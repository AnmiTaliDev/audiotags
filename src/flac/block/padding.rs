pub(crate) fn write(size: usize) -> Vec<u8> {
    vec![0u8; size]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_zeroed_bytes() {
        assert_eq!(write(3), vec![0, 0, 0]);
    }
}
