use crate::flac::error::FlacError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamInfo {
    pub min_block_size: u16,
    pub max_block_size: u16,
    pub min_frame_size: u32,
    pub max_frame_size: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub bits_per_sample: u8,
    pub total_samples: u64,
    pub md5_signature: [u8; 16],
}

impl StreamInfo {
    pub fn duration(&self) -> Option<f64> {
        if self.sample_rate == 0 {
            return None;
        }
        Some(self.total_samples as f64 / self.sample_rate as f64)
    }
}

pub(crate) fn parse(data: &[u8]) -> Result<StreamInfo, FlacError> {
    if data.len() != 34 {
        return Err(FlacError::Truncated);
    }
    let min_block_size = u16::from_be_bytes(data[0..2].try_into().unwrap());
    let max_block_size = u16::from_be_bytes(data[2..4].try_into().unwrap());
    let min_frame_size = u32::from_be_bytes([0, data[4], data[5], data[6]]);
    let max_frame_size = u32::from_be_bytes([0, data[7], data[8], data[9]]);
    let bits = u64::from_be_bytes(data[10..18].try_into().unwrap());
    let sample_rate = ((bits >> 44) & 0xf_ffff) as u32;
    let channels = (((bits >> 41) & 0x7) + 1) as u8;
    let bits_per_sample = (((bits >> 36) & 0x1f) + 1) as u8;
    let total_samples = bits & 0xf_ffff_ffff;
    let mut md5_signature = [0u8; 16];
    md5_signature.copy_from_slice(&data[18..34]);
    Ok(StreamInfo {
        min_block_size,
        max_block_size,
        min_frame_size,
        max_frame_size,
        sample_rate,
        channels,
        bits_per_sample,
        total_samples,
        md5_signature,
    })
}

pub(crate) fn write(info: &StreamInfo) -> Vec<u8> {
    let mut out = Vec::with_capacity(34);
    out.extend_from_slice(&info.min_block_size.to_be_bytes());
    out.extend_from_slice(&info.max_block_size.to_be_bytes());
    out.extend_from_slice(&info.min_frame_size.to_be_bytes()[1..]);
    out.extend_from_slice(&info.max_frame_size.to_be_bytes()[1..]);
    let bits: u64 = ((info.sample_rate as u64 & 0xf_ffff) << 44)
        | ((((info.channels - 1) as u64) & 0x7) << 41)
        | ((((info.bits_per_sample - 1) as u64) & 0x1f) << 36)
        | (info.total_samples & 0xf_ffff_ffff);
    out.extend_from_slice(&bits.to_be_bytes());
    out.extend_from_slice(&info.md5_signature);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_real_streaminfo() {
        let raw: [u8; 34] = [
            0x12, 0x00, 0x12, 0x00, 0x00, 0x10, 0x50, 0x00, 0x2d, 0xc4, 0x0a, 0xc4, 0x42, 0xf0,
            0x00, 0x03, 0xc9, 0x48, 0x20, 0x67, 0xc4, 0x44, 0x8e, 0x2b, 0xd8, 0x75, 0xdf, 0xd5,
            0xd4, 0x55, 0x30, 0x06, 0xfe, 0xe2,
        ];
        let info = parse(&raw).unwrap();
        assert_eq!(info.sample_rate, 44100);
        assert_eq!(info.channels, 2);
        assert_eq!(info.bits_per_sample, 16);
        assert_eq!(info.total_samples, 248136);
        assert_eq!(write(&info), raw);
    }
}
