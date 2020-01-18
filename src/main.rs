extern crate termion;

mod term;
mod newterm;
use term::Layout;

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


/// This struct manages the logic for using a terminal to:
/// - Receive keyboard input
/// - Draw windows
/// - Detect resize events
struct TermController {
  controller: TermSizeController,
  input_reader: TermInputReader,
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

  let l = term::layout::TermionLayout::new();
  let v = l.create_view_window();
  v.refresh();
}
