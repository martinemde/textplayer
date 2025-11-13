//! CommandResult - Encapsulates the result of executing a command

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of operation performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operation {
    Action,
    Start,
    Save,
    Restore,
    Score,
    Quit,
    Error,
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operation::Action => write!(f, "action"),
            Operation::Start => write!(f, "start"),
            Operation::Save => write!(f, "save"),
            Operation::Restore => write!(f, "restore"),
            Operation::Score => write!(f, "score"),
            Operation::Quit => write!(f, "quit"),
            Operation::Error => write!(f, "error"),
        }
    }
}

/// Result of executing a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub input: String,
    pub raw_output: String,
    pub operation: Operation,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(flatten)]
    pub details: HashMap<String, serde_json::Value>,
}

impl CommandResult {
    /// Create a new CommandResult
    pub fn new(
        input: String,
        raw_output: String,
        operation: Operation,
        success: bool,
        message: Option<String>,
    ) -> Self {
        Self {
            input,
            raw_output,
            operation,
            success,
            message,
            details: HashMap::new(),
        }
    }

    /// Create a new CommandResult with details
    pub fn with_details(
        input: String,
        raw_output: String,
        operation: Operation,
        success: bool,
        message: Option<String>,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            input,
            raw_output,
            operation,
            success,
            message,
            details,
        }
    }

    /// Check if this is an action command
    pub fn is_action_command(&self) -> bool {
        self.operation == Operation::Action
    }

    /// Check if this is a system command
    pub fn is_system_command(&self) -> bool {
        !self.is_action_command()
    }

    /// Check if the command succeeded
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Check if the command failed
    pub fn is_failure(&self) -> bool {
        !self.success
    }

    /// Add a detail field
    pub fn add_detail(&mut self, key: String, value: serde_json::Value) {
        self.details.insert(key, value);
    }

    /// Get a detail field
    pub fn get_detail(&self, key: &str) -> Option<&serde_json::Value> {
        self.details.get(key)
    }
}
