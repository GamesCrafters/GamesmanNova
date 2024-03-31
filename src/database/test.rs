//! # Database Test Module
//!
//! This module provides integration tests for the database module.
//!
//! #### Authorship
//!
//! - Benjamin Riley Zimmerman, 3/8/2024 (bz931@berkely.edu)

#[test]
fn parse_unsigned_correctness() {
    let data1 = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let expected_parse1: u128 = 0xDEAD_BEEF_0000_0000;
    assert_eq!(
        parse_unsigned(&data1, data1.len()),
        expected_parse1
    );
    let data2 = vec![0x00, 0x00, 0x0F, 0xF0, 0x0F, 0x0F];
    let expected_parse2: u128 = 0x0000_0FF0_0FF0_0000;
    assert_eq!(parse_unsigned(&data2, data2.len()));
    let data3 = vec![
        0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF, 0x00,
    ];
    let expected_parse3: u128 = 0xDEAD_BEEF_DEAD_BEEF;
    assert_eq!(parse_unsigned(&data3, 128));
}

#[test]
fn parse_unsigned_error_correctness() {
    let mut data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let mut result = panic::catch_unwind(|| parse_unsigned(&data, -1));
    assert!(result.is_err());
    result = panic::catch_unwind(|| parse_unsigned(&data, 127));
    assert!(result.is_err());
    data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF];
    result = panic::catch_unwind(|| parse_unsigned(&data, 127));
    assert!(result.is_err());
}

#[test]
fn parse_signed_correctness() {
    let data1 = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let expected_parse1: i128 = 0xDEAD_BEEF_0000_0000 as i128;
    assert_eq!(parse_signed(&data1, data1.len), expected_parse1);
    let data2 = vec![0x00, 0x00, 0x0F, 0xF0, 0x0F, 0x0F];
    let expected_parse2: i128 = 0x0000_0FF0_0FF0_0000 as i128;
    assert_eq!(parse_signed(&data2, data2.len));
    let data3 = vec![
        0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF, 0x00,
    ];
    let expected_parse3: i128 = 0xDEAD_BEEF_DEAD_BEEF as i128;
    assert_eq!(parse_signed(&data3, 128));
}

#[test]
fn parse_string_correctness() {
    let data1 = vec![0x73, 0x75, 0x6D, 0x6D, 0x72, 0x73];
    let expected_parse1: String = "summrs";
    assert_eq!(parse_string(&data1, data1.len), expected_parse1);
    let data2 = vec![
        0x4C, 0x61, 0x4E, 0x61, 0x20, 0x64, 0x33, 0x31, 0x20, 0x72, 0x33, 0x59,
    ];
    let expected_parse2: String = "LaNa d31 r3Y";
    assert_eq!(parse_signed(&data2, data2.len));
    let data3 = vec![
        0x49, 0x20, 0x67, 0x65, 0x74, 0x20, 0x24, 0x24, 0x24,
    ];
    let expected_parse3: String = "i get $$$";
    assert_eq!(parse_signed(&data3, data3.len));
}

#[test]
fn parse_string_error_correctness() {
    let mut data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let mut result =
        panic::catch_unwind(|| parse_unsigned(&data, data.len - 1));
    assert!(result.is_err());
    result = panic::catch_unwind(|| parse_unsigned(&data, data.len + 8));
    assert!(result.is_err());
}

#[test]
fn parse_f32_correctness() {
    let data1 = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let expected_parse1: f32 = -6.25985e+18;
    assert_eq!(parse_string(&data1, data1.len), expected_parse1);
    let data2 = vec![0x01, 0x23, 0x45, 0x67];
    let expected_parse2: f32 = 2.99882e-38;
    assert_eq!(parse_signed(&data2, data2.len));
}

#[test]
fn parse_f32_error_correctness() {
    let mut data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    let mut result =
        panic::catch_unwind(|| parse_unsigned(&data, data.len - 1));
    assert!(result.is_err());
    data = vec![];
    result = panic::catch_unwind(|| parse_unsigned(&data, 32));
    assert!(result.is_err());
}

