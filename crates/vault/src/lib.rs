//! Coffre de secrets (REQ-019, ADR-0007) — prototype J0.
//!
//! Modèle : maître de clé aléatoire 256 bits (ou secret dérivé d'un mot de
//! passe maître via Argon2id), enveloppe AES-256-GCM. Aucun secret en clair
//! dans les logs ; buffers sensibles zeroizés ; types `secrecy::Secret`.
#![forbid(unsafe_code)]

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use secrecy::{ExposeSecret, SecretBox, SecretString};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;
/// Paramètres Argon2id (RFC 9106 recommandations minimales renforcées).
pub const ARGON2_M_KIB: u32 = 64 * 1024;
pub const ARGON2_T: u32 = 3;
pub const ARGON2_P: u32 = 1;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("mot de passe maître incorrect ou blob corrompu")]
    AuthFailed,
    #[error("paramètres KDF invalides : {0}")]
    Kdf(String),
    #[error("chiffrement impossible : {0}")]
    Cipher(String),
    #[error("blob invalide : {0}")]
    Blob(String),
}

/// Maître de clé en mémoire, zeroizé à la suppression.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; KEY_LEN]);

impl MasterKey {
    /// Nouvelle maître de clé aléatoire.
    pub fn generate() -> Self {
        let mut k = [0u8; KEY_LEN];
        rand::thread_rng().fill_bytes(&mut k);
        Self(k)
    }

    /// Dérive une maître de clé d'un mot de passe maître (Argon2id).
    pub fn from_password(
        password: &SecretString,
        salt: &[u8; SALT_LEN],
    ) -> Result<Self, VaultError> {
        let params = Params::new(ARGON2_M_KIB, ARGON2_T, ARGON2_P, Some(KEY_LEN))
            .map_err(|e| VaultError::Kdf(e.to_string()))?;
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0u8; KEY_LEN];
        argon2
            .hash_password_into(password.expose_secret().as_bytes(), salt, &mut key)
            .map_err(|e| VaultError::Kdf(e.to_string()))?;
        Ok(Self(key))
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

/// Blob chiffré persistable (sans aucun secret en clair).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultBlob {
    pub version: u8,
    pub kdf: String,
    pub argon2_m_kib: u32,
    pub argon2_t: u32,
    pub argon2_p: u32,
    #[serde(with = "hexbytes")]
    pub salt: Vec<u8>,
    #[serde(with = "hexbytes")]
    pub nonce: Vec<u8>,
    #[serde(with = "hexbytes")]
    pub ciphertext: Vec<u8>,
}

mod hexbytes {
    use serde::{Deserialize, Deserializer, Serializer};
    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(v))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        hex::decode(s).map_err(serde::de::Error::custom)
    }
}

/// Scelle un secret avec la maître de clé (enveloppe AES-256-GCM).
pub fn seal(key: &MasterKey, plaintext: &[u8]) -> Result<VaultBlob, VaultError> {
    let cipher =
        Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|e| VaultError::Cipher(e.to_string()))?;
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce_bytes), plaintext)
        .map_err(|e| VaultError::Cipher(e.to_string()))?;
    Ok(VaultBlob {
        version: 1,
        kdf: "none(master-key)".into(),
        argon2_m_kib: 0,
        argon2_t: 0,
        argon2_p: 0,
        salt: vec![],
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

/// Scelle un secret avec un mot de passe maître (Argon2id + AES-256-GCM).
pub fn seal_with_password(
    password: &SecretString,
    plaintext: &[u8],
) -> Result<VaultBlob, VaultError> {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    let key = MasterKey::from_password(password, &salt)?;
    let mut blob = seal(&key, plaintext)?;
    blob.kdf = "argon2id".into();
    blob.argon2_m_kib = ARGON2_M_KIB;
    blob.argon2_t = ARGON2_T;
    blob.argon2_p = ARGON2_P;
    blob.salt = salt.to_vec();
    Ok(blob)
}

/// Ouvre un blob scellé par mot de passe maître ; le clair est rendu dans
/// un type `Secret` et zeroizé à la suppression.
pub fn open_with_password(
    password: &SecretString,
    blob: &VaultBlob,
) -> Result<SecretBox<Vec<u8>>, VaultError> {
    if blob.kdf != "argon2id" || blob.salt.len() != SALT_LEN {
        return Err(VaultError::Blob("kdf/salt inattendus".into()));
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&blob.salt);
    let key = MasterKey::from_password(password, &salt)?;
    open_with_key(&key, blob)
}

/// Ouvre un blob scellé par maître de clé directe.
pub fn open_with_key(key: &MasterKey, blob: &VaultBlob) -> Result<SecretBox<Vec<u8>>, VaultError> {
    if blob.nonce.len() != NONCE_LEN {
        return Err(VaultError::Blob("nonce invalide".into()));
    }
    let cipher =
        Aes256Gcm::new_from_slice(key.as_bytes()).map_err(|e| VaultError::Cipher(e.to_string()))?;
    let plain = cipher
        .decrypt(Nonce::from_slice(&blob.nonce), blob.ciphertext.as_ref())
        .map_err(|_| VaultError::AuthFailed)?;
    Ok(SecretBox::new(Box::new(plain)))
}

/// Génère un nonce aléatoire (utilitaire exposé pour tests).
pub fn random_nonce() -> [u8; NONCE_LEN] {
    let mut n = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut n);
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    /// TEST-VAULT-001 : round-trip Argon2id + AES-256-GCM.
    #[test]
    fn round_trip_password() {
        let pw = SecretString::new("mot-de-passe-maitre-correct".into());
        let secret = b"ssh-pass-TopSecret-42".to_vec();
        let blob = seal_with_password(&pw, &secret).expect("seal");
        let out = open_with_password(&pw, &blob).expect("open");
        assert_eq!(out.expose_secret(), &secret);
    }

    /// TEST-VAULT-001b : mauvais mot de passe → échec, aucun clair divulgué.
    #[test]
    fn wrong_password_fails() {
        let pw = SecretString::new("bon-mot-de-passe".into());
        let bad = SecretString::new("mauvais".into());
        let blob = seal_with_password(&pw, b"secret").expect("seal");
        assert!(matches!(
            open_with_password(&bad, &blob),
            Err(VaultError::AuthFailed)
        ));
    }

    /// TEST-VAULT-001c : le blob sérialisé ne contient ni le clair ni la clé.
    #[test]
    fn blob_leaks_nothing() {
        let pw = SecretString::new("pw".into());
        let secret = b"VALEUR-ULTRA-SECRETE-1234567890";
        let blob = seal_with_password(&pw, secret).expect("seal");
        let json = serde_json::to_string(&blob).expect("serde");
        assert!(!json.contains("VALEUR-ULTRA-SECRETE"));
        assert!(!json.contains(&hex::encode(secret)));
    }

    /// TEST-VAULT-001d : maître de clé directe round-trip + altération détectée.
    #[test]
    fn master_key_round_trip_and_integrity() {
        let key = MasterKey::generate();
        let blob = seal(&key, b"donnee").expect("seal");
        let out = open_with_key(&key, &blob).expect("open");
        assert_eq!(out.expose_secret().as_slice(), b"donnee");
        let mut tampered = blob.clone();
        tampered.ciphertext[0] ^= 0xFF;
        assert!(open_with_key(&key, &tampered).is_err());
    }
}
