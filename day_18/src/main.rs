#![allow(unused_imports)]
use anyhow::{bail, Context, Error, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{
        alpha1, anychar, char, digit1, hex_digit1, line_ending, multispace1, none_of, one_of,
        space0, space1,
    },
    combinator::{map, map_res, not, value},
    error::ParseError,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    ErrorConvert, Finish, IResult,
};
use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::env;
use std::fmt;
use std::fs::read_to_string;
use std::ops::Add;
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
    let numbers = separated_list1(line_ending, SnailfishNumber::parse)(&content)
        .unwrap()
        .1;
    assert_eq!(numbers.len(), 100);

    part1(&numbers[..]);
    part2(&numbers[..]);

    Ok(())
}

fn part1(numbers: &[SnailfishNumber]) {
    let mut result = numbers[0].clone();
    for num in numbers.iter().skip(1) {
        result = &result + num;
    }
    println!("part 1: {}", result.magnitude());
}

fn part2(numbers: &[SnailfishNumber]) {
    println!("part 2: {}", find_largest_magnitude(numbers));
}

#[derive(Debug, Clone)]
enum SnailfishNumber {
    Regular(u64),
    Pair(Box<(SnailfishNumber, SnailfishNumber)>),
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

impl Parseable for SnailfishNumber {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, val) = alt((Self::parse_pair, Self::parse_regular))(i)?;
        Ok((i, val))
    }
}

impl SnailfishNumber {
    fn parse_regular(i: &str) -> IResult<&str, Self> {
        let (i, num) = map(one_of("0123456789"), |n| {
            n.to_string().parse::<u64>().unwrap()
        })(i)?;
        Ok((i, Self::Regular(num)))
    }

    fn parse_pair(i: &str) -> IResult<&str, Self> {
        let (i, _) = tag("[")(i)?;
        let (i, first) = Self::parse(i)?;
        let (i, _) = char(',')(i)?;
        let (i, second) = Self::parse(i)?;
        let (i, _) = tag("]")(i)?;
        Ok((i, Self::Pair(Box::new((first, second)))))
    }

    fn reduce(&mut self) {
        while self.explode(0).0 || self.split() {}
    }

    fn explode(&mut self, level: usize) -> (bool, Option<u64>, Option<u64>) {
        match self {
            Self::Regular(_) => (false, None, None),
            Self::Pair(pair) => {
                if level >= 4 {
                    let pair = pair.clone();
                    *self = Self::Regular(0);
                    match (pair.0, pair.1) {
                        (Self::Regular(left), Self::Regular(right)) => {
                            (true, Some(left), Some(right))
                        }
                        _ => panic!("Tried to explode nested pair."),
                    }
                } else {
                    let (exploded, to_left, to_right) = pair.0.explode(level + 1);
                    if exploded {
                        if let Some(num) = to_right {
                            pair.1.add_from_left(num);
                        }
                        (exploded, to_left, None)
                    } else {
                        let (exploded, to_left, to_right) = pair.1.explode(level + 1);
                        if exploded {
                            if let Some(num) = to_left {
                                pair.0.add_from_right(num);
                            }
                        }
                        (exploded, None, to_right)
                    }
                }
            }
        }
    }

    fn split(&mut self) -> bool {
        match self {
            Self::Pair(pair) => {
                if pair.0.split() {
                    true
                } else {
                    pair.1.split()
                }
            }
            Self::Regular(num) => {
                if *num >= 10 {
                    let left = *num / 2;
                    let right = *num - left;
                    *self = Self::Pair(Box::new((Self::Regular(left), Self::Regular(right))));
                    true
                } else {
                    false
                }
            }
        }
    }

    fn add_from_left(&mut self, to_add: u64) {
        match self {
            Self::Regular(num) => {
                *num += to_add;
            }
            Self::Pair(pair) => {
                pair.0.add_from_left(to_add);
            }
        }
    }

    fn add_from_right(&mut self, to_add: u64) {
        match self {
            Self::Regular(num) => {
                *num += to_add;
            }
            Self::Pair(pair) => {
                pair.1.add_from_right(to_add);
            }
        }
    }

    fn magnitude(&self) -> u64 {
        match self {
            Self::Regular(num) => *num,
            Self::Pair(pair) => 3 * pair.0.magnitude() + 2 * pair.1.magnitude(),
        }
    }
}

impl Add for SnailfishNumber {
    type Output = SnailfishNumber;

    fn add(self, other: Self) -> Self {
        let mut output = SnailfishNumber::Pair(Box::new((self, other)));
        output.reduce();
        output
    }
}

