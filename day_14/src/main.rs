#![allow(unused_imports)]
use anyhow::{bail, Context, Error, Result};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{
        alpha1, anychar, char, digit1, line_ending, multispace1, none_of, one_of, space0, space1,
    },
    combinator::{map, map_res, value, verify},
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

    let polymerizer = Polymerizer::parse(&content).finish().unwrap().1;

    part1(&polymerizer);
    part2(&polymerizer);
    Ok(())
}

fn part1(poly: &Polymerizer) {
    let limits = poly.grow(10).find_limits();
    println!("part 1: {}", limits.1 - limits.0);
}

fn part2(poly: &Polymerizer) {
    let limits = poly.grow_stats(40).find_limits();
    println!("part 2: {}", limits.1 - limits.0);
}

#[derive(Debug, Clone)]
struct Polymerizer {
    template: Vec<PolyElement>,
    rules: Vec<Rule>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
struct PolyElement(char);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Rule {
    first: PolyElement,
    second: PolyElement,
    result: PolyElement,
}

struct PolymerStats {
    counts: HashMap<(PolyElement, PolyElement), usize>,
    first: PolyElement,
    last: PolyElement,
}

type Polymer = Vec<PolyElement>;

impl Polymerizer {
    fn grow(&self, steps: usize) -> Polymer {
        let mut polymer = self.template.clone();
        for _ in 0..steps {
            let mut iter = polymer.into_iter();
            let first = iter.next().unwrap();

            polymer = iter.fold(vec![first], |mut grown, next| {
                let last = grown.last().unwrap();
                for r in self.rules.iter() {
                    if r.applies(last, &next) {
                        grown.push(r.result);
                        break;
                    }
                }
                grown.push(next);
                grown
            });
        }
        polymer
    }

    fn grow_stats(&self, steps: usize) -> PolymerStats {
        let mut counts = self.template_pairs();
        for _ in 0..steps {
            let mut updated = HashMap::new();
            for r in self.rules.iter() {
                if let Some(count) = counts.get(&r.pair()) {
                    for pair in r.produces() {
                        *updated.entry(pair).or_insert(0) += count;
                    }
                }
            }
            counts = updated;
        }
        PolymerStats {
            counts,
            first: *self.template.first().unwrap(),
            last: *self.template.last().unwrap(),
        }
    }

    fn template_pairs(&self) -> HashMap<(PolyElement, PolyElement), usize> {
        let mut iter = self.template.iter().cloned();
        let mut previous = iter.next().unwrap();

        iter.fold(HashMap::new(), |mut counts, next| {
            *counts.entry((previous, next)).or_insert(0) += 1;
            previous = next;
            counts
        })
    }
}

impl Rule {
    fn applies(&self, first: &PolyElement, second: &PolyElement) -> bool {
        first == &self.first && second == &self.second
    }

    fn pair(&self) -> (PolyElement, PolyElement) {
        (self.first, self.second)
    }

    fn produces(&self) -> [(PolyElement, PolyElement); 2] {
        [(self.first, self.result), (self.result, self.second)]
    }
}

impl Parseable for Polymerizer {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, (template, _, rules)) = tuple((
            many1(PolyElement::parse),
            many1(line_ending),
            many1(Rule::parse),
        ))(i)?;

        Ok((i, Self { template, rules }))
    }
}

impl Parseable for PolyElement {
    fn parse(i: &str) -> IResult<&str, Self> {
        map(verify(anychar, |c| c.is_uppercase()), PolyElement)(i)
    }
}

impl Parseable for Rule {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, (first, second, _, result, _)) = tuple((
            PolyElement::parse,
            PolyElement::parse,
            tag(" -> "),
            PolyElement::parse,
            line_ending,
        ))(i)?;

        Ok((
            i,
            Self {
                first,
                second,
                result,
            },
        ))
    }
}

trait Limits {
    fn find_limits(&self) -> (usize, usize);
}

impl Limits for Polymer {
    fn find_limits(&self) -> (usize, usize) {
        let mut counts = HashMap::<PolyElement, usize>::new();
        for elem in self.iter() {
            *counts.entry(*elem).or_insert(0) += 1;
        }
        let max = counts.iter().max_by(|l, r| l.1.cmp(r.1)).unwrap().1;
        let min = counts.iter().min_by(|l, r| l.1.cmp(r.1)).unwrap().1;

        (*min, *max)
    }
}

impl Limits for PolymerStats {
    fn find_limits(&self) -> (usize, usize) {
        let mut counts = HashMap::<PolyElement, usize>::new();
        for ((poly1, poly2), count) in self.counts.iter() {
            *counts.entry(*poly1).or_insert(0) += count;
            *counts.entry(*poly2).or_insert(0) += count;
        }
        // we count everything twice, except for the very first and very last element:
        *counts.get_mut(&self.first).unwrap() += 1;
        *counts.get_mut(&self.last).unwrap() += 1;

        // we count everything twice
        let max = counts.iter().max_by(|l, r| l.1.cmp(r.1)).unwrap().1 / 2;
        let min = counts.iter().min_by(|l, r| l.1.cmp(r.1)).unwrap().1 / 2;

        (min, max)
    }
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let polymerizer = Polymerizer::parse(&content).finish().unwrap().1;

        assert_eq!(polymerizer.grow(5).len(), 97);
        assert_eq!(polymerizer.grow(10).len(), 3073);
        assert_eq!(polymerizer.grow(10).find_limits(), (161, 1749));
    }

    #[test]
    fn test_part2() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let polymerizer = Polymerizer::parse(&content).finish().unwrap().1;

        assert_eq!(polymerizer.grow_stats(10).find_limits(), (161, 1749));
    }
}
