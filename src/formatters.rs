//! Formatters - Different output formatters for command results

use crate::{command_result::CommandResult, PROMPT_REGEX};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Write;

lazy_static::lazy_static! {
    static ref SCORE_PATTERN: Regex = Regex::new(r"(?i)Score:\s*(\d+)").unwrap();
    static ref MOVES_PATTERN: Regex = Regex::new(r"(?i)Moves:\s*(\d+)").unwrap();
    static ref TIME_PATTERN: Regex = Regex::new(r"(\d{1,2}:\d{2}\s*(?:AM|PM))").unwrap();
}

/// Formatter trait
pub trait Formatter {
    fn format(&self, result: &CommandResult) -> String;
    fn write_to(&self, result: &CommandResult, writer: &mut dyn Write) -> std::io::Result<()> {
        write!(writer, "{}", self.format(result))
    }
}

/// Shell formatter - interactive presentation with prompts and colors
pub struct ShellFormatter;

impl Formatter for ShellFormatter {
    fn format(&self, result: &CommandResult) -> String {
        if result.is_action_command() {
            self.format_game_output(result)
        } else {
            self.format_system_feedback(result)
        }
    }

    fn write_to(&self, result: &CommandResult, writer: &mut dyn Write) -> std::io::Result<()> {
        if result.is_action_command() {
            let display = result.message.as_ref().unwrap_or(&result.raw_output);
            let (content, prompt) = self.extract_prompt(display);
            write!(writer, "{}", content)?;
            if let Some(p) = prompt {
                let color = if result.success {
                    "\x1b[32m"
                } else {
                    "\x1b[31m"
                };
                write!(writer, "{}{}\x1b[0m", color, p)?;
            }
        } else {
            write!(writer, "{}", self.format(result))?;
        }
        Ok(())
    }
}

impl ShellFormatter {
    fn format_system_feedback(&self, result: &CommandResult) -> String {
        use crate::command_result::Operation;

        if matches!(result.operation, Operation::Start | Operation::Score) {
            return result.raw_output.clone();
        }

        let prefix = if result.success {
            "\x1b[32m✓\x1b[0m"
        } else {
            "\x1b[31m✗\x1b[0m"
        };
        let message = result.message.as_ref().map(|s| s.as_str()).unwrap_or("");
        let mut feedback = format!(
            "{} {}: {}",
            prefix,
            result.operation.to_string().to_uppercase(),
            message
        );

        // Add details if present
        if !result.details.is_empty() {
            let detail_lines: Vec<String> = result
                .details
                .iter()
                .map(|(k, v)| format!("  {}: {}", k, v))
                .collect();
            feedback.push_str(&format!("\n{}", detail_lines.join("\n")));
        }

        feedback
    }

    fn format_game_output(&self, result: &CommandResult) -> String {
        result
            .message
            .as_ref()
            .unwrap_or(&result.raw_output)
            .clone()
    }

    fn extract_prompt<'a>(&self, content: &'a str) -> (String, Option<&'static str>) {
        if PROMPT_REGEX.is_match(content) {
            let cleaned = PROMPT_REGEX.replace_all(content, "").trim_end().to_string();
            (format!("{}\n\n", cleaned), Some("> "))
        } else {
            (content.to_string(), None)
        }
    }
}

/// Text formatter - returns plain text output
pub struct TextFormatter;

impl Formatter for TextFormatter {
    fn format(&self, result: &CommandResult) -> String {
        let content = result.message.as_ref().unwrap_or(&result.raw_output);
        let cleaned = self.remove_prompt(content);
        format!("{}\n\n", cleaned)
    }
}

impl TextFormatter {
    fn remove_prompt(&self, content: &str) -> String {
        PROMPT_REGEX.replace_all(content, "").trim_end().to_string()
    }
}

