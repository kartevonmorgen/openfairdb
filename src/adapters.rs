// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

pub mod json {

    #[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
    pub struct Entry {
        pub id          : Option<String>,
        pub created     : Option<u64>,
        pub version     : Option<u64>,
        pub title       : String,
        pub description : String,
        pub lat         : f64,
        pub lng         : f64,
        pub street      : Option<String>,
        pub zip         : Option<String>,
        pub city        : Option<String>,
        pub country     : Option<String>,
        pub email       : Option<String>,
        pub telephone   : Option<String>,
        pub homepage    : Option<String>,
        pub categories  : Option<Vec<String>>,
        pub license     : Option<String>,
    }
    
    #[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
    pub struct Category {
        pub id        : Option<String>,
        pub created   : Option<u64>,
        pub version   : Option<u64>,
        pub name      : Option<String>
    }
    
    #[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
    pub struct SearchResult {
        pub visible   : Vec<String>,
        pub invisible : Vec<String>
    }
}
