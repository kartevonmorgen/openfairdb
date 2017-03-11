use entities::*;
use business::usecase::tests::MockRepo;
use business::db::Repo;
use business::error::RepoError;
use std::{io,result};
use r2d2::{self, Pool};

pub struct MockDb {
    entries: MockRepo<Entry>,
    categories: MockRepo<Category>,
    tags: MockRepo<Tag>,
}

impl MockDb {
    pub fn clear_all(&mut self) {
        self.entries.clear();
        self.categories.clear();
        self.tags.clear();
    }
}

type RepoResult<T> = result::Result<T,RepoError>;

impl Repo<Entry> for MockDb {
    fn get(&self, id: &str) -> RepoResult<Entry> {
        self.entries.get(id)
    }
    fn all(&self) -> RepoResult<Vec<Entry>> {
        self.entries.all()
    }
    fn create(&mut self, e: &Entry) -> RepoResult<()> {
        self.entries.create(e)
    }
    fn update(&mut self, e: &Entry) -> RepoResult<()> {
        self.entries.update(e)
    }
}

impl Repo<Category> for MockDb {
    fn get(&self, id: &str) -> RepoResult<Category> {
        self.categories.get(id)
    }
    fn all(&self) -> RepoResult<Vec<Category>> {
        self.categories.all()
    }
    fn create(&mut self, e: &Category) -> RepoResult<()> {
        self.categories.create(e)
    }
    fn update(&mut self, e: &Category) -> RepoResult<()> {
        self.categories.update(e)
    }
}

impl Repo<Tag> for MockDb {
    fn get(&self, id: &str) -> RepoResult<Tag> {
        self.tags.get(id)
    }
    fn all(&self) -> RepoResult<Vec<Tag>> {
        self.tags.all()
    }
    fn create(&mut self, e: &Tag) -> RepoResult<()> {
        self.tags.create(e)
    }
    fn update(&mut self, e: &Tag) -> RepoResult<()> {
        self.tags.update(e)
    }
}

#[derive(Debug)]
pub struct MockDbConnectionManager;

impl r2d2::ManageConnection for MockDbConnectionManager {
  type Connection = MockDb;
  type Error = io::Error;

  fn connect(&self) -> Result<MockDb, io::Error> {
        Ok(MockDb {
            entries: MockRepo::new(),
            categories: MockRepo::new(),
            tags: MockRepo::new(),
        })
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
