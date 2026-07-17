use crate::id3::encoding::{self, TextEncoding};
use crate::id3::error::Id3Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub lang: String,
    pub description: String,
    pub text: String,
}

impl Comment {
    pub(crate) fn parse(bytes: &[u8]) -> Result<Self, Id3Error> {
        let (&encoding_byte, rest) = bytes.split_first().ok_or(Id3Error::Truncated)?;
        let encoding = TextEncoding::from_byte(encoding_byte)?;
        if rest.len() < 3 {
            return Err(Id3Error::Truncated);
        }
        let lang = String::from_utf8_lossy(&rest[..3]).into_owned();
        let (description_bytes, text_bytes) = encoding::split_terminated(encoding, &rest[3..]);
        let description = encoding::decode_string(encoding, description_bytes)?;
        let text = encoding::decode_string(encoding, text_bytes)?
            .trim_end_matches('\0')
            .to_owned();
        Ok(Self {
            lang,
            description,
            text,
        })
    }

    pub(crate) fn write(&self) -> Vec<u8> {
        let encoding = TextEncoding::Utf8;
        let mut out = vec![encoding.as_byte()];
        let mut lang_bytes = *b"XXX";
        for (slot, byte) in lang_bytes.iter_mut().zip(self.lang.as_bytes()) {
            *slot = *byte;
        }
        out.extend_from_slice(&lang_bytes);
        out.extend(encoding::encode_string(encoding, &self.description));
        out.push(0x00);
        out.extend(encoding::encode_string(encoding, &self.text));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let comment = Comment {
            lang: "eng".to_owned(),
            description: String::new(),
            text: "great track".to_owned(),
        };
        let bytes = comment.write();
        assert_eq!(Comment::parse(&bytes).unwrap(), comment);
    }
}