#[test]
fn parse_f64_correctness() {
    let data1 = vec![0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00];
    let expected_parse1: f64 = -1.71465e+38;
    assert_eq!(parse_string(&data1, data1.len), expected_parse1);
    let data2 = vec![0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01];
    let expected_parse2: f64 = 2.36943e-38;
    assert_eq!(parse_signed(&data2, data2.len));
}

#[test]
fn parse_f64_error_correctness() {
    let mut data1 = vec![0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00];
    let mut result =
        panic::catch_unwind(|| parse_unsigned(&data, data.len - 1));
    assert!(result.is_err());
    data = vec![];
    result = panic::catch_unwind(|| parse_unsigned(&data, 32));
    assert!(result.is_err());
}

#[test]
fn parse_enum_correctness() {
    const enum_map: &[(u8, [u8; 8]); 2] = &[
        (1, [b'A', b'R', b'G', b'_', b'1', b'\0']),
        (2, [b'A', b'R', b'G', b'_', b'2', b'\0']),
        (
            3,
            [b'U', b'N', b'D', b'E', b'F', b'I', b'N', b'E'],
        ),
    ];
    let data1: &[u8] = &[1];
    let data2: &[u8] = &[2];
    assert_eq!(parse_enum(data1, enum_map), "ARG_1");
    assert_eq!(parse_enum(data2, enum_map), "ARG_2");
}

#[test]
fn parse_enum_error_correctness() {
    const enum_map: &[(u8, [u8; 8]); 2] = &[
        (1, [b'A', b'R', b'G', b'_', b'1', b'\0']),
        (2, [b'A', b'R', b'G', b'_', b'2', b'\0']),
        (
            3,
            [b'U', b'N', b'D', b'E', b'F', b'I', b'N', b'E'],
        ),
    ];
    let invalid_size_data: &[u8] = &[];
    let invalid_variant_data: &[u8] = &[4];
    let mut result =
        panic::catch_unwind(|| parse_unsigned(invalid_size_data, enum_map));
    assert!(result.is_err());
    result =
        panic::catch_unwind(|| parse_unsigned(invalid_variant_data, enum_map));
    assert!(result.is_err());
}

/* SCHEMA TESTS */
#[test]
fn test_schema_builder_empty() {
    let builder = SchemaBuilder::new();
    let schema = builder.build();
    assert_eq!(schema.size(), 0);
    assert!(schema.iter().next().is_none());
}

#[test]
fn test_schema_builder_single_attribute() {
    let builder = SchemaBuilder::new()
        .add(Attribute::new("age", Datatype::UINT, 8))
        .unwrap();
    let schema = builder.build();
    assert_eq!(schema.size(), 8);
    assert_eq!(
        schema
            .iter()
            .next()
            .unwrap()
            .name(),
        "age"
    );
}

#[test]
fn test_schema_builder_multiple_attributes() {
    let builder = SchemaBuilder::new()
        .add(Attribute::new("name", Datatype::CSTR, 16))
        .unwrap()
        .add(Attribute::new("score", Datatype::SPFP, 32))
        .unwrap();
    let schema = builder.build();
    assert_eq!(schema.size(), 48);
    let mut iter = schema.iter();
    assert_eq!(iter.next().unwrap().name(), "name");
    assert_eq!(iter.next().unwrap().name(), "score");
}

#[test]
#[should_panic]
fn test_schema_builder_invalid_attribute_size() {
    let builder = SchemaBuilder::new()
        .add(Attribute::new("name", Datatype::CSTR, -1))
        .unwrap();
}

#[test]
#[should_panic]
fn test_check_attribute_empty_name() {
    let existing: Vec<Attribute> = vec![];
    let new_attr = Attribute::new("", Datatype::UINT, 8);
    check_attribute_validity(&existing, &new_attr).unwrap();
}

#[test]
#[should_panic]
fn test_check_attribute_duplicate_name() {
    let existing = vec![Attribute::new("age", Datatype::UINT, 8)];
    let new_attr = Attribute::new("age", Datatype::SINT, 16);
    check_attribute_validity(&existing, &new_attr).unwrap();
}

#[test]
fn test_check_attribute_valid() {
    let existing: Vec<Attribute> = vec![];
    let new_attr = Attribute::new("score", Datatype::SPFP, 32);
    let result = check_attribute_validity(&existing, &new_attr);
    assert!(result.is_ok());
}
