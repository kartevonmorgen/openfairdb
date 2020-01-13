use pwhash;
use std::io;

quick_error! {
    #[derive(Debug)]
    pub enum ParameterError {
        Title{
            description("The title is invalid")
        }
        Bbox{
            description("Bounding box is invalid")
        }
        License{
            description("Unsupported license")
        }
        Email{
            description("Invalid email address")
        }
        Phone{
            description("Invalid phone nr")
        }
        Url{
            description("Invalid URL")
        }
        Contact{
            description("Invalid contact")
        }
        RegistrationType{
            description("Invalid registration type")
        }
        UserExists{
            description("The user already exists")
        }
        UserDoesNotExist{
            description("The user does not exist")
        }
        Password{
            description("Invalid password")
        }
        EmptyComment{
            description("Empty comment")
        }
        RatingValue{
            description("Rating value out of range")
        }
        RatingContext(context: String){
            description("Invalid rating context")
        }
        Credentials {
            description("Invalid credentials")
        }
        EmailNotConfirmed {
            description("Email not confirmed")
        }
        Forbidden{
            description("This is not allowed")
        }
        Unauthorized{
            description("This is not allowed without auth")
        }
        DateTimeOutOfRange{
            description("The date/time is out of range")
        }
        EndDateBeforeStart{
            description("The end date is before the start")
        }
        OwnedTag{
            description("The tag is owned by an organization")
        }
        CreatorEmail{
            description("Missing the email of the creator")
        }
        InvalidPosition {
            description("Invalid position")
        }
        InvalidLimit{
            description("Invalid limit")
        }
        Role{
            description("Invalid role")
        }
        TokenInvalid{
            description("Token invalid")
        }
        TokenExpired{
            description("Token expired")
        }
        InvalidNonce{
            description("Invalid nonce")
        }
        EmptyIdList{
            description("Missing id list")
        }
    }
}

quick_error! {
    #[derive(Debug)]
    //TODO: rename to GatewayError
    pub enum RepoError {
        NotFound{
            description("The requested object could not be found")
        }
        TooManyFound {
            description("Too many instead of only one requested object has been found")
        }
        #[cfg(test)]
        AlreadyExists {
            description("The object already exists")
        }
        InvalidVersion {
            description("The version of the object is invalid")
        }
        Io(err: io::Error) {
            from()
            cause(err)
            description(err.to_string())
        }
        Other(err: anyhow::Error) {
            description(err.to_string())
        }
    }
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Parameter(err: ParameterError) {
            from()
            cause(err)
            description(err.description())
        }
        ParseInt(err: std::num::ParseIntError) {
            from()
            cause(err)
            description(err.description())
        }
        Repo(err: RepoError) {
            from()
            cause(err)
            description(err.description())
        }
        Pwhash(err: pwhash::error::Error) {
            from()
            cause(err)
            description(err.description())
        }
        Internal(msg: String) {
            from()
        }
    }
}

impl From<anyhow::Error> for RepoError {
    fn from(from: anyhow::Error) -> Self {
        RepoError::Other(from)
    }
}

impl From<ofdb_core::password::Error> for Error {
    fn from(_: ofdb_core::password::Error) -> Self {
        Error::Parameter(ParameterError::Password)
    }
}

impl From<ofdb_core::RegistrationTypeParseError> for Error {
    fn from(_: ofdb_core::RegistrationTypeParseError) -> Self {
        Error::Parameter(ParameterError::RegistrationType)
    }
}
