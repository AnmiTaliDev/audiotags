mod application;
mod cuesheet;
mod padding;
mod picture;
mod seektable;
mod streaminfo;
mod vorbis_comment;

pub use application::Application;
pub use cuesheet::CueSheet;
pub use picture::{Picture, PictureType};
pub use seektable::SeekTable;
pub use streaminfo::StreamInfo;
pub use vorbis_comment::VorbisComments;

use crate::flac::error::FlacError;

pub(crate) const STREAMINFO: u8 = 0;
pub(crate) const PADDING: u8 = 1;
pub(crate) const APPLICATION: u8 = 2;
pub(crate) const SEEKTABLE: u8 = 3;
pub(crate) const VORBIS_COMMENT: u8 = 4;
pub(crate) const CUESHEET: u8 = 5;
pub(crate) const PICTURE: u8 = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Block {
    StreamInfo(StreamInfo),
    Padding(usize),
    Application(Application),
    SeekTable(SeekTable),
    VorbisComment(VorbisComments),
    CueSheet(CueSheet),
    Picture(Picture),
    Reserved { block_type: u8, data: Vec<u8> },
}

impl Block {
    pub(crate) fn parse(block_type: u8, data: Vec<u8>) -> Result<Self, FlacError> {
        Ok(match block_type {
            STREAMINFO => Block::StreamInfo(streaminfo::parse(&data)?),
            PADDING => Block::Padding(data.len()),
            APPLICATION => Block::Application(application::parse(&data)?),
            SEEKTABLE => Block::SeekTable(seektable::parse(&data)?),
            VORBIS_COMMENT => Block::VorbisComment(vorbis_comment::parse(&data)?),
            CUESHEET => Block::CueSheet(cuesheet::parse(&data)?),
            PICTURE => Block::Picture(picture::parse(&data)?),
            other => Block::Reserved {
                block_type: other,
                data,
            },
        })
    }

    fn block_type(&self) -> u8 {
        match self {
            Block::StreamInfo(_) => STREAMINFO,
            Block::Padding(_) => PADDING,
            Block::Application(_) => APPLICATION,
            Block::SeekTable(_) => SEEKTABLE,
            Block::VorbisComment(_) => VORBIS_COMMENT,
            Block::CueSheet(_) => CUESHEET,
            Block::Picture(_) => PICTURE,
            Block::Reserved { block_type, .. } => *block_type,
        }
    }

    fn payload(&self) -> Vec<u8> {
        match self {
            Block::StreamInfo(info) => streaminfo::write(info),
            Block::Padding(size) => padding::write(*size),
            Block::Application(app) => application::write(app),
            Block::SeekTable(table) => seektable::write(table),
            Block::VorbisComment(vc) => vorbis_comment::write(vc),
            Block::CueSheet(cs) => cuesheet::write(cs),
            Block::Picture(pic) => picture::write(pic),
            Block::Reserved { data, .. } => data.clone(),
        }
    }

    pub(crate) fn write(&self, out: &mut Vec<u8>, is_last: bool) {
        let payload = self.payload();
        let len = payload.len() as u32;
        out.push(self.block_type() | if is_last { 0x80 } else { 0x00 });
        out.extend_from_slice(&len.to_be_bytes()[1..]);
        out.extend_from_slice(&payload);
    }
}
