use std::io::{self, BufRead, Read, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use serde_json::Value;
use xi_core_lib::XiCore;
use xi_rpc::RpcLoop;

/// Wraps an instance of `mpsc::Sender`, implementing `Write`.
///
/// This lets the tx side of an mpsc::channel serve as the destination
/// stream for an RPC loop.
pub struct Writer(Sender<String>);

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = String::from_utf8(buf.to_vec()).unwrap();
        self.0
            .send(s)
            .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("{:?}", err)))
            .map(|_| buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Wraps an instance of `mpsc::Receiver`, providing convenience methods
/// for parsing received messages.
pub struct Reader(Receiver<String>);

impl Read for Reader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        unreachable!("didn't expect xi-rpc to call read");
    }
}

// Note: we don't properly implement BufRead, only the stylized call patterns
// used by xi-rpc.
impl BufRead for Reader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        unreachable!("didn't expect xi-rpc to call fill_buf");
    }

    fn consume(&mut self, _amt: usize) {
        unreachable!("didn't expect xi-rpc to call consume");
    }

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        let event = match self.0.recv() {
            Ok(s) => s,
            // Error should only occur if the channel was intentionally dropped.
            Err(_) => return Ok(0),
        };
        buf.push_str(&event);
        Ok(event.len())
    }
}

pub fn start_xi_core() -> (Writer, Reader, std::thread::JoinHandle<()>) {
    let mut core = XiCore::new();

    // TODO Modify readers to gracefully shutdown.
    //
    // xi-rpc will continue to read from the Reader until an EOF is returned.
    // To gracefully shutdown the reader should return EOF when it is time to exit.

    let (to_core_tx, to_core_rx) = channel();
    let to_core_writer = Writer(to_core_tx);
    let to_core_reader = Reader(to_core_rx);

    let (from_core_tx, from_core_rx) = channel();
    let from_core_writer = Writer(from_core_tx.clone());
    let from_core_reader = Reader(from_core_rx);

    let mut core_event_loop = RpcLoop::new(from_core_writer);

    // Spawn an event loop that will interact with the xi core.
    // The event loop will attempt to use the core to handle RPC's which 
    // we send down the to_core_writer or forward messages from the 
    // core out the from_core_reader to us.
    let event_loop_join = thread::spawn(move || core_event_loop.mainloop(|| to_core_reader, &mut core).unwrap());
    (to_core_writer, from_core_reader, event_loop_join)
}