use crate::flac::error::FlacError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub id: [u8; 4],
    pub data: Vec<u8>,
}

pub(crate) fn parse(data: &[u8]) -> Result<Application, FlacError> {
    if data.len() < 4 {
        return Err(FlacError::Truncated);
    }
    let mut id = [0u8; 4];
    id.copy_from_slice(&data[0..4]);
    Ok(Application {
        id,
        data: data[4..].to_vec(),
    })
}

pub(crate) fn write(app: &Application) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + app.data.len());
    out.extend_from_slice(&app.id);
    out.extend_from_slice(&app.data);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let app = Application {
            id: *b"test",
            data: vec![1, 2, 3, 4, 5],
        };
        let bytes = write(&app);
        assert_eq!(parse(&bytes).unwrap(), app);
    }
}
