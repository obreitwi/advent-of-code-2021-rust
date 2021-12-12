#![allow(unused_imports)]
use anyhow::{bail, Context, Error, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{
        alpha1, anychar, char, digit1, line_ending, multispace1, none_of, one_of, space0, space1,
    },
    combinator::{map, map_res, value},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Err::{Failure, Incomplete},
    ErrorConvert, Finish, IResult,
};
use std::cmp::Ordering;
use std::collections::hash_map::Entry;
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

    let configs = read(&content);

    part1(&configs[..]);
    part2(&configs[..]);
    Ok(())
}

fn part1(configs: &[Configuration]) {
    let count = count_unique_digits(configs);
    println!("part1: {} unique output digits", count);
}

fn part2(config: &[Configuration]) {
    // println!(
    // "part2: {} fuel needed",
    // get_fuel_linear_cost(pos)
    // );
    todo!();
}

#[derive(Debug, Clone)]
struct Configuration {
    signals: Vec<Vec<Signal>>,
    digits: Vec<Vec<Signal>>,
}

impl Parseable for Configuration {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, signals) = separated_list1(space1, many1(Signal::parse))(i)?;
        let (i, _) = tag(" | ")(i)?;
        let (i, digits) = separated_list1(space1, many1(Signal::parse))(i)?;

        assert_eq!(signals.len(), 10);
        assert_eq!(digits.len(), 4);

        Ok((i, Self { signals, digits }))
    }
}

fn read(i: &str) -> Vec<Configuration> {
    if let Ok((_, parsed)) = separated_list1(line_ending, Configuration::parse)(i).finish() {
        parsed
    } else {
        panic!("Could not parse input.");
    }
}

#[derive(Debug, Clone, Copy, Hash)]
enum Signal {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

fn count_unique_digits(configs: &[Configuration]) -> usize {
    configs
        .iter()
        .map(|c| {
            c.digits
                .iter()
                .map(|d| d.len())
                .filter(|d| [2, 3, 4, 7].contains(d))
                .count()
        })
        .sum()
}

impl Parseable for Signal {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            value(Signal::A, char('a')),
            value(Signal::B, char('b')),
            value(Signal::C, char('c')),
            value(Signal::D, char('d')),
            value(Signal::E, char('e')),
            value(Signal::F, char('f')),
            value(Signal::G, char('g')),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let configs = read(&content);
        println!("Read: {:#?}", configs);
        assert_eq!(count_unique_digits(&configs[..]), 26);
    }

    #[test]
    fn test_part2() {}
}
