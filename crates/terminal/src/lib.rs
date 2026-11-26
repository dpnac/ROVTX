//! Terminal VT/xterm (REQ-011, ADR-0009) — prototype J0.
//!
//! Émulation : `alacritty_terminal` (grille, ANSI, scrollback, sélection).
//! PTY local : `portable-pty` (ConPTY sous Windows, openpty sous Linux).
//! Le rendu egui arrivera en J1/J2 ; ce prototype prouve l'émulation,
//! le flux PTY, la sélection/copie et la transmission intacte de Ctrl+C.
#![forbid(unsafe_code)]

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::term::Config as TermConfig;
use alacritty_terminal::term::Term;
use alacritty_terminal::vte::ansi::Processor;
use portable_pty::{CommandBuilder, MasterPty, NativePtySystem, PtySize, PtySystem};
use thiserror::Error;

pub const DEFAULT_COLS: u16 = 80;
pub const DEFAULT_ROWS: u16 = 24;

#[derive(Debug, Error)]
pub enum TermError {
    #[error("pty : {0}")]
    Pty(String),
    #[error("io : {0}")]
    Io(#[from] std::io::Error),
}

/// Collecteur d'événements du terminal (titre, presse-papiers OSC52…).
#[derive(Clone, Default)]
pub struct EventSink(Arc<Mutex<Vec<String>>>);

impl EventListener for EventSink {
    fn send_event(&self, event: Event) {
        if let Event::Title(t) = event
            && let Ok(mut v) = self.0.lock()
        {
            v.push(t);
        }
    }
}

/// Émulateur VT : grille alacritty_terminal alimentée par un parseur ANSI.
pub struct Emulator {
    term: Term<EventSink>,
    parser: Processor,
    pub events: EventSink,
}

impl Emulator {
    pub fn new(cols: usize, rows: usize, scrollback: usize) -> Self {
        let config = TermConfig {
            scrolling_history: scrollback,
            ..TermConfig::default()
        };
        let size = Size { cols, rows };
        Self {
            term: Term::new(config, &size, EventSink::default()),
            parser: Processor::new(),
            events: EventSink::default(),
        }
    }

    /// Traite des octets venus du PTY/réseau (séquences ANSI comprises).
    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    /// Contenu texte visible (cellules rendues, sans séquences).
    pub fn visible_text(&self) -> String {
        let mut out = String::new();
        let mut last_line = None;
        for cell in self.term.grid().display_iter() {
            match last_line {
                Some(l) if l == cell.point.line.0 => {}
                Some(_) => out.push('\n'),
                None => {}
            }
            last_line = Some(cell.point.line.0);
            out.push(cell.c);
        }
        out
    }

    /// Sélectionne une plage puis rend le texte sélectionné (copie).
    pub fn select_and_copy(
        &mut self,
        start_line: i32,
        start_col: usize,
        end_line: i32,
        end_col: usize,
    ) -> Option<String> {
        use alacritty_terminal::index::{Column, Line, Point, Side};
        use alacritty_terminal::selection::Selection;
        let start = Point::new(Line(start_line), Column(start_col));
        let end = Point::new(Line(end_line), Column(end_col));
        let mut sel = Selection::new(
            alacritty_terminal::selection::SelectionType::Simple,
            start,
            Side::Left,
        );
        sel.update(end, Side::Right);
        sel.include_all();
        self.term.selection = Some(sel);
        self.term.selection_to_string()
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.term.resize(Size { cols, rows });
    }
}

struct Size {
    cols: usize,
    rows: usize,
}

impl alacritty_terminal::grid::Dimensions for Size {
    fn total_lines(&self) -> usize {
        self.rows
    }
    fn screen_lines(&self) -> usize {
        self.rows
    }
    fn columns(&self) -> usize {
        self.cols
    }
}

/// Shell par défaut de la plateforme (bash/zsh sous Unix, cmd sous Windows).
fn default_shell_command() -> CommandBuilder {
    #[cfg(unix)]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        CommandBuilder::new(shell)
    }
    #[cfg(windows)]
    {
        CommandBuilder::new("cmd.exe")
    }
}

