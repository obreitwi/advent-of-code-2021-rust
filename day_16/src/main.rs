#![allow(unused_imports)]
use anyhow::{bail, Context, Error, Result};
use nom::{
    bits::bits,
    bits::complete::{tag, take},
    branch::alt,
    bytes::complete::{is_a, is_not, take_while1},
    character::complete::{
        alpha1, anychar, char, digit1, line_ending, multispace1, none_of, one_of, space0, space1, hex_digit1,
    },
    combinator::{map, map_res, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Err::{Failure, Incomplete},
    ErrorConvert, Finish, IResult,
};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .nth(1)
            .with_context(|| "No input provided!")
            .unwrap_or_else(|_| "input.txt".to_owned()),
    );
    println!("Input: {}", input.display());
    let content = read_to_string(&input)?;
    let raw = parse_bytes_from_hex(&content).finish().unwrap().1;

    Ok(())
}

#[derive (Debug, Clone, PartialEq, Eq)]
enum Packet {
    Literal(LiteralPacket),
    Operator(OperatorPacket),
}

#[derive (Debug, Clone, PartialEq, Eq)]
struct LiteralPacket {

}

#[derive (Debug, Clone, PartialEq, Eq)]
struct OperatorPacket {

}

fn parse_bytes_from_hex(i: &str) -> IResult<&str, Vec<u8>> {
    let (i, hex) = hex_digit1(i)?;

    assert!(hex.len() % 2 == 0);

    let hex_lc = hex.to_lowercase();
    let mut iter = hex_lc.chars();
    let mut values = Vec::new();
    // input is expected to be byte aligned
    while let Some(hex1) = iter.next() {
        let hex2 = iter.next().unwrap();

        values.push((hex_to_u8(hex1) << 4) + hex_to_u8(hex2));
    }
    Ok((i, values))
}

fn hex_to_u8(c: char) -> u8 {
    match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' => 10,
        'b' => 11,
        'c' => 12,
        'd' => 13,
        'e' => 14,
        'f' => 15,
        _ => panic!("Invalid value to convert to hex: {}", c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        assert_eq!(parse_bytes_from_hex("10").finish().unwrap().1, [0x10]);
        assert_eq!(parse_bytes_from_hex("AB").finish().unwrap().1, [0xAB]);
        assert_eq!(parse_bytes_from_hex("ef").finish().unwrap().1, [0xef]);
        assert_eq!(parse_bytes_from_hex("1234").finish().unwrap().1, [0x12, 0x34]);
    }

    #[test]
    fn test_part1() {}

    #[test]
    fn test_part2() {}
}
