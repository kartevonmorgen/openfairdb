// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate log;
extern crate slog_envlogger;
#[macro_use]
extern crate nickel;
extern crate rustc_serialize;
extern crate hyper;
extern crate unicase;
extern crate clap;
#[macro_use]
extern crate rusted_cypher;
extern crate r2d2;
extern crate r2d2_cypher;
extern crate typemap;
extern crate uuid;
extern crate geojson;
extern crate emailaddress;
extern crate url;

mod business;
mod adapters;
mod infrastructure;

fn main() {
    slog_envlogger::init().unwrap();
    infrastructure::cli::run();
}
