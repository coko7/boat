use anyhow::Result;
use log::debug;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{csv_io, repository::Repository};

#[derive(Debug)]
pub struct FileRepository {
    path: PathBuf,
}

impl FileRepository {
    pub fn new(path: &Path) -> Self {
        FileRepository {
            path: path.to_path_buf(),
        }
    }
}

impl<T: for<'de> serde::Deserialize<'de> + serde::Serialize + std::fmt::Debug> Repository<T>
    for FileRepository
{
    fn initialize(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        match File::create_new(&self.path) {
            Ok(mut file) => {
                file.flush()?;
                debug!("created new file repository: {}", self.path.display());
            }
            Err(_) => debug!("file repository already exists: {}", self.path.display()),
        }

        Ok(())
    }

    fn load_all(&self) -> Result<Vec<T>> {
        let raw_entries = fs::read_to_string(&self.path)?;
        let entries = csv_io::deserialize::<T>(&raw_entries)?;
        Ok(entries)
    }

    fn save_all(&self, data: &[T]) -> Result<()> {
        let mut file = File::create(&self.path)?;

        let csv_str = csv_io::serialize(data)?;
        debug!("converted data to csv: {data:?}");

        file.write_all(csv_str.as_bytes())?;
        debug!("wrote csv to file: {}", self.path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::repository::Repository;
    use serde::{Deserialize, Serialize};
    use std::fs;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Dummy {
        id: u32,
        name: String,
    }

    fn example_entries() -> Vec<Dummy> {
        vec![
            Dummy {
                id: 1,
                name: "A".into(),
            },
            Dummy {
                id: 2,
                name: "B".into(),
            },
        ]
    }

    #[test]
    fn test_initialize_creates_empty_file() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file_path = dir.path().join("repo.csv");

        let repo = FileRepository::new(&file_path);
        <FileRepository as Repository<Dummy>>::initialize(&repo)?;

        let content = fs::read_to_string(&file_path)?;
        assert!(content.trim().is_empty());
        Ok(())
    }

    #[test]
    fn test_save_and_load_roundtrip() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file_path = dir.path().join("repo.csv");

        let repo = FileRepository::new(&file_path);
        <FileRepository as Repository<Dummy>>::initialize(&repo)?;

        let entries = example_entries();
        repo.save_all(&entries)?;

        let loaded = repo.load_all()?;
        assert_eq!(entries, loaded);
        Ok(())
    }

    #[test]
    fn test_load_all_empty_returns_empty_vec() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let file_path = dir.path().join("repo.csv");

        let repo = FileRepository::new(&file_path);
        <FileRepository as Repository<Dummy>>::initialize(&repo)?;

        let loaded: Vec<Dummy> = repo.load_all()?;
        assert!(loaded.is_empty());
        Ok(())
    }
}
