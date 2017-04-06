use std::io;
use std::error;
use pwhash;

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
        UserName{
            description("Invalid username")
        }
        Password{
            description("Invalid password")
        }
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum RepoError {
        NotFound{
            description("The requested object could not be found")
        }
        AlreadyExists{
            description("The object already exists")
        }
        InvalidVersion{
            description("The version of the object is invalid")
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
        Pwhash(err: pwhash::error::Error){
            from()
            cause(err)
            description(err.description())
        }
    }
}
