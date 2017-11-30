use business::error::Error as BError;
use business::error::RepoError;
use std::error;
use std::io;
use serde_json;
use r2d2;

#[cfg(feature = "neo4j")]
use rusted_cypher::error::GraphError;

#[cfg(feature = "sqlite")]
use diesel::result::Error as DieselError;

#[cfg(feature = "neo4j")]
impl From<GraphError> for RepoError {
    fn from(err: GraphError) -> RepoError {
        RepoError::Other(Box::new(err))
    }
}

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
        R2d2(err: r2d2::GetTimeout){
            from()
        }
        Toml(err: ::toml::de::Error){
            from()
        }
    }
}
