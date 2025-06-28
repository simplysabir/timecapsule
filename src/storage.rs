use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::crypto::TimeLockedMessage;

fn get_storage_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?;
    
    let storage_dir = home_dir.join(".timecapsule");
    
    if !storage_dir.exists() {
        fs::create_dir_all(&storage_dir)?;
    }
    
    Ok(storage_dir)
}

pub fn save_message(message: &TimeLockedMessage) -> Result<String> {
    let storage_dir = get_storage_dir()?;
    let id = Uuid::new_v4().to_string();
    let file_path = storage_dir.join(format!("{}.json", id));
    
    save_to_file(message, &file_path)?;
    Ok(id)
}

pub fn save_to_file(message: &TimeLockedMessage, file_path: &Path) -> Result<()> {
    let json = serde_json::to_string_pretty(message)?;
    fs::write(file_path, json)?;
    Ok(())
}

pub fn load_message(id: &str) -> Result<TimeLockedMessage> {
    let storage_dir = get_storage_dir()?;
    let file_path = storage_dir.join(format!("{}.json", id));
    load_from_file(&file_path)
}

pub fn load_from_file(file_path: &Path) -> Result<TimeLockedMessage> {
    let json = fs::read_to_string(file_path)
        .map_err(|e| anyhow!("Failed to read file {:?}: {}", file_path, e))?;
    
    let message: TimeLockedMessage = serde_json::from_str(&json)
        .map_err(|e| anyhow!("Failed to parse JSON: {}", e))?;
    
    Ok(message)
}

pub fn list_messages() -> Result<HashMap<String, TimeLockedMessage>> {
    let storage_dir = get_storage_dir()?;
    let mut messages = HashMap::new();
    
    if !storage_dir.exists() {
        return Ok(messages);
    }
    
    for entry in fs::read_dir(storage_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Some(file_stem) = path.file_stem() {
                let id = file_stem.to_string_lossy().to_string();
                match load_from_file(&path) {
                    Ok(message) => {
                        messages.insert(id, message);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to load message {}: {}", id, e);
                    }
                }
            }
        }
    }
    
    Ok(messages)
}