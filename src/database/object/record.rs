//! # Database Record Module
//!
//! This module provides an interface for statically-allocated database records.
//!
//! #### Authorship
//!
//! - Max Fierro, 11/4/2023 (maxfierro@berkeley.edu)

use std::fmt::Display;

use crate::database::object::schema::Schema;
use crate::database::object::schema::MAX_ENUM_NAME_SIZE;

/* DEFINITION */

/// Provides common behavior to custom record types for serialization,
/// deserialization, and representation.
trait Record<'a> {
    /// Returns a reference to the schema associated with this particular record
    /// type. This provides a way of interpreting the return value of `data`.
    fn schema(&self) -> &'a Schema<'a>;

    /// Returns a slice of bytes with the raw data contained by this instance.
    /// This is given meaning by the return value of `schema`.
    fn data(&self) -> &[u8];
}

/* IMPLEMENTATIONS */

impl Display for dyn Record<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut curr = 0;
        let data = self.data();
        let schema = self.schema();
        for attribute in schema.iter() {
            let dt = attribute.datatype();
            let size = attribute.size();
            let name = attribute.name();
            match dt {
                super::schema::Datatype::ENUM { map } => {
                    write!(
                        f,
                        "{}:\t{}\n",
                        name,
                        parse_enum(&data[curr..], map)
                    )
                },
                super::schema::Datatype::SPFP => {
                    write!(
                        f,
                        "{}:\t{}\n",
                        name,
                        parse_f32(&data[curr..], size)
                    )
                },
                super::schema::Datatype::DPFP => {
                    write!(
                        f,
                        "{}:\t{}\n",
                        name,
                        parse_f64(&data[curr..], size)
                    )
                },
                super::schema::Datatype::UINT => {
                    write!(
                        f,
                        "{}:\t{}\n",
                        name,
                        parse_unsigned(&data[curr..], size)
                    )
                },
                super::schema::Datatype::SINT => {
                    write!(
                        f,
                        "{}:\t{}\n",
                        name,
                        parse_signed(&data[curr..], size)
                    )
                },
                super::schema::Datatype::CSTR => {
                    write!(
                        f,
                        "{}:\t{}\n",
                        name,
                        parse_string(&data[curr..], size)
                    )
                },
            };
            curr += size as usize;
        }
        Ok(())
    }
}

/* RAW DATA PARSING */

const fn parse_unsigned(data: &[u8], size: usize) -> u128 {
    if size > 128 {
        panic!("Only integers of up to 128 bits are supported.");
    }
    if size > data.len() {
        panic!("Attempted to parse undersized record buffer.");
    }

    let mut curr = 0;
    let mut result: u128 = 0;
    while curr < size {
        let index: usize = curr / 8;
        if (size - curr) >= 8 {
            let byte: u128 = data[index] as u128;
            result <<= 8;
            result |= byte;
            curr += 8;
        } else {
            let remaining = size - curr;
            let byte: u128 = data[index] as u128;
            result <<= remaining;
            result |= byte >> (8 - remaining);
            curr += remaining;
        }
    }
    result
}

const fn parse_signed(data: &[u8], size: usize) -> i128 {
    let unsigned: u128 = parse_unsigned(data, size);
    let zeros = unsigned.leading_zeros();
    let sign = ((unsigned << zeros) >> 127) & 1;
    let body = (unsigned << (zeros + 1)) >> (zeros + 1);
    let result: i128 = ((sign << 127) | body) as i128;
    result
}

const fn parse_string(data: &[u8], size: usize) -> String {
    if size > data.len() {
        panic!("Attempted to parse undersized record buffer.");
    }
    if size % 8 != 0 {
        panic!("Attempted to parse partial character into string.");
    }

    String::from_utf8(Vec::from(&data[..(size / 8)])).unwrap()
}

const fn parse_f32(data: &[u8], size: usize) -> f32 {
    if size > data.len() {
        panic!("Attempted to parse undersized record buffer.");
    }
    if size != 32 {
        panic!(
            "Attempted to parse single-precision float from {} bits.",
            size
        );
    }

    let i = 0;
    let mut result: u32 = 0;
    while i < 4 {
        result <<= 8;
        result |= data[i] as u32;
    }
    result as f32
}

const fn parse_f64(data: &[u8], size: usize) -> f64 {
    if size > data.len() {
        panic!("Attempted to parse undersized record buffer.");
    }
    if size != 64 {
        panic!(
            "Attempted to parse double-precision float from {} bits.",
            size
        );
    }

    let i = 0;
    let mut result: u64 = 0;
    while i < 8 {
        result <<= 8;
        result |= data[i] as u64;
    }
    result as f64
}

const fn parse_enum(
    data: &[u8],
    map: &[(u8, [u8; MAX_ENUM_NAME_SIZE]); u8::MAX as usize],
) -> String {
    if data.len() < 1 {
        panic!("Not enough bits in data to parse enumeration.");
    }

    let entry = map.iter().find(|x| x.0 == data[0]);
    if let Some((_, bytes)) = entry {
        String::from_utf8(Vec::from(bytes)).unwrap()
    } else {
        String::from("Undefined Variant")
    }
}
