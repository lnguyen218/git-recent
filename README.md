# git-recent

git-recent is a small terminal utility that lists the most-recently committed Git branches and lets you interactively select one to checkout. It displays a compact menu of recent branches (sorted by committer date) and supports keyboard navigation and selection.

This tool is implemented in Rust and intended to be lightweight and simple.

## Features

- Lists branches sorted by recent commit date (most recent first).
- Interactive terminal UI with:
  - Arrow keys (Up/Down)
  - Vim-style navigation (k/j)
  - Space or Enter to confirm checkout
  - `q`, `Q`, or `Esc` to cancel
- Moves the checked-out branch to the front of the internal list after a successful checkout.
- Minimal dependencies (only `git` and on Unix-like systems `stty` for raw mode).

## Requirements

- Rust toolchain to build (or pre-built binary)
- Git available in PATH
- Unix-like terminal recommended. The code uses `stty` to enable raw mode (no-ops on non-Unix targets, so interactive key handling may be degraded on some platforms).

## Build & Install

To build from source:

1. Clone the repository:
   git clone https://github.com/lnguyen218/git-recent.git
   cd git-recent

2. Build with cargo:
   cargo build --release

3. The binary will be at:
   target/release/git-recent

Optionally install it with Cargo:
   cargo install --path .

(or copy the `target/release/git-recent` binary into a directory on your PATH)

## Usage

Run the program from a Git repository directory:

   git-recent

The program lists the most-recently committed branches (up to a built-in maximum). Use the keys below to navigate and select:

- Up Arrow, k, or w — move selection up
- Down Arrow, j, or s — move selection down
- Enter or Space — checkout the selected branch
- q, Q, or Esc — cancel and exit

When you select a branch, `git checkout <branch>` is executed. On success, the branch is moved to the front of the internal list and the program exits.

If there are no branches found, the program prints `No branches found` and exits.

## Behavior & Configuration

- The implementation reads the output of `git branch --sort=-committerdate` to get branches sorted by committer date.
- Constants in `src/main.rs` control behavior:
  - `MAX_BRANCHES`: maximum number of branches read (defaults to 200)
  - `NO_OF_VISIBLE_BRANCHES`: number of branches shown at once in the UI (defaults to 5)
  To change these behaviors, edit the constants in `src/main.rs` and rebuild.

- Terminal handling:
  - On Unix, `stty raw -echo` is used while the program runs to provide immediate key input handling; `stty -raw echo` is restored on exit (including panic) via an RAII guard.
  - The program prints basic ANSI escape sequences to clear the screen and highlight selection. This assumes a compatible terminal.

## Limitations & Notes

- The UI is intentionally minimal. It is not a full TUI — it uses simple ANSI control sequences and `stty` for raw mode.
- On non-Unix platforms the raw-mode guard is a no-op; interactive input may not behave identically on Windows terminals.
- The application runs `git checkout` directly. Any Git hooks, merge conflicts, or uncommitted changes will behave the same as when running `git checkout` yourself.

## Troubleshooting

- If the program exits with "git branch failed" or "git show-current failed", make sure you're running in a Git repository and `git` is available.
- If the terminal appears garbled after an unexpected exit, run `stty sane` (on Unix) or open a new terminal window.
- If key inputs don't respond as expected on Windows, try running in WSL or another Unix-like environment.

## Contributing

Contributions are welcome. Please open issues for bugs or feature requests. If you plan to send a pull request:

- Fork the repository and create a feature branch.
- Keep changes focused and provide tests where reasonable.
- Ensure the code builds with stable Rust.

## License

See the LICENSE file in this repository for license details (if present).

## Contact

Created by @lnguyen218 — open an issue or PR on the repository if you need help or want to contribute.
