mod git;
mod platform;

use crate::git::GitStatusCache;
use chrono::{DateTime, Local};
use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Stylize},
};
use humansize::{format_size, DECIMAL};
use ignore::WalkBuilder;
use std::io::{stdout, Result};
use std::path::{Path, PathBuf};

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
    #[arg(long, requires = "long")]
    calculate_sizes: bool,
}

// Data structures
struct FileInfo { path: PathBuf, is_dir: bool, display_size: String, modified_time: DateTime<Local> }
struct DisplayInfo {
    permissions: String, owner: String, size: String, time: String, git: String,
    icon: String, name: String, name_color: Color, is_dir: bool,
}
struct TreeNode { info: DisplayInfo, children: Vec<TreeNode> }
#[derive(Default)]
struct ColumnWidths { owner: usize, size: usize }

// --- MAIN LOGIC ---

fn main() -> Result<()> {
    let args = Args::parse();
    let git_cache = if args.git {
        GitStatusCache::new(&args.path).unwrap_or_else(|e| {
            eprintln!("Error creating git cache: {}", e); None
        })
    } else { None };

    if args.tree {
        print_tree_view(&args, &git_cache)?;
    } else {
        let files = get_entries(&args.path, args.all, args.calculate_sizes)?;
        if args.long {
            print_long_view(&files, &git_cache)?;
        } else {
            print_simple_view(&files, &git_cache)?;
        }
    }
    Ok(())
}

// --- DATA GATHERING FUNCTIONS ---

fn calculate_dir_size(path: &Path, show_hidden: bool) -> u64 {
    WalkBuilder::new(path).hidden(!show_hidden).git_ignore(!show_hidden).build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
        .filter_map(|e| e.metadata().ok()).map(|md| md.len()).sum()
}

