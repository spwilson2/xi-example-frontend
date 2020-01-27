use std::cell::RefCell;
use std::io::{stdout, Write};
use std::rc::Rc;

use termion::clear;
use termion::color::DetectColors;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use termion::sys::attr::{get_terminal_attr, raw_terminal_attr, set_terminal_attr};

use tokio::io::{Stdout};

use super::io;

const STATUS_HEIGHT: u32 = 1;

pub struct TermionScreen{
    height: u32,
    width: u32,
    writer: io::WriteMux<Stdout>,

    //writer: Rc<RefCell<Box<dyn Write>>>,
}

impl TermionScreen {
    pub fn new(writer: io::WriteMux<Stdout>) -> Self {
        let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());

        write!(stdout, "{}", clear::All).unwrap();

        let (width, height) = termion::terminal_size().unwrap();

        Self {
            writer: writer,
            height: u32::from(height),
            width: u32::from(width),
        }
    }

    // NOTE: This function doesn't use the fd of the read/writer. It should.
    pub fn enable_raw_mode() -> std::io::Result<()>{
      let mut ios = get_terminal_attr()?;
      let prev_ios = ios;
      raw_terminal_attr(&mut ios);
      set_terminal_attr(&ios)?;

      Ok(())
    }
}
