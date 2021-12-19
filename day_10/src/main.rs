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
    let input = PathBuf::from("input.txt");
    let content = read_to_string(&input)?;

    let lines = parse_lines(&content).finish().unwrap().1;

    part1(&lines[..]);
    part2(&lines[..]);
    Ok(())
}

fn part1(lines: &[BracketLine]) {
    let score = get_score_syntax(lines);
    println!("part 1: {}", score);
}

fn part2(lines: &[BracketLine]) {
    let score = get_score_closing(lines);
    println!("part 2: {}", score);
}

fn get_score_syntax(lines: &[BracketLine]) -> u64 {
    lines
        .iter()
        .filter_map(|l| get_first_syntax_error(&l[..]))
        .map(|b| b.score())
        .sum()
}

fn get_first_syntax_error(line: &[Bracket]) -> Option<BracketType> {
    let mut need_to_close: VecDeque<BracketType> = VecDeque::new();

    for bracket in line.iter() {
        match bracket {
            (bracket, Open) => {
                need_to_close.push_back(*bracket);
            }

            (bracket, Closed) => {
                if let Some(current) = need_to_close.back() {
                    if current != bracket {
                        // found syntax error
                        return Some(*bracket);
                    }
                }
                need_to_close.pop_back();
            }
        }
    }
    None
}

fn get_score_closing(lines: &[Vec<Bracket>]) -> u64 {
    let mut scores: Vec<_> = lines
        .iter()
        .filter_map(|l| get_closing_brackets(&l[..]))
        .map(|bs| get_line_closing_score(&bs[..]))
        .collect();
    scores.sort_unstable();

    scores[scores.len() / 2]
}

fn get_line_closing_score(line: &[BracketType]) -> u64 {
    let mut score = 0;
    for bracket in line.iter() {
        score *= 5;
        score += bracket.score_completion()
    }
    score
}

fn get_closing_brackets(line: &[Bracket]) -> Option<Vec<BracketType>> {
    let mut need_to_close: VecDeque<BracketType> = VecDeque::new();

    for bracket in line.iter() {
        match bracket {
            (bracket, Open) => {
                need_to_close.push_back(*bracket);
            }

            (bracket, Closed) => {
                if let Some(current) = need_to_close.back() {
                    if current != bracket {
                        // found syntax error
                        return None;
                    }
                }
                need_to_close.pop_back();
            }
        }
    }
    if !need_to_close.is_empty() {
        let mut reversed: Vec<_> = need_to_close.into_iter().collect();
        reversed.reverse();
        Some(reversed)
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum State {
    Open,
    Closed,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum BracketType {
    Round,
    Square,
    Curved,
    Sharp,
}

type Bracket = (BracketType, State);

use BracketType::*;
use State::*;

impl Parseable for Bracket {
    fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            value((Round, Open), char('(')),
            value((Round, Closed), char(')')),
            value((Square, Open), char('[')),
            value((Square, Closed), char(']')),
            value((Curved, Open), char('{')),
            value((Curved, Closed), char('}')),
            value((Sharp, Closed), char('>')),
            value((Sharp, Open), char('<')),
        ))(i)
    }
}

impl BracketType {
    fn score(&self) -> u64 {
        match self {
            Round => 3,
            Square => 57,
            Curved => 1197,
            Sharp => 25137,
        }
    }

    fn score_completion(&self) -> u64 {
        match self {
            Round => 1,
            Square => 2,
            Curved => 3,
            Sharp => 4,
        }
    }
}

type BracketLine = Vec<Bracket>;

fn parse_lines(i: &str) -> IResult<&str, Vec<BracketLine>> {
    separated_list1(line_ending, many1(Bracket::parse))(i)
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let lines = parse_lines(&read_to_string(PathBuf::from("debug.txt")).unwrap())
            .finish()
            .unwrap()
            .1;

        assert_eq!(get_score_syntax(&lines[..]), 26397);
    }

    #[test]
    fn test_part2() {
        let lines = parse_lines(&read_to_string(PathBuf::from("debug.txt")).unwrap())
            .finish()
            .unwrap()
            .1;

        assert_eq!(get_score_closing(&lines[..]), 288957);
    }
}
