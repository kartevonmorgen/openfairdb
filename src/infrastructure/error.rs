use business::error::Error as BError;
use business::error::RepoError;
use rusted_cypher::error::GraphError;
use std::error;
use std::io;
use serde_json;
use r2d2;

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
