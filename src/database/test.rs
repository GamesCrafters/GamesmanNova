//! # Database Test Module
//!
//! This module provides some unit tests for different database types.
//!
//! #### Authorship
//!
//! - Benjamin Riley Zimmerman, 3/8/2024 (bz931@berkely.edu)

/* TESTS */

#[cfg(test)]
mod test {

    use crate::database::*;
    use std::panic;

    /* RECORD TESTS */
    #[test]
    fn parse_unsigned_correctness() {
        let data1 = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let expected_parse1: u128 = 0xDEAD_BEEF_0000_0000;
        assert_eq!(parse_unsigned(&data1, data1.len), expected_parse1)
        let data2 = vec![0x00, 0x00, 0x0F, 0xF0, 0x0F, 0x0F];
        let expected_parse2: u128 = 0x0000_0FF0_0FF0_0000
        assert_eq!(parse_unsigned(&data2, data2.len))
        let data3 = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF, 0x00];
        let expected_parse3: u128 = 0xDEAD_BEEF_DEAD_BEEF;
        assert_eq!(parse_unsigned(&data3, 128))
    }
    
    #[test]
    fn parse_unsigned_error_correctness() {
        let mut data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let mut result = panic::catch_unwind(|| parse_unsigned(&data, -1));
        assert!(result.is_err());
        result = panic::catch_unwind(|| parse_unsigned(&data, 127));
        assert!(result.is_err());
        data = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xDE, 0xAD, 0xBE, 0xEF]
        result = panic::catch_unwind(|| parse_unsigned(&data, 127));
        assert!(result.is_err());
    }
    
    /* SCHEMA TESTS */

    /* ENGINE TESTS */

}