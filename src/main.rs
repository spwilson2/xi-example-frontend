#![allow(
    dead_code, 
    unused_imports,
    unused_variables,
    )]

extern crate xi_rope;
extern crate unicode_segmentation as un;
extern crate signal_hook;
#[macro_use]
extern crate pin_project_lite;
extern crate term as term_rs;
extern crate tokio;
extern crate futures as futures_rs;
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

pub mod exports;
pub mod futures;
pub mod term;

use std::io::prelude::*;
use failure::Error;
use std::io;
use libc::c_int;
pub use libc::termios as Termios;
use std::os::unix::io::{AsRawFd, RawFd};
use term_rs::{Terminal};
use term_rs::terminfo::{TermInfo, TerminfoTerminal};

// Support functions for converting libc return values to io errors {
trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

macro_rules! impl_is_minus_one {
        ($($t:ident)*) => ($(impl IsMinusOne for $t {
            fn is_minus_one(&self) -> bool {
                *self == -1
            }
        })*)
    }

impl_is_minus_one! { i8 i16 i32 i64 isize }

fn cvt<T: IsMinusOne>(t: T) -> io::Result<T> {
    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

// Man ioctl_tty
// Man tcsetattr

pub unsafe fn get_terminal_attr(fd: RawFd) -> io::Result<Termios> {
    let mut termios = std::mem::zeroed();
    cvt(libc::tcgetattr(fd, &mut termios))?;
    Ok(termios)
}

pub unsafe fn set_terminal_attr(fd: RawFd, termios: &Termios) -> io::Result<()> {
    cvt(libc::tcsetattr(fd, libc::TCSANOW, termios)).and(Ok(()))
}

pub fn raw_terminal_attr(termios: &mut Termios) {
    unsafe { libc::cfmakeraw(termios) };
}

use libc::{c_ushort, ioctl, STDOUT_FILENO, TIOCGWINSZ};

#[repr(C)]
struct TermSize {
    row: c_ushort,
    col: c_ushort,
    x: c_ushort,
    y: c_ushort,
}

/// Get the size of the terminal.S
///
/// NOTE: This call is non-portable and may not be supported by all terminals
/// See man ioctl_tty.
pub fn terminal_size() -> io::Result<(u16, u16)> {
    unsafe {
        let mut size: TermSize = std::mem::zeroed();
        cvt(ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut size as *mut _))?;
        Ok((size.col as u16, size.row as u16))
    }
}

pub struct OwnedFd(RawFd);

