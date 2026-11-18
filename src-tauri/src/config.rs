//! Configuration constants for Pixo et les Dossiers.

/// Wipe mode when cleaning folders before an attempt:
/// - "delete": permanently deletes the wiped contents
/// - "backup": moves wiped contents into %APPDATA%\PixoDossiers\backup\<timestamp>
pub const WIPE_MODE: &str = "delete";

/// Whether levels that clean target Known Folders wipe their entire contents.
pub const LEVEL2_WIPE_WHOLE_FOLDERS: bool = true;
