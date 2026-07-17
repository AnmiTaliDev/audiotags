use super::comment::Comment;
use super::picture::Picture;
use super::text::ExtendedText;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Text(String),
    ExtendedText(ExtendedText),
    Comment(Comment),
    Picture(Picture),
    Unknown(Vec<u8>),
}
