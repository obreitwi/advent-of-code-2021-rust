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
    println!(
        "part2: {} total sum",
        config.iter().map(|c| c.decode()).sum::<usize>()
    );
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

impl Configuration {
    fn get_mapping(&self) -> Vec<HashSet<Signal>> {
        let mut num_to_wiring: HashMap<usize, Vec<HashSet<Signal>>> = HashMap::new();
        for signal in self.signals.iter() {
            num_to_wiring
                .entry(signal.len())
                .or_default()
                .push(signal.iter().cloned().collect());
        }

        let mut mapping = vec![HashSet::new(); 10];

        // determine 1, 4, 7, 8
        mapping[1] = num_to_wiring[&2][0].iter().cloned().collect();
        mapping[4] = num_to_wiring[&4][0].iter().cloned().collect();
        mapping[7] = num_to_wiring[&3][0].iter().cloned().collect();
        mapping[8] = num_to_wiring[&7][0].iter().cloned().collect();

        // distinguish 6-wire digits: 0, 6, 9

        // 9 is the one which has full overlap with 4
        for signals in num_to_wiring[&6].iter() {
            if mapping[4].difference(signals).count() == 0 {
                mapping[9] = signals.clone();
                break;
            }
        }

        // 6 is the one which has a single difference with 1
        for signals in num_to_wiring[&6].iter() {
            if mapping[1].difference(signals).count() == 1 {
                mapping[6] = signals.clone();
                break;
            }
        }

        // 0 is the remaining one
        for signals in num_to_wiring[&6].iter() {
            if *signals != mapping[9] && *signals != mapping[6] {
                mapping[0] = signals.clone();
                break;
            }
        }

        // distinguish 2, 3, 5

        // 5 is the one tha that has all but one signal from 6
        for signals in num_to_wiring[&5].iter() {
            if mapping[6].difference(signals).count() == 1 {
                mapping[5] = signals.clone();
                break;
            }
        }

        // 3 has 1 segments less than 9 and is not 5
        for signals in num_to_wiring[&5].iter() {
            if mapping[9].difference(signals).count() == 1 && *signals != mapping[5] {
                mapping[3] = signals.clone();
                break;
            }
        }

        // 2 is all that remains
        for signals in num_to_wiring[&5].iter() {
            if *signals != mapping[3] && *signals != mapping[5] {
                mapping[2] = signals.clone();
                break;
            }
        }

        mapping
    }

    fn decode(&self) -> usize {
        let mapping = self.get_mapping();
        let mut retval = 0;
        for digit in self.digits.iter() {
            retval *= 10;
            let set: HashSet<_> = digit.iter().cloned().collect();
            let mut found = false;
            for (decoded, signals) in mapping.iter().enumerate() {
                if *signals == set {
                    retval += decoded;
                    found = true;
                    break;
                }
            }
            assert!(found, "did not find pattern with {} signals", set.len());
        }
        retval
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
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
        assert_eq!(count_unique_digits(&configs[..]), 26);
    }

    #[test]
    fn test_part2() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let configs = read(&content);
        assert_eq!(configs.len(), 10);
        assert_eq!(configs[0].decode(), 8394);
        assert_eq!(configs[1].decode(), 9781);
        assert_eq!(configs[2].decode(), 1197);
        assert_eq!(configs[3].decode(), 9361);
        assert_eq!(configs[4].decode(), 4873);
        assert_eq!(configs[5].decode(), 8418);
        assert_eq!(configs[6].decode(), 4548);
        assert_eq!(configs[7].decode(), 1625);
        assert_eq!(configs[8].decode(), 8717);
        assert_eq!(configs[9].decode(), 4315);
    }
}
