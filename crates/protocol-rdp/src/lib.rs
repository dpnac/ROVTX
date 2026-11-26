//! RDP (REQ-005/006, ADR-0004) — prototype J0.
//!
//! Périmètre J0 : backend filesystem du canal RDPDR (MS-RDPEFS) avec
//! confinement strict aux dossiers sélectionnés par session. IronRDP fournit
//! le canal (`ironrdp-rdpdr`) mais aucun backend filesystem : c'est notre
//! extension Rust. Chaque IRP du serveur (create/read/write/close) est résolu
//! uniquement dans la racine du partage ; toute évasion → ACCESS_DENIED.
//!
//! La connexion RDP complète (NLA, rendu, presse-papiers) est prototypée en
//! J3 contre un serveur réel ; ici la preuve porte sur le confinement.
#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ironrdp_pdu::PduResult;
use ironrdp_pdu::ironrdp_core::impl_as_any;
use ironrdp_rdpdr::backend::RdpdrBackend;
use ironrdp_rdpdr::pdu::efs::DeviceControlRequest;
use ironrdp_rdpdr::pdu::efs::{
    DeviceCloseResponse, DeviceCreateResponse, DeviceIoResponse, DeviceReadResponse,
    DeviceWriteResponse, NtStatus, ServerDeviceAnnounceResponse, ServerDriveIoRequest,
};
use ironrdp_rdpdr::pdu::esc::{ScardCall, ScardIoCtlCode};
use ironrdp_svc::SvcMessage;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RdpdrError {
    #[error("chemin hors du partage : {0}")]
    Escape(String),
    #[error("io : {0}")]
    Io(#[from] std::io::Error),
}

/// Un partage de dossiers : racine locale + nom vu par l'hôte distant
/// (`\\tsclient\<nom>`).
#[derive(Debug, Clone)]
pub struct ShareMapping {
    pub root: PathBuf,
    pub share_name: String,
}

/// Résolution confinée d'un chemin RDPDR (format Windows `\a\b.txt`) dans la
/// racine du partage. Refuse `..`, chemins absolus locaux et séparateurs
/// mixtes sortants.
pub fn resolve_confined(root: &Path, rdpdr_path: &str) -> Result<PathBuf, RdpdrError> {
    // Les chemins RDPDR utilisent des séparateurs Windows : on les traite
    // explicitement (le séparateur natif Linux n'est pas '\\').
    let cleaned = rdpdr_path.trim_start_matches(['\\', '/']);
    if cleaned.contains(':') {
        return Err(RdpdrError::Escape(rdpdr_path.to_string()));
    }
    let mut joined = root.to_path_buf();
    for comp in cleaned.split(['\\', '/']) {
        match comp {
            "" | "." => {}
            ".." => return Err(RdpdrError::Escape(rdpdr_path.to_string())),
            name => joined.push(name),
        }
    }
    // Vérification canonique quand le chemin existe (symlinks compris).
    if joined.exists() {
        let canon_root = root.canonicalize()?;
        let canon = joined.canonicalize()?;
        if !canon.starts_with(&canon_root) {
            return Err(RdpdrError::Escape(rdpdr_path.to_string()));
        }
    } else if let Some(parent) = joined.parent()
        && parent.exists()
    {
        let canon_root = root.canonicalize()?;
        let canon_parent = parent.canonicalize()?;
        if !canon_parent.starts_with(&canon_root) {
            return Err(RdpdrError::Escape(rdpdr_path.to_string()));
        }
    }
    Ok(joined)
}

/// Backend filesystem RDPDR confiné : sert les IRP create/read/write/close
/// d'un partage unique, sans jamais sortir de sa racine.
#[derive(Debug)]
pub struct ConfinedDriveBackend {
    pub mapping: ShareMapping,
    /// file_id → chemin local ouvert.
    open_files: HashMap<u32, PathBuf>,
    next_file_id: u32,
}

impl ConfinedDriveBackend {
    pub fn new(mapping: ShareMapping) -> Self {
        Self {
            mapping,
            open_files: HashMap::new(),
            next_file_id: 1,
        }
    }

    fn resolve(&self, path: &str) -> Result<PathBuf, RdpdrError> {
        resolve_confined(&self.mapping.root, path)
    }

    /// IRP_MJ_CREATE : ouvre un chemin du partage ; `file_id` rendu si OK.
    pub fn create(&mut self, path: &str) -> Result<u32, NtStatus> {
        let local = self.resolve(path).map_err(|_| NtStatus::ACCESS_DENIED)?;
        let file_id = self.next_file_id;
        self.next_file_id += 1;
        self.open_files.insert(file_id, local);
        Ok(file_id)
    }

    /// IRP_MJ_READ : lecture bornée dans un fichier ouvert.
    pub fn read(&self, file_id: u32, offset: u64, length: u32) -> Result<Vec<u8>, NtStatus> {
        use std::io::{Read, Seek, SeekFrom};
        let path = self
            .open_files
            .get(&file_id)
            .ok_or(NtStatus::ACCESS_DENIED)?;
        let mut f = std::fs::File::open(path).map_err(|_| NtStatus::ACCESS_DENIED)?;
        f.seek(SeekFrom::Start(offset))
            .map_err(|_| NtStatus::ACCESS_DENIED)?;
        let mut buf = vec![0u8; length as usize];
        let n = f.read(&mut buf).map_err(|_| NtStatus::ACCESS_DENIED)?;
        buf.truncate(n);
        Ok(buf)
    }

    /// IRP_MJ_WRITE : écriture dans un fichier ouvert.
    pub fn write(&mut self, file_id: u32, offset: u64, data: &[u8]) -> Result<u32, NtStatus> {
        use std::io::{Seek, SeekFrom, Write};
        let path = self
            .open_files
            .get(&file_id)
            .ok_or(NtStatus::ACCESS_DENIED)?
            .clone();
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .map_err(|_| NtStatus::ACCESS_DENIED)?;
        f.seek(SeekFrom::Start(offset))
            .map_err(|_| NtStatus::ACCESS_DENIED)?;
        let n = f.write(data).map_err(|_| NtStatus::ACCESS_DENIED)?;
        Ok(n as u32)
    }

    /// IRP_MJ_CLOSE : ferme le descripteur.
    pub fn close(&mut self, file_id: u32) {
        self.open_files.remove(&file_id);
    }
}

impl_as_any!(ConfinedDriveBackend);

impl RdpdrBackend for ConfinedDriveBackend {
    fn handle_server_device_announce_response(
        &mut self,
        _pdu: ServerDeviceAnnounceResponse,
    ) -> PduResult<()> {
        Ok(())
    }

    fn handle_scard_call(
        &mut self,
        _req: DeviceControlRequest<ScardIoCtlCode>,
        _call: ScardCall,
    ) -> PduResult<Vec<SvcMessage>> {
        Ok(vec![])
    }

    fn handle_drive_io_request(&mut self, req: ServerDriveIoRequest) -> PduResult<Vec<SvcMessage>> {
        use ironrdp_rdpdr::pdu::RdpdrPdu;
        match req {
            ServerDriveIoRequest::ServerCreateDriveRequest(create) => {
                let base = create.device_io_request.clone();
                let (status, file_id) = match self.create(&create.path) {
                    Ok(id) => (NtStatus::SUCCESS, id),
                    Err(st) => (st, 0),
                };
                Ok(vec![SvcMessage::from(RdpdrPdu::DeviceCreateResponse(
                    DeviceCreateResponse {
                        device_io_reply: DeviceIoResponse::new(base, status),
                        file_id,
                        information: ironrdp_rdpdr::pdu::efs::Information::file_opened(),
                    },
                ))])
            }
            ServerDriveIoRequest::DeviceReadRequest(read) => {
                let base = read.device_io_request.clone();
                match self.read(read.device_io_request.file_id, read.offset, read.length) {
                    Ok(data) => Ok(vec![SvcMessage::from(RdpdrPdu::DeviceReadResponse(
                        DeviceReadResponse {
                            device_io_reply: DeviceIoResponse::new(base, NtStatus::SUCCESS),
                            read_data: data,
                        },
                    ))]),
                    Err(status) => Ok(vec![SvcMessage::from(RdpdrPdu::DeviceReadResponse(
                        DeviceReadResponse {
                            device_io_reply: DeviceIoResponse::new(base, status),
                            read_data: vec![],
                        },
                    ))]),
                }
            }
            ServerDriveIoRequest::DeviceWriteRequest(write) => {
                let base = write.device_io_request.clone();
                match self.write(
                    write.device_io_request.file_id,
                    write.offset,
                    &write.write_data,
                ) {
                    Ok(n) => Ok(vec![SvcMessage::from(RdpdrPdu::DeviceWriteResponse(
                        DeviceWriteResponse {
                            device_io_reply: DeviceIoResponse::new(base, NtStatus::SUCCESS),
                            length: n,
                        },
                    ))]),
                    Err(status) => Ok(vec![SvcMessage::from(RdpdrPdu::DeviceWriteResponse(
                        DeviceWriteResponse {
                            device_io_reply: DeviceIoResponse::new(base, status),
                            length: 0,
                        },
                    ))]),
                }
            }
            ServerDriveIoRequest::DeviceCloseRequest(close) => {
                let base = close.device_io_request.clone();
                self.close(close.device_io_request.file_id);
                Ok(vec![SvcMessage::from(RdpdrPdu::DeviceCloseResponse(
                    DeviceCloseResponse {
                        device_io_response: DeviceIoResponse::new(base, NtStatus::SUCCESS),
                    },
                ))])
            }
            other => {
                // Les autres IRP (query info, directory, security…) seront
                // implémentés en J3 ; au prototype ils sont refusés proprement.
                tracing::debug!(?other, "IRP non implémenté au prototype J0");
                Ok(vec![])
            }
        }
    }
}
