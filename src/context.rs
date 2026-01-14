use std::fs::File;
use std::{fs, io};
use std::io::Read;
use std::path::PathBuf;
use fs2::FileExt;
use crate::types;

#[derive(Clone)]
pub struct FrogContext {
    pub run_dir_base: PathBuf,
}

impl FrogContext {
    pub fn new(run_dir_base: impl Into<PathBuf>) -> Self {
        Self {
            run_dir_base: run_dir_base.into(),
        }
    }

    pub fn container_run_dir(&self, container_id: &str) -> PathBuf {
        self.run_dir_base.join(container_id)
    }

    pub fn lock_container(&self, container_id: &str) -> io::Result<File> {
        let run_dir = self.container_run_dir(container_id);
        fs::create_dir_all(&run_dir)?;

        let lock_file_path = run_dir.join("lock");
        let lock_file = File::create(lock_file_path)?;
        lock_file.try_lock_exclusive()?;

        Ok(lock_file)
    }

    pub fn state_file_path(&self, container_id: &str) -> PathBuf {
        self.container_run_dir(container_id).join("state.json")
    }

    pub fn write_state(&self, container_id: &str, state: types::ContainerState) -> io::Result<()> {
        let state_file_path = self.state_file_path(container_id);
        let state_file = File::create(state_file_path)?;
        serde_json::to_writer_pretty(&state_file, &state)?;
        state_file.sync_all()?;
        Ok(())
    }

    pub fn read_state(&self, container_id: &str) -> io::Result<types::ContainerState> {
        let path = self.state_file_path(container_id);
        let mut file = File::open(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        Ok(serde_json::from_str(&buffer)?)
    }
}
