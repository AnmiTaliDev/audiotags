mod comment;
mod content;
mod picture;
mod text;

pub use comment::Comment;
pub use content::Content;
pub use picture::{Picture, PictureType};

use crate::id3::error::Id3Error;
use crate::id3::header::{decode_synchsafe, encode_synchsafe};
use crate::id3::unsynch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub id: String,
    pub content: Content,
}

impl Frame {
    pub fn text(id: &str, value: impl Into<String>) -> Self {
        Self {
            id: id.to_owned(),
            content: Content::Text(value.into()),
        }
    }

    pub fn content(&self) -> &Content {
        &self.content
    }

    pub(crate) fn parse(major_version: u8, data: &[u8]) -> Result<Option<(Self, usize)>, Id3Error> {
        if major_version == 2 {
            parse_v2(data)
        } else {
            parse_v34(major_version, data)
        }
    }

    pub(crate) fn write(&self, out: &mut Vec<u8>) {
        if self.id.len() != 4 {
            return;
        }
        let raw_body = write_content(&self.content);
        let escaped_body = unsynch::encode(&raw_body);
        let unsynchronised = escaped_body.len() != raw_body.len();
        let body = if unsynchronised { &escaped_body } else { &raw_body };
        out.extend_from_slice(self.id.as_bytes());
        out.extend_from_slice(&encode_synchsafe(body.len() as u32));
        out.extend_from_slice(&[0, if unsynchronised { 0x02 } else { 0x00 }]);
        out.extend_from_slice(body);
    }
}

impl From<Picture> for Frame {
    fn from(picture: Picture) -> Self {
        Self {
            id: "APIC".to_owned(),
            content: Content::Picture(picture),
        }
    }
}

impl From<Comment> for Frame {
    fn from(comment: Comment) -> Self {
        Self {
            id: "COMM".to_owned(),
            content: Content::Comment(comment),
        }
    }
}

fn parse_v2(data: &[u8]) -> Result<Option<(Frame, usize)>, Id3Error> {
    const HEADER_SIZE: usize = 6;
    if data.len() < HEADER_SIZE || data[0] == 0 {
        return Ok(None);
    }
    let raw_id = &data[0..3];
    let size = u32::from_be_bytes([0, data[3], data[4], data[5]]) as usize;
    let total = HEADER_SIZE + size;
    if data.len() < total {
        return Err(Id3Error::Truncated);
    }
    let body = &data[HEADER_SIZE..total];
    let id = upgrade_v2_id(raw_id);
    let content = if raw_id == b"PIC" {
        Picture::parse_v2(body).map(Content::Picture)?
    } else {
        parse_content(&id, body)?
    };
    Ok(Some((Frame { id, content }, total)))
}

fn parse_v34(major_version: u8, data: &[u8]) -> Result<Option<(Frame, usize)>, Id3Error> {
    const HEADER_SIZE: usize = 10;
    if data.len() < HEADER_SIZE || data[0] == 0 {
        return Ok(None);
    }
    let id = String::from_utf8_lossy(&data[0..4]).into_owned();
    let size_bytes: [u8; 4] = data[4..8].try_into().unwrap();
    let size = if major_version >= 4 {
        decode_synchsafe(size_bytes)
    } else {
        u32::from_be_bytes(size_bytes)
    } as usize;
    let flags = u16::from_be_bytes([data[8], data[9]]);
    let total = HEADER_SIZE + size;
    if data.len() < total {
        return Err(Id3Error::Truncated);
    }
    let raw_body = &data[HEADER_SIZE..total];
    let body = if major_version >= 4 && flags & 0x0002 != 0 {
        unsynch::decode(raw_body)
    } else {
        raw_body.to_vec()
    };
    let content = parse_content(&id, &body)?;
    Ok(Some((Frame { id, content }, total)))
}

