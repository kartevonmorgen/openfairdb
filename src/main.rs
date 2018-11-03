// Copyright (c) 2015 - 2018 Markus Kohlhase <mail@markus-kohlhase.de>

#![feature(plugin, custom_derive, test, transpose_result, proc_macro_hygiene, decl_macro)]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde;
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
