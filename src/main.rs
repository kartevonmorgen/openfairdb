// Copyright (c) 2015 - 2018 Markus Kohlhase <mail@markus-kohlhase.de>

#![feature(plugin, custom_derive, test, transpose_result)]
#![plugin(rocket_codegen)]
#![allow(proc_macro_derive_resolution_fallback)]

extern crate chrono;

extern crate clap;

extern crate csv;

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
extern crate serde;

extern crate serde_json;

extern crate toml;

extern crate url;

extern crate uuid;

#[cfg(test)]
extern crate test;

mod adapters;
mod core;
mod infrastructure;
mod ports;

fn main() {
    env_logger::init();
    ports::cli::run();
}
