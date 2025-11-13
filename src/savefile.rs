//! Savefile - Utilities for saving and restoring game state

use crate::AUTO_SAVE_SLOT;
use std::path::Path;

/// Represents a save file for a game
#[derive(Debug, Clone)]
pub struct Savefile {
    pub game_name: Option<String>,
    pub slot: String,
}

impl Savefile {
    /// Create a new Savefile
    pub fn new(game_name: Option<String>, slot: Option<String>) -> Self {
        let slot = slot
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| AUTO_SAVE_SLOT.to_string());

        Self { game_name, slot }
    }

    /// Get the filename for this save
    pub fn filename(&self) -> String {
        let basename = if let Some(ref game_name) = self.game_name {
            format!("{}_{}", game_name, self.slot)
        } else {
            self.slot.clone()
        };
        format!("saves/{}.qzl", basename)
    }

    /// Check if the save file exists
    pub fn exists(&self) -> bool {
        Path::new(&self.filename()).exists()
    }

    /// Delete the save file
    pub fn delete(&self) -> std::io::Result<()> {
        let path = self.filename();
        if Path::new(&path).exists() {
            std::fs::remove_file(path)
        } else {
            Ok(())
        }
    }
}
