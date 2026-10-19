use crate::{Error, Result};
use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
pub(crate) struct Journal(File);
impl Journal {
    pub fn begin(directory: &Path, digest: &str) -> Result<Self> {
        zix_cli::files::parents(&directory.join("receipt"))?;
        let meta = std::fs::symlink_metadata(directory).map_err(|_| Error("unsafe_journal"))?;
        let own = std::fs::metadata("/proc/self")
            .map_err(|_| Error("unsafe_journal"))?
            .uid();
        if !meta.is_dir()
            || meta.file_type().is_symlink()
            || meta.uid() != own
            || meta.mode() & 0o077 != 0
            || !zix_cli::files::valid_digest(digest)
        {
            return Err(Error("unsafe_journal"));
        }
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(directory.join(format!("{digest}.json")))
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    Error("replay_forbidden")
                } else {
                    Error("journal_unavailable")
                }
            })?;
        let mut journal = Self(file);
        journal.update(&serde_json::json!({"state":"write_starting_do_not_replay"}))?;
        File::open(directory)
            .and_then(|d| d.sync_all())
            .map_err(|_| Error("journal_unavailable"))?;
        Ok(journal)
    }
    pub fn update(&mut self, value: &serde_json::Value) -> Result<()> {
        let bytes = serde_json::to_vec(value).map_err(|_| Error("journal_unavailable"))?;
        self.0
            .set_len(0)
            .and_then(|_| self.0.seek(SeekFrom::Start(0)))
            .and_then(|_| self.0.write_all(&bytes))
            .and_then(|_| self.0.sync_all())
            .map_err(|_| Error("unknown_result"))
    }
}
