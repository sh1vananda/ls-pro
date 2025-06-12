use clap::Parser;
use std::path::PathBuf;
use crossterm::{style::{Color, Print, ResetColor, SetForegroundColor}, execute};
use std::io::stdout;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    for entry in fs::read_dir(args.path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        if path.is_dir() {
            // Directory: Print in blue with a folder icon
            execute!(
                stdout(),
                SetForegroundColor(Color::Blue),
                Print(" "), // Nerd Font icon for folder
                Print(file_name),
                ResetColor,
                Print("\n")
            )?;
        } else {
            // File: Print in white with a file icon
            execute!(
                stdout(),
                SetForegroundColor(Color::White),
                Print(" "), // Nerd Font icon for file
                Print(file_name),
                ResetColor,
                Print("\n")
            )?;
        }
    }

    Ok(())
}