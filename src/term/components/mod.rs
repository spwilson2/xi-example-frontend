/// This module contains different components for use when interacting with the terminal
///

use failure::Error;
trait DetectResize {
    fn detect_size() -> Result<(usize, usize), Error>;
}

trait NotifyResize {
    fn notify_resize(_: (usize, usize)) ;
}

struct ResizeController<D: DetectResize, N: NotifyResize> {
    dector: D,
    notifier: N,
}

/// This version of the resize detector detects the size of the terminal by using escape sequences.
struct IOResizeDetector {
    // TODO Take the WriteMux for terminal
    // TODO Hold a notifier for the 
}

impl DetectResize for IOResizeDetector {
    fn detect_size() -> Result<(usize, usize), Error> {
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

