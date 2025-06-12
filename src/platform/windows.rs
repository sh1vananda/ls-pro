use std::fs::Metadata;
use std::os::windows::fs::MetadataExt;

const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x10;
const FILE_ATTRIBUTE_READONLY: u32 = 0x1;

pub fn format_permissions(metadata: &Metadata) -> String {
    let attributes = metadata.file_attributes();
    let dir = if (attributes & FILE_ATTRIBUTE_DIRECTORY) != 0 { 'd' } else { '-' };
    let readonly = if (attributes & FILE_ATTRIBUTE_READONLY) != 0 { 'r' } else { '-' };
    let archive = if dir == 'd' { '-' } else { 'a' };
    format!("{}{}{}{}{}", dir, archive, readonly, "-", "-")
}

pub fn get_owner(_metadata: &Metadata) -> String {
    "user".to_string()
}