impl AsRawFd for OwnedFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Drop for OwnedFd {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

impl OwnedFd {
    /// Sets close on exec and nonblock on the inner file descriptor.
    fn set_flags(&self) -> Result<(), failure::Error> {
        unsafe {
            let flags = cvt(libc::fcntl(self.as_raw_fd(), libc::F_GETFL, 0))?;
            let flags = flags | libc::O_NONBLOCK | libc::O_CLOEXEC;
            cvt(libc::fcntl(self.as_raw_fd(), libc::F_SETFL, flags))?;
        };
        Ok(())
    }
}

pub fn setup_winch_handler() -> i32 {
    let mut fds = [0; 2];
    unsafe { assert_eq!(0, libc::pipe(fds.as_mut_ptr())) };
    let read = fds[0];
    let write = OwnedFd(fds[1]);
    signal_hook::pipe::register(signal_hook::SIGWINCH, write).unwrap();
    read
}

/// Is this stream a TTY?
pub fn is_tty<T: AsRawFd>(stream: &T) -> bool {
    unsafe { libc::isatty(stream.as_raw_fd()) == 1 }
}

use term_rs::terminfo::parm::Param;

// NOTE: See manpage terminfo
fn paint_screen() {
    use std::char;
    let term = TermInfo::from_env().unwrap();
    let params = [Param::Number(0),Param::Number(0)];

    unsafe {
        let attrs = get_terminal_attr(1).unwrap();
        let mut attrs_new = attrs.clone();
        raw_terminal_attr(&mut attrs_new);
        set_terminal_attr(1, &attrs_new).unwrap();
        // Enable alternate screen buffer.
        term.apply_cap("smcup", &[], &mut std::io::stdout()).unwrap();

        let (cols, rows) = terminal_size().unwrap();

        // Move cursor
        term.apply_cap("cup", &params, &mut std::io::stdout()).unwrap();

        let mut s = String::new();
        for _ in 0..(rows+1) {
            for v in  0..cols {
                s.push('.');
            }
            s.push('\r');
            s.push('\n');
            print!("{}", s);
            term.apply_cap("cup", &params, &mut std::io::stdout()).unwrap();
        }

        use std::io::prelude::*;
        io::stdout().flush().unwrap();
        // Move cursor
        term.apply_cap("cup", &params, &mut std::io::stdout()).unwrap();
        io::stdout().flush().unwrap();

        //std::thread::sleep_ms(3000);

        // Restore attrs
        term.apply_cap("rmcup", &[], &mut std::io::stdout()).unwrap();
        set_terminal_attr(1, &attrs).unwrap();
    }

}

use std::os::unix::prelude::*;
use exports::io::prelude::*;
async fn test_detect_size_as(mut fd: term::io::raw::AsyncRawFd) {
    let mut buf = [0u8;16];

    loop{
        fd.read(&mut buf).await.unwrap();
        let (row,col) = terminal_size().unwrap();
        println!("{} x {}", row, col);
    }
}

fn test_detect_size() {
    let readfd = setup_winch_handler();
    let readfd = unsafe {term::io::raw::RawFd::from_raw_fd(readfd)};
    let readfd = term::io::raw::AsyncRawFd::new(readfd);

    let mut rt = exports::runtime::Runtime::new().unwrap();
    rt.block_on(test_detect_size_as(readfd));
}


/// This struct defines a drawable Window of the terminal
pub struct WindowSegment {
    // Space to store graphemes
    cols : usize,
    rows : usize,
    // NOTE: The rope structure doesn't impose size limitations, our cols and rows components do.
    pub rope : xi_rope::Rope,
}

use std::str::FromStr;

impl WindowSegment {
    fn graphs(&self) -> (usize, usize) {
        return (self.rows, self.cols)
    }
}

trait Draw {
    fn draw(self: &Self, window: &mut WindowSegment); 
}
struct CountDrawer {
}
impl Draw for CountDrawer {
    fn draw(self: &Self, window: &mut WindowSegment) {
        let (rows, cols) = window.graphs();
        let buffer = vec!['.' as u8; rows*cols];
        let slice = std::str::from_utf8(&buffer).unwrap();
        window.rope.edit(0..buffer.len(), slice);
    }
}

pub struct Point {
    pub x: usize,
    pub y: usize,
}

fn move_cursor(term: &mut TermInfo, loc: &Point) {
    let params = [Param::Number(loc.y as i32),Param::Number(loc.x as i32)];
    term.apply_cap("cup", &params, &mut std::io::stdout()).unwrap();
}

impl WindowSegment {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            cols: cols,
            rows: rows,
            rope: xi_rope::Rope::default(),
        }
    }
    // TODO Terminal as arg
    fn paint(self: &Self, start: &Point) {
        // TODO Move cursor to start point
        let mut term = TermInfo::from_env().unwrap();
        let mut rope_cur = xi_rope::Cursor::new(&self.rope, 0);

        for row in 0..self.rows {
            move_cursor(&mut term, &Point {x: start.x, y: start.y+row});

            let start = rope_cur.pos();
            for _ in 0..self.cols {
                // Gather graphemes in this row
                match rope_cur.next_grapheme() {
                    Some(idx) => rope_cur.set(idx),
                    None => panic!(),
                };

            }
            //print!("{},{}", start, rope_cur.pos());
            print!("{}", self.rope.slice(start..rope_cur.pos()));
        }
        io::stdout().flush().unwrap();
    }
}

fn paint_part() {
    let mut seg = WindowSegment::new(10,10);
    let d = CountDrawer{};
    d.draw(&mut seg);
    seg.paint(&Point{x:5,y:5});
}

fn main() {
    //paint_screen();
    paint_part();
    //test_detect_size();

    // TODO Paint screen method
    
    println!("Hello, world!");
}
