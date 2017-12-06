use business::error::Error as BError;
use business::error::RepoError;
use diesel_migrations::RunMigrationsError;
use std::error;
use std::io;
use serde_json;
use r2d2;

#[cfg(feature = "sqlite")]
use diesel::result::Error as DieselError;

impl From<RepoError> for AppError {
    fn from(err: RepoError) -> AppError {
        AppError::Business(BError::Repo(err))
    }
}

#[cfg(feature = "sqlite")]
impl From<DieselError> for RepoError {
    fn from(err: DieselError) -> RepoError {
        RepoError::Other(Box::new(err))
    }
}

#[cfg(feature = "sqlite")]
impl From<RunMigrationsError> for AppError {
    fn from(err: RunMigrationsError) -> AppError {
        AppError::Other(Box::new(err))
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum AppError {
        Business(err: BError){
            from()
            cause(err)
            description(err.description())
        }
        Serialize(err: serde_json::Error){
            from()
            cause(err)
            description(err.description())
        }
        Other(err: Box<error::Error + Send + Sync>){
            from()
            description(err.description())
            from(err: io::Error) -> (Box::new(err))
        }
        R2d2(err: r2d2::Error){
            from()
        }
        Toml(err: ::toml::de::Error){
            from()
        }
    }
}
