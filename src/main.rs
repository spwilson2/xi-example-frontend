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

#[macro_export]
macro_rules! cast_err {
    ($e:expr) => {
        match $e {
            Ok(_ok) => Ok(_ok),
            Err(_fail) => Err(Error::from(_fail)),
        }
    }
}

pub mod exports;
pub mod futures;
pub mod term;

fn main() {
    println!("Hello, world!");
}
