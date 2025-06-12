mod platform;

use chrono::{DateTime, Local};
use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use ignore::WalkBuilder;
use std::fs::Metadata;
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

    /// Show hidden files and directories and do not respect .gitignore
    #[arg(short, long)]
    all: bool,
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
        print_tree(&args.path, "", 0, args.depth, args.long, args.all)?;
    } else {
        // --- Flat View Logic ---
        let files = get_entries(&args.path, args.all)?;

        if args.long {
            print_long_view(&files)?;
        } else {
            print_simple_view(&files)?;
        }
    }

    Ok(())
}

/// Gathers directory entries, respecting .gitignore and hidden file rules.
fn get_entries(path: &PathBuf, show_hidden: bool) -> Result<Vec<FileInfo>> {
    let mut entries = Vec::new();
    
    // Use the `ignore` crate's WalkBuilder to respect .gitignore and hidden file rules.
    let walk = WalkBuilder::new(path)
        .hidden(!show_hidden)       // If show_hidden is false, respect hidden files.
        .git_ignore(!show_hidden)   // If show_hidden is false, respect .gitignore.
        .max_depth(Some(1))            // We only want to list the direct children.
        .build();

    for result in walk {
        match result {
            Ok(entry) => {
                // The first entry (depth 0) is the root directory itself, skip it.
                if entry.depth() == 0 {
                    continue;
                }
                if let Ok(metadata) = entry.metadata() {
                    let is_dir = metadata.is_dir();
                    entries.push(FileInfo {
                        path: entry.into_path(),
                        metadata,
                        is_dir,
                    });
                }
            }
            Err(e) => eprintln!("Error processing entry: {}", e),
        }
    }
    
    // Sort entries: directories first, then by name
    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir { std::cmp::Ordering::Less } 
        else if !a.is_dir && b.is_dir { std::cmp::Ordering::Greater } 
        else { a.path.file_name().cmp(&b.path.file_name()) }
    });

    Ok(entries)
}


fn print_simple_view(files: &[FileInfo]) -> Result<()> {
    for file in files {
        print_file_line(file, "", false, None)?;
    }
    Ok(())
}

fn print_long_view(files: &[FileInfo]) -> Result<()> {
    for file in files {
        print_file_line(file, "", true, None)?;
    }
    Ok(())
}

fn print_tree(
    path: &PathBuf,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    long_format: bool,
    show_hidden: bool,
) -> Result<()> {
    if depth >= max_depth {
        return Ok(());
    }

    let Ok(entries) = get_entries(path, show_hidden) else { return Ok(()); };
    let mut peekable_entries = entries.iter().peekable();

    while let Some(file) = peekable_entries.next() {
        let is_last = peekable_entries.peek().is_none();
        let connector = if is_last { "└── " } else { "├── " };
        print_file_line(file, &format!("{}{}", prefix, connector), long_format, None)?;

        if file.is_dir {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            print_tree(&file.path, &new_prefix, depth + 1, max_depth, long_format, show_hidden)?;
        }
    }

    Ok(())
}

// NOTE: Added a new `git_status` parameter, which is unused for now. We'll use it in the next phase.
fn print_file_line(file: &FileInfo, prefix: &str, long_format: bool, git_status: Option<char>) -> Result<()> {
    let mut stdout = stdout();
    let file_name = file.path.file_name().unwrap_or_default().to_string_lossy();

    let git_char = git_status.map(|s| s.to_string()).unwrap_or_else(|| " ".to_string());

    let base_info = if long_format {
        let perms = platform::format_permissions(&file.metadata);
        let owner = platform::get_owner(&file.metadata);
        let size = file.metadata.len();
        let modified_time: DateTime<Local> = file.metadata.modified()?.into();
        let time_str = modified_time.format("%b %e %H:%M").to_string();
        format!("{perms} {owner:<12} {size:>8} {time_str:<12} {git_char}")
    } else {
        "".to_string()
    };
    
    let display_name = if long_format {
        format!(" {}", file_name)
    } else {
        format!("{}{}", git_char, file_name)
    };

    if file.is_dir {
        execute!(
            stdout,
            Print(prefix),
            Print(&base_info),
            SetForegroundColor(Color::Blue),
            Print(" "),
            Print(format!("{}/\n", display_name.trim())),
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
            Print(format!("{}\n", display_name.trim())),
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