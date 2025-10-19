use std::error::Error;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

const MAX_BRANCHES: usize = 200;
const NO_OF_VISIBLE_BRANCHES: usize = 5;

/// Load up to MAX_BRANCHES most recently committed branches.
/// Returns an error if the git command fails.
fn load_recent() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["branch", "--sort=-committerdate"])
        .output()?;
    if !output.status.success() {
        return Err(format!("git branch failed: {}", output.status).into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let branches: Vec<String> = stdout
        .lines()
        .map(|s| {
            // branch lines will be like "* main" or "  feature"
            s.trim().trim_start_matches('*').trim().to_string()
        })
        .filter(|s| !s.is_empty())
        .take(MAX_BRANCHES)
        .collect();

    Ok(branches)
}

/// Get the current branch name (git branch --show-current).
fn get_current_branch() -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()?;
    if !output.status.success() {
        return Err(format!("git show-current failed: {}", output.status).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// RAII guard that enables raw mode while alive and restores terminal state on Drop.
/// Uses `stty` on unix. On non-unix this is a no-op.
struct RawModeGuard {
    enabled: bool,
}

impl RawModeGuard {
    fn new() -> Self {
        let mut enabled = false;
        if cfg!(unix) {
            // Enable raw mode and disable echo for cleaner key handling.
            let _ = Command::new("stty")
                .arg("raw")
                .arg("-echo")
                .stdin(Stdio::inherit())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            enabled = true;
        }
        RawModeGuard { enabled }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.enabled && cfg!(unix) {
            // Restore canonical mode and re-enable echo.
            let _ = Command::new("stty")
                .arg("-raw")
                .arg("echo")
                .stdin(Stdio::inherit())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }
}

/// Application state and logic.
struct App {
    branches: Vec<String>,
    current_branch: String,
    selected: usize,
    offset: usize,
}

impl App {
    fn new(branches: Vec<String>, current_branch: String) -> Self {
        App {
            branches,
            current_branch,
            offset: 0,
            selected: 0,
        }
    }

    fn render(&self) -> io::Result<()> {
        // Clear screen and render menu
        print!("\x1b[H\x1b[J");
        println!("Select recent branch:");
        print!("\x1b[G");
        if self.offset > 0 {
            println!("  \x1b[47;30m(less)\x1b[0m")
        } else {
            println!("  \x1b[30m(less)\x1b[0m")
        }
        for (i, b) in self.branches[self.offset..(self.offset + NO_OF_VISIBLE_BRANCHES)]
            .iter()
            .enumerate()
        {
            print!("\x1b[G");
            let current_mark = if b == &self.current_branch { "*" } else { " " };
            if i == self.selected - self.offset {
                // Highlight selection: blue background, black text
                println!(" \x1b[44;30m{current_mark} {b}\x1b[0m");
            } else {
                println!(" {current_mark} {b}");
            }
        }
        print!("\x1b[G");
        if self.offset + NO_OF_VISIBLE_BRANCHES < self.branches.len() {
            println!("  \x1b[47;30m(more)\x1b[0m")
        } else {
            println!("  \x1b[30m(more)\x1b[0m")
        }
        io::stdout().flush()
    }

    fn handle_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
        if self.offset > self.selected {
            self.offset -= 1;
        }
    }

    fn handle_down(&mut self) {
        if self.selected + 1 < self.branches.len() {
            self.selected += 1;
        }
        if self.offset + NO_OF_VISIBLE_BRANCHES - 1 < self.selected {
            self.offset += 1;
        }
    }

    /// Read a single key (or escape sequence) and update selected index accordingly.
    /// Returns true when user confirms selection (Enter/Space).
        fn handle_input(&mut self) -> io::Result<Option<bool>> {
        // Buffer to accommodate escape sequences (e.g. "\x1b[<A>")
        let mut buffer = [0u8; 3];
        let n = io::stdin().read(&mut buffer)?;
        if n == 0 {
            return Ok(None);
        }

        match buffer[0] {
            27 => {
                // ESC. Try to read up to two more bytes (arrow sequences). If no more bytes arrive quickly,
                // read will block - but arrow keys send bytes immediately so this works in practice.
                if n >= 3 {
                    match buffer[2] {
                        65 => self.handle_up(),   // Up Arrow
                        66 => self.handle_down(), // Down Arrow
                        _ => {}
                    }
                    return Ok(None)
                } else {
                    // Single ESC press -> treat as cancel
                    return Ok(Some(false))
                }
            }
            107 | 119 => {
                // k | w
                self.handle_up();
                return Ok(None)
            }
            106 | 115 => {
                // j | s
                self.handle_down();
                return Ok(None)
            }
            10 | 13 | 32 => {
                // Enter (\n or \r) or Space
                return Ok(Some(true))
            }
            113 | 81 => {
                // q | Q -> quit/cancel
                return Ok(Some(false))
            }
            _ => return Ok(None),
        }

        Ok(Some(false))
    }


    fn checkout_selected(&mut self) -> Result<bool, Box<dyn Error>> {
        let chosen = &self.branches[self.selected];
        println!("\x1b[H\x1b[J");
        println!("\nChecking out branch: {chosen}");
        print!("\x1b[G");

        let status = Command::new("git").args(["checkout", chosen]).status()?;
        if status.success() {
            // Move chosen branch to the front of the list
            let chosen_clone = chosen.clone();
            self.branches.retain(|b| b != &chosen_clone);
            self.branches.insert(0, chosen_clone);
            Ok(true)
        } else {
            Err(format!("git checkout failed: {}", status).into())
        }
    }

        fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Create RAII guard to restore terminal state on panic/exit.
        let _raw_guard = RawModeGuard::new();

        // Hide cursor
        print!("\x1b[?25l");
        io::stdout().flush()?;

        let mut confirmed = false;
        loop {
            self.render()?;
            match self.handle_input()? {
                None => continue,
                Some(true) => {
                    confirmed = true;
                    break;
                }
                Some(false) => {
                    confirmed = false;
                    break;
                }
            }
        }

        // Show cursor (RawModeGuard will restore the other state)
        drop(_raw_guard);
        print!("\x1b[?25h");
        io::stdout().flush()?;

        // Perform checkout and update history if successful
        if confirmed {
            match self.checkout_selected() {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Ok(())
        }
    }

}

fn main() {
    if let Err(e) = run_app() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_app() -> Result<(), Box<dyn Error>> {
    let branches = load_recent()?;
    if branches.is_empty() {
        println!("No branches found");
        return Ok(());
    }
    let current_branch = get_current_branch().unwrap_or_default();

    let mut app = App::new(branches, current_branch);
    app.run()
}
