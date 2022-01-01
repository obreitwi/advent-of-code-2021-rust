#![allow(unused_imports)]
#![feature(map_first_last)]
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
use std::cmp::{Ord, Ordering, PartialOrd};
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
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

    let grid = Grid::read(&content);

    part1(&grid);
    part2(&grid);
    Ok(())
}

fn part1(grid: &Grid) {
    let parent_cum_risk = grid.get_lowest_risk_paths();
    println!(
        "part 1: {}",
        parent_cum_risk[grid.size_y - 1][grid.size_x - 1].risk
    )
}

fn part2(grid: &Grid) {
    let grid = grid.grow(5);
    let parent_cum_risk = grid.get_lowest_risk_paths();
    println!(
        "part 2: {}",
        parent_cum_risk[grid.size_y - 1][grid.size_x - 1].risk
    )
}

#[derive(Debug, Clone)]
struct Grid {
    size_x: usize,
    size_y: usize,
    risk: Vec<Vec<usize>>,
}

impl Parseable for Grid {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, risk) = separated_list1(line_ending, many1(num1::<usize>))(i)?;

        let size_y = risk.len();
        let size_x = risk[0].len();

        for line in risk.iter() {
            assert_eq!(line.len(), size_x);
        }

        Ok((
            i,
            Self {
                risk,
                size_x,
                size_y,
            },
        ))
    }
}

impl Grid {
    fn read(i: &str) -> Self {
        if let Ok((_, parsed)) = Self::parse(i).finish() {
            parsed
        } else {
            panic!("Could not parse input.");
        }
    }

    fn get_lowest_risk_paths(&self) -> Vec<Vec<Point>> {
        // encode which point is the parent and the cumulative costs
        let mut parent_cum_risk: Vec<Vec<Point>> = vec![
            vec![
                Point {
                    x: usize::MAX,
                    y: usize::MAX,
                    risk: usize::MAX
                };
                self.size_x
            ];
            self.size_y
        ];
        let mut queue: BTreeSet<Point> = BTreeSet::new();
        queue.insert(Point {
            x: 0,
            y: 0,
            risk: 0,
        });
        parent_cum_risk[0][0] = Point {
            x: 0,
            y: 0,
            risk: self.risk[0][0],
        };

        while let Some(current) = queue.pop_first() {
            // check if we already found a better alternative
            if parent_cum_risk[current.y][current.x].risk < current.risk {
                assert_ne!(parent_cum_risk[current.y][current.x].risk, usize::MAX);
                // continue;
            }

            if current.x > 0 {
                self.update(
                    &current,
                    current.x - 1,
                    current.y,
                    &mut queue,
                    &mut parent_cum_risk,
                );
            }
            if current.y > 0 {
                self.update(
                    &current,
                    current.x,
                    current.y - 1,
                    &mut queue,
                    &mut parent_cum_risk,
                );
            }
            if current.x < self.size_x - 1 {
                self.update(
                    &current,
                    current.x + 1,
                    current.y,
                    &mut queue,
                    &mut parent_cum_risk,
                );
            }
            if current.y < self.size_y - 1 {
                self.update(
                    &current,
                    current.x,
                    current.y + 1,
                    &mut queue,
                    &mut parent_cum_risk,
                );
            }
        }
        assert_eq!(queue.len(), 0);
        parent_cum_risk
    }

    // check if given point is reachable from parent Point with lower totla risk and update
    // accordingly
    fn update(
        &self,
        parent: &Point,
        new_x: usize,
        new_y: usize,
        queue: &mut BTreeSet<Point>,
        parent_cum_risk: &mut Vec<Vec<Point>>,
    ) {
        let risk_current = parent_cum_risk[new_y][new_x].risk;
        let risk_step = self.risk[new_y][new_x];
        let risk_new = parent.risk + risk_step;
        if risk_new < risk_current {
            parent_cum_risk[new_y][new_x] = Point {
                risk: risk_new,
                ..*parent
            };
            queue.insert(Point {
                x: new_x,
                y: new_y,
                risk: risk_new,
            });
        }
    }

    fn grow(&self, steps: usize) -> Self {
        let mut risk = vec![vec![0; self.size_x * steps]; self.size_y * steps];
        for (y, row) in risk.iter_mut().enumerate() {
            for (x, elem) in row.iter_mut().enumerate() {
                let mut new_val =
                    self.risk[y % self.size_y][x % self.size_x] + x / self.size_x + y / self.size_y;
                while new_val > 9 {
                    new_val -= 9;
                }
                *elem = new_val;
            }
        }

        Self {
            size_x: self.size_x * steps,
            size_y: self.size_y * steps,
            risk,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Point {
    risk: usize,
    x: usize,
    y: usize,
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        self.risk
            .cmp(&other.risk)
            .then_with(|| self.y.cmp(&other.y))
            .then_with(|| self.x.cmp(&other.x))
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

fn num1<T: std::str::FromStr>(i: &str) -> IResult<&str, T> {
    map(one_of("0123456789"), |c: char| {
        c.to_string()
            .parse::<T>()
            .unwrap_or_else(|_| panic!("could not parse number"))
    })(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let grid = Grid::read(&content);

        let parent_cum_risk = grid.get_lowest_risk_paths();

        for row in grid.risk.iter() {
            for elem in row.iter() {
                eprint!("{}", elem);
            }
            eprintln!();
        }
        eprintln!();

        for row in parent_cum_risk.iter() {
            for elem in row.iter() {
                if elem.risk < usize::MAX {
                    eprint!("{:02} ", elem.risk);
                } else {
                    eprint!("XX ");
                }
            }
            eprintln!();
        }

        assert_eq!(parent_cum_risk[grid.size_y - 1][grid.size_x - 1].risk, 40);
    }

    #[test]
    fn test_part2() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let grid = Grid::read(&content).grow(5);

        let parent_cum_risk = grid.get_lowest_risk_paths();

        for row in grid.risk.iter() {
            for elem in row.iter() {
                eprint!("{}", elem);
            }
            eprintln!();
        }
        eprintln!();

        for row in parent_cum_risk.iter() {
            for elem in row.iter() {
                if elem.risk < usize::MAX {
                    eprint!("{:03} ", elem.risk);
                } else {
                    eprint!("XX ");
                }
            }
            eprintln!();
        }

        assert_eq!(parent_cum_risk[grid.size_y - 1][grid.size_x - 1].risk, 315);
    }
}
