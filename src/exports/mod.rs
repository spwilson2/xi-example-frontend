/// This file contains redefinitions of particular dependencies which might be swapped in or out.
///


pub mod sync {
    pub use tokio::sync::*;
}
pub mod runtime {
    pub use tokio::runtime::*;
    pub use tokio::task::JoinHandle as Promise;
}
pub mod io {
    pub use futures::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
    pub mod prelude {
        pub use futures::prelude::*;
    }

}

