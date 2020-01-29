//use std::io::{StdoutLock};
use std::sync::Arc;
use crate::future::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use crate::future::io::prelude::*;
use crate::future::sync::{broadcast, RwLock, Mutex, MutexGuard, RwLockWriteGuard, Semaphore, SemaphorePermit};

use failure::{Error, Fail};

use std::cell::UnsafeCell;

///// This controller manages detection of the terminal screen size.
///// Screen size of terminals can be difficult to maintain. It may require
///// asynchronous messages.
//pub struct TermSizeController<W: AsyncWrite + Unpin> {
//  // Last pervious known size
//  size: Arc<Mutex<(usize, usize)>>, // Width, Height
//
//  // Write to the terminal to send messages requesting the size of the terminal
//  term_writer: WriteMux<W>,
//
//  // Notification/Read access to the terminal for resize responses
//  escape_receiver: Option<escapeSequenceReceiver>,
//}
//
pub struct InputDistributor<R : AsyncRead + Unpin> {
  term_reader: R,
  chan: broadcast::Sender::<char>,
}

impl<R: AsyncRead + Unpin> InputDistributor<R> {
  pub fn new(reader: R) -> Self {
    Self {
      term_reader: reader,
      chan: broadcast::channel(10).0,
    }
  }

  /// Schedule an async read call on the given input distributor.
  pub async fn read(&mut self) -> Result<(), Error>{
      use std::str;

      let mut buf = [0u8;4];
      let bytes = self.term_reader.read(&mut buf).await?;
      let s = str::from_utf8(&buf)?;

      for c in s.chars() {
          // Ignore the error, if there were no senders it's not a problem.
          // Someone might subscribe later.
          let _ = self.chan.send(c);
      }
      Ok(())
  }

  pub fn subscribe(&self) -> broadcast::Receiver<char> {
      self.chan.subscribe()
  }
}

/***********************/
/** Write Mux Methods **/
/***********************/

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
  pub async fn write(&self, buf: &[u8]) -> Result<usize, Error> {
    let _l = self.state.exclusive_lock.acquire().await;
    // NOTE: This method is safe because we only allow access either with ALL
    // reader semaphors (in the case of acquire) or the exclusive lock
    // semaphore (in the case of this call).
    let ret;
    unsafe {
      ret = self.write_unlocked(buf).await;
    }
    drop(_l);
    cast_err!(ret)
  }

  pub async fn flush(&self) -> Result<(), Error> {
    let _l = self.state.exclusive_lock.acquire().await;
    let ret;
    unsafe {
      ret = self.flush_unlocked().await;
    }
    drop(_l);
    cast_err!(ret)
  }

  pub(self) async unsafe fn flush_unlocked(&self) -> Result<(), Error> {
      cast_err!((*self.state.writer.get()).flush().await)
  }

  pub(self) async unsafe fn write_unlocked(&self, buf: &[u8]) -> Result<usize, Error> {
      cast_err!((*self.state.writer.get()).write(buf).await)
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
  pub async fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
    unsafe {
      self.state.write_unlocked(buf).await
    }
  }

  pub async fn flush(&mut self) -> Result<(), Error> {
    unsafe {
      self.state.flush_unlocked().await
    }
  }
}
