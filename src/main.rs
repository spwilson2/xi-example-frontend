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


mod core;
mod depends;

fn main() {
    depends::setup_log();

    #[cfg(feature = "tracing")]
    trace::start_tracer();

    // Start the XI Core and RPC parser thread, get writer and reader to/from the core.
    let (to_core_channel, from_core_channel, event_loop_join) = core::start_xi_core();

    // TODO:
    // Initilize the RPC channels used by the GUI thread
    ui_controller = ui::Controller::new();

    let mut to_ui_chan = ui_controller.input_chan.borrow_mut();
    let mut from_ui_chan = ui_controller.output_chan.borrow_mut();

    // Start our event processing event loop
    // This thread should receive input events (from GUI or possible plugins) and forward control messages
    // to whatever event handler should receive them (likely the GUI and xi-core)
    //
    // TODO 
    // It'll be the main_controller's responsiblity to close channels when shutdown takes space.
    let main_controller = controller::new(to_core_channel, from_core_channel, to_ui_chan, from_ui_chan));

    // Run the GUI in the main thread - (some graphics frameworks expect to run from the main thread)
}