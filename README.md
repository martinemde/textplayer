# textplayer

A Rust library and CLI for running text-based interactive fiction games using the Frotz Z-Machine interpreter. This library provides structured access to classic text adventure games with multiple output formatters for different use cases.

Inspired by [@danielricks/textplayer](https://github.com/danielricks/textplayer) - the original Python implementation.

I have chosen not to distribute the games in the crate. You'll need to clone this repository to use the games directly without the full pathname. This is out of an abundance of caution and respect to the owners. Listing them for download, as is done regularly, may be interpreted differently than distributing them in a package.

I am grateful for the ability to use these games for learning and building. Zork is the game that got me started on MUDs as a kid, which is the reason I'm a programmer now.

## Requirements

textplayer requires [Frotz](http://frotz.sourceforge.net/), a Z-Machine interpreter written by Stefan Jokisch in 1995-1997.

Use Homebrew to install the `frotz` package:

```bash
brew install frotz
```

If you don't have homebrew, download the source code, build and install.

```bash
git clone https://github.com/DavidGriffith/frotz.git
cd frotz
make dumb
make dumb_install # optional, but recommended so that dfrotz is in path
```

The `dfrotz` (dumb frotz) binary must be available in your PATH or you will need to pass the path to the dfrotz executable as an argument to textplayer.

## Installation

### Using Cargo

Add to your `Cargo.toml`:

```toml
[dependencies]
textplayer = "0.1.0"
```

Or install the CLI:

```bash
cargo install textplayer
```

### From Source

Clone the repository and build:

```bash
git clone git@github.com:martinemde/text-player-rs.git
cd text-player-rs
cargo build --release
```

The binary will be available at `target/release/textplayer`.

## Usage

### Command Line

You can use the command line to play games:

```bash
# Play a game (interactive mode)
$ textplayer play zork1

# Or use the shorthand
$ textplayer zork1

# Specify a formatter
$ textplayer play zork1 --formatter json

# Specify custom dfrotz path
$ textplayer play zork1 --dfrotz ~/bin/dfrotz
```

### Library Usage

The point of this library is to allow you to run text based adventure games programmatically.

```rust
use textplayer::{Gamefile, Session};

fn main() -> textplayer::Result<()> {
    // Create a new game session
    let gamefile = Gamefile::from_input("games/zork1.z5")?;
    let mut session = Session::new(gamefile, None)?;

    // Start the game
    let start_result = session.start()?;
    println!("{}", start_result.raw_output);

    // Execute commands
    let response = session.call("go north")?;
    println!("{}", response.raw_output);

    // Get current score
    let score = session.score()?;
    if let Some(score_value) = score.get_detail("score") {
        println!("Score: {}", score_value);
    }

    // Save and restore
    session.save(Some("my_save".to_string()))?;
    session.restore(Some("my_save".to_string()))?;

    // Quit the game
    session.quit()?;

    Ok(())
}
```

### Save and Restore Operations

```rust
use textplayer::{Gamefile, Session};

fn main() -> textplayer::Result<()> {
    let gamefile = Gamefile::from_input("zork1.z5")?;
    let mut session = Session::new(gamefile, None)?;
    session.start()?;

    // Save to default slot (autosave)
    let save_result = session.save(None)?;
    println!("{}", save_result.message.unwrap_or_default());

    // Save to named slot
    session.save(Some("before_dragon".to_string()))?;

    // Restore from default slot
    session.restore(None)?;

    // Restore from named slot
    session.restore(Some("before_dragon".to_string()))?;

    Ok(())
}
```

### Interactive Shell Example

```rust
use std::io::{self, BufRead, Write};
use textplayer::{Gamefile, Session, Formatters};

fn main() -> textplayer::Result<()> {
    let gamefile = Gamefile::from_input("zork1.z5")?;
    let mut session = Session::new(gamefile, None)?;

    let formatter = Formatters::by_name("shell");
    let stdin = io::stdin();
    let mut stdin_lock = stdin.lock();
    let mut stdout = io::stdout();

    session.run(|result| {
        formatter.write_to(result, &mut stdout).ok()?;
        stdout.flush().ok()?;

        let mut line = String::new();
        stdin_lock.read_line(&mut line).ok()?;
        Some(line.trim().to_string())
    })?;

    Ok(())
}
```

### Configuring dfrotz Path

By default, textplayer looks for the `dfrotz` executable in the system PATH. You can specify a custom path:

```rust
use textplayer::{Gamefile, Session};

fn main() -> textplayer::Result<()> {
    let gamefile = Gamefile::from_input("zork1.z5")?;

    // Use local path to compiled dfrotz
    let mut session = Session::new(gamefile.clone(), Some("./frotz/dfrotz".to_string()))?;

    // Use absolute path
    let mut session = Session::new(gamefile, Some("/usr/local/bin/dfrotz".to_string()))?;

    Ok(())
}
```

You can also set the `DFROTZ_PATH` environment variable:

```bash
export DFROTZ_PATH=/usr/local/bin/dfrotz
```

## Development

After checking out the repo, build the project:

```bash
cargo build
```

Run tests:

```bash
cargo test
```

Run the CLI during development:

```bash
cargo run -- play zork1
```

To release a new version, update the version number in `Cargo.toml` and `src/lib.rs`, then:

```bash
cargo publish
```

## Contributing

Bug reports and pull requests are welcome on GitHub at <https://github.com/martinemde/text-player-rs>.

## Game Files

You'll need Z-Machine game files (`.z3`, `.z5`, `.z8` extensions) to play. Many classic interactive fiction games are available from:

- [The Interactive Fiction Archive](https://www.ifarchive.org/)
- [Infocom games](http://www.infocom-if.org/downloads/downloads.html)

## License

The crate is available as open source under the terms of the [MIT License](https://opensource.org/licenses/MIT).

I have included the same games from [@danielricks/textplayer](https://github.com/danielricks/textplayer), assuming that in the last ~10 years that it has not been a problem.

The games are copyright and licensed by their respective owners.

**Please open an issue on the repository or contact me directly if there are any concerns.**

## Credits

This Rust implementation was inspired and influenced by [@danielricks/textplayer](https://github.com/danielricks/textplayer), who wrote a Python interface for Frotz to facilitate training models to automatically play the game.

This is a port of my Ruby version, [text_player](https://github.com/martinemde/text_player).
