//! Commands - Game command implementations

use crate::{
    command_result::{CommandResult, Operation},
    dfrotz::Dfrotz,
    savefile::Savefile,
    Result, FAILURE_PATTERNS, FILENAME_PROMPT_REGEX, PROMPT_REGEX, SCORE_REGEX,
};
use regex::Regex;
use std::collections::HashMap;

/// Parse save/restore command from input
fn parse_save_restore(input: &str, game_name: Option<&str>) -> Option<Savefile> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let slot = if parts.len() > 1 {
        Some(parts[1].to_string())
    } else {
        None
    };

    Some(Savefile::new(game_name.map(|s| s.to_string()), slot))
}

/// Commands factory
pub struct Commands;

impl Commands {
    /// Create a command from user input
    pub fn create(input: &str, game_name: Option<&str>) -> Box<dyn Command> {
        let trimmed = input.trim().to_lowercase();

        match trimmed.as_str() {
            "score" => Box::new(ScoreCommand),
            "quit" => Box::new(QuitCommand),
            s if s.starts_with("save") => {
                if let Some(savefile) = parse_save_restore(input, game_name) {
                    Box::new(SaveCommand { savefile })
                } else {
                    Box::new(ActionCommand {
                        input: input.to_string(),
                    })
                }
            }
            s if s.starts_with("restore") => {
                if let Some(savefile) = parse_save_restore(input, game_name) {
                    Box::new(RestoreCommand { savefile })
                } else {
                    Box::new(ActionCommand {
                        input: input.to_string(),
                    })
                }
            }
            _ => Box::new(ActionCommand {
                input: input.to_string(),
            }),
        }
    }
}

/// Trait for executable commands
pub trait Command {
    fn execute(&self, game: &mut Dfrotz) -> Result<CommandResult>;
    fn input(&self) -> String;
}

/// Start command - initialize the game
pub struct StartCommand;

impl Command for StartCommand {
    fn execute(&self, game: &mut Dfrotz) -> Result<CommandResult> {
        let raw_output = game.read_until(Some(&PROMPT_REGEX))?;

        Ok(CommandResult::new(
            String::new(),
            raw_output,
            Operation::Start,
            true,
            None,
        ))
    }

    fn input(&self) -> String {
        String::new()
    }
}

/// Action command - generic game actions (look, go north, etc.)
pub struct ActionCommand {
    pub input: String,
}

impl Command for ActionCommand {
    fn execute(&self, game: &mut Dfrotz) -> Result<CommandResult> {
        game.write(&self.input)?;
        let raw_output = game.read_until(Some(&PROMPT_REGEX))?;

        let success = !Self::failure_detected(&raw_output);

        Ok(CommandResult::new(
            self.input.clone(),
            raw_output,
            Operation::Action,
            success,
            None,
        ))
    }

    fn input(&self) -> String {
        self.input.clone()
    }
}

impl ActionCommand {
    fn failure_detected(output: &str) -> bool {
        FAILURE_PATTERNS
            .iter()
            .any(|pattern| pattern.is_match(output))
    }
}

/// Score command
pub struct ScoreCommand;

impl Command for ScoreCommand {
    fn execute(&self, game: &mut Dfrotz) -> Result<CommandResult> {
        game.write("score")?;
        let raw_output = game.read_until(Some(&PROMPT_REGEX))?;

        let mut details = HashMap::new();
        let mut message = None;

        if let Some(captures) = SCORE_REGEX.captures(&raw_output) {
            if let Some(score_match) = captures.get(1) {
                let score: i32 = score_match.as_str().parse().unwrap_or(0);
                details.insert("score".to_string(), serde_json::json!(score));

                if let Some(max_match) = captures.get(2) {
                    let max_score: i32 = max_match.as_str().parse().unwrap_or(0);
                    details.insert("out_of".to_string(), serde_json::json!(max_score));
                    message = Some(format!("Score: {}/{}", score, max_score));
                } else {
                    message = Some(format!("Score: {}", score));
                }
            }
        }

        Ok(CommandResult::with_details(
            "score".to_string(),
            raw_output,
            Operation::Score,
            true,
            message,
            details,
        ))
    }

    fn input(&self) -> String {
        "score".to_string()
    }
}

/// Save command
pub struct SaveCommand {
    pub savefile: Savefile,
}

impl Command for SaveCommand {
    fn execute(&self, game: &mut Dfrotz) -> Result<CommandResult> {
        // Ensure saves directory exists
        std::fs::create_dir_all("saves").ok();

        game.write("save")?;
        game.read_until(Some(&FILENAME_PROMPT_REGEX))?;
        game.write(&self.savefile.filename())?;

        let mut result = game.read_until(Some(
            &Regex::new(r"(?i)Overwrite existing file\? |Ok\.|Failed\.|>").unwrap(),
        ))?;

        if result.contains("Overwrite existing file?") {
            game.write("y")?;
            result.push_str(&game.read_until(Some(&Regex::new(r"(?i)Ok\.|Failed\.|>").unwrap()))?);
        }

        let success = result.contains("Ok.");
        let message = if success {
            Some(format!("[{}] Game saved successfully", self.savefile.slot))
        } else {
            Some("Save operation failed".to_string())
        };

        let mut details = HashMap::new();
        details.insert("slot".to_string(), serde_json::json!(self.savefile.slot));
        details.insert(
            "filename".to_string(),
            serde_json::json!(self.savefile.filename()),
        );

        Ok(CommandResult::with_details(
            "save".to_string(),
            result,
            Operation::Save,
            success,
            message,
            details,
        ))
    }

    fn input(&self) -> String {
        "save".to_string()
    }
}

/// Restore command
pub struct RestoreCommand {
    pub savefile: Savefile,
}

impl Command for RestoreCommand {
    fn execute(&self, game: &mut Dfrotz) -> Result<CommandResult> {
        game.write("restore")?;
        game.read_until(Some(&FILENAME_PROMPT_REGEX))?;
        game.write(&self.savefile.filename())?;

        let result = game.read_until(Some(&Regex::new(r"(?i)Ok\.|Failed\.|>").unwrap()))?;

        let success = result.contains("Ok.");
        let message = if success {
            Some(format!(
                "[{}] Game restored successfully",
                self.savefile.slot
            ))
        } else {
            Some("Restore operation failed".to_string())
        };

        let mut details = HashMap::new();
        details.insert("slot".to_string(), serde_json::json!(self.savefile.slot));
        details.insert(
            "filename".to_string(),
            serde_json::json!(self.savefile.filename()),
        );

        Ok(CommandResult::with_details(
            "restore".to_string(),
            result,
            Operation::Restore,
            success,
            message,
            details,
        ))
    }

    fn input(&self) -> String {
        "restore".to_string()
    }
}

/// Quit command
pub struct QuitCommand;

impl Command for QuitCommand {
    fn execute(&self, game: &mut Dfrotz) -> Result<CommandResult> {
        game.write("quit")?;
        let raw_output = game.read_until(Some(&Regex::new(r"(?i)Are you sure|>").unwrap()))?;

        if raw_output.to_lowercase().contains("are you sure") {
            game.write("y")?;
            // Give it a moment to process
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        game.terminate()?;

        Ok(CommandResult::new(
            "quit".to_string(),
            raw_output,
            Operation::Quit,
            true,
            Some("Game ended".to_string()),
        ))
    }

    fn input(&self) -> String {
        "quit".to_string()
    }
}
