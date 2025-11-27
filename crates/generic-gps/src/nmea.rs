//! TODO

use core::str::Utf8Error;

use jiff::Timestamp;

/// A generic NMEA sentence.
#[derive(Debug, Clone, PartialEq)]
pub struct NmeaSentence {
    /// The talker ID.
    pub talker: [char; 2],
    /// The sentence kind.
    pub kind: NmeaSentenceKind,
}

/// The kind of NMEA sentence.
#[derive(Debug, Clone, PartialEq)]
#[expect(missing_docs, reason = "Message descriptors")]
pub enum NmeaSentenceKind {
    GNSS { latitude: Latitude, longitude: Longitude, timestamp: Timestamp },
    GLSS { latitude: Latitude, longitude: Longitude, timestamp: Timestamp },
}

/// A latitude value.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Latitude {
    /// North latitude.
    North(f64),
    /// South latitude.
    South(f64),
}

/// A longitude value.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Longitude {
    /// East longitude.
    East(f64),
    /// West longitude.
    West(f64),
}

// -------------------------------------------------------------------------------------------------

/// Parse a NMEA sentence from the provided buffer.
///
/// # Errors
///
/// Returns an error if the sentence is malformed.
pub fn parse_sentence<T>(buffer: &[u8]) -> Result<NmeaSentence, NmeaError<T>> {
    let buffer = core::str::from_utf8(buffer).map_err(NmeaError::Utf8)?;
    let mut sections = buffer.split(',');

    // Read the sentence identifier
    let ident = sections.next().ok_or(NmeaError::Malformed)?;
    let mut chars = ident.chars();

    // Check for starting '$'
    if chars.next() != Some('$') {
        return Err(NmeaError::Malformed);
    }

    // // Read talker ID
    // let talker_a = chars.next().ok_or(NmeaError::MalformedSentence)?;
    // let talker_b = chars.next().ok_or(NmeaError::MalformedSentence)?;
    // let talker = [talker_a, talker_b];

    match &ident[chars.count()..] {
        "GNS" => todo!(),
        "GLL" => todo!(),
        _ => Err(NmeaError::UnknownType),
    }
}

/// Parse a latitude from two NMEA fields.
fn _parse_latitude<T>(_degrees: &str, _direction: &str) -> Result<Latitude, NmeaError<T>> {
    todo!()
}

/// Parse a longitude from two NMEA fields.
fn _parse_longitude<T>(_degrees: &str, _direction: &str) -> Result<Longitude, NmeaError<T>> {
    todo!()
}

/// Parse a timestamp from a NMEA field.
fn _parse_timestamp<T>(_timestamp: &str) -> Result<Timestamp, NmeaError<T>> { todo!() }

// -------------------------------------------------------------------------------------------------

/// An error that can occur when parsing NMEA sentences.
#[derive(Debug, Clone)]
pub enum NmeaError<Error> {
    /// The sentence was malformed.
    Malformed,
    /// The sentence type was not recognized.
    UnknownType,

    /// A time error occurred.
    Time(jiff::Error),
    /// The sentence was not valid UTF-8.
    Utf8(Utf8Error),
    /// An other error occurred.
    Other(Error),
}
