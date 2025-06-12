# ls-pro

`ls-pro` is a modern, feature-rich replacement for the classic `ls` command, written in Rust. Inspired by tools like `exa`, it aims to be more informative, visually appealing, and developer-friendly.



## Features

*   **Colorful Output:** Uses colors to distinguish between file types, permissions, and sizes.
*   **Nerd Font Icons:** Provides beautiful icons for different file types for at-a-glance recognition.
*   **Tree View:** A built-in `--tree` (`-t`) flag to display directories recursively.
*   **Git Integration:** Instantly see the Git status of every file in your repository with the `--git` flag (`M` for modified, `A` for new, `?` for untracked, etc.).
*   **Smart Ignoring:** Automatically respects your `.gitignore` files to hide irrelevant files (like `target/` or `node_modules/`). Use `--all` (`-a`) to see everything.
*   **Developer-Focused Layout:** A clean, aligned, and readable layout designed for developers.
*   **Calculated Directory Sizes:** Opt-in to calculate the total size of directories with `--calculate-sizes`.
*   **Fast:** Built with Rust for excellent performance.

## Prerequisites

There are two essential things you need before installing `ls-pro`:

1.  **Rust:** You need the Rust toolchain to install `ls-pro`. If you don't have it, get it from [rustup.rs](https://rustup.rs/).
2.  **A Nerd Font:** This is **critical** for the icons to display correctly.
    *   Go to the [Nerd Fonts website](https://www.nerdfonts.com/font-downloads).
    *   Download and install a font of your choice (e.g., FiraCode Nerd Font, JetBrainsMono Nerd Font).
    *   **Important:** Configure your terminal emulator (Windows Terminal, iTerm2, Kitty, etc.) to **use the Nerd Font** you just installed. Otherwise, you will see `□` instead of icons.

## Installation

### Option 1: From Crates.io (Recommended)

Once `ls-pro` is published, you can install it directly with Cargo:
```bash
cargo install ls-pro
```

### Option 2: From Source

If you want to build from the source code:
```bash
# 1. Clone the repository
git clone https://github.com/sh1vananda/ls-pro
cd ls-pro

# 2. Build and install the binary
cargo install --path .
```
This will compile `ls-pro` and place the executable in your Cargo bin path so you can run it from anywhere.

## Usage

You can use `ls-pro` just like you would use `ls`.

### Options

Here are the available flags. You can see them by running `ls-pro --help`.

```
A modern ls / exa clone with Git integration and icons.

Usage: ls-pro [OPTIONS] [PATH]

Arguments:
  [PATH]  The path to the directory or file to list [default: .]

Options:
  -l, --long               Use a long listing format
  -t, --tree               List files in a tree-like format
  -a, --all                Show hidden files and directories and do not respect .gitignore
      --git                Show git status for each file (if in a repository)
      --calculate-sizes    Recursively calculate and display the total size of directories
      --depth <DEPTH>      Set the maximum depth for the tree view [default: 18446744073709551615]
  -h, --help               Print help
  -V, --version            Print version
```

### Examples

**1. A simple listing with icons and Git status:**
```bash
ls-pro --git
```

**2. A long, detailed listing:**
```bash
ls-pro -l
```


**3. The ultimate tree view:**
This shows the file tree with full details, Git status, and calculated directory sizes.
```bash
ls-pro --tree --long --git --calculate-sizes
```
