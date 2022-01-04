#![allow(unused_imports)]
use anyhow::{bail, Context, Error, Result};
use nom::{
    bits::bits,
    bits::complete::{tag, take},
    branch::alt,
    bytes::complete::{is_a, is_not, take_while1},
    character::complete::{
        alpha1, anychar, char, digit1, hex_digit1, line_ending, multispace1, none_of, one_of,
        space0, space1,
    },
    combinator::{map, map_res, not, value},
    error::ParseError,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
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

    part1(&raw[..]);

    Ok(())
}

fn part1(i: &[u8]) {
    let (i, pkt) = Packet::parse((i, 0usize)).finish().unwrap();
    assert!(i.0.len() <= 1);

    println!("part 1: version sum = {}", pkt.version_sum());
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Packet {
    Literal(LiteralPacket),
    Operator(OperatorPacket),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LiteralPacket {
    value: u64,
    version: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OperatorPacket {
    type_id: u8,
    version: u8,
    packets: Vec<Packet>,
}

type InputBits<'a> = (&'a [u8], usize);

impl Parseable for Packet {
    fn parse(i: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        alt((Self::parse_literal, Self::parse_operator))(i)
    }
}

impl Packet {
    fn parse_literal(i: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        map(LiteralPacket::parse, Packet::Literal)(i)
    }

    fn parse_operator(i: (&[u8], usize)) -> IResult<(&[u8], usize), Self> {
        map(OperatorPacket::parse, Packet::Operator)(i)
    }

    fn version_sum(&self) -> usize {
        match self {
            Self::Operator(pkt) => pkt.version_sum(),
            Self::Literal(pkt) => pkt.version_sum(),
        }
    }
}

impl LiteralPacket {
    const TYPE_ID: u8 = 4;

    fn version_sum(&self) -> usize {
        self.version as usize
    }
}

trait Parseable: Sized {
    fn parse(i: InputBits) -> IResult<InputBits, Self>;
}

impl Parseable for LiteralPacket {
    fn parse(i: InputBits) -> IResult<InputBits, Self> {
        let (i, version): (InputBits, u8) = take(3usize)(i)?;
        let (mut i, _) = tag(Self::TYPE_ID, 3usize)(i)?;
        let mut value: u64 = 0;
        let mut count = 0;

        let mut is_last_part = false;
        while !is_last_part {
            assert!(count < 16, "Cannot store value in 64 bits.");
            let (ii, part): (InputBits, u64) = take(5usize)(i)?;
            i = ii;
            is_last_part = (part >> 4) == 0;
            let payload = part & 0xF;
            value = (value << 4) + payload;
            if is_last_part {
                break;
            }
            count += 1
        }
        Ok((i, Self { version, value }))
    }
}

impl OperatorPacket {
    fn parse(i: InputBits) -> IResult<InputBits, Self> {
        let (i, version): (InputBits, u8) = take(3usize)(i)?;
        let _ = not(tag(LiteralPacket::TYPE_ID, 3usize))(i)?;
        let (i, type_id): (InputBits, u8) = take(3usize)(i)?;
        let (i, packets) = Vec::<Packet>::parse(i)?;

        Ok((
            i,
            Self {
                type_id,
                version,
                packets,
            },
        ))
    }
}

impl Parseable for Vec<Packet> {
    fn parse(i: InputBits) -> IResult<InputBits, Self> {
        let (i, size_tag): (InputBits, u8) = take(1usize)(i)?;
        if size_tag == 0 {
            let (i, total_bits): (InputBits, usize) = take(15usize)(i)?;
            let get_error = || {
                Err(nom::Err::Error(
                    nom::error::Error::<InputBits>::from_error_kind(
                        i,
                        nom::error::ErrorKind::NonEmpty,
                    ),
                ))
            };
            let (i, bits_subpackets) = take_many(total_bits)(i)?;
            let parsed_packets = many1(Packet::parse)((&bits_subpackets[..], 0)).finish();
            if let Ok((ii, packets)) = parsed_packets {
                let expected_offset = total_bits % 8;
                if expected_offset != ii.1
                    || (expected_offset == 0 && !ii.0.is_empty())
                    || (expected_offset > 0 && ii.0.len() != 1)
                {
                    eprintln!(
                        "parsed {} packets: ii.0.len(): {}, expected_offset: {}, actual: {}",
                        packets.len(),
                        ii.0.len(),
                        expected_offset,
                        ii.1
                    );
                    get_error()
                } else {
                    Ok((i, packets))
                }
            } else {
                get_error()
            }
        } else {
            let (mut i, total_packets): (InputBits, usize) = take(11usize)(i)?;

            let mut packets = Vec::new();
            for _ in 0..total_packets {
                let (ii, packet) = Packet::parse(i)?;
                packets.push(packet);
                i = ii;
            }

            Ok((i, packets))
        }
    }
}

impl OperatorPacket {
    fn version_sum(&self) -> usize {
        let packet_sum: usize = self.packets.iter().map(|p: &Packet| p.version_sum()).sum();
        self.version as usize + packet_sum
    }
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

fn take_many<'a>(count: usize) -> impl Fn(InputBits<'a>) -> IResult<InputBits<'a>, Vec<u8>> {
    move |mut i: InputBits<'a>| -> IResult<InputBits<'a>, Vec<u8>> {
        let mut values: Vec<u8> = Vec::new();
        let mut to_read = count;

        while to_read > 8 {
            let (ii, val): (InputBits, u8) = take(8usize)(i)?;
            values.push(val);
            i = ii;
            to_read -= 8;
        }
        let (i, last): (InputBits, u8) = take(to_read)(i)?;
        values.push(last << (8 - to_read));
        Ok((i, values))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex_to_bytes_must(i: &str) -> Vec<u8> {
        parse_bytes_from_hex(i).finish().unwrap().1
    }

    #[test]
    fn test_parsing() {
        assert_eq!(parse_bytes_from_hex("10").finish().unwrap().1, [0x10]);
        assert_eq!(parse_bytes_from_hex("AB").finish().unwrap().1, [0xAB]);
        assert_eq!(parse_bytes_from_hex("ef").finish().unwrap().1, [0xef]);
        assert_eq!(
            parse_bytes_from_hex("1234").finish().unwrap().1,
            [0x12, 0x34]
        );
    }

    #[test]
    fn test_take_many() {
        let given: Vec<u8> = vec![0xab; 10];
        let want: Vec<u8> = {
            let mut want = vec![0xab; 9];
            want[8] = 0xab & (u8::MAX << 2);
            want
        };
        assert_eq!(
            take_many(70usize)((&given[..], 0)),
            Ok(((&given[8..], 6usize), want))
        );
    }

    #[test]
    fn test_part1_1() {
        let input = hex_to_bytes_must("8A004A801A8002F478");
        let got = Packet::parse((&input[..], 0)).finish().unwrap().1;
        assert_eq!(got.version_sum(), 16);
    }

    #[test]
    fn test_part1_2() {
        let input = hex_to_bytes_must("620080001611562C8802118E34");
        let got = Packet::parse((&input[..], 0)).finish().unwrap().1;
        assert_eq!(got.version_sum(), 12);
    }

    #[test]
    fn test_part1_3() {
        let input = hex_to_bytes_must("C0015000016115A2E0802F182340");
        let got = Packet::parse((&input[..], 0)).finish().unwrap().1;
        assert_eq!(got.version_sum(), 23);
    }

    #[test]
    fn test_part1_4() {
        let input = hex_to_bytes_must("A0016C880162017C3686B18A3D4780");
        let got = Packet::parse((&input[..], 0)).finish().unwrap().1;
        assert_eq!(got.version_sum(), 31);
    }

    #[test]
    fn test_part1_5() {
        let input = hex_to_bytes_must("EE00D40C823060");
        let want = Packet::Operator(OperatorPacket {
            version: 7,
            type_id: 3,
            packets: vec![
                Packet::Literal(LiteralPacket {
                    value: 1,
                    version: 2,
                }),
                Packet::Literal(LiteralPacket {
                    value: 2,
                    version: 4,
                }),
                Packet::Literal(LiteralPacket {
                    value: 3,
                    version: 1,
                }),
            ],
        });
        let got = Packet::parse((&input[..], 0)).finish().unwrap().1;
        assert_eq!(want, got);
    }

    #[test]
    fn test_part1_6() {
        let input = hex_to_bytes_must("38006F45291200");
        let want = Packet::Operator(OperatorPacket {
            version: 1,
            type_id: 6,
            packets: vec![
                Packet::Literal(LiteralPacket {
                    value: 10,
                    version: 6,
                }),
                Packet::Literal(LiteralPacket {
                    value: 20,
                    version: 2,
                }),
            ],
        });
        let got = Packet::parse((&input[..], 0)).finish().unwrap().1;
        assert_eq!(want, got);
    }

    #[test]
    fn test_part2() {}
}
