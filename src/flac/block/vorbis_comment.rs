use crate::flac::bytes::Reader;
use crate::flac::error::FlacError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VorbisComments {
    pub vendor: String,
    pub comments: Vec<(String, String)>,
}

impl Default for VorbisComments {
    fn default() -> Self {
        Self {
            vendor: "audiometa".to_owned(),
            comments: Vec::new(),
        }
    }
}

impl VorbisComments {
    pub fn get_first(&self, key: &str) -> Option<&str> {
        self.comments
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.remove(key);
        self.comments.push((key.to_owned(), value.to_owned()));
    }

    pub fn remove(&mut self, key: &str) {
        self.comments.retain(|(k, _)| !k.eq_ignore_ascii_case(key));
    }
}

pub(crate) fn parse(data: &[u8]) -> Result<VorbisComments, FlacError> {
    let mut r = Reader::new(data);
    let vendor_len = r.read_u32_le()? as usize;
    let vendor = r.read_string(vendor_len)?;
    let count = r.read_u32_le()?;
    let mut comments = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let len = r.read_u32_le()? as usize;
        let entry = r.read_string(len)?;
        if let Some(eq) = entry.find('=') {
            comments.push((entry[..eq].to_owned(), entry[eq + 1..].to_owned()));
        }
    }
    Ok(VorbisComments { vendor, comments })
}

pub(crate) fn write(vc: &VorbisComments) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(vc.vendor.len() as u32).to_le_bytes());
    out.extend_from_slice(vc.vendor.as_bytes());
    out.extend_from_slice(&(vc.comments.len() as u32).to_le_bytes());
    for (key, value) in &vc.comments {
        let entry = format!("{key}={value}");
        out.extend_from_slice(&(entry.len() as u32).to_le_bytes());
        out.extend_from_slice(entry.as_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let mut vc = VorbisComments::default();
        vc.set("TITLE", "song title");
        vc.set("ARTIST", "artist name");
        let bytes = write(&vc);
        let parsed = parse(&bytes).unwrap();
        assert_eq!(parsed, vc);
    }

    #[test]
    fn get_is_case_insensitive() {
        let mut vc = VorbisComments::default();
        vc.set("title", "song title");
        assert_eq!(vc.get_first("TITLE"), Some("song title"));
    }

    #[test]
    fn set_replaces_existing_value() {
        let mut vc = VorbisComments::default();
        vc.set("TITLE", "first");
        vc.set("TITLE", "second");
        assert_eq!(vc.comments.len(), 1);
        assert_eq!(vc.get_first("TITLE"), Some("second"));
    }
}
