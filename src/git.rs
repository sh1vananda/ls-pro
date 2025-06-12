use crossterm::style::Color;
use git2::{Error, Repository, Status};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, Error>;

pub struct GitStatusCache {
    statuses: HashMap<PathBuf, Status>,
}

impl GitStatusCache {
    pub fn new(path: &Path) -> Result<Option<Self>> {
        match Repository::discover(path) {
            Ok(repo) => {
                let mut status_opts = git2::StatusOptions::new();
                status_opts.include_untracked(true).recurse_untracked_dirs(true);
                let statuses = repo.statuses(Some(&mut status_opts))?;
                let mut status_map = HashMap::new();

                if let Some(repo_root) = repo.workdir() {
                    for entry in statuses.iter() {
                        if let Some(path_str) = entry.path() {
                            let full_path = repo_root.join(path_str);
                            if let Ok(canonical_path) = full_path.canonicalize() {
                                status_map.insert(canonical_path, entry.status());
                            } else {
                                status_map.insert(full_path, entry.status());
                            }
                        }
                    }
                }
                Ok(Some(Self { statuses: status_map }))
            }
            Err(_) => Ok(None),
        }
    }

    pub fn get(&self, path: &Path) -> Option<(char, Color)> {
        self.statuses.get(path).map(Self::status_to_char_color)
    }

    fn status_to_char_color(status: &Status) -> (char, Color) {
        if status.is_index_new() { ('A', Color::Green) }
        else if status.is_index_modified() { ('M', Color::Green) }
        else if status.is_index_deleted() { ('D', Color::Red) }
        else if status.is_index_renamed() { ('R', Color::Green) }
        else if status.is_index_typechange() { ('T', Color::Green) }
        else if status.is_wt_new() { ('?', Color::Cyan) }
        else if status.is_wt_modified() { ('M', Color::Yellow) }
        else if status.is_wt_deleted() { ('D', Color::Red) }
        else if status.is_wt_renamed() { ('R', Color::Yellow) }
        else if status.is_wt_typechange() { ('T', Color::Yellow) }
        else if status.is_ignored() { ('I', Color::DarkGrey) }
        else if status.is_conflicted() { ('C', Color::Red) }
        else { (' ', Color::White) }
    }
}