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

    let pos = read(&content);

    part1(&pos[..]);
    part2(&pos[..]);
    Ok(())
}

fn part1(pos: &[i64]) {
    println!(
        "part1: {} fuel needed",
        get_fuel(pos)
    );
}

fn part2(pos: &[i64]) {
    todo!();
}

fn read(i: &str) -> Vec<i64> {
    match separated_list1(char(','), num1::<i64>)(i).finish() {
        Ok((_, parsed)) => parsed,
        Err(e) => {
            panic!("Error parsing: {}", e);
        }
    }
}

fn get_fuel(pos: &[i64]) -> i64 {
    let min = *pos.iter().min().unwrap();
    let max = *pos.iter().max().unwrap();
    (min..max+1).map(|com| pos.iter().map(|p| (p - com).abs()).sum()).min().unwrap()
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
        let pos = read(&content);
        assert_eq!(get_fuel(&pos[..]), 37);
    }

    #[test]
    fn test_part2() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let pos = read(&content);
    }
}