impl Add for &SnailfishNumber {
    type Output = SnailfishNumber;

    fn add(self, other: Self) -> SnailfishNumber {
        let mut output = SnailfishNumber::Pair(Box::new((self.clone(), other.clone())));
        output.reduce();
        output
    }
}

impl fmt::Display for SnailfishNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Regular(num) => {
                write!(f, "{}", num)
            }
            Self::Pair(pair) => {
                write!(f, "[{},{}]", pair.0, pair.1)
            }
        }
    }
}

fn find_largest_magnitude(numbers: &[SnailfishNumber]) -> u64 {
    numbers
        .iter()
        .flat_map(|x| numbers.iter().map(|y| (x, y)).collect::<Vec<_>>())
        .filter_map(|(x, y)| {
            if x != y {
                Some((x + y).magnitude())
            } else {
                None
            }
        })
        .max()
        .unwrap()
}

impl PartialEq for SnailfishNumber {
    fn eq(&self, other: &SnailfishNumber) -> bool {
        match (self, other) {
            (SnailfishNumber::Regular(left), &SnailfishNumber::Regular(right)) => *left == right,
            (SnailfishNumber::Pair(ref left), &SnailfishNumber::Pair(ref right)) => {
                left.0 == right.0 && left.1 == right.1
            }
            (_, _) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_1() {
        compare(
            &["[1,1]", "[2,2]", "[3,3]", "[4,4]"],
            "[[[[1,1],[2,2]],[3,3]],[4,4]]",
        );
    }

    #[test]
    fn part1_2() {
        compare(
            &["[1,1]", "[2,2]", "[3,3]", "[4,4]", "[5,5]"],
            "[[[[3,0],[5,3]],[4,4]],[5,5]]",
        );
    }

    #[test]
    fn part1_3() {
        compare(
            &["[1,1]", "[2,2]", "[3,3]", "[4,4]", "[5,5]", "[6,6]"],
            "[[[[5,0],[7,4]],[5,5]],[6,6]]",
        );
    }

    #[test]
    fn part1_4() {
        compare(
            &[
                "[[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]",
                "[7,[[[3,7],[4,3]],[[6,3],[8,8]]]]",
                "[[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]",
                "[[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]",
                "[7,[5,[[3,8],[1,4]]]]",
                "[[2,[2,2]],[8,[8,1]]]",
                "[2,9]",
                "[1,[[[9,3],9],[[9,0],[0,7]]]]",
                "[[[5,[7,4]],7],1]",
                "[[[[4,2],2],6],[8,7]]",
            ],
            "[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]",
        )
    }

    #[test]
    fn part1_5() {
        assert_eq!(
            SnailfishNumber::parse("[[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]")
                .unwrap()
                .1
                .magnitude(),
            3488
        );
    }

    #[test]
    fn part2() {
        let numbers: Vec<_> = [
            "[[[0,[5,8]],[[1,7],[9,6]]],[[4,[1,2]],[[1,4],2]]]",
            "[[[5,[2,8]],4],[5,[[9,9],0]]]",
            "[6,[[[6,2],[5,6]],[[7,6],[4,7]]]]",
            "[[[6,[0,7]],[0,9]],[4,[9,[9,0]]]]",
            "[[[7,[6,4]],[3,[1,3]]],[[[5,5],1],9]]",
            "[[6,[[7,3],[3,2]]],[[[3,8],[5,7]],4]]",
            "[[[[5,4],[7,7]],8],[[8,3],8]]",
            "[[9,3],[[9,9],[6,[4,9]]]]",
            "[[2,[[7,7],7]],[[5,8],[[9,3],[0,2]]]]",
            "[[[[5,2],5],[8,[3,7]]],[[5,[7,5]],[4,4]]]",
        ]
        .iter()
        .map(|n| SnailfishNumber::parse(n).unwrap().1)
        .collect();

        assert_eq!(numbers.len(), 10);

        assert_eq!(find_largest_magnitude(&numbers[..]), 3993);
    }

    fn compare(input: &[&str], want: &str) {
        let numbers: Vec<_> = input
            .iter()
            .map(|n| SnailfishNumber::parse(n).unwrap().1)
            .collect();

        let want = SnailfishNumber::parse(want).unwrap().1;

        let mut got = numbers[0].clone();
        for num in numbers.iter().skip(1) {
            got = got + num.clone();
        }

        eprintln!("Want: {}", want);
        eprintln!("Got: {}", got);

        assert_eq!(want, got);
    }
}
