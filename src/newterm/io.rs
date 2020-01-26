//use std::io::{StdoutLock};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite, Stdout};

use tokio::prelude::*;
use tokio::sync::broadcast;
//use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::{RwLock, Mutex, MutexGuard, RwLockWriteGuard, Semaphore, SemaphorePermit};

use std::cell::UnsafeCell;

/// This controller manages detection of the terminal screen size.
/// Screen size of terminals can be difficult to maintain. It may require
/// asynchronous messages.
pub struct TermSizeController {
  // Last pervious known size
  size: Arc<Mutex<(usize, usize)>>, // Width, Height

  // Write to the terminal to send messages requesting the size of the terminal
  term_writer: WriteMux<Stdout>,

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
  pub fn new<R: AsyncRead>(writer: WriteMux<Stdout>, receiver: &TermInputReader<R>) -> Self {
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
    self.term_writer.write(RESIZE_ESCAPE_REQ.as_bytes()).await.unwrap();
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
  pub fn new(reader: R) -> Self {
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

struct WriteMuxState<W: AsyncWrite + Unpin> {
  writer: UnsafeCell<W>,
  exclusive_lock: Semaphore,
}

pub struct WriteMux<W: AsyncWrite + Unpin> {
  state: Arc<WriteMuxState<W>>,
}

impl<W: AsyncWrite + Unpin> Clone for WriteMux<W> {
  fn clone(&self) -> Self {
    Self {
      state: self.state.clone(),
    }
  }
}

impl<W: AsyncWrite + Unpin> WriteMuxState<W> {
  fn new(writer: W) -> Self {
    Self {
      writer: std::cell::UnsafeCell::from(writer),
      exclusive_lock: Semaphore::new(1),
    }
  }
}

impl<W: AsyncWrite + Unpin> WriteMux<W> {
  pub fn new(writer: W) -> Self {
    Self {
      state: Arc::from(WriteMuxState::new(writer))
    }
  }
  pub async fn write(&self, buf: &[u8]) -> Result<usize, std::io::Error> {
    let _l = self.state.exclusive_lock.acquire().await;
    // NOTE: This method is safe because we only allow access either with ALL
    // reader semaphors (in the case of acquire) or the exclusive lock
    // semaphore (in the case of this call).
    let ret;
    unsafe {
      ret = self.write_unlocked(buf).await;
    }
    drop(_l);
    ret
  }

  pub async fn flush(&self) -> Result<(), std::io::Error> {
    let _l = self.state.exclusive_lock.acquire().await;
    let ret;
    unsafe {
      ret = self.flush_unlocked().await;
    }
    drop(_l);
    ret
  }

  pub(self) async unsafe fn flush_unlocked(&self) -> Result<(), std::io::Error> {
      (*self.state.writer.get()).flush().await
  }

  pub(self) async unsafe fn write_unlocked(&self, buf: &[u8]) -> Result<usize, std::io::Error> {
      (*self.state.writer.get()).write(buf).await
  }

  pub async fn acquire<'a>(&'a self) -> WriteMuxGuard<'a, W> {
    WriteMuxGuard::<'a, W> {
      state: self,
      _permit: self.state.exclusive_lock.acquire().await,
    }
  }
}

pub struct WriteMuxGuard<'a, W: AsyncWrite + Unpin> {
  state: &'a WriteMux<W>,
  _permit: SemaphorePermit<'a>,
}

impl<'a, W: AsyncWrite + Unpin> WriteMuxGuard<'a, W> {
  pub async fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
    unsafe {
      self.state.write_unlocked(buf).await
    }
  }

  pub async fn flush(&mut self) -> Result<(), std::io::Error> {
    unsafe {
      self.state.flush_unlocked().await
    }
  }
}
