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
    part2(&jellyfish[..]);
    Ok(())
}

fn part1(jellyfish: Vec<u64>) {
    println!(
        "part1: {} fish after 80 days",
        count_jellyfish(jellyfish, 80)
    );
}

fn part2(jellyfish: &[u64]) {
    println!(
        "part2: {} fish after 256",
        count_jellyfish_fast(jellyfish, 256)
    );
}

fn count_jellyfish(mut jellyfish: Vec<u64>, rounds: usize) -> usize {
    for r in 0..rounds {
        println!("Round #{}: {} fish", r, jellyfish.len());
        let mut num_additions = 0;
        for jf in jellyfish.iter_mut() {
            if *jf == 0 {
                *jf = 6;
                num_additions += 1;
            } else {
                *jf -= 1;
            }
        }
        jellyfish.resize(jellyfish.len() + num_additions, 8);
    }
    jellyfish.len()
}

fn count_jellyfish_fast(jellyfish: &[u64], rounds: usize) -> usize {
    let mut state_to_count = jellyfish_inventur(jellyfish);
    for _ in 0..rounds {
        let mut state_to_count_next = vec![0; state_to_count.len()];
        state_to_count_next[8] = state_to_count[0];
        state_to_count_next[..8].clone_from_slice(&state_to_count[1..]);
        state_to_count_next[6] += state_to_count[0];
        std::mem::swap(&mut state_to_count, &mut state_to_count_next);
    }
    state_to_count.iter().sum()
}

fn jellyfish_inventur(jellyfish: &[u64]) -> Vec<usize> {
    let mut counts = vec![0; 9];
    for jf in jellyfish {
        counts[*jf as usize] += 1;
    }
    counts
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
        assert_eq!(count_jellyfish_fast(&jellyfish[..], 256), 26984457539);
    }
}