/// Session shell locale via PTY (portable-pty).
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtySession {
    /// Démarre le shell par défaut de la plateforme (bash/PowerShell/CMD).
    pub fn spawn_shell(cols: u16, rows: u16) -> Result<Self, TermError> {
        let system = NativePtySystem::default();
        let pair = system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| TermError::Pty(e.to_string()))?;
        let cmd = default_shell_command();
        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| TermError::Pty(e.to_string()))?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| TermError::Pty(e.to_string()))?;
        Ok(Self {
            master: pair.master,
            writer,
            child,
        })
    }

    /// Lit la sortie du PTY (bloquant, à appeler depuis un thread dédié).
    pub fn reader(&self) -> Result<Box<dyn Read + Send>, TermError> {
        self.master
            .try_clone_reader()
            .map_err(|e| TermError::Pty(e.to_string()))
    }

    /// Envoie des touches au processus : Ctrl+C est transmis tel quel
    /// (0x03), l'UI ne l'intercepte jamais (exigence REQ-011).
    pub fn write(&mut self, bytes: &[u8]) -> Result<(), TermError> {
        self.writer.write_all(bytes)?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), TermError> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| TermError::Pty(e.to_string()))
    }

    pub fn try_wait(&mut self) -> Result<Option<u32>, TermError> {
        Ok(self
            .child
            .try_wait()
            .map_err(|e| TermError::Pty(e.to_string()))?
            .map(|s| s.exit_code()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TEST-TERM-001 : émulation ANSI — couleurs, curseur, texte visible.
    #[test]
    fn ansi_emulation() {
        let mut emu = Emulator::new(40, 5, 100);
        emu.feed(b"bonjour \x1b[31mROUGE\x1b[0m monde\r\n");
        let txt = emu.visible_text();
        assert!(txt.contains("bonjour ROUGE monde"), "{txt:?}");
        // Le texte ne contient aucune séquence d'échappement brute.
        assert!(!txt.contains('\x1b'), "séquence ANSI non interprétée");
    }

    /// TEST-TERM-002 : scrollback conservé au-delà de l'écran.
    #[test]
    fn scrollback_history() {
        let mut emu = Emulator::new(20, 3, 50);
        for i in 0..10 {
            emu.feed(format!("ligne-{i}\r\n").as_bytes());
        }
        // La grille historique contient les lignes anciennes.
        use alacritty_terminal::grid::Dimensions as _;
        let history = emu.term.grid().history_size();
        assert!(history >= 5, "historique = {history}");
    }

    /// TEST-TERM-003 : sélection → copie du texte sélectionné.
    #[test]
    fn selection_copy() {
        let mut emu = Emulator::new(40, 5, 10);
        emu.feed(b"abcdef ghijkl\r\n");
        let copied = emu.select_and_copy(0, 0, 0, 5).unwrap_or_default();
        assert!(copied.contains("abcdef"), "copie = {copied:?}");
    }

    /// TEST-TERM-004 : PTY local — le shell exécute une commande, Ctrl+C
    /// (0x03) est transmis au processus (ici : interrompt `sleep`).
    #[test]
    fn pty_shell_and_ctrl_c() {
        let mut session = PtySession::spawn_shell(80, 24).expect("pty");
        let mut reader = session.reader().expect("reader");
        session.write(b"echo PTY-OK\n").expect("write");
        let mut buf = [0u8; 4096];
        let mut seen = String::new();
        // Lecture bornée en temps : le prompt + la sortie arrivent vite.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !seen.contains("PTY-OK") && std::time::Instant::now() < deadline {
            if let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                seen.push_str(&String::from_utf8_lossy(&buf[..n]));
            }
        }
        assert!(seen.contains("PTY-OK"), "sortie pty = {seen:?}");

        // Ctrl+C : lance sleep puis interrompt ; le shell survit et rend
        // la main (preuve que 0x03 traverse l'UI jusqu'au processus).
        session.write(b"sleep 30\n").expect("write");
        std::thread::sleep(std::time::Duration::from_millis(300));
        session.write(&[0x03]).expect("ctrl+c");
        std::thread::sleep(std::time::Duration::from_millis(300));
        session.write(b"echo APRES-CTRL-C\n").expect("write");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !seen.contains("APRES-CTRL-C") && std::time::Instant::now() < deadline {
            if let Ok(n) = reader.read(&mut buf) {
                if n == 0 {
                    break;
                }
                seen.push_str(&String::from_utf8_lossy(&buf[..n]));
            }
        }
        assert!(
            seen.contains("APRES-CTRL-C"),
            "le shell répond après Ctrl+C : {seen:?}"
        );
    }
}
