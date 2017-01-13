use std::{io, fmt};
use std::error::Error;
use rustc_serialize::json;
use nickel::status::StatusCode;
use rusted_cypher::error::{GraphError, Neo4jError};
use r2d2::GetTimeout;
use url::ParseError as UrlParseError;

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
    pub enum ValidationError {
        License{
            description("Unsupported license")
        }
        Email{
            description("Invalid email address")
        }
        Url(err: UrlParseError){
            from()
            cause(err)
            description(err.description())
        }
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum ParameterError {
        Id{
            description("Requested ID is invalid")
        }
        Bbox{
            description("Requested bounding box is invalid")

        }
        Categories{
            description("Requested categories are invalid")
        }
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum AppError {
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
        Store(err: StoreError){
            from()
            cause(err)
            description(err.description())
        }
        Io(err: io::Error){
            from()
            cause(err)
            description(err.description())
        }
        Parameter(err: ParameterError){
            from()
            cause(err)
            description(err.description())
        }
        Validation(err: ValidationError){
            from()
            cause(err)
            description(err.description())
        }
    }
}


impl<'a> From<&'a AppError> for StatusCode {
    fn from(err: &AppError) -> StatusCode {
        match *err {
            AppError::Parse(_)        |
            AppError::Io(_)           |
            AppError::Parameter(_)    |
            AppError::Validation(_)   => StatusCode::BadRequest,
            AppError::Encode(_)       => StatusCode::InternalServerError,
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
