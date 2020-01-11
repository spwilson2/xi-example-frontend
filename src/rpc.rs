extern crate tokio;
extern crate futures;

use tokio::prelude::*;
use futures::prelude::*;
use tokio::sync::mpsc;

use crate::futures::StreamExt;

use serde_json::Value;
use xi_core_lib::XiCore;

use serde::de::DeserializeOwned;

/// A trait for types which can handle RPCs.
///
/// Types which implement `MethodHandler` are also responsible for implementing
/// `Parser`; `Parser` is provided when Self::Notification and Self::Request
/// can be used with serde::DeserializeOwned.
pub trait Handler {
    type Notification: DeserializeOwned;
    type Request: DeserializeOwned;
    fn handle_notification(&mut self, ctx: &RpcCtx, rpc: Self::Notification);
    fn handle_request(&mut self, ctx: &RpcCtx, rpc: Self::Request) -> Result<Value, RemoteError>;
    #[allow(unused_variables)]
    fn idle(&mut self, ctx: &RpcCtx, token: usize) {}
}

// On the old verison of the loop
// Reader passed into loop would recv messages and those messages would be forwarded out
// to all peers of the loop

// Reader passed to the ::new method would be used to receive RPC calls and execute them
// utilizing the handler
async fn rpc_loop() {

async pub fn rpc_loop<H>(&mut self, handler: &mut H) -> Result<(), ReadError>
    where
        H: Handler,
    {

    loop {
        // TODO: Select from channels we're spinning on

        // TODO: Pass the message to the handler
    }
}

pub fn start_xi_core() -> (Writer, Reader, std::thread::JoinHandle<()>) {
    let mut core = XiCore::new();



    let (to_core_tx, to_core_rx) = mpsc::unbounded_channel();

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


async fn run(mut sender: mpsc::UnboundedSender::<String>, mut recv: mpsc::UnboundedReceiver::<String>) {
    tokio::spawn(async move {
      sender.send("Hello".to_string()).unwrap();
      sender.send("world!".to_string()).unwrap()
    });

    //while let Some(res) = recv.next().await {
    //  println!("Received {}", res);
    //}
    recv.for_each( |res|{
      println!("Received {}", res);
      future::ready(())
    }).await;

}

fn main() {
  // Create the runtime
  let (sender, recv) = mpsc::unbounded_channel::<String>();
  let mut rt = tokio::runtime::Runtime::new().unwrap();

  rt.block_on(run(sender, recv));
  
}
