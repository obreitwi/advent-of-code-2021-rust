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

    let lines = VentLine::read_lines(&content);

    part1(&lines);
    part2(&lines);
    Ok(())
}

fn part1(lines: &[VentLine]) {
    println!("part1: {} overlaps", count_overlaps_straight(lines));
}

fn part2(lines: &[VentLine]) {
    println!("part2: {} overlaps", count_overlaps(lines));
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

struct VentLine {
    from: (i64, i64),
    to: (i64, i64),
}

impl Parseable for VentLine {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, from) = point(i)?;
        let (i, _) = tag(" -> ")(i)?;
        let (i, to) = point(i)?;
        Ok((i, Self { from, to }))
    }
}

type VentLines = Vec<VentLine>;

impl Parseable for VentLines {
    fn parse(i: &str) -> IResult<&str, Self> {
        separated_list1(line_ending, VentLine::parse)(i)
    }
}

impl VentLine {
    fn read_lines(i: &str) -> VentLines {
        match VentLines::parse(i).finish() {
            Ok((_, parsed)) => parsed,
            Err(e) => {
                panic!("Error parsing: {}", e);
            }
        }
    }

    fn is_vertical(&self) -> bool {
        self.from.0 == self.to.0
    }

    fn is_horizontal(&self) -> bool {
        self.from.1 == self.to.1
    }

    fn is_straight(&self) -> bool {
        self.is_horizontal() || self.is_vertical()
    }

    fn get_direction(&self) -> (i64, i64) {
        (
            ordering_to_direction(self.to.0.cmp(&self.from.0)),
            ordering_to_direction(self.to.1.cmp(&self.from.1)),
        )
    }

    fn get_points(&self) -> Vec<(i64, i64)> {
        let dir = self.get_direction();
        let mut current = self.from;

        let mut points = vec![current];

        while current != self.to {
            current = (current.0 + dir.0, current.1 + dir.1);
            points.push(current);
        }
        points
    }
}

fn count_overlaps_straight(lines: &[VentLine]) -> usize {
    let mut num_overlaps: HashMap<(i64, i64), usize> = HashMap::new();

    for line in lines.iter().filter(|l| l.is_straight()) {
        for point in line.get_points().into_iter() {
            num_overlaps
                .entry(point)
                .and_modify(|c| {
                    *c += 1;
                })
                .or_insert(1);
        }
    }

    num_overlaps.values().filter(|v| **v > 1).count()
}

fn count_overlaps(lines: &[VentLine]) -> usize {
    let mut num_overlaps: HashMap<(i64, i64), usize> = HashMap::new();

    for line in lines.iter() {
        for point in line.get_points().into_iter() {
            num_overlaps
                .entry(point)
                .and_modify(|c| {
                    *c += 1;
                })
                .or_insert(1);
        }
    }

    num_overlaps.values().filter(|v| **v > 1).count()
}

fn ordering_to_direction(ord: Ordering) -> i64 {
    match ord {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

fn point(i: &str) -> IResult<&str, (i64, i64)> {
    separated_pair(num1, char(','), num1)(i)
}

fn num1(i: &str) -> IResult<&str, i64> {
    map(digit1, |n: &str| n.parse::<i64>().unwrap())(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let lines = VentLine::read_lines(&content);
        assert_eq!(lines.len(), 10, "Did not read all lines");
        assert_eq!(count_overlaps_straight(&lines[..]), 5);
    }

    #[test]
    fn test_part2() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let lines = VentLine::read_lines(&content);
        assert_eq!(lines.len(), 10, "Did not read all lines");
        assert_eq!(count_overlaps(&lines[..]), 12);
    }
}
