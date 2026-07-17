use crate::id3::error::Id3Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextEncoding {
    Latin1,
    Utf16,
    Utf16Be,
    Utf8,
}

impl TextEncoding {
    pub fn from_byte(byte: u8) -> Result<Self, Id3Error> {
        Ok(match byte {
            0 => Self::Latin1,
            1 => Self::Utf16,
            2 => Self::Utf16Be,
            3 => Self::Utf8,
            other => return Err(Id3Error::InvalidEncoding(other)),
        })
    }

    pub fn as_byte(self) -> u8 {
        match self {
            Self::Latin1 => 0,
            Self::Utf16 => 1,
            Self::Utf16Be => 2,
            Self::Utf8 => 3,
        }
    }
}

pub(crate) fn decode_string(encoding: TextEncoding, bytes: &[u8]) -> Result<String, Id3Error> {
    match encoding {
        TextEncoding::Latin1 => Ok(bytes.iter().map(|&b| b as char).collect()),
        TextEncoding::Utf8 => std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Id3Error::InvalidUtf8),
        TextEncoding::Utf16 => decode_utf16(bytes, None),
        TextEncoding::Utf16Be => decode_utf16(bytes, Some(true)),
    }
}

fn decode_utf16(bytes: &[u8], forced_big_endian: Option<bool>) -> Result<String, Id3Error> {
    let (big_endian, bytes) = match forced_big_endian {
        Some(be) => (be, bytes),
        None => match bytes {
            [0xff, 0xfe, rest @ ..] => (false, rest),
            [0xfe, 0xff, rest @ ..] => (true, rest),
            rest => (false, rest),
        },
    };
    if bytes.len() % 2 != 0 {
        return Err(Id3Error::InvalidUtf16);
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| {
            if big_endian {
                u16::from_be_bytes([pair[0], pair[1]])
            } else {
                u16::from_le_bytes([pair[0], pair[1]])
            }
        })
        .collect();
    String::from_utf16(&units).map_err(|_| Id3Error::InvalidUtf16)
}

pub(crate) fn split_terminated(encoding: TextEncoding, bytes: &[u8]) -> (&[u8], &[u8]) {
    match encoding {
        TextEncoding::Latin1 | TextEncoding::Utf8 => match bytes.iter().position(|&b| b == 0x00) {
            Some(idx) => (&bytes[..idx], &bytes[idx + 1..]),
            None => (bytes, &[]),
        },
        TextEncoding::Utf16 | TextEncoding::Utf16Be => {
            let mut idx = 0;
            while idx + 1 < bytes.len() {
                if bytes[idx] == 0x00 && bytes[idx + 1] == 0x00 {
                    return (&bytes[..idx], &bytes[idx + 2..]);
                }
                idx += 2;
            }
            (bytes, &[])
        }
    }
}

pub(crate) fn encode_string(encoding: TextEncoding, value: &str) -> Vec<u8> {
    match encoding {
        TextEncoding::Latin1 => value.chars().map(|c| c as u8).collect(),
        TextEncoding::Utf8 => value.as_bytes().to_vec(),
        TextEncoding::Utf16 => {
            let mut out = vec![0xff, 0xfe];
            out.extend(value.encode_utf16().flat_map(|unit| unit.to_le_bytes()));
            out
        }
        TextEncoding::Utf16Be => value
            .encode_utf16()
            .flat_map(|unit| unit.to_be_bytes())
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_encoding() {
        for encoding in [
            TextEncoding::Latin1,
            TextEncoding::Utf16,
            TextEncoding::Utf16Be,
            TextEncoding::Utf8,
        ] {
            let value = "hello";
            let bytes = encode_string(encoding, value);
            assert_eq!(decode_string(encoding, &bytes).unwrap(), value);
        }
    }

    #[test]
    fn decodes_utf8_unicode() {
        let value = "héllo wörld";
        let bytes = encode_string(TextEncoding::Utf8, value);
        assert_eq!(decode_string(TextEncoding::Utf8, &bytes).unwrap(), value);
    }
}
