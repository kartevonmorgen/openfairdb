use entities::*;
use business::usecase::tests::MockDb;
use business::db::Repo;
use business::error::RepoError;
use std::{io,result};
use r2d2::{self, Pool};

type RepoResult<T> = result::Result<T,RepoError>;

#[derive(Debug)]
pub struct MockDbConnectionManager;

impl r2d2::ManageConnection for MockDbConnectionManager {
  type Connection = MockDb;
  type Error = io::Error;

  fn connect(&self) -> Result<MockDb, io::Error> {
        Ok(MockDb::new())
  }

  fn is_valid(&self, conn: &mut MockDb) -> Result<(), io::Error> {
      Ok(())
  }

  fn has_broken(&self, _: &mut MockDb) -> bool {
        false
  }
}

lazy_static! {
    pub static ref DB_POOL: r2d2::Pool<MockDbConnectionManager> = {
        let config = r2d2::Config::builder().pool_size(1).build();
        let manager = MockDbConnectionManager{};
        Pool::new(config, manager).expect("Failed to create pool.")
    };
}

pub struct DB(r2d2::PooledConnection<MockDbConnectionManager>);

impl DB {
    pub fn conn(&mut self) -> &mut MockDb {
        &mut self.0
    }
}

pub fn db() -> io::Result<DB> {
    match DB_POOL.get() {
        Ok(conn) => Ok(DB(conn)),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
    }
}
