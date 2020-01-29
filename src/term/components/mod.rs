/// This module contains different components for use when interacting with the terminal
///

use crate::future::runtime::{Handle, Promise};
use crate::future::sync::watch;
type Size = (usize, usize);

use failure::Error;
trait DetectResize {
    fn detect_size(&mut self) -> Promise<Result<(usize, usize), Error>>;
}

trait NotifyResize {
    fn notify_resize(_: (usize, usize)) ;
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

/// This version of the resize detector detects the size of the terminal by using escape sequences.
struct IOResizeDetector {
    // TODO Take the WriteMux for terminal
    // TODO Hold a notifier for the 
}

impl DetectResize for IOResizeDetector {
    fn detect_size(&mut self) -> Promise<Result<(usize, usize), Error>>{
        todo!()
    }
}

impl IOResizeDetector {
    //pub fn new(mut dist : &mut InputDistributor) -> Self {
    //    Self {
    //    }
    //}
}

/// This version of the resize detector detects the size of the terminal by making calls to the
/// operating system
struct SyscallResizeDetector {
}

