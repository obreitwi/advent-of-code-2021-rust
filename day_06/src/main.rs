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

    let jellyfish = read_jellyfish(&content);

    part1(jellyfish.clone());
    part2(jellyfish);
    Ok(())
}

fn part1(jellyfish: Vec<u64>) {
    println!("part1: {} fish after 80 days", count_jellyfish(jellyfish, 80));
}

fn part2(_jellyfish: Vec<u64>) {
    // println!("part2: {} overlaps", count_jellyfish(lines));
}

fn count_jellyfish(mut jellyfish: Vec<u64>, rounds: usize) -> usize {
    for _ in 0..rounds {
        let mut num_additions = 0;
        for jf in jellyfish.iter_mut() {
            if *jf == 0 {
                *jf = 6;
                num_additions += 1;
            }
            else {
                *jf -= 1;
            }
        }
        jellyfish.resize(jellyfish.len() + num_additions, 8);
    }
    jellyfish.len()
}

fn read_jellyfish(i: &str) -> Vec<u64> {
    match separated_list1(char(','), num1::<u64>)(i).finish() {
        Ok((_, parsed)) => parsed,
        Err(e) => {
            panic!("Error parsing: {}", e);
        }
    }
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

fn num1<T: std::str::FromStr>(i: &str) -> IResult<&str, T> {
    map(digit1, |n: &str| {
        n.parse::<T>()
            .unwrap_or_else(|_| panic!("could not parse number"))
    })(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let jellyfish = read_jellyfish(&content);
        assert_eq!(count_jellyfish(jellyfish.clone(), 18), 26);
        assert_eq!(count_jellyfish(jellyfish, 80), 5934);
    }

    #[test]
    fn test_part2() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let jellyfish = read_jellyfish(&content);
    }
}
