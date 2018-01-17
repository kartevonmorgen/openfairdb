// Copyright (c) 2015 - 2018 Markus Kohlhase <mail@markus-kohlhase.de>

#![feature(plugin, custom_derive, test)]
#![plugin(rocket_codegen)]
#![recursion_limit = "256"]

extern crate chrono;
extern crate clap;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;
extern crate env_logger;
extern crate fast_chemail;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate pwhash;
#[macro_use]
extern crate quick_error;
extern crate quoted_printable;
extern crate regex;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[cfg(test)]
extern crate test;
extern crate toml;
extern crate url;
extern crate uuid;

mod entities;
mod business;
mod adapters;
mod infrastructure;

fn main() {
    env_logger::init();
    infrastructure::cli::run();
}
