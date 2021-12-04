#![allow(unused_imports)]
use anyhow::{bail, Context, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{alpha1, anychar, char, digit1, line_ending, none_of, one_of, space0},
    combinator::{map, map_res, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    ErrorConvert, Finish, IResult,
};
use std::collections::{HashMap, HashSet};
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

    let diagnostics = Diagnostics::from(&content);

    part1(&diagnostics);
    part2(&diagnostics);
    Ok(())
}

fn part1(diag: &Diagnostics) {
    println!("part1: {}", diag.gamma() * diag.epsilon());
}

fn part2(_diag: &Diagnostics) {
    // println!("part2: {}", pos * depth);
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

struct Diagnostics {
    numbers: Vec<u64>,
    num_bits: usize,
}

impl Parseable for Diagnostics {
    fn parse(i: &str) -> IResult<&str, Diagnostics> {
        let (i, num_strings) = separated_list1(line_ending, digit1)(i)?;
        let mut max = 0;

        let numbers = {
            let mut numbers = Vec::with_capacity(num_strings.len());
            for num_str in num_strings {
                let num = binary_str_to_num(num_str);
                if num > max {
                    max = num;
                }
                numbers.push(num);
            }
            numbers
        };
        Ok((
            i,
            Self {
                numbers,
                num_bits: num_bits(max),
            },
        ))
    }
}

impl Diagnostics {
    fn from(i: &str) -> Self {
        match Self::parse(i).finish() {
            Ok((i, parsed)) => {
                assert!(i == "\n", "Did not consume full string.");
                parsed
            }
            Err(e) => {
                panic!("Error parsing: {}", e);
            }
        }
    }

    fn get_most_common_bits(&self) -> usize {
        let half = self.numbers.len() / 2;
        let mut retval = 0;
        for i in 0..self.num_bits {
            // shift to left
            retval *= 2;

            let bitcount = get_bitcount(self.num_bits - 1 - i, &self.numbers[..]);
            if bitcount > half {
                // add bit if most common
                retval += 1;
            }
        }
        retval
    }

    fn gamma(&self) -> usize {
        self.get_most_common_bits()
    }

    fn epsilon(&self) -> usize {
        let least_common = !self.get_most_common_bits();
        // clear upper bits
        let mask: usize = usize::MAX << self.num_bits;
        least_common & !mask
    }
}

fn binary_str_to_num(i: &str) -> u64 {
    let mut retval = 0;
    for c in i.chars() {
        retval *= 2;
        if c == '1' {
            retval += 1;
        }
    }
    retval
}

fn num_bits(n: u64) -> usize {
    let mut n = n;
    let mut ld2 = 0;
    while n > 0 {
        ld2 += 1;
        n /= 2;
    }
    ld2
}

fn get_bitcount(pos: usize, numbers: &[u64]) -> usize {
    let selector = 1 << pos;
    numbers.iter().filter(|n| **n & selector > 0).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEBUG_DATA: &str = "00100\n\
                              11110\n\
                              10110\n\
                              10111\n\
                              10101\n\
                              01111\n\
                              00111\n\
                              11100\n\
                              10000\n\
                              11001\n\
                              00010\n\
                              01010\n";

    #[test]
    fn test_num_bits() {
        assert_eq!(num_bits(0), 0);
        assert_eq!(num_bits(1), 1);
        assert_eq!(num_bits(2), 2);
        assert_eq!(num_bits(3), 2);
        assert_eq!(num_bits(4), 3);
        assert_eq!(num_bits(5), 3);
        assert_eq!(num_bits(8), 4);
        assert_eq!(num_bits(9), 4);
    }

    #[test]
    fn test_part1() {
        let diag = Diagnostics::from(DEBUG_DATA);
        println!("{:?}", diag.numbers);
        assert_eq!(diag.num_bits, 5);
        assert_eq!(diag.gamma(), 22);
        assert_eq!(diag.epsilon(), 9);
    }

    #[test]
    fn test_part2() {}
}
