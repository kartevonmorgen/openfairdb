use business::error::Error as BError;
use rusted_cypher::error::{GraphError, Neo4jError};
use r2d2::GetTimeout;
use std::error;
use std::io;
use rustc_serialize::json;
use adapters::error::ValidationError;

quick_error!{
    #[derive(Debug)]
    pub enum StoreError {
        NotFound{
            description("Could not find object")
        }
        InvalidVersion{
            description("The version property is invalid")
            }
        InvalidId{
            description("The ID is not valid")
            }
        Graph(err: GraphError){
            from()
            cause(err)
            description(err.description())
        }
        Neo(err: Neo4jError){
            from()
            description(&err.message)
        }
        Pool(err: GetTimeout){
            from()
            cause(err)
            description(err.description())
        }
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
        Store(err: StoreError){
            from()
            cause(err)
            description(err.description())
        }
        Encode(err: json::EncoderError){
            from()
            cause(err)
            description(err.description())
        }
        Parse(err: json::ParserError){
            from()
            cause(err)
            description(err.description())
        }
        Validation(err: ValidationError){
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
