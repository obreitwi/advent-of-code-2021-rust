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
use std::collections::{HashMap, HashSet, VecDeque};
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

    let (i, points) = Points::parse(&content).finish().unwrap();
    let (_, instructions) = Vec::<Fold>::parse(i).finish().unwrap();

    part1(&points, instructions[0].clone());
    part2(&points, &instructions[..]);
    Ok(())
}

fn part1(points: &Points, instruction: Fold) {
    let points = instruction.apply(points);
    println!("part 1: There are {} points", points.len());
}

fn part2(points: &Points, instructions: &[Fold]) {
    let mut points = points.clone();
    for instr in instructions.iter() {
        points = instr.apply(&points);
    }
    println!("part2:");
    print_paper(&points);
}

type Points = HashSet<(u64, u64)>;

fn print_paper(points: &Points) {
    let max_x = points.iter().map(|p| p.0).max().unwrap();
    let max_y = points.iter().map(|p| p.1).max().unwrap();

    for y in 0..(max_y+1) {
        for x in 0..(max_x+1) {
            if points.contains(&(x, y)) {
                print!("#")
            }
            else {
                print!(".")
            }
        }
        println!()
    }
}
impl Parseable for Points {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, vec) = separated_list1(
            line_ending,
            separated_pair(num1::<u64>, char(','), num1::<u64>),
        )(i)?;

        Ok((i, vec.into_iter().collect()))
    }
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Fold {
    Horizontal(u64),
    Vertical(u64),
}

impl Parseable for Fold {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((Fold::parse_vertical, Fold::parse_horizontal))(i)
    }
}

impl Parseable for Vec<Fold> {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, _) = many0(line_ending)(i)?;
        separated_list1(line_ending, Fold::parse)(i)
    }
}

impl Fold {
    fn parse_vertical(i: &str) -> IResult<&str, Self> {
        let (i, _) = tag("fold along y=")(i)?;
        map(num1, Self::Vertical)(i)
    }

    fn parse_horizontal(i: &str) -> IResult<&str, Self> {
        let (i, _) = tag("fold along x=")(i)?;
        map(num1, Self::Horizontal)(i)
    }

    pub fn apply(&self, points: &Points) -> Points {
        match self {
            Self::Horizontal(fold_x) => points
                .iter()
                .map(|(x, y)| (if x < fold_x { *x } else { *fold_x * 2 - x }, *y))
                .collect(),
            Self::Vertical(fold_y) => points
                .iter()
                .map(|(x, y)| (*x, if y < fold_y { *y } else { *fold_y * 2 - y }))
                .collect(),
        }
    }
}

fn num1<T: std::str::FromStr>(i: &str) -> IResult<&str, T> {
    map(digit1, |s: &str| {
            s.parse::<T>()
            .unwrap_or_else(|_| panic!("could not parse number"))
    })(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let (i, points) = Points::parse(&content).finish().unwrap();

        println!("{:#?}", points);

        assert_eq!(points.len(), 18);

        let folds = Vec::<Fold>::parse(i).finish().unwrap().1;

        assert_eq!(folds[0], Fold::Vertical(7));
        assert_eq!(folds[0].apply(&points).len(), 17);
    }

    #[test]
    fn test_part2() {
    }
}
