
//mod screen;

use crate::exports::io::{AsyncWrite, AsyncRead};
use super::io::raw::{AsyncRawFd};
use super::io::util::{WriteMux};


use tokio::io::{stdout, stdin};
use tokio::io::{Stdin, Stdout};
use std::io::Write;

use tokio::prelude::*;

use termion::raw::IntoRawMode;

use std::sync::Arc;
use std::mem::MaybeUninit;

/// This struct manages the logic for using a terminal to:
/// - Receive keyboard input
/// - Draw windows
/// - Detect resize events
pub struct TermController {
    //input_reader: io::TermInputReader<Stdin>,
    //input_distributor: io::InputDistributor<AsyncRawFd>,
    write_mux: WriteMux<AsyncRawFd>,
}

struct MyStdout(Stdout);

// Initialize the TermController
//pub fn initialize_term_controller() -> TermController {
//    let fd = RawFd(STDOUT_FNUM);
//    let fd = AsyncRawFd::new(fd);
//
//    TermController::new(fd)
//}
//
//impl TermController {
//  pub fn new(mut out: AsyncRawFd) -> Self {
//    //let reader = io::TermInputReader::new(stdin());
//    //
//
//    let mut s = Self {
//     // controller: None,
//      //input_reader: reader,
//      write_mux: WriteMux::new(out),
//    };
//
//    //s.controller = Some(io::TermSizeController::new(writer, &s.input_reader));
//    s
//  }
//}
