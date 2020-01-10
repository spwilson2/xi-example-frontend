extern crate xi_core_lib;
extern crate xi_rpc;

extern crate chrono;
#[macro_use]
extern crate log;
extern crate fern;

#[macro_use]
extern crate serde_json;

//use std::cell::RefCell;
//use std::fs::File;
//use std::io::prelude::*;
//use std::process::exit;
//use std::rc::Rc;
use std::thread;

use xi_rpc::{Peer, RpcLoop};

mod core;
mod logging;
mod events;
mod ui;

use ui::keyboard::TermionKeyboard;
use ui::InputController;

static CONFIG_DIR :  &'static str = "/localhome/sean.wilson/.config/xi-example";

fn setup_config(core: &dyn Peer) {

  core.send_rpc_notification(
    "client_started",
    &json!({ "config_dir": CONFIG_DIR, }),
    );
}

fn setup_logger() {
    let logging_path = CONFIG_DIR.to_owned() + "/xi-example-log.txt";
    logging::setup(&std::path::Path::new(&logging_path)).expect("Failed to setup logger")
}

fn main() {

  setup_logger();

  let (client_to_core_writer, core_to_client_reader, client_to_client_writer) =
    core::start_xi_core();

  let mut front_event_loop = RpcLoop::new(client_to_core_writer);

  let raw_peer = front_event_loop.get_raw_peer();
  setup_config(&raw_peer);

  let child = thread::spawn(move || {

    let mut event_handler = events::EventController::new();
    front_event_loop
      .mainloop(|| core_to_client_reader, &mut event_handler)
      .unwrap();
  });


  // TODO Initialize UI

  // Create a reader from the std input stream.
  let mut keyboard_device = TermionKeyboard::from_reader(std::io::stdin());

  // Create a controller which will be used to map inputs from the keyboard
  // to actions for xi
  //let mut input_controller = InputController::new(
  //  keyboard_device, 
  //  client_to_client_writer);

  // Begin the keyboard controller's event loop. We will now 
  // start processing input events from the keyboard
  //if let Err(err) = input_controller.start_keyboard_event_loop(&raw_peer) {
  //    println!("an error occured: {}", err);
  //    exit(1);
  //}

  child.join().unwrap();

}
