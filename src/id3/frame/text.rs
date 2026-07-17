use crate::id3::encoding::{self, TextEncoding};
use crate::id3::error::Id3Error;

pub(crate) fn parse_text(bytes: &[u8]) -> Result<String, Id3Error> {
    let (&encoding_byte, rest) = bytes.split_first().ok_or(Id3Error::Truncated)?;
    let encoding = TextEncoding::from_byte(encoding_byte)?;
    let value = encoding::decode_string(encoding, rest)?;
    Ok(value.trim_end_matches('\0').to_owned())
}

pub(crate) fn write_text(value: &str) -> Vec<u8> {
    let encoding = TextEncoding::Utf8;
    let mut out = vec![encoding.as_byte()];
    out.extend(encoding::encode_string(encoding, value));
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedText {
    pub description: String,
    pub value: String,
}

pub(crate) fn parse_extended_text(bytes: &[u8]) -> Result<ExtendedText, Id3Error> {
    let (&encoding_byte, rest) = bytes.split_first().ok_or(Id3Error::Truncated)?;
    let encoding = TextEncoding::from_byte(encoding_byte)?;
    let (description_bytes, value_bytes) = encoding::split_terminated(encoding, rest);
    let description = encoding::decode_string(encoding, description_bytes)?;
    let value = encoding::decode_string(encoding, value_bytes)?
        .trim_end_matches('\0')
        .to_owned();
    Ok(ExtendedText { description, value })
}

pub(crate) fn write_extended_text(ext: &ExtendedText) -> Vec<u8> {
    let encoding = TextEncoding::Utf8;
    let mut out = vec![encoding.as_byte()];
    out.extend(encoding::encode_string(encoding, &ext.description));
    out.push(0x00);
    out.extend(encoding::encode_string(encoding, &ext.value));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_text() {
        let bytes = write_text("hello world");
        assert_eq!(parse_text(&bytes).unwrap(), "hello world");
    }

    #[test]
    fn round_trips_extended_text() {
        let ext = ExtendedText {
            description: "replaygain_track_gain".to_owned(),
            value: "-6.5 dB".to_owned(),
        };
        let bytes = write_extended_text(&ext);
        assert_eq!(parse_extended_text(&bytes).unwrap(), ext);
    }
}
