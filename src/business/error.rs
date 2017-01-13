use std::io;
use rustc_serialize::json;
use url::ParseError as UrlParseError;

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
    pub enum Error {
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
