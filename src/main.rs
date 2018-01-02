// Copyright (c) 2015 - 2017 Markus Kohlhase <mail@markus-kohlhase.de>

#![feature(plugin,custom_derive,test)]
#![plugin(rocket_codegen)]
#![recursion_limit="256"]

#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate quick_error;
extern crate clap;
extern crate r2d2;
extern crate uuid;
extern crate fast_chemail;
extern crate url;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate pwhash;
extern crate quoted_printable;
extern crate toml;
extern crate dotenv;

#[cfg(feature = "sqlite")]
extern crate r2d2_diesel;

#[cfg(feature = "sqlite")]
#[macro_use]
extern crate diesel;

#[cfg(feature = "sqlite")]
#[macro_use]
extern crate diesel_migrations;

#[cfg(test)]
extern crate test;

mod entities;
mod business;
mod adapters;
mod infrastructure;

fn main() {
    env_logger::init().unwrap();
    infrastructure::cli::run();
}
