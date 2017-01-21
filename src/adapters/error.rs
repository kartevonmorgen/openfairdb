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
