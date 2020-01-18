mod io;
use tokio::io::{stdout, stdin};
use tokio::io::{Stdin, Stdout};
use std::sync::Arc;

/// This struct manages the logic for using a terminal to:
/// - Receive keyboard input
/// - Draw windows
/// - Detect resize events
struct TermController {
  controller: Option<io::TermSizeController>,
  input_reader: Option<io::TermInputReader<Stdin>>,

  // TODO Gonna need a handle to stdout
  //stdin_handle: Stdin,
  writer: io::TermWriteMux,
}

impl TermController {
  pub fn new(sin: Stdin, sout: Stdout) -> Self {
    let mut s = Self {
      controller: None, 
      input_reader: None,
      writer: io::TermWriteMux::new(&stdout),
    };

    s.input_reader = Some(io::TermInputReader::new(sin));
    s.controller = Some(io::TermSizeController::new(s.writer.clone(), &s.input_reader.as_ref().unwrap()));
    s
  }
}
