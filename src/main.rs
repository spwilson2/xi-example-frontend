extern crate termion;
#[macro_use]
extern crate pin_project_lite;

mod term;
mod newterm;
use term::Layout;
use tokio::io::{stdout, stdin};

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

impl Widget {
}

struct LayoutWidget {
}

struct TextViewWidget {
}


// Note, To be able to get a cached view of the xi main we'll probably register
// ourself as a plugin, that or create our own version of the cache.

fn main() {


  // Initialize core runner

  //let l = term::layout::TermionLayout::new();
  //let v = l.create_view_window();
  //v.refresh();
  let c = newterm::TermController::new();
}
