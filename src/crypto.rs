use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key,
};
use anyhow::{anyhow, Result};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{rand_core::RngCore, SaltString}};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};

#[derive(Serialize, Deserialize, Clone)]
pub struct TimeLockedMessage {
    pub encrypted_content: String,
    pub nonce: String,
    pub salt: String,
    pub password_hash: String,
    pub unlock_date: DateTime<Utc>,
    pub label: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl TimeLockedMessage {
    pub fn new(
        content: &str, 
        password: &str, 
        unlock_date: DateTime<Utc>,
        label: Option<String>
    ) -> Result<Self> {
        // Generate random salt
        let salt = SaltString::generate(&mut OsRng);
        
        // Hash the password with Argon2
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?
            .to_string();
        
        // Derive encryption key from password
        let key = derive_key(password, salt.as_str())?;
        
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the content
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let encrypted_bytes = cipher
            .encrypt(nonce, content.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        // Encode to base64
        let encrypted_content = general_purpose::STANDARD.encode(encrypted_bytes);
        let nonce_str = general_purpose::STANDARD.encode(nonce_bytes);
        
        Ok(TimeLockedMessage {
            encrypted_content,
            nonce: nonce_str,
            salt: salt.to_string(),
            password_hash,
            unlock_date,
            label,
            created_at: Utc::now(),
        })
    }
    
    pub fn unlock(&self, password: &str) -> Result<String> {
        // Verify password first
        let parsed_hash = PasswordHash::new(&self.password_hash)
            .map_err(|e| anyhow!("Invalid password hash: {}", e))?;
        
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow!("Invalid password"))?;
        
        // Check if unlock time has passed
        if self.unlock_date > Utc::now() {
            return Err(anyhow!("Message is still time-locked"));
        }
        
        // Derive the same key
        let key = derive_key(password, &self.salt)?;
        
        // Decode base64
        let encrypted_bytes = general_purpose::STANDARD
            .decode(&self.encrypted_content)
            .map_err(|e| anyhow!("Failed to decode encrypted content: {}", e))?;
        
        let nonce_bytes = general_purpose::STANDARD
            .decode(&self.nonce)
            .map_err(|e| anyhow!("Failed to decode nonce: {}", e))?;
        
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Decrypt
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key));
        let decrypted_bytes = cipher
            .decrypt(nonce, encrypted_bytes.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;
        
        let content = String::from_utf8(decrypted_bytes)
            .map_err(|e| anyhow!("Invalid UTF-8 in decrypted content: {}", e))?;
        
        Ok(content)
    }
}

fn derive_key(password: &str, salt: &str) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();
    let salt_string = SaltString::from_b64(salt)
        .map_err(|e| anyhow!("Invalid salt: {}", e))?;
    
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt_string.as_str().as_bytes(), &mut key)
        .map_err(|e| anyhow!("Key derivation failed: {}", e))?;
    
    Ok(key)
}