use super::error::RepoError;
use std::result;
use entities::*;

type Result<T> = result::Result<T, RepoError>;

pub trait Repo<T> {
    fn get(&self, &str) -> Result<T>;
    fn all(&self) -> Result<Vec<T>>;
    fn create(&mut self, &T) -> Result<()>;
    fn update(&mut self, &T) -> Result<()>;
}

pub trait Db {

   fn create_entry(&mut self, &Entry) -> Result<()>;

   fn get_entry(&self, &str) -> Result<Entry>;

   fn all_entries(&self) -> Result<Vec<Entry>>;
   fn all_categories(&self) -> Result<Vec<Category>>;

   fn update_entry(&mut self, &Entry) -> Result<()>;

}
