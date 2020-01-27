#![allow(
    dead_code, 
    unused_imports,
    unused_variables,
    )]

extern crate termion;
extern crate tokio;
extern crate futures;
extern crate libc;
#[macro_use]
extern crate failure;

#[macro_export]
macro_rules! cast_err {
    ($e:expr) => {
        match $e {
            Ok(_ok) => Ok(_ok),
            Err(_fail) => Err(Error::from(_fail)),
        }
    }
}

mod term;
mod future;


// Window:
//  - Widget

// Frame Widget -- Contains and divides other widgets:
// - Widget(s): 

// Text Buffer Widget:
// - XI View of a buffer
// - Cursor (location)


// Term Screen:
//  WindowWidget
//   LayoutWidget
//    TextViewWidget



trait Widget {
  fn redraw(&mut self, dims: &(usize, usize));  // TODO Buffer param
  fn resize(&mut self, dims: &(usize, usize)) { // TODO Buffer param
    self.redraw(dims)
  }
}



//
struct WindowWidget {
}

impl Widget for WindowWidget {
  fn redraw(&mut self, dims: &(usize, usize)) {todo!()}  // TODO Buffer param
}

struct LayoutWidget {
}

struct TextViewWidget {
}


// Note, To be able to get a cached view of the xi main we'll probably register
// ourself as a plugin, that or create our own version of the cache.
fn main() {

  let c = term::initialize_term_controller();
}
