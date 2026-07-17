use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::id3::error::Id3Error;
use crate::id3::frame::{Comment, Content, Frame, Picture, PictureType};
use crate::id3::header::{self, decode_synchsafe, TagHeader};
use crate::id3::unsynch;
use crate::Timestamp;

#[derive(Debug, Clone, Default)]
pub struct Tag {
    frames: Vec<Frame>,
}

impl Tag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_from_path(path: impl AsRef<Path>) -> Result<Self, Id3Error> {
        let mut file = File::open(path)?;
        let mut header_bytes = [0u8; header::HEADER_SIZE];
        if file.read(&mut header_bytes)? < header::HEADER_SIZE {
            return Ok(Self::default());
        }
        let header = match TagHeader::parse(&header_bytes) {
            Ok(header) => header,
            Err(Id3Error::NoTag) => return Ok(Self::default()),
            Err(err) => return Err(err),
        };
        let mut body = vec![0u8; header.tag_size as usize];
        file.read_exact(&mut body)?;
        Self::parse_body(&header, &body)
    }

    fn parse_body(header: &TagHeader, body: &[u8]) -> Result<Self, Id3Error> {
        let mut offset = 0;
        if header.extended_header {
            if body.len() < 4 {
                return Err(Id3Error::Truncated);
            }
            let ext_size_bytes: [u8; 4] = body[0..4].try_into().unwrap();
            let ext_size = if header.major_version >= 4 {
                decode_synchsafe(ext_size_bytes) as usize
            } else {
                u32::from_be_bytes(ext_size_bytes) as usize
            };
            offset = if header.major_version >= 4 {
                ext_size
            } else {
                4 + ext_size
            };
        }
        let mut data = body[offset.min(body.len())..].to_vec();
        if header.unsynchronisation {
            data = unsynch::decode(&data);
        }

        let mut frames = Vec::new();
        let mut cursor = 0;
        while cursor < data.len() {
            match Frame::parse(header.major_version, &data[cursor..])? {
                Some((frame, consumed)) => {
                    frames.push(frame);
                    cursor += consumed;
                }
                None => break,
            }
        }
        Ok(Self { frames })
    }

    pub fn write_to(&self, file: &mut File) -> Result<(), Id3Error> {
        file.seek(SeekFrom::Start(0))?;
        let mut original = Vec::new();
        file.read_to_end(&mut original)?;

        let audio_start = match TagHeader::parse(&original) {
            Ok(header) => header::HEADER_SIZE + header.tag_size as usize,
            Err(_) => 0,
        };
        let audio = &original[audio_start.min(original.len())..];

        let mut body = Vec::new();
        for frame in &self.frames {
            frame.write(&mut body);
        }
        let header_bytes = TagHeader::write(body.len() as u32);

        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)?;
        file.write_all(&header_bytes)?;
        file.write_all(&body)?;
        file.write_all(audio)?;
        file.flush()?;
        Ok(())
    }

    pub fn write_to_path(&self, path: impl AsRef<Path>) -> Result<(), Id3Error> {
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)?;
        self.write_to(&mut file)
    }

    pub fn get(&self, id: &str) -> Option<&Frame> {
        self.frames.iter().find(|f| f.id == id)
    }

    pub fn remove(&mut self, id: &str) {
        self.frames.retain(|f| f.id != id);
    }

    pub fn add_frame(&mut self, frame: impl Into<Frame>) {
        let frame = frame.into();
        match &frame.content {
            Content::Text(_) | Content::ExtendedText(_) => {
                self.frames.retain(|f| f.id != frame.id);
            }
            Content::Comment(comment) => {
                let lang = comment.lang.clone();
                let description = comment.description.clone();
                self.frames.retain(|f| {
                    !(f.id == frame.id
                        && matches!(&f.content, Content::Comment(c) if c.lang == lang && c.description == description))
                });
            }
            Content::Picture(picture) => {
                let picture_type = picture.picture_type;
                self.frames.retain(|f| {
                    !(f.id == frame.id
                        && matches!(&f.content, Content::Picture(p) if p.picture_type == picture_type))
                });
            }
            Content::Unknown(_) => {}
        }
        self.frames.push(frame);
    }

    fn text(&self, id: &str) -> Option<&str> {
        self.frames.iter().find_map(|f| {
            if f.id != id {
                return None;
            }
            match &f.content {
                Content::Text(value) => Some(value.as_str()),
                _ => None,
            }
        })
    }

    fn set_text(&mut self, id: &str, value: &str) {
        self.add_frame(Frame::text(id, value));
    }

    fn number_pair(&self, id: &str) -> (Option<u32>, Option<u32>) {
        match self.text(id) {
            Some(value) => {
                let mut parts = value.splitn(2, '/');
                let first = parts.next().and_then(|s| s.trim().parse().ok());
                let second = parts.next().and_then(|s| s.trim().parse().ok());
                (first, second)
            }
            None => (None, None),
        }
    }

    fn set_number_pair(&mut self, id: &str, first: Option<u32>, second: Option<u32>) {
        let value = match (first, second) {
            (Some(a), Some(b)) => format!("{a}/{b}"),
            (Some(a), None) => a.to_string(),
            (None, Some(b)) => format!("/{b}"),
            (None, None) => {
                self.remove(id);
                return;
            }
        };
        self.set_text(id, &value);
    }

    pub fn title(&self) -> Option<&str> {
        self.text("TIT2")
    }
    pub fn set_title(&mut self, value: &str) {
        self.set_text("TIT2", value);
    }
    pub fn remove_title(&mut self) {
        self.remove("TIT2");
    }

    pub fn artist(&self) -> Option<&str> {
        self.text("TPE1")
    }
    pub fn set_artist(&mut self, value: &str) {
        self.set_text("TPE1", value);
    }
    pub fn remove_artist(&mut self) {
        self.remove("TPE1");
    }

    pub fn album(&self) -> Option<&str> {
        self.text("TALB")
    }
    pub fn set_album(&mut self, value: &str) {
        self.set_text("TALB", value);
    }
    pub fn remove_album(&mut self) {
        self.remove("TALB");
    }

    pub fn album_artist(&self) -> Option<&str> {
        self.text("TPE2")
    }
    pub fn set_album_artist(&mut self, value: &str) {
        self.set_text("TPE2", value);
    }
    pub fn remove_album_artist(&mut self) {
        self.remove("TPE2");
    }

    pub fn genre(&self) -> Option<&str> {
        self.text("TCON")
    }
    pub fn set_genre(&mut self, value: &str) {
        self.set_text("TCON", value);
    }
    pub fn remove_genre(&mut self) {
        self.remove("TCON");
    }

    pub fn date_recorded(&self) -> Option<Timestamp> {
        if let Some(text) = self.text("TDRC")
            && let Ok(ts) = text.parse()
        {
            return Some(ts);
        }
        let year: i32 = self.text("TYER")?.parse().ok()?;
        let mut ts = Timestamp {
            year,
            ..Timestamp::default()
        };
        if let Some(tdat) = self.text("TDAT")
            && tdat.len() == 4
        {
            ts.day = tdat[0..2].parse().ok();
            ts.month = tdat[2..4].parse().ok();
        }
        if let Some(time) = self.text("TIME")
            && time.len() == 4
        {
            ts.hour = time[0..2].parse().ok();
            ts.minute = time[2..4].parse().ok();
        }
        Some(ts)
    }
    pub fn set_date_recorded(&mut self, timestamp: Timestamp) {
        self.set_text("TDRC", &timestamp.to_string());
        self.remove("TDAT");
        self.remove("TIME");
    }
    pub fn remove_date_recorded(&mut self) {
        self.remove("TDRC");
        self.remove("TDAT");
        self.remove("TIME");
    }

    pub fn year(&self) -> Option<i32> {
        if let Some(text) = self.text("TYER")
            && let Ok(year) = text.parse()
        {
            return Some(year);
        }
        self.date_recorded().map(|ts| ts.year)
    }
    pub fn set_year(&mut self, year: i32) {
        self.set_text("TYER", &year.to_string());
    }
    pub fn remove_year(&mut self) {
        self.remove("TYER");
    }

    pub fn duration(&self) -> Option<u32> {
        self.text("TLEN")?.parse::<u32>().ok().map(|ms| ms / 1000)
    }

    pub fn track(&self) -> Option<u32> {
        self.number_pair("TRCK").0
    }
    pub fn set_track(&mut self, track: u32) {
        let total = self.total_tracks();
        self.set_number_pair("TRCK", Some(track), total);
    }
    pub fn remove_track(&mut self) {
        match self.total_tracks() {
            Some(total) => self.set_number_pair("TRCK", None, Some(total)),
            None => self.remove("TRCK"),
        }
    }

    pub fn total_tracks(&self) -> Option<u32> {
        self.number_pair("TRCK").1
    }
    pub fn set_total_tracks(&mut self, total: u32) {
        let track = self.track();
        self.set_number_pair("TRCK", track, Some(total));
    }
    pub fn remove_total_tracks(&mut self) {
        match self.track() {
            Some(track) => self.set_number_pair("TRCK", Some(track), None),
            None => self.remove("TRCK"),
        }
    }

    pub fn disc(&self) -> Option<u32> {
        self.number_pair("TPOS").0
    }
    pub fn set_disc(&mut self, disc: u32) {
        let total = self.total_discs();
        self.set_number_pair("TPOS", Some(disc), total);
    }
    pub fn remove_disc(&mut self) {
        match self.total_discs() {
            Some(total) => self.set_number_pair("TPOS", None, Some(total)),
            None => self.remove("TPOS"),
        }
    }

    pub fn total_discs(&self) -> Option<u32> {
        self.number_pair("TPOS").1
    }
    pub fn set_total_discs(&mut self, total: u32) {
        let disc = self.disc();
        self.set_number_pair("TPOS", disc, Some(total));
    }
    pub fn remove_total_discs(&mut self) {
        match self.disc() {
            Some(disc) => self.set_number_pair("TPOS", Some(disc), None),
            None => self.remove("TPOS"),
        }
    }

    pub fn pictures(&self) -> impl Iterator<Item = &Picture> {
        self.frames.iter().filter_map(|f| match &f.content {
            Content::Picture(picture) => Some(picture),
            _ => None,
        })
    }
    pub fn remove_picture_by_type(&mut self, picture_type: PictureType) {
        self.frames.retain(|f| {
            !matches!(&f.content, Content::Picture(p) if p.picture_type == picture_type)
        });
    }

    pub fn comments(&self) -> impl Iterator<Item = &Comment> {
        self.frames.iter().filter_map(|f| match &f.content {
            Content::Comment(comment) => Some(comment),
            _ => None,
        })
    }
}
