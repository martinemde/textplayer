//! TextPlayer - A Rust interface for running text-based interactive fiction games
//!
//! This library provides structured access to classic text adventure games using
//! the Frotz Z-Machine interpreter.

use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

pub mod command_result;
pub mod commands;
pub mod dfrotz;
pub mod formatters;
pub mod gamefile;
pub mod savefile;
pub mod session;

pub use command_result::CommandResult;
pub use commands::Commands;
pub use dfrotz::Dfrotz;
pub use formatters::Formatters;
pub use gamefile::Gamefile;
pub use savefile::Savefile;
pub use session::Session;

/// Default autosave slot name
pub const AUTO_SAVE_SLOT: &str = "autosave";

/// Default game directory relative to project root
pub const GAME_DIR: &str = "games";

lazy_static! {
    /// Regex pattern for filename prompt from dfrotz
    pub static ref FILENAME_PROMPT_REGEX: Regex =
        Regex::new(r"Please enter a filename \[.*\]: ").unwrap();

    /// Regex pattern for the game prompt
    pub static ref PROMPT_REGEX: Regex =
        Regex::new(r"(?m)^>\s*$").unwrap();

    /// Regex pattern for score parsing
    pub static ref SCORE_REGEX: Regex =
        Regex::new(r"([0-9]+) ?(?:\(total [points ]*[out ]*of [a mxiuof]*[a posible]*([0-9]+)\))?").unwrap();

    /// Common failure patterns in text adventure games
    pub static ref FAILURE_PATTERNS: Vec<Regex> = vec![
        Regex::new(r"(?i)I don't understand").unwrap(),
        Regex::new(r"(?i)I don't know").unwrap(),
        Regex::new(r"(?i)You can't").unwrap(),
        Regex::new(r"(?i)You're not").unwrap(),
        Regex::new(r"(?i)I can't see").unwrap(),
        Regex::new(r"(?i)That doesn't make sense").unwrap(),
        Regex::new(r"(?i)That's not a verb I recognize").unwrap(),
        Regex::new(r"(?i)What do you want to").unwrap(),
        Regex::new(r"(?i)You don't see").unwrap(),
        Regex::new(r"(?i)There is no").unwrap(),
        Regex::new(r"(?i)I don't see").unwrap(),
        Regex::new(r"(?i)I beg your pardon").unwrap(),
    ];
}

/// Get the default games directory
pub fn game_dir() -> PathBuf {
    PathBuf::from(GAME_DIR)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Process error: {0}")]
    Process(String),

    #[error("Game not found: {0}")]
    GameNotFound(String),

    #[error("Dfrotz not found: {0}")]
    DfrotzNotFound(String),

    #[error("Multiple games found for '{0}': {1:?}")]
    MultipleGamesFound(String, Vec<String>),

    #[error("Game not running")]
    GameNotRunning,

    #[error("Save operation failed")]
    SaveFailed,

    #[error("Restore operation failed")]
    RestoreFailed,
}

pub type Result<T> = std::result::Result<T, Error>;
