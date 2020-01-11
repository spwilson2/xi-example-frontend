use std::io::{self, BufRead, Read, Write};

pub struct Controller {
}

impl Controller {
  pub fn new(
    to_core_chan : impl Write, 
    to_ui_chan : impl Write, 
    from_ui_chan: impl Read) -> Self {
    Controller {}
  }

  pub fn mainloop(&mut self) {

  }
}
