//! Serialize cooperating imports; this lock never contains credential data.
use crate::{Error, Result};
use std::{
    fs::{File, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::PathBuf,
};
pub(crate) fn acquire() -> Result<File> {
    let path = PathBuf::from(
        std::env::var_os("XDG_RUNTIME_DIR").ok_or(Error("private_runtime_required"))?,
    );
    if !path.is_absolute() {
        return Err(Error("unsafe_runtime"));
    }
    let uid = std::fs::metadata("/proc/self")
        .map_err(|_| Error("unsafe_runtime"))?
        .uid();
    let metadata = std::fs::symlink_metadata(&path).map_err(|_| Error("unsafe_runtime"))?;
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 {
        return Err(Error("unsafe_runtime"));
    }
    let path = path.join("zix-docker-custody-import.lock");
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(&path)
        .map_err(|_| Error("custody_import_locked"))?;
    let metadata = file
        .metadata()
        .map_err(|_| Error("custody_import_locked"))?;
    let link = std::fs::symlink_metadata(&path).map_err(|_| Error("custody_import_locked"))?;
    if !metadata.is_file()
        || metadata.uid() != uid
        || metadata.mode() & 0o077 != 0
        || link.file_type().is_symlink()
        || link.ino() != metadata.ino()
        || link.dev() != metadata.dev()
    {
        return Err(Error("custody_import_locked"));
    }
    file.try_lock()
        .map_err(|_| Error("custody_import_locked"))?;
    Ok(file)
}
