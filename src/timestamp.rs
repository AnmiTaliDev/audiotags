use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Timestamp {
    pub year: i32,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseTimestampError;

impl fmt::Display for ParseTimestampError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid timestamp")
    }
}

impl std::error::Error for ParseTimestampError {}

fn parse_field(field: Option<&str>) -> Result<Option<u8>, ParseTimestampError> {
    match field {
        None | Some("") => Ok(None),
        Some(s) => s.parse().map(Some).map_err(|_| ParseTimestampError),
    }
}

impl FromStr for Timestamp {
    type Err = ParseTimestampError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (date_part, time_part) = match s.find('T') {
            Some(idx) => (&s[..idx], &s[idx + 1..]),
            None => (s, ""),
        };

        let mut date_fields = date_part.split('-');
        let year = date_fields
            .next()
            .filter(|s| !s.is_empty())
            .ok_or(ParseTimestampError)?
            .parse()
            .map_err(|_| ParseTimestampError)?;
        let month = parse_field(date_fields.next())?;
        let day = parse_field(date_fields.next())?;

        let mut time_fields = time_part.split(':');
        let hour = parse_field(time_fields.next())?;
        let minute = parse_field(time_fields.next())?;
        let second = parse_field(time_fields.next())?;

        if month.is_none() && day.is_some() {
            return Err(ParseTimestampError);
        }
        if day.is_none() && hour.is_some() {
            return Err(ParseTimestampError);
        }
        if hour.is_none() && minute.is_some() {
            return Err(ParseTimestampError);
        }
        if minute.is_none() && second.is_some() {
            return Err(ParseTimestampError);
        }

        Ok(Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        })
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}", self.year)?;
        let Some(month) = self.month else {
            return Ok(());
        };
        write!(f, "-{month:02}")?;
        let Some(day) = self.day else {
            return Ok(());
        };
        write!(f, "-{day:02}")?;
        let Some(hour) = self.hour else {
            return Ok(());
        };
        write!(f, "T{hour:02}")?;
        let Some(minute) = self.minute else {
            return Ok(());
        };
        write!(f, ":{minute:02}")?;
        let Some(second) = self.second else {
            return Ok(());
        };
        write!(f, ":{second:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_full_precision() {
        let ts = Timestamp::from_str("2020-05-22T13:45:07").unwrap();
        assert_eq!(ts.to_string(), "2020-05-22T13:45:07");
    }

    #[test]
    fn round_trips_partial_precision() {
        for s in ["2020", "2020-05", "2020-05-22", "2020-05-22T13"] {
            assert_eq!(Timestamp::from_str(s).unwrap().to_string(), s);
        }
    }

    #[test]
    fn rejects_gaps_in_precision() {
        assert!(Timestamp::from_str("2020-05-22T:30").is_err());
        assert!(Timestamp::from_str("2020--22").is_err());
    }
}
