use business::usecase::tests::MockDb;
use std::io;
use diesel::r2d2::{ManageConnection, Pool, PoolError};

#[derive(Debug)]
pub struct MockDbConnectionManager;

impl ManageConnection for MockDbConnectionManager {
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

pub fn create_connection_pool(_: &str) -> Result<ConnectionPool, PoolError> {
    let manager = MockDbConnectionManager {};
    Pool::builder().max_size(1).build(manager)
}
