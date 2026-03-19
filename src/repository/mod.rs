use anyhow::Result;
use std::fmt::Debug;

pub mod file_repository;

pub trait Repository<T>: Debug {
    fn initialize(&self) -> Result<()>;
    fn load_all(&self) -> Result<Vec<T>>;
    fn save_all(&self, data: &[T]) -> Result<()>;
    // fn save_single(&self, data: &T) -> Result<()>;
}
