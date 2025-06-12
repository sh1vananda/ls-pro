// If the target OS is unix, include and expose the `unix` module's contents.
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::*;

// If the target OS is windows, include and expose the `windows` module's contents.
#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;