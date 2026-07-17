use crate::id3::encoding::{self, TextEncoding};
use crate::id3::error::Id3Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PictureType {
    Other,
    Icon,
    OtherIcon,
    CoverFront,
    CoverBack,
    Leaflet,
    Media,
    LeadArtist,
    Artist,
    Conductor,
    Band,
    Composer,
    Lyricist,
    RecordingLocation,
    DuringRecording,
    DuringPerformance,
    ScreenCapture,
    BrightFish,
    Illustration,
    BandLogo,
    PublisherLogo,
    Undefined(u8),
}

impl PictureType {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::Other,
            0x01 => Self::Icon,
            0x02 => Self::OtherIcon,
            0x03 => Self::CoverFront,
            0x04 => Self::CoverBack,
            0x05 => Self::Leaflet,
            0x06 => Self::Media,
            0x07 => Self::LeadArtist,
            0x08 => Self::Artist,
            0x09 => Self::Conductor,
            0x0a => Self::Band,
            0x0b => Self::Composer,
            0x0c => Self::Lyricist,
            0x0d => Self::RecordingLocation,
            0x0e => Self::DuringRecording,
            0x0f => Self::DuringPerformance,
            0x10 => Self::ScreenCapture,
            0x11 => Self::BrightFish,
            0x12 => Self::Illustration,
            0x13 => Self::BandLogo,
            0x14 => Self::PublisherLogo,
            other => Self::Undefined(other),
        }
    }

    fn as_byte(self) -> u8 {
        match self {
            Self::Other => 0x00,
            Self::Icon => 0x01,
            Self::OtherIcon => 0x02,
            Self::CoverFront => 0x03,
            Self::CoverBack => 0x04,
            Self::Leaflet => 0x05,
            Self::Media => 0x06,
            Self::LeadArtist => 0x07,
            Self::Artist => 0x08,
            Self::Conductor => 0x09,
            Self::Band => 0x0a,
            Self::Composer => 0x0b,
            Self::Lyricist => 0x0c,
            Self::RecordingLocation => 0x0d,
            Self::DuringRecording => 0x0e,
            Self::DuringPerformance => 0x0f,
            Self::ScreenCapture => 0x10,
            Self::BrightFish => 0x11,
            Self::Illustration => 0x12,
            Self::BandLogo => 0x13,
            Self::PublisherLogo => 0x14,
            Self::Undefined(other) => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    pub mime_type: String,
    pub picture_type: PictureType,
    pub description: String,
    pub data: Vec<u8>,
}

impl Picture {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, Id3Error> {
        let (&encoding_byte, rest) = bytes.split_first().ok_or(Id3Error::Truncated)?;
        let encoding = TextEncoding::from_byte(encoding_byte)?;
        let mime_end = rest
            .iter()
            .position(|&b| b == 0x00)
            .ok_or(Id3Error::Truncated)?;
        let mime_type = std::str::from_utf8(&rest[..mime_end])
            .map_err(|_| Id3Error::InvalidUtf8)?
            .to_owned();
        Self::finish_parsing(encoding, mime_type, &rest[mime_end + 1..])
    }

    pub(crate) fn parse_v2(bytes: &[u8]) -> Result<Self, Id3Error> {
        let (&encoding_byte, rest) = bytes.split_first().ok_or(Id3Error::Truncated)?;
        let encoding = TextEncoding::from_byte(encoding_byte)?;
        if rest.len() < 3 {
            return Err(Id3Error::Truncated);
        }
        let mime_type = match &rest[..3] {
            b"PNG" => "image/png",
            b"JPG" => "image/jpeg",
            b"BMP" => "image/bmp",
            b"GIF" => "image/gif",
            b"TIF" => "image/tiff",
            _ => "application/octet-stream",
        }
        .to_owned();
        Self::finish_parsing(encoding, mime_type, &rest[3..])
    }

    fn finish_parsing(
        encoding: TextEncoding,
        mime_type: String,
        rest: &[u8],
    ) -> Result<Self, Id3Error> {
        let (&picture_type_byte, rest) = rest.split_first().ok_or(Id3Error::Truncated)?;
        let picture_type = PictureType::from_byte(picture_type_byte);
        let (description_bytes, data) = encoding::split_terminated(encoding, rest);
        let description = encoding::decode_string(encoding, description_bytes)?;
        Ok(Self {
            mime_type,
            picture_type,
            description,
            data: data.to_vec(),
        })
    }

    pub(crate) fn write(&self) -> Vec<u8> {
        let encoding = TextEncoding::Utf8;
        let mut out = vec![encoding.as_byte()];
        out.extend_from_slice(self.mime_type.as_bytes());
        out.push(0x00);
        out.push(self.picture_type.as_byte());
        out.extend(encoding::encode_string(encoding, &self.description));
        out.push(0x00);
        out.extend_from_slice(&self.data);
        out
    }
}
