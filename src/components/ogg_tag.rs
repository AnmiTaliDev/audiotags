use crate::*;
use oggvorbismeta::{read_comment_header, replace_comment_header, CommentHeader, VorbisComments};
use std::fs::OpenOptions;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::path::Path;
use std::str::FromStr;

pub struct OggInnerTag {
    header: CommentHeader,
}

impl Default for OggInnerTag {
    fn default() -> Self {
        Self {
            header: CommentHeader::new(),
        }
    }
}

impl OggInnerTag {
    pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Self> {
        let f = File::open(path)?;
        let header = read_comment_header(f)?;
        Ok(Self { header })
    }
}

impl_tag!(OggTag, OggInnerTag, TagType::Ogg);

impl<'a> From<AnyTag<'a>> for OggTag {
    fn from(inp: AnyTag<'a>) -> Self {
        let mut t = OggTag::default();
        if let Some(v) = inp.title() {
            t.set_title(v)
        }
        if let Some(v) = inp.artists_as_string() {
            t.set_artist(&v)
        }
        if let Some(v) = inp.date {
            t.set_date(v)
        }
        if let Some(v) = inp.year {
            t.set_year(v)
        }
        if let Some(v) = inp.album_title() {
            t.set_album_title(v)
        }
        if let Some(v) = inp.album_artists_as_string() {
            t.set_album_artist(&v)
        }
        if let Some(v) = inp.track_number() {
            t.set_track_number(v)
        }
        if let Some(v) = inp.total_tracks() {
            t.set_total_tracks(v)
        }
        if let Some(v) = inp.disc_number() {
            t.set_disc_number(v)
        }
        if let Some(v) = inp.total_discs() {
            t.set_total_discs(v)
        }
        t
    }
}

impl<'a> From<&'a OggTag> for AnyTag<'a> {
    fn from(inp: &'a OggTag) -> Self {
        Self {
            title: inp.title(),
            artists: inp.artists(),
            date: inp.date(),
            year: inp.year(),
            duration: inp.duration(),
            album_title: inp.album_title(),
            album_artists: inp.album_artists(),
            album_cover: inp.album_cover(),
            track_number: inp.track_number(),
            total_tracks: inp.total_tracks(),
            disc_number: inp.disc_number(),
            total_discs: inp.total_discs(),
            genre: inp.genre(),
            composer: inp.composer(),
            comment: inp.comment(),
            ..Self::default()
        }
    }
}

impl OggTag {
    fn get_first(&self, key: &str) -> Option<&str> {
        self.inner
            .header
            .comment_list
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    fn set_first(&mut self, key: &str, val: &str) {
        self.inner.header.clear_tag(key);
        self.inner.header.add_tag_single(key, val);
    }

    fn remove(&mut self, key: &str) {
        self.inner.header.clear_tag(key);
    }
}

impl AudioTagEdit for OggTag {
    fn title(&self) -> Option<&str> {
        self.get_first("title")
    }
    fn set_title(&mut self, title: &str) {
        self.set_first("title", title);
    }
    fn remove_title(&mut self) {
        self.remove("title");
    }

    fn artist(&self) -> Option<&str> {
        self.get_first("artist")
    }
    fn set_artist(&mut self, artist: &str) {
        self.set_first("artist", artist);
    }
    fn remove_artist(&mut self) {
        self.remove("artist");
    }

    fn date(&self) -> Option<Timestamp> {
        self.get_first("date")
            .and_then(|s| Timestamp::from_str(s).ok())
    }
    fn set_date(&mut self, date: Timestamp) {
        self.set_first("date", &date.to_string());
    }
    fn remove_date(&mut self) {
        self.remove("date");
    }

    fn year(&self) -> Option<i32> {
        self.get_first("year")
            .and_then(|s| s.parse().ok())
            .or_else(|| {
                self.get_first("date")
                    .and_then(|s| s.chars().take(4).collect::<String>().parse().ok())
            })
    }
    fn set_year(&mut self, year: i32) {
        self.set_first("year", &year.to_string());
    }
    fn remove_year(&mut self) {
        self.remove("year");
        self.remove("date");
    }

    fn duration(&self) -> Option<f64> {
        None
    }

    fn album_title(&self) -> Option<&str> {
        self.get_first("album")
    }
    fn set_album_title(&mut self, title: &str) {
        self.set_first("album", title);
    }
    fn remove_album_title(&mut self) {
        self.remove("album");
    }

    fn album_artist(&self) -> Option<&str> {
        self.get_first("albumartist")
    }
    fn set_album_artist(&mut self, v: &str) {
        self.set_first("albumartist", v);
    }
    fn remove_album_artist(&mut self) {
        self.remove("albumartist");
    }

    fn album_cover(&self) -> Option<Picture<'_>> {
        None
    }
    fn set_album_cover(&mut self, _cover: Picture) {}
    fn remove_album_cover(&mut self) {}

    fn track_number(&self) -> Option<u16> {
        self.get_first("tracknumber")
            .and_then(|s| s.parse().ok())
    }
    fn set_track_number(&mut self, v: u16) {
        self.set_first("tracknumber", &v.to_string());
    }
    fn remove_track_number(&mut self) {
        self.remove("tracknumber");
    }

    fn total_tracks(&self) -> Option<u16> {
        self.get_first("totaltracks").and_then(|s| s.parse().ok())
    }
    fn set_total_tracks(&mut self, v: u16) {
        self.set_first("totaltracks", &v.to_string());
    }
    fn remove_total_tracks(&mut self) {
        self.remove("totaltracks");
    }

    fn disc_number(&self) -> Option<u16> {
        self.get_first("discnumber").and_then(|s| s.parse().ok())
    }
    fn set_disc_number(&mut self, v: u16) {
        self.set_first("discnumber", &v.to_string());
    }
    fn remove_disc_number(&mut self) {
        self.remove("discnumber");
    }

    fn total_discs(&self) -> Option<u16> {
        self.get_first("totaldiscs").and_then(|s| s.parse().ok())
    }
    fn set_total_discs(&mut self, v: u16) {
        self.set_first("totaldiscs", &v.to_string());
    }
    fn remove_total_discs(&mut self) {
        self.remove("totaldiscs");
    }

    fn genre(&self) -> Option<&str> {
        self.get_first("genre")
    }
    fn set_genre(&mut self, v: &str) {
        self.set_first("genre", v);
    }
    fn remove_genre(&mut self) {
        self.remove("genre");
    }

    fn composer(&self) -> Option<&str> {
        self.get_first("composer")
    }
    fn set_composer(&mut self, v: String) {
        self.set_first("composer", &v);
    }
    fn remove_composer(&mut self) {
        self.remove("composer");
    }

    fn comment(&self) -> Option<&str> {
        self.get_first("comment")
    }
    fn set_comment(&mut self, v: String) {
        self.set_first("comment", &v);
    }
    fn remove_comment(&mut self) {
        self.remove("comment");
    }
}

impl AudioTagWrite for OggTag {
    fn write_to(&mut self, file: &mut File) -> crate::Result<()> {
        file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let mut new_content = replace_comment_header(Cursor::new(&bytes), &self.inner.header)?;
        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)?;
        std::io::copy(&mut new_content, file)?;
        Ok(())
    }
    fn write_to_path(&mut self, path: &str) -> crate::Result<()> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;
        self.write_to(&mut file)
    }
}
