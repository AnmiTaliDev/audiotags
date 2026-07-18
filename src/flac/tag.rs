use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::flac::block::{
    Application, Block, CueSheet, Picture, PictureType, SeekTable, StreamInfo, VorbisComments,
};
use crate::flac::error::FlacError;

const MARKER: &[u8; 4] = b"fLaC";

#[derive(Debug, Clone, Default)]
pub struct Tag {
    blocks: Vec<Block>,
}

impl Tag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_from_path(path: impl AsRef<Path>) -> Result<Self, FlacError> {
        let mut file = File::open(path)?;
        let mut marker = [0u8; 4];
        file.read_exact(&mut marker)?;
        if &marker != MARKER {
            return Err(FlacError::NotFlac);
        }

        let mut blocks = Vec::new();
        loop {
            let mut header = [0u8; 4];
            file.read_exact(&mut header)?;
            let is_last = header[0] & 0x80 != 0;
            let block_type = header[0] & 0x7f;
            let length = u32::from_be_bytes([0, header[1], header[2], header[3]]) as usize;
            let mut data = vec![0u8; length];
            file.read_exact(&mut data)?;
            blocks.push(Block::parse(block_type, data)?);
            if is_last {
                break;
            }
        }
        Ok(Self { blocks })
    }

    fn locate_audio_start(original: &[u8]) -> Result<usize, FlacError> {
        if original.len() < 4 || &original[0..4] != MARKER {
            return Err(FlacError::NotFlac);
        }
        let mut offset = 4;
        loop {
            if offset + 4 > original.len() {
                return Err(FlacError::Truncated);
            }
            let is_last = original[offset] & 0x80 != 0;
            let length = u32::from_be_bytes([
                0,
                original[offset + 1],
                original[offset + 2],
                original[offset + 3],
            ]) as usize;
            offset += 4 + length;
            if is_last {
                break;
            }
        }
        Ok(offset)
    }

    pub fn write_to(&self, file: &mut File) -> Result<(), FlacError> {
        file.seek(SeekFrom::Start(0))?;
        let mut original = Vec::new();
        file.read_to_end(&mut original)?;
        let audio_start = Self::locate_audio_start(&original)?;
        let audio = &original[audio_start..];

        let mut out = Vec::new();
        out.extend_from_slice(MARKER);
        let last_index = self.blocks.len().saturating_sub(1);
        for (i, block) in self.blocks.iter().enumerate() {
            block.write(&mut out, i == last_index);
        }
        out.extend_from_slice(audio);

        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)?;
        file.write_all(&out)?;
        file.flush()?;
        Ok(())
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), FlacError> {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        self.write_to(&mut file)
    }

    pub fn vorbis_comments(&self) -> Option<&VorbisComments> {
        self.blocks.iter().find_map(|b| match b {
            Block::VorbisComment(vc) => Some(vc),
            _ => None,
        })
    }

    pub fn vorbis_comments_mut(&mut self) -> &mut VorbisComments {
        let idx = match self
            .blocks
            .iter()
            .position(|b| matches!(b, Block::VorbisComment(_)))
        {
            Some(idx) => idx,
            None => {
                let insert_at = usize::from(matches!(self.blocks.first(), Some(Block::StreamInfo(_))));
                self.blocks
                    .insert(insert_at, Block::VorbisComment(VorbisComments::default()));
                insert_at
            }
        };
        match &mut self.blocks[idx] {
            Block::VorbisComment(vc) => vc,
            _ => unreachable!(),
        }
    }

    pub fn duration(&self) -> Option<f64> {
        self.blocks.iter().find_map(|b| match b {
            Block::StreamInfo(info) => info.duration(),
            _ => None,
        })
    }

    pub fn pictures(&self) -> impl Iterator<Item = &Picture> {
        self.blocks.iter().filter_map(|b| match b {
            Block::Picture(pic) => Some(pic),
            _ => None,
        })
    }

    pub fn add_picture(&mut self, mime_type: String, picture_type: PictureType, data: Vec<u8>) {
        self.remove_picture_type(picture_type);
        self.blocks.push(Block::Picture(Picture {
            picture_type,
            mime_type,
            description: String::new(),
            width: 0,
            height: 0,
            color_depth: 0,
            colors_used: 0,
            data,
        }));
    }

    pub fn remove_picture_type(&mut self, picture_type: PictureType) {
        self.blocks
            .retain(|b| !matches!(b, Block::Picture(p) if p.picture_type == picture_type));
    }

    pub fn stream_info(&self) -> Option<&StreamInfo> {
        self.blocks.iter().find_map(|b| match b {
            Block::StreamInfo(info) => Some(info),
            _ => None,
        })
    }

    pub fn cuesheet(&self) -> Option<&CueSheet> {
        self.blocks.iter().find_map(|b| match b {
            Block::CueSheet(cs) => Some(cs),
            _ => None,
        })
    }

    pub fn seektable(&self) -> Option<&SeekTable> {
        self.blocks.iter().find_map(|b| match b {
            Block::SeekTable(table) => Some(table),
            _ => None,
        })
    }

    pub fn applications(&self) -> impl Iterator<Item = &Application> {
        self.blocks.iter().filter_map(|b| match b {
            Block::Application(app) => Some(app),
            _ => None,
        })
    }

    pub fn padding(&self) -> usize {
        self.blocks
            .iter()
            .map(|b| match b {
                Block::Padding(size) => *size,
                _ => 0,
            })
            .sum()
    }
}