/// JSON formatter - returns JSON string of structured data
pub struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format(&self, result: &CommandResult) -> String {
        serde_json::to_string(result).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Data formatter - parses game-specific data and returns structured output
pub struct DataFormatter;

impl DataFormatter {
    fn extract_location(&self, text: &str) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        let first_line = lines.first()?.trim();

        if first_line.is_empty() {
            return None;
        }

        // Try different location extraction strategies
        let location = self
            .extract_location_with_stats(first_line)
            .or_else(|| self.extract_standalone_location(first_line))?;

        if self.valid_location(&location) {
            Some(location)
        } else {
            None
        }
    }

    fn extract_location_with_stats(&self, line: &str) -> Option<String> {
        // Handle formats like: " Canyon Bottom                    Score: 0        Moves: 26"
        let parts: Vec<&str> = line.split("   ").collect();
        if parts.len() >= 2 {
            let candidate = parts[0].trim();
            if !candidate.is_empty() {
                return Some(candidate.to_string());
            }
        }
        None
    }

    fn extract_standalone_location(&self, line: &str) -> Option<String> {
        let candidate = line.trim();

        // Only consider it a location if it doesn't contain stats and isn't an error message
        if !candidate.contains("Score")
            && !candidate.contains("Moves")
            && !candidate.contains("AM")
            && !candidate.contains("PM")
            && !candidate.contains(":")
            && !candidate.ends_with('.')
            && !candidate.ends_with('!')
            && !candidate.ends_with('?')
            && candidate.len() < 50
            && !candidate.to_lowercase().contains("response")
        {
            Some(candidate.to_string())
        } else {
            None
        }
    }

    fn extract_score(&self, text: &str) -> Option<i32> {
        SCORE_PATTERN
            .captures(text)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    fn extract_moves(&self, text: &str) -> Option<i32> {
        MOVES_PATTERN
            .captures(text)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse().ok())
    }

    fn extract_time(&self, text: &str) -> Option<String> {
        TIME_PATTERN
            .captures(text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    fn valid_location(&self, location: &str) -> bool {
        !location.is_empty()
            && !location.starts_with("I don't ")
            && !location.starts_with("I can't ")
            && !location.starts_with("What do you ")
            && !location.starts_with("You're ")
            && !location.starts_with("You ")
            && !location.starts_with("That's not ")
            && !location.starts_with("I beg your pardon")
    }

    pub fn parse(&self, result: &CommandResult) -> HashMap<String, Value> {
        let text = &result.raw_output;

        let location = self.extract_location(text);
        let score = self.extract_score(text);
        let moves = self.extract_moves(text);
        let time = self.extract_time(text);
        let has_prompt = PROMPT_REGEX.is_match(text);

        let mut cleaned = text.clone();

        // Remove extracted patterns
        if score.is_some() {
            cleaned = SCORE_PATTERN.replace_all(&cleaned, "").to_string();
        }
        if moves.is_some() {
            cleaned = MOVES_PATTERN.replace_all(&cleaned, "").to_string();
        }
        if let Some(ref t) = time {
            cleaned = cleaned.replace(t, "");
        }
        if has_prompt {
            cleaned = PROMPT_REGEX.replace_all(&cleaned, "").to_string();
        }

        // Final cleanup
        let output = self.final_cleanup(&cleaned);

        let mut data = HashMap::new();
        if let Some(loc) = location {
            data.insert("location".to_string(), Value::String(loc));
        }
        if let Some(s) = score {
            data.insert("score".to_string(), Value::Number(s.into()));
        }
        if let Some(m) = moves {
            data.insert("moves".to_string(), Value::Number(m.into()));
        }
        if let Some(t) = time {
            data.insert("time".to_string(), Value::String(t));
        }
        data.insert("prompt".to_string(), Value::String(">".to_string()));
        data.insert("output".to_string(), Value::String(output));
        data.insert("has_prompt".to_string(), Value::Bool(has_prompt));

        data
    }

    fn final_cleanup(&self, text: &str) -> String {
        // Clean up excessive whitespace but preserve paragraph structure
        let cleaned = text.trim();
        let cleaned = Regex::new(r"\n{3,}").unwrap().replace_all(cleaned, "\n\n");
        let cleaned = Regex::new(r"(?m)^\s+$").unwrap().replace_all(&cleaned, "");
        let cleaned = Regex::new(r"[ \t]+$").unwrap().replace_all(&cleaned, "");
        cleaned.trim().to_string()
    }
}

impl Formatter for DataFormatter {
    fn format(&self, result: &CommandResult) -> String {
        let data = self.parse(result);
        serde_json::to_string_pretty(&data).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Formatters module - factory for creating formatters
pub struct Formatters;

impl Formatters {
    pub fn by_name(name: &str) -> Box<dyn Formatter> {
        match name.to_lowercase().as_str() {
            "shell" => Box::new(ShellFormatter),
            "text" => Box::new(TextFormatter),
            "json" => Box::new(JsonFormatter),
            "data" => Box::new(DataFormatter),
            _ => Box::new(ShellFormatter),
        }
    }
}
