//! Dfrotz - Direct interface to dfrotz interpreter

use crate::{Error, Result};
use regex::Regex;
use std::io::{BufReader, BufWriter, Read, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const TIMEOUT_SECS: u64 = 1;
const CHUNK_SIZE: usize = 1024;
const COMMAND_DELAY_MS: u64 = 100;
const SYSTEM_PATH: &str = "dfrotz";

/// Direct interface to the dfrotz (dumb frotz) interpreter
pub struct Dfrotz {
    game_path: String,
    dfrotz_path: String,
    timeout: Duration,
    command_delay: Duration,
    child: Option<Child>,
    stdin: Option<BufWriter<ChildStdin>>,
    stdout_reader: Option<Arc<Mutex<BufReader<ChildStdout>>>>,
}

impl Dfrotz {
    /// Create a new Dfrotz instance
    pub fn new(game_path: String, dfrotz_path: Option<String>) -> Result<Self> {
        let dfrotz = dfrotz_path.unwrap_or_else(|| {
            std::env::var("DFROTZ_PATH").unwrap_or_else(|_| SYSTEM_PATH.to_string())
        });

        if !Self::is_executable(&dfrotz) {
            return Err(Error::DfrotzNotFound(dfrotz));
        }

        Ok(Self {
            game_path,
            dfrotz_path: dfrotz,
            timeout: Duration::from_secs(TIMEOUT_SECS),
            command_delay: Duration::from_millis(COMMAND_DELAY_MS),
            child: None,
            stdin: None,
            stdout_reader: None,
        })
    }

    /// Check if a path is executable
    fn is_executable(path: &str) -> bool {
        // Try to execute 'which' command to check if the executable exists
        Command::new("which")
            .arg(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Start the dfrotz process
    pub fn start(&mut self) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }

        let mut child = Command::new(&self.dfrotz_path)
            .arg(&self.game_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let stdin = BufWriter::new(child.stdin.take().unwrap());
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let stdout_reader = Arc::new(Mutex::new(stdout));

        self.child = Some(child);
        self.stdin = Some(stdin);
        self.stdout_reader = Some(stdout_reader);

        Ok(())
    }

    /// Write a command to the game
    ///
    /// Automatically sleeps for COMMAND_DELAY, keeping callers simple.
    /// It takes time for every command to return output.
    pub fn write(&mut self, cmd: &str) -> Result<()> {
        if !self.is_running() {
            return Err(Error::GameNotRunning);
        }

        if let Some(ref mut stdin) = self.stdin {
            writeln!(stdin, "{}", cmd)?;
            stdin.flush()?;
            thread::sleep(self.command_delay);
            Ok(())
        } else {
            Err(Error::GameNotRunning)
        }
    }

    /// Read all available output
    pub fn read_all(&self) -> Result<String> {
        self.read_until(None)
    }

    /// Read until a pattern is matched or timeout occurs
    pub fn read_until(&self, pattern: Option<&Regex>) -> Result<String> {
        if !self.is_running() {
            return Ok(String::new());
        }

        let stdout_reader = self.stdout_reader.as_ref().unwrap().clone();
        let timeout = self.timeout;
        let pattern_clone = pattern.map(|p| p.clone());

        // Spawn a thread to read with timeout
        let handle = thread::spawn(move || -> String {
            let mut output = String::new();
            let mut buffer = [0u8; CHUNK_SIZE];
            let start = std::time::Instant::now();

            loop {
                if start.elapsed() >= timeout {
                    break;
                }

                // Try to get lock with short timeout
                if let Ok(mut reader) = stdout_reader.try_lock() {
                    // Set a very short read timeout
                    match reader.get_mut().read(&mut buffer) {
                        Ok(0) => {
                            // EOF reached
                            thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                        Ok(n) => {
                            if let Ok(chunk) = std::str::from_utf8(&buffer[..n]) {
                                output.push_str(chunk);

                                // Check if pattern matched
                                if let Some(ref pattern) = pattern_clone {
                                    if pattern.is_match(&output) {
                                        break;
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            // Would block or error, wait a bit
                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                } else {
                    thread::sleep(Duration::from_millis(10));
                }
            }
            output
        });

        Ok(handle.join().unwrap_or_default())
    }

    /// Check if the dfrotz process is running
    pub fn is_running(&self) -> bool {
        if let Some(ref child) = self.child {
            // Try to check if process is still alive without blocking
            child.id() > 0
        } else {
            false
        }
    }

    /// Terminate the dfrotz process
    pub fn terminate(&mut self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        // Close stdin/stdout first
        self.stdin = None;
        self.stdout_reader = None;

        // Kill the process
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }

        self.child = None;
        Ok(())
    }
}

impl Drop for Dfrotz {
    fn drop(&mut self) {
        let _ = self.terminate();
    }
}
