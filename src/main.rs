mod platform;

use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::fs::{self, Metadata};
use std::io::{stdout, Result};
use std::path::PathBuf;
use chrono::{DateTime, Local};

// --- Structs ---
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to the directory or file to list
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Use a long listing format
    #[arg(short, long)]
    long: bool,
}

struct FileInfo {
    path: PathBuf,
    metadata: Metadata,
    is_dir: bool,
}

// --- Main Function ---
fn main() -> Result<()> {
    let args = Args::parse();
    let mut files: Vec<FileInfo> = Vec::new();

    // 1. Gather file information
    for entry in fs::read_dir(args.path)? {
        let entry = entry?;
        let path = entry.path();
        if let Ok(metadata) = entry.metadata() {
            let is_dir = metadata.is_dir();
            files.push(FileInfo { path, metadata, is_dir });
        }
    }

    // 2. Sort entries (directories first, then alphabetically)
    files.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.path.file_name().cmp(&b.path.file_name())
        }
    });

    // 3. Display based on flag
    if args.long {
        print_long_view(&files)?;
    } else {
        print_simple_view(&files)?;
    }

    Ok(())
}

// --- Display Functions ---
fn print_simple_view(files: &[FileInfo]) -> Result<()> {
    let mut stdout = stdout();
    for file in files {
        let file_name = file.path.file_name().unwrap_or_default().to_string_lossy();
        
        if file.is_dir {
            execute!(
                stdout,
                SetForegroundColor(Color::Blue),
                Print(" "), // Folder icon
                Print(format!("{}/\n", file_name)),
                ResetColor
            )?;
        } else {
            let icon = get_icon_for_file(&file_name);
            execute!(
                stdout,
                SetForegroundColor(Color::White),
                Print(icon),
                Print(format!("{}\n", file_name)),
                ResetColor
            )?;
        }
    }
    Ok(())
}

fn print_long_view(files: &[FileInfo]) -> Result<()> {
    let mut stdout = stdout();
    for file in files {
        // --- Platform-specific calls ---
        let perms = platform::format_permissions(&file.metadata);
        let owner = platform::get_owner(&file.metadata);
        // -------------------------------

        let size = file.metadata.len();
        let modified_time: DateTime<Local> = file.metadata.modified()?.into();
        let time_str = modified_time.format("%b %e %H:%M").to_string();
        let file_name = file.path.file_name().unwrap_or_default().to_string_lossy();

        let output_line = format!("{perms} {owner:<12} {size:>8} {time_str:<12} ");

        if file.is_dir {
            execute!(
                stdout,
                Print(&output_line),
                SetForegroundColor(Color::Blue),
                Print(" "), // Folder icon
                Print(format!("{}/\n", file_name)),
                ResetColor
            )?;
        } else {
            let icon = get_icon_for_file(&file_name);
            execute!(
                stdout,
                Print(&output_line),
                SetForegroundColor(Color::White),
                Print(icon),
                Print(format!("{}\n", file_name)),
                ResetColor
            )?;
        }
    }
    Ok(())
}

// --- Helper Functions ---
fn get_icon_for_file(file_name: &str) -> &str {
    if file_name.ends_with(".rs") { " " }
    else if file_name.ends_with(".md") { " " }
    else if file_name.ends_with(".toml") || file_name == "Cargo.lock" { " " }
    else if file_name.starts_with(".git") { " " }
    else { " " }
}