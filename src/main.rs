//! TextPlayer CLI - Play text adventure games from the command line

use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Write};
use textplayer::{Formatters, Gamefile, Session};

#[derive(Parser)]
#[command(name = "textplayer")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Game to play (defaults to play command if no subcommand given)
    game: Option<String>,

    /// Output formatter (text, data, json, shell)
    #[arg(short, long, default_value = "shell")]
    formatter: String,

    /// Path to dfrotz executable
    #[arg(long)]
    dfrotz: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Play a text adventure game
    Play {
        /// Game file or name to play
        game: String,

        /// Output formatter (text, data, json, shell)
        #[arg(short, long, default_value = "shell")]
        formatter: String,

        /// Path to dfrotz executable
        #[arg(long)]
        dfrotz: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    // Determine game and formatter based on whether subcommand was used
    let (game_name, formatter_name, dfrotz_path) = match cli.command {
        Some(Commands::Play {
            game,
            formatter,
            dfrotz,
        }) => (game, formatter, dfrotz),
        None => {
            if let Some(game) = cli.game {
                (game, cli.formatter, cli.dfrotz)
            } else {
                eprintln!("Error: Game name required");
                eprintln!("Usage: textplayer [GAME] or textplayer play [GAME]");
                std::process::exit(1);
            }
        }
    };

    if let Err(e) = run_game(&game_name, &formatter_name, dfrotz_path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_game(
    game_name: &str,
    formatter_name: &str,
    dfrotz_path: Option<String>,
) -> textplayer::Result<()> {
    // Find the game file
    let gamefile = Gamefile::from_input(game_name)?;

    if !gamefile.exists() {
        return Err(textplayer::Error::GameNotFound(game_name.to_string()));
    }

    // Create session
    let mut session = Session::new(gamefile, dfrotz_path)?;

    // Get formatter
    let formatter = Formatters::by_name(formatter_name);

    // Setup stdin reader
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut stdout = io::stdout();

    // Run the game loop
    session.run(|result| {
        // Write formatted output
        if let Err(e) = formatter.write_to(result, &mut stdout) {
            eprintln!("Output error: {}", e);
            return None;
        }
        if let Err(e) = stdout.flush() {
            eprintln!("Flush error: {}", e);
            return None;
        }

        // Read next command
        let mut line = String::new();
        match stdin_lock.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => Some(line.trim().to_string()),
            Err(e) => {
                eprintln!("Input error: {}", e);
                None
            }
        }
    })?;

    Ok(())
}
