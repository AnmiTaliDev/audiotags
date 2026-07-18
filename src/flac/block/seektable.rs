use crate::flac::bytes::Reader;
use crate::flac::error::FlacError;

const SEEK_POINT_SIZE: usize = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeekPoint {
    pub sample_number: u64,
    pub stream_offset: u64,
    pub frame_samples: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SeekTable {
    pub points: Vec<SeekPoint>,
}

pub(crate) fn parse(data: &[u8]) -> Result<SeekTable, FlacError> {
    if !data.len().is_multiple_of(SEEK_POINT_SIZE) {
        return Err(FlacError::Truncated);
    }
    let mut points = Vec::with_capacity(data.len() / SEEK_POINT_SIZE);
    let mut r = Reader::new(data);
    for _ in 0..(data.len() / SEEK_POINT_SIZE) {
        let sample_number = r.read_u64_be()?;
        let stream_offset = r.read_u64_be()?;
        let frame_samples = u16::from_be_bytes(r.read_bytes(2)?.try_into().unwrap());
        points.push(SeekPoint {
            sample_number,
            stream_offset,
            frame_samples,
        });
    }
    Ok(SeekTable { points })
}

pub(crate) fn write(table: &SeekTable) -> Vec<u8> {
    let mut out = Vec::with_capacity(table.points.len() * SEEK_POINT_SIZE);
    for p in &table.points {
        out.extend_from_slice(&p.sample_number.to_be_bytes());
        out.extend_from_slice(&p.stream_offset.to_be_bytes());
        out.extend_from_slice(&p.frame_samples.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let table = SeekTable {
            points: vec![
                SeekPoint {
                    sample_number: 0,
                    stream_offset: 0,
                    frame_samples: 4096,
                },
                SeekPoint {
                    sample_number: 44100,
                    stream_offset: 12345,
                    frame_samples: 4096,
                },
            ],
        };
        let bytes = write(&table);
        assert_eq!(parse(&bytes).unwrap(), table);
    }
}
