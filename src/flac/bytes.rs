use crate::flac::error::FlacError;

pub(crate) struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], FlacError> {
        if self.pos + n > self.data.len() {
            return Err(FlacError::Truncated);
        }
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    pub fn read_string(&mut self, n: usize) -> Result<String, FlacError> {
        String::from_utf8(self.read_bytes(n)?.to_vec()).map_err(|_| FlacError::InvalidUtf8)
    }

    pub fn read_u32_be(&mut self) -> Result<u32, FlacError> {
        Ok(u32::from_be_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }

    pub fn read_u32_le(&mut self) -> Result<u32, FlacError> {
        Ok(u32::from_le_bytes(self.read_bytes(4)?.try_into().unwrap()))
    }

    pub fn read_u8(&mut self) -> Result<u8, FlacError> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_u64_be(&mut self) -> Result<u64, FlacError> {
        Ok(u64::from_be_bytes(self.read_bytes(8)?.try_into().unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_in_order() {
        let data = [0x00, 0x00, 0x00, 0x05, b'h', b'e', b'l', b'l', b'o'];
        let mut r = Reader::new(&data);
        let len = r.read_u32_be().unwrap();
        assert_eq!(len, 5);
        assert_eq!(r.read_string(len as usize).unwrap(), "hello");
    }

    #[test]
    fn errors_on_overrun() {
        let data = [0x00, 0x01];
        let mut r = Reader::new(&data);
        assert!(r.read_u32_be().is_err());
    }
}
