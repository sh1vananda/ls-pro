mod git;
mod platform;

use crate::git::GitStatusCache;
use chrono::{DateTime, Local};
use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
};
use ignore::WalkBuilder;
use std::fs::Metadata;
use std::io::{stdout, Result};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,
    #[arg(short, long)]
    long: bool,
    #[arg(short, long)]
    tree: bool,
    #[arg(long, default_value_t = usize::MAX, requires = "tree")]
    depth: usize,
    #[arg(short, long)]
    all: bool,
    #[arg(long)]
    git: bool,
}

struct FileInfo {
    path: PathBuf,
    metadata: Metadata,
    is_dir: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let git_cache = if args.git {
        GitStatusCache::new(&args.path).unwrap_or_else(|e| {
            eprintln!("Error creating git cache: {}", e);
            None
        })
    } else {
        None
    };

    if args.tree {
        println!("{}", args.path.display());
        print_tree(&args.path, "", 0, args.depth, args.long, args.all, &git_cache)?;
    } else {
        let files = get_entries(&args.path, args.all)?;
        if args.long {
            print_long_view(&files, &git_cache)?;
        } else {
            print_simple_view(&files, &git_cache)?;
        }
    }

    Ok(())
}

fn get_entries(path: &PathBuf, show_hidden: bool) -> Result<Vec<FileInfo>> {
    let mut entries = Vec::new();
    let walk = WalkBuilder::new(path)
        .hidden(!show_hidden)
        .git_ignore(!show_hidden)
        .max_depth(Some(1))
        .build();

    for result in walk {
        if let Ok(entry) = result {
            if entry.depth() == 0 { continue; }
            if let Ok(metadata) = entry.metadata() {
                entries.push(FileInfo {
                    path: entry.into_path(),
                    is_dir: metadata.is_dir(),
                    metadata,
                });
            }
        }
    }

    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir { std::cmp::Ordering::Less }
        else if !a.is_dir && b.is_dir { std::cmp::Ordering::Greater }
        else { a.path.file_name().cmp(&b.path.file_name()) }
    });

    Ok(entries)
}

fn print_simple_view(files: &[FileInfo], git_cache: &Option<GitStatusCache>) -> Result<()> {
    for file in files {
        let git_status = git_cache
            .as_ref()
            .and_then(|cache| file.path.canonicalize().ok().and_then(|p| cache.get(&p)));
        print_file_line(file, "", false, git_status)?;
    }
    Ok(())
}

fn print_long_view(files: &[FileInfo], git_cache: &Option<GitStatusCache>) -> Result<()> {
    for file in files {
        let git_status = git_cache
            .as_ref()
            .and_then(|cache| file.path.canonicalize().ok().and_then(|p| cache.get(&p)));
        print_file_line(file, "", true, git_status)?;
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
    git_cache: &Option<GitStatusCache>,
) -> Result<()> {
    if depth >= max_depth { return Ok(()); }

    let Ok(entries) = get_entries(path, show_hidden) else { return Ok(()); };
    let mut peekable_entries = entries.iter().peekable();

    while let Some(file) = peekable_entries.next() {
        let is_last = peekable_entries.peek().is_none();
        let connector = if is_last { "└── " } else { "├── " };
        let git_status = git_cache
            .as_ref()
            .and_then(|cache| file.path.canonicalize().ok().and_then(|p| cache.get(&p)));
        print_file_line(file, &format!("{}{}", prefix, connector), long_format, git_status)?;

        if file.is_dir {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            print_tree(&file.path, &new_prefix, depth + 1, max_depth, long_format, show_hidden, git_cache)?;
        }
    }
    Ok(())
}

fn print_file_line(
    file: &FileInfo,
    prefix: &str,
    long_format: bool,
    git_status: Option<(char, Color)>,
) -> Result<()> {
    let mut stdout = stdout();
    let file_name = file.path.file_name().unwrap_or_default().to_string_lossy();
    let (git_char, git_color) = git_status.unwrap_or((' ', Color::Reset));

    let base_info = if long_format {
        let perms = platform::format_permissions(&file.metadata);
        let owner = platform::get_owner(&file.metadata);
        let size_str = if file.is_dir { "-".to_string() } else { file.metadata.len().to_string() };
        let modified_time: DateTime<Local> = file.metadata.modified()?.into();
        let time_str = modified_time.format("%b %e %H:%M").to_string();
        let git_indicator = format!("{} ", git_char.with(git_color));
        format!("{perms} {owner:<12} {size:>8} {time_str:<12} {git_indicator}", size = size_str)
    } else {
        format!("{} ", git_char.with(git_color))
    };

    let name_color = if git_status.is_some() { git_color } else { Color::White };

    if file.is_dir {
        let dir_color = if git_status.is_some() { git_color } else { Color::Blue };
        execute!(
            stdout,
            Print(prefix),
            Print(&base_info),
            SetForegroundColor(dir_color),
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
            SetForegroundColor(name_color),
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