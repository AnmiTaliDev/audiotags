use crate::flac::bytes::Reader;
use crate::flac::error::FlacError;

const CATALOG_NUMBER_SIZE: usize = 128;
const CUESHEET_RESERVED_SIZE: usize = 258;
const ISRC_SIZE: usize = 12;
const TRACK_RESERVED_SIZE: usize = 13;
const INDEX_RESERVED_SIZE: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CueSheetTrackIndex {
    pub offset: u64,
    pub index_point_number: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CueSheetTrack {
    pub offset: u64,
    pub number: u8,
    pub isrc: String,
    pub is_audio: bool,
    pub pre_emphasis: bool,
    pub indices: Vec<CueSheetTrackIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CueSheet {
    pub media_catalog_number: String,
    pub lead_in_samples: u64,
    pub is_cd: bool,
    pub tracks: Vec<CueSheetTrack>,
}

fn fixed_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .to_owned()
}

fn write_fixed(out: &mut Vec<u8>, value: &str, size: usize) {
    let mut buf = vec![0u8; size];
    let value_bytes = value.as_bytes();
    let len = value_bytes.len().min(size);
    buf[..len].copy_from_slice(&value_bytes[..len]);
    out.extend_from_slice(&buf);
}

pub(crate) fn parse(data: &[u8]) -> Result<CueSheet, FlacError> {
    let mut r = Reader::new(data);
    let media_catalog_number = fixed_string(r.read_bytes(CATALOG_NUMBER_SIZE)?);
    let lead_in_samples = r.read_u64_be()?;
    let flags = r.read_u8()?;
    let is_cd = flags & 0x80 != 0;
    r.read_bytes(CUESHEET_RESERVED_SIZE)?;
    let track_count = r.read_u8()?;

    let mut tracks = Vec::with_capacity(track_count as usize);
    for _ in 0..track_count {
        let offset = r.read_u64_be()?;
        let number = r.read_u8()?;
        let isrc = fixed_string(r.read_bytes(ISRC_SIZE)?);
        let track_flags = r.read_u8()?;
        let is_audio = track_flags & 0x80 == 0;
        let pre_emphasis = track_flags & 0x40 != 0;
        r.read_bytes(TRACK_RESERVED_SIZE)?;
        let index_count = r.read_u8()?;

        let mut indices = Vec::with_capacity(index_count as usize);
        for _ in 0..index_count {
            let idx_offset = r.read_u64_be()?;
            let index_point_number = r.read_u8()?;
            r.read_bytes(INDEX_RESERVED_SIZE)?;
            indices.push(CueSheetTrackIndex {
                offset: idx_offset,
                index_point_number,
            });
        }

        tracks.push(CueSheetTrack {
            offset,
            number,
            isrc,
            is_audio,
            pre_emphasis,
            indices,
        });
    }

    Ok(CueSheet {
        media_catalog_number,
        lead_in_samples,
        is_cd,
        tracks,
    })
}

pub(crate) fn write(cs: &CueSheet) -> Vec<u8> {
    let mut out = Vec::new();
    write_fixed(&mut out, &cs.media_catalog_number, CATALOG_NUMBER_SIZE);
    out.extend_from_slice(&cs.lead_in_samples.to_be_bytes());
    out.push(if cs.is_cd { 0x80 } else { 0x00 });
    out.extend(std::iter::repeat_n(0u8, CUESHEET_RESERVED_SIZE));
    out.push(cs.tracks.len() as u8);

    for t in &cs.tracks {
        out.extend_from_slice(&t.offset.to_be_bytes());
        out.push(t.number);
        write_fixed(&mut out, &t.isrc, ISRC_SIZE);
        let mut track_flags = 0u8;
        if !t.is_audio {
            track_flags |= 0x80;
        }
        if t.pre_emphasis {
            track_flags |= 0x40;
        }
        out.push(track_flags);
        out.extend(std::iter::repeat_n(0u8, TRACK_RESERVED_SIZE));
        out.push(t.indices.len() as u8);

        for idx in &t.indices {
            out.extend_from_slice(&idx.offset.to_be_bytes());
            out.push(idx.index_point_number);
            out.extend(std::iter::repeat_n(0u8, INDEX_RESERVED_SIZE));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let cs = CueSheet {
            media_catalog_number: "1234567890123".to_owned(),
            lead_in_samples: 88200,
            is_cd: true,
            tracks: vec![
                CueSheetTrack {
                    offset: 0,
                    number: 1,
                    isrc: "ABCDE1234567".to_owned(),
                    is_audio: true,
                    pre_emphasis: false,
                    indices: vec![CueSheetTrackIndex {
                        offset: 0,
                        index_point_number: 1,
                    }],
                },
                CueSheetTrack {
                    offset: 44100 * 180,
                    number: 170,
                    isrc: String::new(),
                    is_audio: true,
                    pre_emphasis: false,
                    indices: vec![],
                },
            ],
        };
        let bytes = write(&cs);
        assert_eq!(parse(&bytes).unwrap(), cs);
    }
}
