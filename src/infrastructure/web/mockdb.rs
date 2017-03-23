use business::usecase::tests::MockDb;
use business::error::RepoError;
use std::{io,result};
use r2d2::{self, Pool, InitializationError};

type RepoResult<T> = result::Result<T,RepoError>;

#[derive(Debug)]
pub struct MockDbConnectionManager;

impl r2d2::ManageConnection for MockDbConnectionManager {
  type Connection = MockDb;
  type Error = io::Error;

  fn connect(&self) -> Result<MockDb, io::Error> {
        Ok(MockDb::new())
  }

  fn is_valid(&self, _: &mut MockDb) -> Result<(), io::Error> {
      Ok(())
  }

  fn has_broken(&self, _: &mut MockDb) -> bool {
        false
  }
}

pub type ConnectionPool = Pool<MockDbConnectionManager>;

pub fn create_connection_pool() -> result::Result<ConnectionPool, InitializationError> {
    let config = r2d2::Config::builder().pool_size(1).build();
    let manager = MockDbConnectionManager{};
    Pool::new(config, manager)
}
