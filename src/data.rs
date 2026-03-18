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
    parser,
};

const DATA_DIR_PATH: &str = "data";
const DEFINITIONS_FILE_NAME: &str = "definitions.csv";
const LOGS_FILE_NAME: &str = "logs.csv";

pub fn initialize() -> Result<()> {
    let dir_path = Path::new(DATA_DIR_PATH);
    fs::create_dir_all(dir_path)?;

    init_data_store(DEFINITIONS_FILE_NAME)?;
    init_data_store(LOGS_FILE_NAME)?;

    info!("successfully initialized data stores");
    Ok(())
}

fn init_data_store(file_name: &str) -> Result<()> {
    let store_path = store_path(file_name);

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

fn store_path(file_name: &str) -> PathBuf {
    let data_dir = Path::new(DATA_DIR_PATH);
    Path::join(data_dir, file_name)
}

pub fn load_all_activity_definitions() -> Result<Vec<ActivityDefinition>> {
    load_all_entries::<ActivityDefinition>(DEFINITIONS_FILE_NAME)
}

pub fn load_all_activity_logs() -> Result<Vec<ActivityLog>> {
    load_all_entries::<ActivityLog>(LOGS_FILE_NAME)
}

fn load_all_entries<T>(file_name: &str) -> Result<Vec<T>>
where
    T: for<'de> Deserialize<'de> + std::fmt::Debug,
{
    let store_path = store_path(file_name);
    let raw_entries = fs::read_to_string(store_path)?;
    let entries = parser::parse_csv::<T>(&raw_entries)?;
    Ok(entries)
}
