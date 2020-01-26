mod io;
mod screen;

use tokio::io::{stdout, stdin};
use tokio::io::{Stdin, Stdout};
use tokio::io::{AsyncWrite, AsyncRead};
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
  controller: Option<io::TermSizeController>,
  input_reader: io::TermInputReader<Stdin>,
  writer: io::WriteMux<Stdout>,
}

struct MyStdout(Stdout);

//impl Write for MyStdout {
//  fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
//
//    self.0.poll_write();
//
//  }
//}
//
//    fn into_raw_mode<W: Write>(w:&W) -> std::io::Result<()> {
//        let mut ios = get_terminal_attr()?;
//        let prev_ios = ios;
//
//        raw_terminal_attr(&mut ios);
//
//        set_terminal_attr(&ios)?;
//
//        Ok(())
//    }

impl TermController {
  pub fn new() -> Self {
    let reader = io::TermInputReader::new(stdin());
    let writer = io::WriteMux::new(stdout());

    let mut s = Self {
      controller: None,
      input_reader: reader,
      writer: writer.clone(),
    };

    s.controller = Some(io::TermSizeController::new(writer, &s.input_reader));
    s
  }
}
