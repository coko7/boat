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
        fs::create_dir_all(&self.path)?;

        match File::create_new(&self.path) {
            Ok(mut file) => {
                file.write_all(b"[]")?;
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
