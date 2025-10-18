use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

fn load_recent() -> Vec<String> {
    let output = Command::new("git")
        .args(["branch", "--sort=-committerdate"])
        .output()
        .expect("Failed to read branches");
    let list = String::from_utf8_lossy(&output.stdout);
    list.lines()
        .map(|s| s.trim().trim_start_matches('*').trim().to_string())
        .filter(|s| !s.is_empty())
        .take(5)
        .collect()
}

fn get_current_branch() -> String {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .expect("Failed to read branches");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn set_raw_mode(enable: bool) {
    if cfg!(unix) {
        let mode = if enable { "raw" } else { "-raw" };
        let _ = Command::new("stty")
            .arg(mode)
            .stdin(Stdio::inherit())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}

fn main() {
    // Load 5 most recent branches
    let mut branches = load_recent();
    let current_branch = get_current_branch();

    if branches.is_empty() {
        println!("No branches found");
        return;
    }

    set_raw_mode(true);
    print!("\x1b[?25l");
    io::stdout().flush().unwrap();

    let mut selected = 0usize;

    loop {
        print!("\x1b[H\x1b[J");
        println!("Select recent branch:\n");
        for (i, b) in branches.iter().enumerate() {
            let mut show_current = " ";
            if *b == current_branch {
                show_current = "*"
            }
            print!("\x1b[G");
            if i == selected {
                println!(" \x1b[44;30m{show_current} {b}\x1b[0m");
            } else {
                println!(" {show_current} {b}")
            }
        }
        io::stdout().flush().unwrap();

        let mut buffer = [0u8; 3];
        if let Ok(n) = io::stdin().read(&mut buffer) {
            if n == 0 {
                continue;
            }
            match buffer[0] {
                27 => {
                    // Escape sequences start with 27
                    if n >= 3 {
                        match buffer[2] {
                            65 => {
                                // Up Arrow
                                if selected > 0 {
                                    selected -= 1;
                                }
                            }
                            66 => {
                                // Down Arrow
                                if selected + 1 < branches.len() {
                                    selected += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                107 | 119 => {
                    // k | w
                    if selected > 0 {
                        selected -= 1;
                    }
                }
                106 | 115 => {
                    // j | s
                    if selected + 1 < branches.len() {
                        selected += 1;
                    }
                }
                10 | 13 | 32 => {
                    // Enter key (\n or \r) or Spacebar
                    break;
                }
                _ => {}
            }
        }
    }

    set_raw_mode(false);
    println!("\x1b[?25h");

    let chosen = branches[selected].clone();
    println!("\nChecking out branch: {chosen}");

    let status = Command::new("git")
        .args(["checkout", &chosen])
        .status()
        .expect("Failed to run git");

    if status.success() {
        branches.retain(|b| b != &chosen);
        branches.insert(0, chosen);
        if branches.len() > 5 {
            branches.truncate(5);
        }
    }
}
