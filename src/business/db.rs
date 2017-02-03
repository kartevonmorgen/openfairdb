use super::error::RepoError;
use std::result;

type Result<T> = result::Result<T, RepoError>;

pub trait Repo<T> {
    type Id;

    fn get(&self, Self::Id) -> Result<T>;
    fn all(&self) -> Result<Vec<T>>;
    fn create(&mut self, &T) -> Result<()>;
    fn update(&mut self, &T) -> Result<()>;
}
