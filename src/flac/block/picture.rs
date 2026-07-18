use crate::flac::bytes::Reader;
use crate::flac::error::FlacError;

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
    Undefined(u32),
}

impl PictureType {
    fn from_u32(value: u32) -> Self {
        match value {
            0 => Self::Other,
            1 => Self::Icon,
            2 => Self::OtherIcon,
            3 => Self::CoverFront,
            4 => Self::CoverBack,
            5 => Self::Leaflet,
            6 => Self::Media,
            7 => Self::LeadArtist,
            8 => Self::Artist,
            9 => Self::Conductor,
            10 => Self::Band,
            11 => Self::Composer,
            12 => Self::Lyricist,
            13 => Self::RecordingLocation,
            14 => Self::DuringRecording,
            15 => Self::DuringPerformance,
            16 => Self::ScreenCapture,
            17 => Self::BrightFish,
            18 => Self::Illustration,
            19 => Self::BandLogo,
            20 => Self::PublisherLogo,
            other => Self::Undefined(other),
        }
    }

    fn as_u32(self) -> u32 {
        match self {
            Self::Other => 0,
            Self::Icon => 1,
            Self::OtherIcon => 2,
            Self::CoverFront => 3,
            Self::CoverBack => 4,
            Self::Leaflet => 5,
            Self::Media => 6,
            Self::LeadArtist => 7,
            Self::Artist => 8,
            Self::Conductor => 9,
            Self::Band => 10,
            Self::Composer => 11,
            Self::Lyricist => 12,
            Self::RecordingLocation => 13,
            Self::DuringRecording => 14,
            Self::DuringPerformance => 15,
            Self::ScreenCapture => 16,
            Self::BrightFish => 17,
            Self::Illustration => 18,
            Self::BandLogo => 19,
            Self::PublisherLogo => 20,
            Self::Undefined(other) => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    pub picture_type: PictureType,
    pub mime_type: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub color_depth: u32,
    pub colors_used: u32,
    pub data: Vec<u8>,
}

pub(crate) fn parse(data: &[u8]) -> Result<Picture, FlacError> {
    let mut r = Reader::new(data);
    let picture_type = PictureType::from_u32(r.read_u32_be()?);
    let mime_len = r.read_u32_be()? as usize;
    let mime_type = r.read_string(mime_len)?;
    let desc_len = r.read_u32_be()? as usize;
    let description = r.read_string(desc_len)?;
    let width = r.read_u32_be()?;
    let height = r.read_u32_be()?;
    let color_depth = r.read_u32_be()?;
    let colors_used = r.read_u32_be()?;
    let data_len = r.read_u32_be()? as usize;
    let data = r.read_bytes(data_len)?.to_vec();
    Ok(Picture {
        picture_type,
        mime_type,
        description,
        width,
        height,
        color_depth,
        colors_used,
        data,
    })
}

pub(crate) fn write(pic: &Picture) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&pic.picture_type.as_u32().to_be_bytes());
    out.extend_from_slice(&(pic.mime_type.len() as u32).to_be_bytes());
    out.extend_from_slice(pic.mime_type.as_bytes());
    out.extend_from_slice(&(pic.description.len() as u32).to_be_bytes());
    out.extend_from_slice(pic.description.as_bytes());
    out.extend_from_slice(&pic.width.to_be_bytes());
    out.extend_from_slice(&pic.height.to_be_bytes());
    out.extend_from_slice(&pic.color_depth.to_be_bytes());
    out.extend_from_slice(&pic.colors_used.to_be_bytes());
    out.extend_from_slice(&(pic.data.len() as u32).to_be_bytes());
    out.extend_from_slice(&pic.data);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let pic = Picture {
            picture_type: PictureType::CoverFront,
            mime_type: "image/jpeg".to_owned(),
            description: String::new(),
            width: 0,
            height: 0,
            color_depth: 0,
            colors_used: 0,
            data: vec![1, 2, 3, 4, 5],
        };
        let bytes = write(&pic);
        assert_eq!(parse(&bytes).unwrap(), pic);
    }
}
