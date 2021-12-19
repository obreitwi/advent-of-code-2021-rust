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

    let mut grid = Grid::read(&content);

    part1(&mut grid.clone());
    part2(&mut grid);
    Ok(())
}

fn part1(grid: &mut Grid) {
    grid.evolve(100);
    println!("part 1: {} flashes", grid.flashes_total);
}
fn part2(grid: &mut Grid) {
    println!("part 2: round {}", grid.find_synchronous_flash());
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Grid {
    grid: Vec<Vec<usize>>,
    size_x: usize,
    size_y: usize,

    flashes_total: usize,
}

impl Parseable for Grid {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, grid) = separated_list1(line_ending, many1(num1::<usize>))(i)?;

        let size_y = grid.len();
        let size_x = grid[0].len();

        for line in grid.iter() {
            assert_eq!(line.len(), size_x);
        }

        Ok((
            i,
            Self {
                grid,
                size_x,
                size_y,
                flashes_total: 0,
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

    fn evolve(&mut self, rounds: usize) {
        for _ in 0..rounds {
            let mut flashed = HashSet::new();

            for y in 0..self.size_y {
                for x in 0..self.size_x {
                    self.juice_up(Point { x, y }, &mut flashed);
                }
            }
            for Point { x, y } in flashed {
                self.grid[y][x] = 0;
            }
        }
    }

    fn find_synchronous_flash(&mut self) -> usize {
        let mut round = 0;
        loop {
            let flashes = self.flashes_total;
            self.evolve(1);
            round += 1;
            if self.flashes_total - flashes == self.size_x * self.size_y {
                return round;
            }
        }
    }

    fn juice_up(&mut self, point: Point, flashed: &mut HashSet<Point>) {
        let Point { x, y } = point;

        self.grid[y][x] += 1;
        if self.grid[y][x] > 9 && !flashed.contains(&point) {
            self.flash(point, flashed);
        }
    }

    fn flash(&mut self, point: Point, flashed: &mut HashSet<Point>) {
        let Point { x, y } = point;
        self.grid[y][x] = 0;

        flashed.insert(point);

        self.flashes_total += 1;

        let mut indices = Vec::new();
        for dx in -1..2 {
            for dy in -1..2 {
                indices.push((x as i64 + dx, y as i64 + dy));
            }
        }

        let compute_limits = |n, size| {
            (
                if n > 0 { n - 1 } else { n },
                if n < size - 1 { n + 1 } else { n },
            )
        };
        let (y_min, y_max) = compute_limits(y, self.size_y);
        let (x_min, x_max) = compute_limits(x, self.size_x);

        for y in y_min..y_max + 1 {
            for x in x_min..x_max + 1 {
                if x == point.x && y == point.y {
                    continue;
                }
                self.juice_up(Point { x, y }, flashed);
            }
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The `f` value implements the `Write` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        for line in self.grid.iter() {
            for elem in line.iter() {
                write!(f, "{}", elem)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {}

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
    fn test_part1() -> Result<()> {
        let mut grid = Grid::read(&read_to_string(PathBuf::from("debug.txt"))?);

        assert_eq!(grid.size_x, 10);
        assert_eq!(grid.size_y, 10);

        grid.evolve(100);
        assert_eq!(grid.flashes_total, 1656);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let mut grid = Grid::read(&read_to_string(PathBuf::from("debug.txt"))?);

        assert_eq!(grid.find_synchronous_flash(), 195);
        Ok(())
    }
}
