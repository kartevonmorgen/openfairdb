use business::error::Error as BError;
use business::error::RepoError;
use adapters::error::Error as AError;
use rusted_cypher::error::GraphError;
use std::error;
use std::io;
use rustc_serialize::json;
use serde_json;

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
        Adapter(err: AError){
            from()
            cause(err)
            description(err.description())
        }
        Encode(err: json::EncoderError){
            from()
            cause(err)
            description(err.description())
        }
        Serialize(err: serde_json::Error){
            from()
            cause(err)
            description(err.description())
        }
        Parse(err: json::ParserError){
            from()
            cause(err)
            description(err.description())
        }
        Other(err: Box<error::Error + Send + Sync>){
            from()
            description(err.description())
            from(err: io::Error) -> (Box::new(err))
        }
    }
}
