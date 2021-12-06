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

    let bingo = Bingo::from(&content);

    part1(bingo.clone());
    part2(bingo);
    Ok(())
}

fn part1(mut bingo: Bingo) {
    println!("part1: {}", bingo.draw_till_bingo());
}

fn part2(mut bingo: Bingo) {
    println!("part2: {}", bingo.find_last_winner());
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

#[derive(Debug, Clone)]
struct Bingo {
    draws: Vec<u64>,
    cards: Vec<Card>,
}

#[derive(Debug, Clone)]
struct Card {
    val_to_num: HashMap<u64, Number>,

    col_to_bingo: Vec<usize>,
    row_to_bingo: Vec<usize>,

    has_bingo: bool,
}

#[derive(Debug, Hash, Clone)]
struct Number {
    col: usize,
    row: usize,
    marked: bool,
}

impl Parseable for Bingo {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, draws) = terminated(separated_list1(char(','), num1), many1(line_ending))(i)?;
        let (i, cards) = separated_list1(pair(many1(line_ending), space0), Card::parse)(i)?;

        Ok((i, Self { draws, cards }))
    }
}

impl Bingo {
    fn from(i: &str) -> Self {
        match Self::parse(i).finish() {
            Ok((_, parsed)) => parsed,
            Err(e) => {
                panic!("Error parsing: {}", e);
            }
        }
    }

    // draws till bingo and returns score
    fn draw_till_bingo(&mut self) -> u64 {
        for drawn in self.draws.iter() {
            for card in self.cards.iter_mut() {
                if card.check(*drawn) {
                    return drawn * card.score();
                }
            }
        }
        panic!("There was no bingo!");
    }

    // draws till bingo and returns score
    fn find_last_winner(&mut self) -> u64 {
        let mut boards_won = 0;
        let num_cards = self.cards.len();
        for drawn in self.draws.iter() {
            for card in self.cards.iter_mut().filter(|c| !c.has_bingo()) {
                if card.check(*drawn) {
                    boards_won += 1;
                    if boards_won == num_cards {
                        return drawn * card.score();
                    }
                }
            }
        }
        panic!("There was no bingo!");
    }
}

impl Parseable for Card {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, lines) = terminated(
            separated_list1(pair(line_ending, space0), numline),
            line_ending,
        )(i)?;
        let num_rows = lines.len();

        let mut val_to_num = HashMap::new();

        for (row, line) in lines.into_iter().enumerate() {
            // TODO: Integrate into nom errors
            assert_eq!(line.len(), num_rows, "Card not square.");

            for (col, num) in line.into_iter().enumerate() {
                val_to_num.insert(
                    num,
                    Number {
                        col,
                        row,
                        marked: false,
                    },
                );
            }
        }
        Ok((
            i,
            Self {
                val_to_num,
                col_to_bingo: vec![0; num_rows],
                row_to_bingo: vec![0; num_rows],
                has_bingo: false,
            },
        ))
    }
}

impl Card {
    // returns if card has bingo
    fn check(&mut self, num: u64) -> bool {
        let needed = self.needed_for_bingo();
        if let Entry::Occupied(num) = self.val_to_num.entry(num) {
            let mut num = num.into_mut();
            if !num.marked {
                num.marked = true;
                self.col_to_bingo[num.col] += 1;
                self.row_to_bingo[num.row] += 1;
                if self.col_to_bingo[num.col] == needed || self.row_to_bingo[num.row] == needed {
                    self.has_bingo = true;
                }
            }
        }
        self.has_bingo
    }

    fn has_bingo(&self) -> bool {
        self.has_bingo
    }

    fn score(&self) -> u64 {
        self.val_to_num
            .iter()
            .filter_map(|(val, num)| if !num.marked { Some(val) } else { None })
            .sum()
    }

    fn needed_for_bingo(&self) -> usize {
        self.row_to_bingo.len()
    }
}

fn numline(i: &str) -> IResult<&str, Vec<u64>> {
    separated_list1(space1, num1)(i)
}

fn num1(i: &str) -> IResult<&str, u64> {
    map(digit1, |n: &str| n.parse::<u64>().unwrap())(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let mut bingo = Bingo::from(&content);
        assert_eq!(bingo.cards.len(), 3, "did not parse all cards");
        assert_eq!(
            bingo.cards[0].col_to_bingo.len(),
            5,
            "cards have wrong dimensions"
        );
        assert_eq!(
            bingo.cards[1].col_to_bingo.len(),
            5,
            "cards have wrong dimensions"
        );
        assert_eq!(
            bingo.cards[2].col_to_bingo.len(),
            5,
            "cards have wrong dimensions"
        );

        assert_eq!(bingo.draw_till_bingo(), 4512);
    }

    #[test]
    fn test_part2() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let mut bingo = Bingo::from(&content);
        assert_eq!(bingo.find_last_winner(), 1924);
    }
}
