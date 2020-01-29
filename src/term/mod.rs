pub mod io;
//mod screen;
pub mod components;

use tokio::io::{stdout, stdin};
use tokio::io::{Stdin, Stdout};

use crate::future::io::{AsyncWrite, AsyncRead, AsyncRawFd};
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
    write_mux: io::WriteMux<AsyncRawFd>,
}

struct MyStdout(Stdout);

/// Initialize the TermController
pub fn initialize_term_controller() -> TermController {
    let fd = crate::future::io::RawFd(crate::future::io::STDOUT_FNUM);
    let fd = crate::future::io::AsyncRawFd::new(fd);

    TermController::new(fd)
}

impl TermController {
  pub fn new(mut out: AsyncRawFd) -> Self {
    //let reader = io::TermInputReader::new(stdin());
    //

    let mut s = Self {
     // controller: None,
      //input_reader: reader,
      write_mux: io::WriteMux::new(out),
    };

    //s.controller = Some(io::TermSizeController::new(writer, &s.input_reader));
    s
  }
}
