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
    let input = PathBuf::from(env::args().nth(1).with_context(|| "No input provided!")?);
    println!("Input: {}", input.display());
    let content = read_to_string(&input)?;

    let directions = Direction::from(&content);

    part1(&directions[..]);
    part2(&directions[..]);
    Ok(())
}

fn part1(directions: &[Direction]) {
    let (pos, depth) = get_pos_depth(directions);
    println!("part1: {}", pos * depth);
}

fn part2(directions: &[Direction]) {
    let (pos, depth) = get_pos_depth_w_aim(directions);
    println!("part2: {}", pos * depth);
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Forward(u64),
    Up(u64),
    Down(u64),
}

type Directions = Vec<Direction>;

impl Direction {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((Self::parse_forward, Self::parse_up, Self::parse_down))(i)
    }

    fn from(i: &str) -> Directions {
        match Directions::parse(i).finish() {
            Ok((i, parsed)) => {
                assert!(i == "\n", "Did not consume full string.");
                parsed
            }
            Err(e) => {
                panic!("Error parsing: {}", e);
            }
        }
    }

    fn parse_forward(i: &str) -> IResult<&str, Self> {
        let (i, count) = preceded(tag("forward "), digit1)(i)?;
        Ok((i, Self::Forward(count.parse::<u64>().unwrap())))
    }

    fn parse_up(i: &str) -> IResult<&str, Self> {
        let (i, count) = preceded(tag("up "), digit1)(i)?;
        Ok((i, Self::Up(count.parse::<u64>().unwrap())))
    }

    fn parse_down(i: &str) -> IResult<&str, Self> {
        let (i, count) = preceded(tag("down "), digit1)(i)?;
        Ok((i, Self::Down(count.parse::<u64>().unwrap())))
    }
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

impl Parseable for Directions {
    fn parse(i: &str) -> IResult<&str, Directions> {
        separated_list1(line_ending, Direction::parse)(i)
    }
}

fn get_pos_depth(dirs: &[Direction]) -> (u64, u64) {
    let mut pos = 0;
    let mut depth = 0;
    use Direction::*;
    for d in dirs.iter() {
        match d {
            Forward(steps) => {
                pos += steps;
            }
            Up(steps) => {
                depth -= steps;
            }
            Down(steps) => {
                depth += steps;
            }
        }
    }
    (pos, depth)
}

fn get_pos_depth_w_aim(dirs: &[Direction]) -> (u64, u64) {
    let mut pos = 0;
    let mut depth = 0;
    let mut aim: i64 = 0;
    use Direction::*;
    for d in dirs.iter() {
        match d {
            Forward(steps) => {
                pos += steps;
                depth += *steps as i64 * aim;
            }
            Up(steps) => {
                aim -= *steps as i64;
            }
            Down(steps) => {
                aim += *steps as i64;
            }
        }
    }
    (pos, depth as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEBUG_DATA: &str = "forward 5\n\
                              down 5\n\
                              forward 8\n\
                              up 3\n\
                              down 8\n\
                              forward 2\n";

    #[test]
    fn test_part1() {
        let directions = Direction::from(DEBUG_DATA);
        assert_eq!(directions.len(), 6, "did not parse all instructions");

        let (debug_pos, debug_depth) = get_pos_depth(&directions[..]);

        println!("{:?}", directions);

        assert_eq!(debug_pos, 15, "pos does not match");
        assert_eq!(debug_depth, 10, "depth does not match");
        assert_eq!(debug_pos * debug_depth, 150);
    }

    #[test]
    fn test_part2() {
        let directions = Direction::from(DEBUG_DATA);
        assert_eq!(directions.len(), 6, "did not parse all instructions");

        let (debug_pos, debug_depth) = get_pos_depth_w_aim(&directions[..]);

        println!("{:?}", directions);

        assert_eq!(debug_pos, 15, "pos does not match");
        assert_eq!(debug_depth, 60, "depth does not match");
        assert_eq!(debug_pos * debug_depth, 900);
    }
}
