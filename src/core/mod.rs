pub mod directories;
pub mod files;
pub mod io;

pub use directories::Directory;
pub use files::{File, FileChunk};
pub use io::Io;

pub trait Memorable {
    fn memorize(&self) -> String;
}
