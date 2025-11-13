//! Session - Manages game session lifecycle and output formatting

use crate::{
    command_result::{CommandResult, Operation},
    commands::{
        Command, Commands, QuitCommand, RestoreCommand, SaveCommand, ScoreCommand, StartCommand,
    },
    dfrotz::Dfrotz,
    gamefile::Gamefile,
    savefile::Savefile,
    Result,
};

/// Mid-level: Manages game session lifecycle
pub struct Session {
    gamefile: Gamefile,
    game: Dfrotz,
    started: bool,
    start_result: Option<CommandResult>,
}

impl Session {
    /// Create a new game session
    pub fn new(gamefile: Gamefile, dfrotz_path: Option<String>) -> Result<Self> {
        let game = Dfrotz::new(gamefile.full_path()?, dfrotz_path)?;

        Ok(Self {
            gamefile,
            game,
            started: false,
            start_result: None,
        })
    }

    /// Run the game with a closure that processes results
    ///
    /// The closure receives the result and should return the next command.
    /// Return None to exit the game loop.
    pub fn run<F>(&mut self, mut handler: F) -> Result<()>
    where
        F: FnMut(&CommandResult) -> Option<String>,
    {
        let mut result = self.start()?;

        while self.is_running() {
            if let Some(command) = handler(&result) {
                result = self.call(&command)?;
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Start the game
    pub fn start(&mut self) -> Result<CommandResult> {
        if self.started {
            return Ok(self.start_result.as_ref().unwrap().clone());
        }

        self.game.start()?;
        self.started = true;

        let start_command = StartCommand;
        let result = self.execute_command(&start_command)?;
        self.start_result = Some(result.clone());

        Ok(result)
    }

    /// Check if the game is running
    pub fn is_running(&self) -> bool {
        self.started && self.game.is_running()
    }

    /// Execute a command
    ///
    /// We intentionally intercept certain commands for security and convenience:
    /// - save/restore commands are restricted to the saves directory
    /// - quit is intercepted to ensure clean shutdown
    pub fn call(&mut self, cmd: &str) -> Result<CommandResult> {
        let command = Commands::create(cmd, Some(&self.gamefile.name));
        self.execute_command(command.as_ref())
    }

    /// Get the current score
    pub fn score(&mut self) -> Result<CommandResult> {
        let command = ScoreCommand;
        self.execute_command(&command)
    }

    /// Save the game to a slot
    pub fn save(&mut self, slot: Option<String>) -> Result<CommandResult> {
        let savefile = Savefile::new(Some(self.gamefile.name.clone()), slot);
        let command = SaveCommand { savefile };
        self.execute_command(&command)
    }

    /// Restore the game from a slot
    pub fn restore(&mut self, slot: Option<String>) -> Result<CommandResult> {
        let savefile = Savefile::new(Some(self.gamefile.name.clone()), slot);
        let command = RestoreCommand { savefile };
        self.execute_command(&command)
    }

    /// Quit the game
    pub fn quit(&mut self) -> Result<CommandResult> {
        let command = QuitCommand;
        self.execute_command(&command)
    }

    /// Execute a command
    fn execute_command(&mut self, command: &dyn Command) -> Result<CommandResult> {
        if self.is_running() {
            command.execute(&mut self.game)
        } else {
            Ok(CommandResult::new(
                command.input(),
                String::new(),
                Operation::Error,
                false,
                Some("Game not running".to_string()),
            ))
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        // Ensure game is properly terminated
        let _ = self.game.terminate();
    }
}
