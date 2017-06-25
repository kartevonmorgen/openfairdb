// Copyright (c) 2015 - 2017 Markus Kohlhase <mail@markus-kohlhase.de>

#![feature(plugin,custom_derive)]
#![plugin(rocket_codegen)]
#![allow(unmanaged_state)]

#[macro_use]
extern crate log;
extern crate slog_envlogger;
#[macro_use]
extern crate quick_error;
extern crate clap;
#[macro_use]
extern crate rusted_cypher;
extern crate r2d2;
extern crate r2d2_cypher;
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

mod entities;
mod business;
mod adapters;
mod infrastructure;

fn main() {
    // TODO: setup proper logging with rocket!
    // let _guard = slog_envlogger::init().unwrap();
    infrastructure::cli::run();
}
