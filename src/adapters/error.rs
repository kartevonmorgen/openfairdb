quick_error!{
    #[derive(Debug)]
    pub enum ConversionError {
        Id{
            description("No ID was found")
        }
        Created{
            description("No timestamp was found")
        }
        Version{
            description("No version was found")
        }
        Name{
            description("No name was found")
        }
        Categories{
            description("No categories were found")
        }
    }
}

quick_error!{
    #[derive(Debug)]
    pub enum Error {
        Conversion(err: ConversionError){
            from()
            cause(err)
            description(err.description())

        }
    }
}
