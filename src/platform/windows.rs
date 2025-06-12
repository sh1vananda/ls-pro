use std::fs::Metadata;

pub fn format_permissions(_metadata: &Metadata) -> String {
    // Windows permissions are not easily represented in rwxrwxrwx format.
    "----------".to_string()
}

pub fn get_owner(_metadata: &Metadata) -> String {
    // Getting file owner on Windows is more involved.
    "user group".to_string()
}