fn parse_content(id: &str, body: &[u8]) -> Result<Content, Id3Error> {
    Ok(match id {
        "APIC" => Content::Picture(Picture::parse(body)?),
        "COMM" => Content::Comment(Comment::parse(body)?),
        "TXXX" => Content::ExtendedText(text::parse_extended_text(body)?),
        id if id.starts_with('T') => Content::Text(text::parse_text(body)?),
        _ => Content::Unknown(body.to_vec()),
    })
}

fn write_content(content: &Content) -> Vec<u8> {
    match content {
        Content::Text(value) => text::write_text(value),
        Content::ExtendedText(ext) => text::write_extended_text(ext),
        Content::Comment(comment) => comment.write(),
        Content::Picture(picture) => picture.write(),
        Content::Unknown(bytes) => bytes.clone(),
    }
}

fn upgrade_v2_id(raw: &[u8]) -> String {
    let upgraded: Option<&str> = match raw {
        b"TT1" => Some("TIT1"),
        b"TT2" => Some("TIT2"),
        b"TT3" => Some("TIT3"),
        b"TP1" => Some("TPE1"),
        b"TP2" => Some("TPE2"),
        b"TP3" => Some("TPE3"),
        b"TP4" => Some("TPE4"),
        b"TCM" => Some("TCOM"),
        b"TXT" => Some("TEXT"),
        b"TLA" => Some("TLAN"),
        b"TCO" => Some("TCON"),
        b"TAL" => Some("TALB"),
        b"TPA" => Some("TPOS"),
        b"TRK" => Some("TRCK"),
        b"TRC" => Some("TSRC"),
        b"TYE" => Some("TYER"),
        b"TDA" => Some("TDAT"),
        b"TIM" => Some("TIME"),
        b"TMT" => Some("TMED"),
        b"TFT" => Some("TFLT"),
        b"TBP" => Some("TBPM"),
        b"TCR" => Some("TCOP"),
        b"TPB" => Some("TPUB"),
        b"TEN" => Some("TENC"),
        b"TSS" => Some("TSSE"),
        b"TLE" => Some("TLEN"),
        b"TSI" => Some("TSIZ"),
        b"TDY" => Some("TDLY"),
        b"TKE" => Some("TKEY"),
        b"TOT" => Some("TOAL"),
        b"TOA" => Some("TOPE"),
        b"TOL" => Some("TOLY"),
        b"TOR" => Some("TORY"),
        b"TXX" => Some("TXXX"),
        b"COM" => Some("COMM"),
        b"PIC" => Some("APIC"),
        b"UFI" => Some("UFID"),
        b"ULT" => Some("USLT"),
        b"MCI" => Some("MCDI"),
        b"POP" => Some("POPM"),
        b"GEO" => Some("GEOB"),
        b"CNT" => Some("PCNT"),
        b"CRA" => Some("AENC"),
        _ => None,
    };
    match upgraded {
        Some(id) => id.to_owned(),
        None => String::from_utf8_lossy(raw).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_text_frame() {
        let frame = Frame::text("TIT2", "song title");
        let mut out = Vec::new();
        frame.write(&mut out);
        let (parsed, consumed) = Frame::parse(4, &out).unwrap().unwrap();
        assert_eq!(consumed, out.len());
        assert_eq!(parsed, frame);
    }

    #[test]
    fn parses_v2_upgraded_ids() {
        let mut data = Vec::new();
        data.extend_from_slice(b"TP1");
        let body = text::write_text("artist name");
        data.extend_from_slice(&(body.len() as u32).to_be_bytes()[1..]);
        data.extend_from_slice(&body);
        let (frame, consumed) = Frame::parse(2, &data).unwrap().unwrap();
        assert_eq!(consumed, data.len());
        assert_eq!(frame.id, "TPE1");
        assert_eq!(frame.content, Content::Text("artist name".to_owned()));
    }

    #[test]
    fn stops_at_padding() {
        let data = [0u8; 20];
        assert!(Frame::parse(4, &data).unwrap().is_none());
    }
}
