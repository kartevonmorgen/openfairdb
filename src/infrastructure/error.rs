use business::error::Error as BError;
use rusted_cypher::error::{GraphError, Neo4jError};
use r2d2::GetTimeout;
use nickel::status::StatusCode;

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
        Save{
            description("Could not save object")
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
    }
}

impl<'a> From<&'a AppError> for StatusCode {
    fn from(err: &AppError) -> StatusCode {
        match *err {
            AppError::Business(ref err)  =>
                match *err {
                    BError::Parse(_)        |
                    BError::Io(_)           |
                    BError::Parameter(_)    |
                    BError::Validation(_)   => StatusCode::BadRequest,
                    BError::Encode(_)       => StatusCode::InternalServerError,
                },
            AppError::Store(ref err)  =>
                match *err {
                    StoreError::NotFound        => StatusCode::NotFound,
                    StoreError::InvalidVersion  |
                    StoreError::InvalidId       => StatusCode::BadRequest,
                    _                           => StatusCode::InternalServerError
            }
        }
    }
}
