use std::fs::Metadata;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use users::{Users, UsersCache};

lazy_static::lazy_static! {
    static ref USERS_CACHE: UsersCache = UsersCache::new();
}

pub fn format_permissions(metadata: &Metadata) -> String {
    let mode = metadata.permissions().mode();
    format!(
        "{}{}{}{}{}{}{}{}{}{}",
        if metadata.is_dir() { 'd' } else { '-' },
        if mode & 0o400 != 0 { 'r' } else { '-' },
        if mode & 0o200 != 0 { 'w' } else { '-' },
        if mode & 0o100 != 0 { 'x' } else { '-' },
        if mode & 0o040 != 0 { 'r' } else { '-' },
        if mode & 0o020 != 0 { 'w' } else { '-' },
        if mode & 0o010 != 0 { 'x' } else { '-' },
        if mode & 0o004 != 0 { 'r' } else { '-' },
        if mode & 0o002 != 0 { 'w' } else { '-' },
        if mode & 0o001 != 0 { 'x' } else { '-' },
    )
}

pub fn get_owner(metadata: &Metadata) -> String {
    let user = USERS_CACHE.get_user_by_uid(metadata.uid());
    let group = USERS_CACHE.get_group_by_gid(metadata.gid());
    let user_name = user.map_or_else(|| metadata.uid().to_string(), |u| u.name().to_string_lossy().into_owned());
    let group_name = group.map_or_else(|| metadata.gid().to_string(), |g| g.name().to_string_lossy().into_owned());
    format!("{} {}", user_name, group_name)
}