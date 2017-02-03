use std::io;
use std::error;

quick_error!{
    #[derive(Debug)]
    pub enum ParameterError {
        Bbox{
            description("Requested bounding box is invalid")
        }
        License{
            description("Unsupported license")
        }
        Email{
            description("Invalid email address")
        }
        Url{
            description("Invalid URL")
        }
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum RepoError {
        NotFound{
            description("The requested object could not be found")
        }
        Io(err: io::Error) {
            from()
            cause(err)
            description(err.description())
        }
        Other(err: Box<error::Error>){
            description(err.description())
        }
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum Error {
        Parameter(err: ParameterError){
            from()
            cause(err)
            description(err.description())
        }
        Repo(err: RepoError){
            from()
            cause(err)
            description(err.description())
        }
    }
}
