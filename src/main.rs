#![allow(dead_code, unused_imports)]

#[macro_use]
extern crate serde_json;
extern crate xi_core_lib;
extern crate xi_rpc;
#[macro_use]
extern crate log;
//#[macro_use]
//extern crate clap;
extern crate dirs;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
extern crate termion;
extern crate toml;
#[cfg(feature = "tracing")]
extern crate xi_trace;

#[cfg(feature = "sync")]
extern crate tokio;

mod core;
mod depends;
mod ui;
mod base;

fn main() {
    depends::setup_log();

    #[cfg(feature = "tracing")]
    trace::start_tracer();

    // Start the XI Core and RPC parser thread, get writer and reader to/from the core.
    let (to_core_chan, from_core_chan, core_event_thread) = core::start_xi_core();

    // Allocate/init the RPC channels used by the GUI thread
    //let mut ui_controller = ui::Controller::new();
    //let (to_ui_chan, from_ui_chan) = ui_controller.borrow_channels_mut();

    //// Allocate our main controller
    ////
    //// This controller will be responsible for managing routing of RPC requests
    //// to the correct channels an overall program logic such as:
    //// - Saving files
    //// - Quitting
    //// It'll be the main_controller's responsiblity to close channels when
    //// shutdown takes place.
    //let mut main_controller = base::Controller::new(to_core_chan, to_ui_chan, from_ui_chan);

    //// Start our event processing event loop
    //// This thread should receive input events (from GUI or possible plugins)
    //// and forward control messages to whatever event handler should receive
    //// them (likely the GUI and xi-core)
    //let controller_thread = std::thread::spawn(move || {
    //    main_controller.mainloop();
    //});

    //// Run the GUI in the main thread - (some graphics frameworks expect to run
    //// from the main thread)
    ////
    //// Loop will shutdown when the main_controller exits.
    //ui_controller.mainloop();
    //controller_thread.join();
    //core_event_thread.join();
}