use super::error::RepoError;
use std::result;

type Result<T> = result::Result<T, RepoError>;

pub trait Repo<T> {
    fn get(&self, &str) -> Result<T>;
    fn all(&self) -> Result<Vec<T>>;
    fn create(&mut self, &T) -> Result<()>;
    fn update(&mut self, &T) -> Result<()>;
}
