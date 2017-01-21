use std::io;

quick_error!{
    #[derive(Debug)]
    pub enum ParameterError {
        Bbox{
            description("Requested bounding box is invalid")
        }
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum Error {
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
    }
}
