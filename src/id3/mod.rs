mod encoding;
mod error;
mod frame;
mod header;
mod tag;
mod unsynch;

pub use error::Id3Error;
pub use frame::{Comment, Content, Frame, Picture, PictureType};
pub use tag::Tag;
