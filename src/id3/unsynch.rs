pub(crate) fn decode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        out.push(data[i]);
        if data[i] == 0xff && data.get(i + 1) == Some(&0x00) {
            i += 1;
        }
        i += 1;
    }
    out
}

pub(crate) fn encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut prev_was_ff = false;
    for &byte in data {
        if prev_was_ff && (byte == 0x00 || byte & 0xe0 == 0xe0) {
            out.push(0x00);
        }
        out.push(byte);
        prev_was_ff = byte == 0xff;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_arbitrary_bytes() {
        let data = [0x00, 0xff, 0x00, 0xff, 0xe0, 0xff, 0xff, 0x01];
        assert_eq!(decode(&encode(&data)), data);
    }

    #[test]
    fn leaves_plain_data_untouched() {
        let data = [0x01, 0x02, 0x03];
        assert_eq!(encode(&data), data);
        assert_eq!(decode(&data), data);
    }
}
