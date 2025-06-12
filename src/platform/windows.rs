use std::fs::Metadata;

pub fn format_permissions(_metadata: &Metadata) -> String {
    // Windows permissions are more complex (ACLs) and not easily
    // represented in the rwxrwxrwx format. We'll return a placeholder.
    "----------".to_string()
}

pub fn get_owner(_metadata: &Metadata) -> String {
    // Getting file owner on Windows is possible but requires the windows-sys crate
    "user group".to_string()
}