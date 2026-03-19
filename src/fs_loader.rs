use anyhow::Result;
use log::{debug, info};
use serde::Deserialize;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    activity::{ActivityDefinition, ActivityLog},
    loader::DataLoader,
    parser,
};

#[derive(Debug)]
pub struct FileSystemLoader {
    data_directory: PathBuf,
}

impl FileSystemLoader {
    const DEFINITIONS_FILE_NAME: &str = "definitions.csv";
    const LOGS_FILE_NAME: &str = "logs.csv";

    pub fn new(directory: &Path) -> Self {
        FileSystemLoader {
            data_directory: directory.to_path_buf(),
        }
    }

    fn init_data_store(&self, file_name: &str) -> Result<()> {
        let store_path = self.get_file_path(file_name);

        match File::create_new(store_path) {
            Ok(mut file) => {
                file.write_all(b"[]")?;
                file.flush()?;
                debug!("created new data store at {:?}", file_name);
            }
            Err(_) => debug!("data store already exists at {:?}", file_name),
        }

        Ok(())
    }

    fn get_file_path(&self, file_name: &str) -> PathBuf {
        Path::join(&self.data_directory, file_name)
    }

    fn load_all_entries<T>(&self, file_name: &str) -> Result<Vec<T>>
    where
        T: for<'de> Deserialize<'de> + std::fmt::Debug,
    {
        let store_path = self.get_file_path(file_name);
        let raw_entries = fs::read_to_string(store_path)?;
        let entries = parser::parse_csv::<T>(&raw_entries)?;
        Ok(entries)
    }
}

impl DataLoader for FileSystemLoader {
    fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.data_directory)?;

        self.init_data_store(Self::DEFINITIONS_FILE_NAME)?;
        self.init_data_store(Self::LOGS_FILE_NAME)?;

        info!("successfully initialized data stores");
        Ok(())
    }

    fn load_all_activity_definitions(&self) -> Result<Vec<ActivityDefinition>> {
        self.load_all_entries::<ActivityDefinition>(Self::DEFINITIONS_FILE_NAME)
    }

    fn load_all_activity_logs(&self) -> Result<Vec<ActivityLog>> {
        self.load_all_entries::<ActivityLog>(Self::LOGS_FILE_NAME)
    }
}
