//use std::io::{StdoutLock};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, Stdout};

use tokio::prelude::*;
use tokio::sync::broadcast;
use tokio::sync::{Mutex, Semaphore, SemaphorePermit};

/// This controller manages detection of the terminal screen size.
/// Screen size of terminals can be difficult to maintain. It may require
/// asynchronous messages.
pub struct TermSizeController {
  // Last pervious known size
  size: Arc<Mutex<(usize, usize)>>, // Width, Height

  // Write to the terminal to send messages requesting the size of the terminal
  term_writer: TermWriteMux,

  // Notification/Read access to the terminal for resize responses
  escape_receiver: Option<escapeSequenceReceiver>,
}

const RESIZE_ESCAPE_REQ: &'static str = "";
const RESIZE_ESCAPE_RET: &'static str = "";
const REFRESH_SIZE_RATE: u64 = 10;

fn handle_escape(size: &mut (usize,usize), es: &str) {
  // TODO Parse the size fromt he escape sequence
  if es == RESIZE_ESCAPE_RET {
    *size = (1,1);
  }
}

/// This controller manages detection of the terminal screen size.
/// Screen size of terminals can be difficult to maintain. It may require
/// asynchronous messages.
impl TermSizeController {
  pub fn new<R: AsyncRead>(writer: TermWriteMux, receiver: &TermInputReader<R>) -> Self {
    Self {
      size: Arc::from(Mutex::new((0,0))),
      term_writer: writer,
      escape_receiver: Some(receiver.sub_escape()),
    }
  }

  pub async fn begin_loop(&mut self) {

    // Start a task where we wait for updates.
    // NOTE: This task will own the escape receiver from here on out.
    let size = self.size.clone();
    let mut escape_receiver = self.escape_receiver.take().unwrap();
    tokio::spawn(async move {
      loop {
        let data = escape_receiver.recv().await.unwrap();
        let mut locked_size = size.lock().await;
        handle_escape(&mut locked_size, data.as_str());
      }
    });

    loop {
      self.tick().await;
      tokio::time::delay_for(std::time::Duration::from_millis(REFRESH_SIZE_RATE)).await;
    }
  }

  async fn tick(&mut self) {
    self.term_writer.write_stdout(RESIZE_ESCAPE_REQ.as_bytes()).await.unwrap();
  }
}

type escapeSequence = String;
type escapeSequenceReceiver = broadcast::Receiver::<escapeSequence>;

pub enum TermInputValue {
  text(String),
  escapeSequence(escapeSequence), // TODO use events rather than the string
}

/// TermInputReader reads from the terminal input stream and translates the
/// results into different events 
pub struct TermInputReader<R : AsyncRead> {
  term_reader: R,

  escape_chan: (broadcast::Sender::<escapeSequence>, escapeSequenceReceiver),
  text_chan: (broadcast::Sender::<String>, broadcast::Receiver::<String>),
}

impl<R: AsyncRead> TermInputReader<R> {
  pub fn new(mut reader: R) -> Self {
    Self {
      term_reader: reader,
      escape_chan: broadcast::channel(10),
      text_chan: broadcast::channel(10),
    }
  }

  pub fn sub_escape(&self) -> escapeSequenceReceiver{
    self.escape_chan.0.subscribe()
  }
}

const TERM_WRITER_PERMITS: usize = 4;

/// This struct handles multiplexing of writes to the terminal
///
/// This is necessary because multiple writers will be required to 
/// do things like asynchronous redraw detection
pub struct TermWriteMux {
  //writer: Stdout,
  // Reader/writer semaphore
  callback: &'static dyn Fn() -> Stdout,
  writer: Stdout,
  sem: Arc<Semaphore>,
}

impl Clone for TermWriteMux {
  fn clone(&self) -> Self {
    Self {
      sem: self.sem.clone(),
      writer: (self.callback)(),
      callback: self.callback,
    }
  }
}

impl TermWriteMux {
  pub fn new(cb: &'static dyn Fn() -> Stdout) -> Self {
    Self {
      callback: cb,
      writer: cb(),
      sem: Arc::from(Semaphore::new(TERM_WRITER_PERMITS)),
    }
  }

  pub async fn write_stdout(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
    let _p = self.sem.acquire().await;
    self.writer.write(buf).await
  }

  pub async fn write_stdout_exclusive(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
    let mut permits: [Option<SemaphorePermit>; TERM_WRITER_PERMITS] = Default::default();
    for permit in permits.iter_mut() {
      *permit = Some(self.sem.acquire().await);
    }
    self.writer.write(buf).await
  }
}
