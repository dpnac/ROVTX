//! Serveur DHCPv4 (REQ-020/021, ADR-0005) — prototype J0.
//!
//! Périmètre J0 : éditeur d'options typées avec aperçu des octets sur le fil,
//! règles de priorité (réservation > classe > scope > global), encodage via
//! `dhcproto`. Le moteur DORA complet et les baux persistants arrivent en J4.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum OptionError {
    #[error("code d'option réservé ou géré par le moteur : {0}")]
    Reserved(u8),
    #[error("type incompatible avec le code {code} : attendu {expected}")]
    TypeMismatch { code: u8, expected: &'static str },
    #[error("valeur hexadécimale invalide : {0}")]
    BadHex(String),
    #[error("texte non représentable en ASCII : {0}")]
    BadText(String),
    #[error("adresse IPv4 invalide : {0}")]
    BadIpv4(String),
    #[error("liste IPv4 vide ou invalide")]
    BadIpv4List,
    #[error("entier hors plage pour le type demandé")]
    OutOfRange,
    #[error("longueur hors limites ({len} octets, max {max})")]
    TooLong { len: usize, max: usize },
}

/// Codes gérés automatiquement par le moteur (non surchargeables).
pub const ENGINE_MANAGED: &[u8] = &[0, 1, 2, 53, 54, 50, 51, 58, 59, 61, 62, 63, 255];
/// Codes typés mis en avant par le cahier des charges (CDC §04).
pub const WELL_KNOWN: &[u8] = &[3, 6, 15, 43, 51, 60, 66, 67, 121, 150];

/// Types de valeur de l'éditeur d'options (CDC §04).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum OptionValue {
    /// HEX brut : espaces, deux-points ou préfixe 0x acceptés.
    Hex(String),
    /// Texte avec encodage explicite (ASCII pour 66/67).
    Text(String),
    Ipv4(Ipv4Addr),
    Ipv4List(Vec<Ipv4Addr>),
    U8(u8),
    U16(u16),
    U32(u32),
    Bool(bool),
    /// Routes classeless (RFC 3442) : (préfixe, destination, passerelle).
    Routes(Vec<(u8, Ipv4Addr, Ipv4Addr)>),
}

/// Portée d'une option, du plus spécifique au plus général.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Scope {
    Reservation = 0,
    Class = 1,
    Scope = 2,
    Global = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpOption {
    pub code: u8,
    pub name: String,
    pub value: OptionValue,
    pub scope: Scope,
    /// Priorité numérique : plus petit = plus prioritaire à portée égale.
    pub priority: u16,
}

/// Aperçu des octets sur le fil (représentation exacte envoyée).
pub fn encode_bytes(value: &OptionValue) -> Result<Vec<u8>, OptionError> {
    Ok(match value {
        OptionValue::Hex(s) => parse_hex(s)?,
        OptionValue::Text(s) => {
            if !s.is_ascii() {
                return Err(OptionError::BadText(s.clone()));
            }
            s.as_bytes().to_vec()
        }
        OptionValue::Ipv4(ip) => ip.octets().to_vec(),
        OptionValue::Ipv4List(list) => {
            if list.is_empty() {
                return Err(OptionError::BadIpv4List);
            }
            list.iter().flat_map(|ip| ip.octets()).collect()
        }
        OptionValue::U8(v) => v.to_be_bytes().to_vec(),
        OptionValue::U16(v) => v.to_be_bytes().to_vec(),
        OptionValue::U32(v) => v.to_be_bytes().to_vec(),
        OptionValue::Bool(b) => vec![u8::from(*b)],
        OptionValue::Routes(routes) => {
            let mut out = Vec::new();
            for (prefix_len, dest, gw) in routes {
                if *prefix_len > 32 {
                    return Err(OptionError::OutOfRange);
                }
                let sig = significant_octets(*prefix_len);
                out.push(*prefix_len);
                out.extend_from_slice(&dest.octets()[..sig]);
                out.extend_from_slice(&gw.octets());
            }
            out
        }
    })
}

