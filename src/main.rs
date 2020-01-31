#![allow(
    dead_code, 
    unused_imports,
    unused_variables,
    )]

#[macro_use]
extern crate pin_project_lite;
extern crate termion;
extern crate tokio;
extern crate futures as futures_rs;
extern crate libc;
#[macro_use]
extern crate failure;

pub mod exports;
pub mod futures;

fn main() {
    println!("Hello, world!");
}
