mod platform;

use chrono::{DateTime, Local};
use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use std::fs::{self, Metadata};
use std::io::{stdout, Result};
use std::path::PathBuf;

/// A modern ls / exa clone with Git integration and icons.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to the directory or file to list
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Use a long listing format
    #[arg(short, long)]
    long: bool,

    /// List files in a tree-like format
    #[arg(short, long)]
    tree: bool,

    /// Set the maximum depth for the tree view
    #[arg(long, default_value_t = usize::MAX, requires = "tree")]
    depth: usize,
}

struct FileInfo {
    path: PathBuf,
    metadata: Metadata,
    is_dir: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.tree {
        // --- Tree View Logic ---
        println!("{}", args.path.display());
        print_tree(&args.path, "", 0, args.depth, args.long)?;
    } else {
        // --- Flat View Logic ---
        let mut files: Vec<FileInfo> = Vec::new();
        for entry in fs::read_dir(&args.path)? {
            let entry = entry?;
            if let Ok(metadata) = entry.metadata() {
                let is_dir = metadata.is_dir();
                files.push(FileInfo {
                    path: entry.path(),
                    metadata,
                    is_dir,
                });
            }
        }

        files.sort_by(|a, b| {
            if a.is_dir && !b.is_dir {
                std::cmp::Ordering::Less
            } else if !a.is_dir && b.is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.path.file_name().cmp(&b.path.file_name())
            }
        });

        if args.long {
            print_long_view(&files)?;
        } else {
            print_simple_view(&files)?;
        }
    }

    Ok(())
}

fn print_simple_view(files: &[FileInfo]) -> Result<()> {
    for file in files {
        print_file_line(file, "", false)?;
    }
    Ok(())
}

fn print_long_view(files: &[FileInfo]) -> Result<()> {
    for file in files {
        print_file_line(file, "", true)?;
    }
    Ok(())
}

fn print_tree(
    path: &PathBuf,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    long_format: bool,
) -> Result<()> {
    if depth >= max_depth {
        return Ok(());
    }

    let Ok(entries_iter) = fs::read_dir(path) else { return Ok(()); };

    let mut entries = entries_iter
        .filter_map(|res| res.ok())
        .filter_map(|entry| {
            if let Ok(metadata) = entry.metadata() {
                let is_dir = metadata.is_dir();
                Some(FileInfo {
                    path: entry.path(),
                    metadata,
                    is_dir,
                })
            } else {
                None
            }
        })
        .collect::<Vec<FileInfo>>();

    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.path.file_name().cmp(&b.path.file_name())
        }
    });

    let mut peekable_entries = entries.iter().peekable();

    while let Some(file) = peekable_entries.next() {
        let is_last = peekable_entries.peek().is_none();
        let connector = if is_last { "└── " } else { "├── " };
        print_file_line(file, &format!("{}{}", prefix, connector), long_format)?;

        if file.is_dir {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            print_tree(&file.path, &new_prefix, depth + 1, max_depth, long_format)?;
        }
    }

    Ok(())
}

fn print_file_line(file: &FileInfo, prefix: &str, long_format: bool) -> Result<()> {
    let mut stdout = stdout();
    let file_name = file.path.file_name().unwrap_or_default().to_string_lossy();

    let base_info = if long_format {
        // Call the platform-agnostic functions
        let perms = platform::format_permissions(&file.metadata);
        let owner = platform::get_owner(&file.metadata);
        let size = file.metadata.len();
        let modified_time: DateTime<Local> = file.metadata.modified()?.into();
        let time_str = modified_time.format("%b %e %H:%M").to_string();
        format!("{perms} {owner:<12} {size:>8} {time_str:<12} ")
    } else {
        "".to_string()
    };

    if file.is_dir {
        execute!(
            stdout,
            Print(prefix),
            Print(&base_info),
            SetForegroundColor(Color::Blue),
            Print(" "),
            Print(format!("{}/\n", file_name)),
            ResetColor
        )
    } else {
        let icon = get_icon_for_file(&file_name);
        execute!(
            stdout,
            Print(prefix),
            Print(&base_info),
            SetForegroundColor(Color::White),
            Print(icon),
            Print(format!("{}\n", file_name)),
            ResetColor
        )
    }
}

fn get_icon_for_file(file_name: &str) -> &str {
    if file_name.ends_with(".rs") { " " }      
    else if file_name.ends_with(".md") { " " }      
    else if file_name.ends_with(".toml") { " " } 
    else if file_name == "Cargo.lock" { " " }       
    else if file_name.starts_with(".git") { " " } 
    else { " " }                                 
}