fn get_entries(path: &Path, show_hidden: bool, calc_sizes: bool) -> Result<Vec<FileInfo>> {
    let mut entries = Vec::new();
    let walk = WalkBuilder::new(path).hidden(!show_hidden).git_ignore(!show_hidden).max_depth(Some(1)).build();
    for result in walk {
        if let Ok(entry) = result {
            if entry.depth() == 0 { continue; }
            if let Ok(metadata) = entry.metadata() {
                let path = entry.into_path();
                let is_dir = metadata.is_dir();
                let display_size = if is_dir {
                    if calc_sizes { format_size(calculate_dir_size(&path, show_hidden), DECIMAL) } 
                    else { "-".to_string() }
                } else { format_size(metadata.len(), DECIMAL) };
                entries.push(FileInfo { path, is_dir, display_size, modified_time: metadata.modified()?.into() });
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

fn build_tree_nodes(path: &Path, depth: usize, max_depth: usize, show_hidden: bool, calc_sizes: bool, git_cache: &Option<GitStatusCache>) -> Result<Vec<TreeNode>> {
    if depth >= max_depth { return Ok(Vec::new()); }
    let entries = get_entries(path, show_hidden, calc_sizes)?;
    let mut nodes = Vec::new();
    for file in entries {
        let metadata = file.path.metadata()?;
        let (git_char, git_color) = git_cache.as_ref().and_then(|cache| file.path.canonicalize().ok().and_then(|p| cache.get(&p))).unwrap_or((' ', Color::Reset));
        let file_name_str = file.path.file_name().unwrap().to_string_lossy();
        let info = DisplayInfo {
            permissions: platform::format_permissions(&metadata), owner: platform::get_owner(&metadata),
            size: file.display_size.clone(), time: file.modified_time.format("%d-%m-%Y %H:%M").to_string(),
            git: format!("{}", git_char.with(git_color)),
            icon: if file.is_dir { " ".to_string() } else { get_icon_for_file(&file_name_str).to_string() },
            name: file_name_str.to_string(),
            name_color: if git_char != ' ' { git_color } else { if file.is_dir { Color::Blue } else { Color::White } },
            is_dir: file.is_dir,
        };
        let children = if file.is_dir { build_tree_nodes(&file.path, depth + 1, max_depth, show_hidden, calc_sizes, git_cache)? } else { Vec::new() };
        nodes.push(TreeNode { info, children });
    }
    Ok(nodes)
}

// --- FLAT VIEW PRINTING ---

fn print_simple_view(files: &[FileInfo], git_cache: &Option<GitStatusCache>) -> Result<()> {
    let mut stdout = stdout();
    for file in files {
        let (git_char, git_color) = git_cache.as_ref().and_then(|cache| file.path.canonicalize().ok().and_then(|p| cache.get(&p))).unwrap_or((' ', Color::Reset));
        let file_name = file.path.file_name().unwrap().to_string_lossy();
        let name_color = if git_char != ' ' { git_color } else { Color::White };
        let dir_color = if git_char != ' ' { git_color } else { Color::Blue };
        let icon = if file.is_dir { " " } else { get_icon_for_file(&file_name) };
        execute!(stdout, Print(format!("{} ", git_char.with(git_color))),
            SetForegroundColor(if file.is_dir { dir_color } else { name_color }), Print(icon),
            Print(format!("{}{}\n", file_name, if file.is_dir { "/" } else { "" })), ResetColor)?;
    }
    Ok(())
}

fn print_long_view(files: &[FileInfo], git_cache: &Option<GitStatusCache>) -> Result<()> {
    if files.is_empty() { return Ok(()); }
    let mut display_infos = Vec::new();
    let mut widths = ColumnWidths::default();
    for file in files {
        let metadata = file.path.metadata()?;
        let (git_char, git_color) = git_cache.as_ref().and_then(|cache| file.path.canonicalize().ok().and_then(|p| cache.get(&p))).unwrap_or((' ', Color::Reset));
        let owner = platform::get_owner(&metadata);
        if owner.len() > widths.owner { widths.owner = owner.len(); }
        let size = &file.display_size;
        if size.len() > widths.size { widths.size = size.len(); }
        let file_name_str = file.path.file_name().unwrap().to_string_lossy();
        display_infos.push(DisplayInfo {
            permissions: platform::format_permissions(&metadata), owner, size: size.clone(), time: file.modified_time.format("%d-%m-%Y %H:%M").to_string(),
            git: format!("{}", git_char.with(git_color)),
            icon: if file.is_dir { " ".to_string() } else { get_icon_for_file(&file_name_str).to_string() },
            name: file_name_str.to_string(),
            name_color: if git_char != ' ' { git_color } else { if file.is_dir { Color::Blue } else { Color::White } },
            is_dir: file.is_dir,
        });
    }

    let mut stdout = stdout();
    execute!(stdout, SetForegroundColor(Color::Green),
        Print(format!("{:<11} ", "Permissions")), Print(format!("{:<width$}  ", "Owner", width = widths.owner)),
        Print(format!("{:>width$} ", "Size", width = widths.size)), Print("Last Modified    "), Print("Git "), Print("Name\n"),
        Print(format!("{:<11} ", "-----------")), Print(format!("{}  ", "─".repeat(widths.owner))),
        Print(format!("{} ", "─".repeat(widths.size))), Print("---------------- "), Print("--- "), Print("----\n"), ResetColor)?;

    for info in display_infos {
        let owner_padded = format!("{:<width$}", info.owner, width = widths.owner);
        let size_padded = format!("{:>width$}", info.size, width = widths.size);
        execute!(stdout, Print(format!("{:<11} ", info.permissions)), Print(format!("{}  ", owner_padded)),
            Print(format!("{} ", size_padded)), Print(format!("{} ", info.time)), Print(format!("{}  ", info.git)),
            SetForegroundColor(info.name_color), Print(&info.icon),
            Print(format!("{}{}\n", info.name, if info.is_dir { "/" } else { "" })), ResetColor)?;
    }
    Ok(())
}

// --- FINAL TREE VIEW FUNCTIONS ---

fn print_tree_view(args: &Args, git_cache: &Option<GitStatusCache>) -> Result<()> {
    let nodes = build_tree_nodes(&args.path, 0, args.depth, args.all, args.calculate_sizes, git_cache)?;
    let mut stdout = stdout();
    println!("{}", args.path.display());

    if args.long {
        let mut widths = ColumnWidths::default();
        calculate_data_widths(&nodes, &mut widths);
        
        // Print Header for long tree view
        execute!(stdout, SetForegroundColor(Color::Green),
            Print(format!("{:<11} ", "Permissions")), Print(format!("{:<width$}  ", "Owner", width = widths.owner)),
            Print(format!("{:>width$} ", "Size", width = widths.size)), Print("Last Modified    "), Print("Git "), Print("Name\n"),
            Print(format!("{:<11} ", "-----------")), Print(format!("{}  ", "─".repeat(widths.owner))),
            Print(format!("{} ", "─".repeat(widths.size))), Print("---------------- "), Print("--- "), Print("----\n"), ResetColor)?;
        
        print_tree_nodes_long(&nodes, "", &widths, &mut stdout)?;
    } else {
        print_tree_nodes_simple(&nodes, "", &mut stdout)?;
    }
    Ok(())
}

fn calculate_data_widths(nodes: &[TreeNode], widths: &mut ColumnWidths) {
    for node in nodes {
        if node.info.owner.len() > widths.owner { widths.owner = node.info.owner.len(); }
        if node.info.size.len() > widths.size { widths.size = node.info.size.len(); }
        calculate_data_widths(&node.children, widths);
    }
}

fn print_tree_nodes_long(nodes: &[TreeNode], prefix: &str, widths: &ColumnWidths, stdout: &mut std::io::Stdout) -> Result<()> {
    let mut peekable_nodes = nodes.iter().peekable();
    while let Some(node) = peekable_nodes.next() {
        let is_last = peekable_nodes.peek().is_none();
        
        let owner_padded = format!("{:<width$}", node.info.owner, width = widths.owner);
        let size_padded = format!("{:>width$}", node.info.size, width = widths.size);
        
        execute!(stdout,
            Print(format!("{:<11} ", node.info.permissions)),
            Print(format!("{}  ", owner_padded)),
            Print(format!("{} ", size_padded)),
            Print(format!("{} ", node.info.time)),
            Print(format!("{}  ", node.info.git)),
        )?;

        let tree_prefix = format!("{}{}", prefix, if is_last { "└── " } else { "├── " });
        execute!(stdout,
            Print(tree_prefix),
            SetForegroundColor(node.info.name_color),
            Print(&node.info.icon),
            Print(format!("{}{}\n", node.info.name, if node.info.is_dir { "/" } else { "" })),
            ResetColor,
        )?;

        if !node.children.is_empty() {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            print_tree_nodes_long(&node.children, &new_prefix, widths, stdout)?;
        }
    }
    Ok(())
}

fn print_tree_nodes_simple(nodes: &[TreeNode], prefix: &str, stdout: &mut std::io::Stdout) -> Result<()> {
    let mut peekable_nodes = nodes.iter().peekable();
    while let Some(node) = peekable_nodes.next() {
        let is_last = peekable_nodes.peek().is_none();
        let tree_prefix = format!("{}{}", prefix, if is_last { "└── " } else { "├── " });

        execute!(stdout,
            Print(tree_prefix),
            Print(format!("{} ", node.info.git)),
            SetForegroundColor(node.info.name_color),
            Print(&node.info.icon),
            Print(format!("{}{}\n", node.info.name, if node.info.is_dir { "/" } else { "" })),
            ResetColor,
        )?;

        if !node.children.is_empty() {
            let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });
            print_tree_nodes_simple(&node.children, &new_prefix, stdout)?;
        }
    }
    Ok(())
}

fn get_icon_for_file(file_name: &str) -> &str {
    if file_name.ends_with(".rs") { " " }
    else if file_name.ends_with(".md") { " " }
    else if file_name.ends_with(".toml") { " " }
    else if file_name == "Cargo.lock" { " " }
    else if file_name.starts_with(".git") { " " }
    else { " " }
}