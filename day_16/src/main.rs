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

    let grid = Grid::read(&content);

    part1(&grid);
    part2(&grid);
    Ok(())
}

fn part1(grid: &Grid) {
    let low_points = grid.get_low_points();

    println!(
        "part1: {} low points with risk of {}",
        low_points.len(),
        low_points.iter().map(|p| p.risk_level()).sum::<usize>()
    );
}

fn part2(grid: &Grid) {
    println!("part 2: product: {}", grid.get_largest_basins().iter().product::<usize>());
}

#[derive(Debug, Clone)]
struct Grid {
    size_x: usize,
    size_y: usize,
    grid: Vec<Vec<usize>>,
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

    fn get_low_points(&self) -> Vec<Point> {
        let mut points = Vec::new();

        for y in 0..self.size_y {
            for x in 0..self.size_x {
                let height = self.grid[y][x];

                if y > 0 && self.grid[y - 1][x] <= height {
                    continue;
                }
                if x > 0 && self.grid[y][x - 1] <= height {
                    continue;
                }
                if x < self.size_x - 1 && self.grid[y][x + 1] <= height {
                    continue;
                }
                if y < self.size_y - 1 && self.grid[y + 1][x] <= height {
                        continue;
                }
                points.push(Point { x, y, height });
            }
        }

        points
    }

    fn get_basin(&self, point: &Point) -> HashSet<(usize, usize)> {
        let origin = (point.x, point.y);

        let mut basin_queue = BasinQueue::new(self);
        basin_queue.checked_push(origin);

        while let Some(current) = basin_queue.pop_front() {
            if current.1 > 0 {
                basin_queue.checked_push((current.0, current.1 - 1));
            }
            if current.0 > 0 {
                basin_queue.checked_push((current.0 - 1, current.1));
            }
            if current.0 < self.size_x - 1 {
                basin_queue.checked_push((current.0 + 1, current.1));
            }
            if current.1 < self.size_y - 1 {
                basin_queue.checked_push((current.0, current.1 + 1));
            }
        }

        basin_queue.basin
    }

    fn get_basin_sizes(&self) -> Vec<usize> {
        self.get_low_points().iter().map(|p| self.get_basin(p).len()).collect()
    }

    fn get_largest_basins(&self) -> Vec<usize> {
        let mut basin_sizes = self.get_basin_sizes();
        basin_sizes.sort_unstable();
        let all_but_three = basin_sizes.len() - 3;
        basin_sizes.into_iter().skip(all_but_three).collect()
    }
}
#[derive(Debug, Clone)]
struct BasinQueue<'a> {
    queue: VecDeque<(usize, usize)>,
    basin: HashSet<(usize, usize)>,
    grid: &'a Grid,
}

impl<'a> BasinQueue<'a> {
    fn new(grid: &'a Grid) -> Self {
        Self {
            queue: VecDeque::new(),
            basin: HashSet::new(),
            grid
        }
    }

    fn checked_push(&mut self, point: (usize, usize)) {
        if !self.basin.contains(&point) && self.grid.grid[point.1][point.0] < 9 {
            self.queue.push_back(point);
            self.basin.insert(point);
        }
    }

    fn pop_front(&mut self) -> Option<(usize, usize)> {
        self.queue.pop_front()
    }
}

#[derive(Debug, Clone)]
struct Point {
    height: usize,
    x: usize,
    y: usize,
}

impl Point {
    fn risk_level(&self) -> usize {
        self.height + 1
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
        let low_points = grid.get_low_points();
        assert_eq!(low_points.len(), 4);
        assert_eq!(low_points.iter().map(|p| p.risk_level()).sum::<usize>(), 15);
    }

    #[test]
    fn test_part2() {
        let content = read_to_string(PathBuf::from("debug.txt")).unwrap();
        let grid = Grid::read(&content);
        println!("{:#?}", grid.get_basin_sizes());
        assert_eq!(grid.get_largest_basins().iter().product::<usize>(), 1134);
    }
}