fn significant_octets(prefix_len: u8) -> usize {
    match prefix_len {
        0 => 0,
        1..=8 => 1,
        9..=16 => 2,
        17..=24 => 3,
        _ => 4,
    }
}

/// Accepte `01:04:C0:A8:32:0A`, `01 04 C0 A8 32 0A`, `0x0104C0A8320A`.
pub fn parse_hex(input: &str) -> Result<Vec<u8>, OptionError> {
    let cleaned: String = input
        .trim()
        .strip_prefix("0x")
        .unwrap_or(input.trim())
        .chars()
        .filter(|c| !matches!(c, ':' | ' ' | '-' | '_'))
        .collect();
    if cleaned.is_empty() || !cleaned.len().is_multiple_of(2) {
        return Err(OptionError::BadHex(input.to_string()));
    }
    hex::decode(cleaned).map_err(|_| OptionError::BadHex(input.to_string()))
}

/// Valide une option : code non réservé au moteur, type cohérent, taille OK.
pub fn validate(option: &DhcpOption) -> Result<Vec<u8>, OptionError> {
    if ENGINE_MANAGED.contains(&option.code) {
        return Err(OptionError::Reserved(option.code));
    }
    let bytes = encode_bytes(&option.value)?;
    if bytes.len() > 255 {
        return Err(OptionError::TooLong {
            len: bytes.len(),
            max: 255,
        });
    }
    check_type_for_code(option.code, &option.value)?;
    Ok(bytes)
}

fn check_type_for_code(code: u8, value: &OptionValue) -> Result<(), OptionError> {
    let ok = match code {
        3 | 6 | 150 => matches!(value, OptionValue::Ipv4(_) | OptionValue::Ipv4List(_)),
        66 | 67 | 15 => matches!(value, OptionValue::Text(_)),
        51 => matches!(value, OptionValue::U32(_)),
        121 => matches!(value, OptionValue::Routes(_)),
        43 | 60 | 12 | 14 => matches!(value, OptionValue::Hex(_) | OptionValue::Text(_)),
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        Err(OptionError::TypeMismatch {
            code,
            expected: expected_type_name(code),
        })
    }
}

fn expected_type_name(code: u8) -> &'static str {
    match code {
        3 | 6 | 150 => "IPv4 ou liste IPv4",
        66 | 67 | 15 => "texte",
        51 => "uint32",
        121 => "routes (RFC 3442)",
        _ => "type libre",
    }
}

/// Résout la valeur effective d'un code : réservation > classe > scope >
/// global ; à portée égale, la priorité numérique la plus petite gagne.
pub fn resolve_effective(options: &[DhcpOption], code: u8) -> Option<&DhcpOption> {
    options
        .iter()
        .filter(|o| o.code == code)
        .min_by_key(|o| (o.scope, o.priority))
}

/// Encode une option validée au format RFC 2132 (code, longueur, octets).
pub fn encode_tlv(option: &DhcpOption) -> Result<Vec<u8>, OptionError> {
    let bytes = validate(option)?;
    let mut tlv = vec![option.code, bytes.len() as u8];
    tlv.extend_from_slice(&bytes);
    Ok(tlv)
}

