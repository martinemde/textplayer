//! Gamefile - Represents a game file and its metadata

use crate::{game_dir, Error, Result};
use std::path::PathBuf;

/// Represents a Z-Machine game file
#[derive(Debug, Clone)]
pub struct Gamefile {
    pub name: String,
    pub path: PathBuf,
}

impl Gamefile {
    /// Create a new Gamefile from a name and path
    pub fn new(name: String, path: PathBuf) -> Self {
        Self { name, path }
    }

    /// Create a Gamefile from user input
    ///
    /// If the input contains a path separator, it's treated as a full path.
    /// Otherwise, it's treated as a game name and searched in the games directory.
    pub fn from_input(input: &str) -> Result<Self> {
        if input.contains('/') || input.contains('\\') {
            let path = PathBuf::from(input);
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(input)
                .to_string();
            Ok(Self::new(name, path))
        } else {
            // Search in games directory
            let game_dir = game_dir();

            let mut matches = Vec::new();
            if let Ok(entries) = std::fs::read_dir(&game_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            if file_name.starts_with(input) {
                                matches.push(path);
                            }
                        }
                    }
                }
            }

            match matches.len() {
                0 => Err(Error::GameNotFound(input.to_string())),
                1 => {
                    let path = matches.into_iter().next().unwrap();
                    let name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(input)
                        .to_string();
                    Ok(Self::new(name, path))
                }
                _ => {
                    let names: Vec<String> = matches
                        .iter()
                        .filter_map(|p| p.file_name())
                        .filter_map(|n| n.to_str())
                        .map(|s| s.to_string())
                        .collect();
                    Err(Error::MultipleGamesFound(input.to_string(), names))
                }
            }
        }
    }

    /// Check if the game file exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    /// Get the full path to the game file
    pub fn full_path(&self) -> Result<String> {
        self.path
            .canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .or_else(|_| Ok(self.path.to_string_lossy().to_string()))
    }
}
