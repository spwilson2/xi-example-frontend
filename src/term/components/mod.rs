/// This module contains different components for use when interacting with the terminal
///

use crate::future::runtime::{Handle, Promise};
use crate::future::sync::watch;
type Size = (usize, usize);

use failure::Error;

use crate::term::io::{WriteMux, InputDistributor};
use crate::future::io::{AsyncWrite, AsyncRead, AsyncRawFd};
use crate::future::sync::{broadcast, RwLock, Mutex, MutexGuard, RwLockWriteGuard, Semaphore, SemaphorePermit};

use tokio::task::JoinHandle;

trait DetectResize {
    fn detect_size(&mut self) -> Promise<Result<(usize, usize), Error>>;
}

trait NotifyResize {
    fn notify_resize(self, _: (usize, usize));
}

pin_project! {
/// The resize controller is in charge of detecting and signaling resize events.
pub struct ResizeController<D: DetectResize> {
    size: Size,
    sender: watch::Sender<Size>,
    receiver: watch::Receiver<Size>,
    #[pin]
    dector: D,

}
}

impl<D: DetectResize + Send + Sync>  ResizeController<D> {
    fn new (dector: D) -> Self {
        let (s,r) = watch::channel((0,0));
        Self {
            receiver: r,
            dector: dector,
            size: (0, 0),
            sender: s,
        }
    }
    async fn check_size_change(old_size: Size, detector: std::pin::Pin<&mut D>, sender: &watch::Sender<Size>) {
        let size = detector.detect_size().await.unwrap().unwrap();
        if size != old_size {
        }
    }

    pub fn subscribe(&self) -> watch::Receiver<Size> {
        self.receiver.clone()
    }

    pub fn tick(self: std::pin::Pin<&mut Self>, handle: &Handle) -> Promise<()> {
        let this = self.project();
        handle.spawn(Self::check_size_change(self.size, this.dector, &self.sender))
    }
}

impl <D: DetectResize, N: NotifyResize> ResizeController<D,N> {
    pub fn new(detector: D, notifier: N) -> Self {
        Self {
            dector: detector,
            notifier: notifier,
        }
    }

    fn tick(/*handler*/) {
        // Give the detector handle to runtime, schedules itself returns a join-handle
        // Give the handler + joinhandle to the notifier

    }
}

/// This version of the resize detector detects the size of the terminal by using escape sequences.
struct IOResizeDetector<N: NotifyResize> {
    writer: WriteMux<AsyncRawFd>,
    input: broadcast::Receiver<char>,
    notifier: N,
}

static ESC: char = 0x1Bu8 as char;

impl<N: NotifyResize> IOResizeDetector<N> {
    pub fn new<R: AsyncRead>(writer: WriteMux<AsyncRawFd>, input: broadcast::Receiver<char>, N) -> Self {
        Self {
            writer: writer,
            input: input,
            notifier: notifier,
        }
    }

    pub async fn tick(&mut self) {
        self.writer.write(RESIZE_ESCAPE_REQ.as_bytes()).await.unwrap();

        'a: loop {
            let c = self.input.recv().await.unwrap();
            if c == ESC {
                let c = self.input.recv().await.unwrap();
                if c == '[' {
                    // Read until ;
                    let mut row = 0usize;
                    loop {
                        let c = self.input.recv().await.unwrap();
                        if c.is_ascii_digit() {
                            row = row * 10 +  c.to_digit(10).unwrap() as usize;
                        }
                        else if c == ';' {
                            break;
                        } else {
                            // Just ignore, we didn't find the sequence we were looking for
                            continue 'a;
                        }
                    }

                    let mut col = 0usize;
                    // Read until R
                    loop {
                        let c = self.input.recv().await.unwrap();
                        if c.is_ascii_digit() {
                            col = col * 10 +  c.to_digit(10).unwrap() as usize;
                        }
                        else if c == 'R' {
                            break;
                        } else {
                            // Just ignore, we didn't find the sequence we were looking for
                            continue 'a;
                        }
                    }
                    self.notifier.notify_resize((col, row));
                }
                // Get the remaineder of the escape sequence
            }
        }
        // Receive on the channel, discard all that are not the returned escape sequence
    }
}

/// This version of the resize detector detects the size of the terminal by making calls to the
/// operating system
struct SyscallResizeDetector {
}


const RESIZE_ESCAPE_REQ: &'static str = "";
const RESIZE_ESCAPE_RET: &'static str = "";
//const REFRESH_SIZE_RATE: u64 = 10;

//
//fn handle_escape(size: &mut (usize,usize), es: &str) {
//  // TODO Parse the size fromt he escape sequence
//  if es == RESIZE_ESCAPE_RET {
//    *size = (1,1);
//  }
//}
//
///// This controller manages detection of the terminal screen size.
///// Screen size of terminals can be difficult to maintain. It may require
///// asynchronous messages.
//impl<W: AsyncWrite + Unpin> TermSizeController<W> {
//  pub fn new<R: AsyncRead>(writer: WriteMux<W>, receiver: &InputDistributor<R>) -> Self {
//    Self {
//      size: Arc::from(Mutex::new((0,0))),
//      term_writer: writer,
//      escape_receiver: Some(receiver.sub_escape()),
//    }
//  }
//
//  pub async fn begin_loop(&mut self) {
//
//    // Start a task where we wait for updates.
//    // NOTE: This task will own the escape receiver from here on out.
//    let size = self.size.clone();
//    let mut escape_receiver = self.escape_receiver.take().unwrap();
//    tokio::spawn(async move {
//      loop {
//        let data = escape_receiver.recv().await.unwrap();
//        let mut locked_size = size.lock().await;
//        handle_escape(&mut locked_size, data.as_str());
//      }
//    });
//
//    loop {
//      self.tick().await;
//      tokio::time::delay_for(std::time::Duration::from_millis(REFRESH_SIZE_RATE)).await;
//    }
//  }
//
//  async fn tick(&mut self) {
//    self.term_writer.write(RESIZE_ESCAPE_REQ.as_bytes()).await.unwrap();
//  }
//}