/// Construit une carte d'options effectives (code → octets) pour un client,
/// après résolution des portées/priorités.
pub fn effective_option_map(options: &[DhcpOption]) -> Result<BTreeMap<u8, Vec<u8>>, OptionError> {
    let mut map = BTreeMap::new();
    let codes: BTreeMap<u8, ()> = options.iter().map(|o| (o.code, ())).collect();
    for code in codes.keys() {
        if let Some(opt) = resolve_effective(options, *code) {
            map.insert(*code, validate(opt)?);
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opt(code: u8, value: OptionValue, scope: Scope) -> DhcpOption {
        DhcpOption {
            code,
            name: format!("option-{code}"),
            value,
            scope,
            priority: 100,
        }
    }

    /// TEST-DHCP-001 : option 43 HEX `01:04:C0:A8:32:0A` → `01 04 C0 A8 32 0A`.
    #[test]
    fn option_43_hex_bytes() {
        let o = opt(
            43,
            OptionValue::Hex("01:04:C0:A8:32:0A".into()),
            Scope::Scope,
        );
        assert_eq!(
            validate(&o).unwrap(),
            vec![0x01, 0x04, 0xC0, 0xA8, 0x32, 0x0A]
        );
    }

    /// TEST-DHCP-002 : option 66 texte → ASCII, pas hex.
    #[test]
    fn option_66_text_is_ascii() {
        let o = opt(66, OptionValue::Text("192.168.50.10".into()), Scope::Scope);
        assert_eq!(validate(&o).unwrap(), b"192.168.50.10".to_vec());
    }

    /// TEST-DHCP-003 : option 67 bootfile → ASCII.
    #[test]
    fn option_67_bootfile_ascii() {
        let o = opt(67, OptionValue::Text("bootx64.efi".into()), Scope::Scope);
        assert_eq!(validate(&o).unwrap(), b"bootx64.efi".to_vec());
    }

    /// TEST-DHCP-004 : option 150 liste IPv4 → `C0 A8 32 0A`.
    #[test]
    fn option_150_ipv4_list() {
        let o = opt(
            150,
            OptionValue::Ipv4List(vec!["192.168.50.10".parse().unwrap()]),
            Scope::Scope,
        );
        assert_eq!(validate(&o).unwrap(), vec![0xC0, 0xA8, 0x32, 0x0A]);
    }

    /// TEST-DHCP-005 : option 121 routes structurées (RFC 3442).
    #[test]
    fn option_121_routes() {
        let o = opt(
            121,
            OptionValue::Routes(vec![(
                24,
                "10.0.0.0".parse().unwrap(),
                "192.168.1.1".parse().unwrap(),
            )]),
            Scope::Scope,
        );
        assert_eq!(validate(&o).unwrap(), vec![24, 10, 0, 0, 192, 168, 1, 1]);
    }

    /// TEST-DHCP-008 : priorité réservation > classe > scope > global.
    #[test]
    fn priority_resolution() {
        let global = opt(66, OptionValue::Text("global".into()), Scope::Global);
        let scope = opt(66, OptionValue::Text("scope".into()), Scope::Scope);
        let resv = opt(66, OptionValue::Text("reserv".into()), Scope::Reservation);
        let class = opt(66, OptionValue::Text("class".into()), Scope::Class);
        let all = [global, scope, resv, class];
        let eff = resolve_effective(&all, 66).unwrap();
        assert_eq!(eff.scope, Scope::Reservation);
        assert_eq!(validate(eff).unwrap(), b"reserv".to_vec());
    }

    /// TEST-DHCP-009 : les codes gérés par le moteur ne sont pas surchargeables.
    #[test]
    fn engine_managed_rejected() {
        for code in [1u8, 51, 53, 54, 61, 255] {
            assert!(ENGINE_MANAGED.contains(&code), "précondition {code}");
            let o = opt(code, OptionValue::U32(3600), Scope::Global);
            assert_eq!(validate(&o), Err(OptionError::Reserved(code)));
        }
        // Codes configurables avec le type attendu : validation OK.
        let r3 = opt(
            3,
            OptionValue::Ipv4("192.168.1.1".parse().unwrap()),
            Scope::Global,
        );
        assert!(validate(&r3).is_ok());
        let r6 = opt(
            6,
            OptionValue::Ipv4List(vec!["9.9.9.9".parse().unwrap()]),
            Scope::Global,
        );
        assert!(validate(&r6).is_ok());
        // Code configurable mais type incohérent : rejet typé.
        let bad = opt(3, OptionValue::U32(3600), Scope::Global);
        assert!(matches!(
            validate(&bad),
            Err(OptionError::TypeMismatch { code: 3, .. })
        ));
    }

    /// TEST-DHCP-010 : formats hex variés acceptés, invalides rejetés.
    #[test]
    fn hex_formats() {
        assert_eq!(parse_hex("0x0104").unwrap(), vec![1, 4]);
        assert_eq!(parse_hex("01 04").unwrap(), vec![1, 4]);
        assert_eq!(parse_hex("01:04").unwrap(), vec![1, 4]);
        assert!(parse_hex("0x010").is_err());
        assert!(parse_hex("zz").is_err());
    }

    /// TEST-DHCP-011 : TLV RFC 2132 (code, longueur, valeur).
    #[test]
    fn tlv_encoding() {
        let o = opt(
            150,
            OptionValue::Ipv4("192.168.50.10".parse().unwrap()),
            Scope::Global,
        );
        assert_eq!(
            encode_tlv(&o).unwrap(),
            vec![150, 4, 0xC0, 0xA8, 0x32, 0x0A]
        );
    }

    /// TEST-DHCP-012 : texte non ASCII rejeté (encodage explicite).
    #[test]
    fn non_ascii_text_rejected() {
        let o = opt(66, OptionValue::Text("hôte".into()), Scope::Global);
        assert!(matches!(validate(&o), Err(OptionError::BadText(_))));
    }

    /// TEST-DHCP-013 : cross-vérification de l'encodage des options 43/66/67/
    /// 121/150 contre `dhcproto` (bibliothèque indépendante), octet par octet.
    #[test]
    fn cross_check_against_dhcproto() {
        use dhcproto::v4::{DhcpOption as DOpt, DhcpOptions, OptionCode as DCode};
        use ipnet::Ipv4Net;

        let mut opts = DhcpOptions::new();
        opts.insert(DOpt::VendorExtensions(vec![
            0x01, 0x04, 0xC0, 0xA8, 0x32, 0x0A,
        ]));
        opts.insert(DOpt::TFTPServerName(b"192.168.50.10".to_vec()));
        opts.insert(DOpt::BootfileName(b"bootx64.efi".to_vec()));
        opts.insert(DOpt::TFTPServerAddress("192.168.50.10".parse().unwrap()));
        opts.insert(DOpt::ClasslessStaticRoute(vec![(
            Ipv4Net::new("10.0.0.0".parse::<std::net::Ipv4Addr>().unwrap(), 24).unwrap(),
            "192.168.1.1".parse().unwrap(),
        )]));

        let expected: BTreeMap<u8, Vec<u8>> = [
            (43, vec![0x01, 0x04, 0xC0, 0xA8, 0x32, 0x0A]),
            (66, b"192.168.50.10".to_vec()),
            (67, b"bootx64.efi".to_vec()),
            (150, vec![0xC0, 0xA8, 0x32, 0x0A]),
            (121, vec![24, 10, 0, 0, 192, 168, 1, 1]),
        ]
        .into_iter()
        .collect();

        // Nos valeurs typées doivent produire exactement les mêmes octets.
        let ours: Vec<DhcpOption> = vec![
            opt(
                43,
                OptionValue::Hex("01:04:C0:A8:32:0A".into()),
                Scope::Global,
            ),
            opt(66, OptionValue::Text("192.168.50.10".into()), Scope::Global),
            opt(67, OptionValue::Text("bootx64.efi".into()), Scope::Global),
            opt(
                150,
                OptionValue::Ipv4("192.168.50.10".parse().unwrap()),
                Scope::Global,
            ),
            opt(
                121,
                OptionValue::Routes(vec![(
                    24,
                    "10.0.0.0".parse().unwrap(),
                    "192.168.1.1".parse().unwrap(),
                )]),
                Scope::Global,
            ),
        ];
        for o in &ours {
            let bytes = validate(o).expect("validate");
            assert_eq!(bytes, expected[&o.code], "code {}", o.code);
        }

        // Et dhcproto ré-encode ces mêmes options à l'identique (TLV).
        for (code, want) in &expected {
            let dopt = opts.get(DCode::from(*code));
            assert!(dopt.is_some(), "dhcproto connaît le code {code}");
            let mut buf = Vec::new();
            let mut enc = dhcproto::encoder::Encoder::new(&mut buf);
            dhcproto::Encodable::encode(dopt.unwrap(), &mut enc).expect("encode");
            let tlv = enc.buffer_filled().to_vec();
            assert_eq!(tlv[0], *code, "TLV code {code}");
            assert_eq!(tlv[1] as usize, want.len(), "TLV len {code}");
            assert_eq!(&tlv[2..], &want[..], "TLV valeur {code}");
        }
    }
}
