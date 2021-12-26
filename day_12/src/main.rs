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
use std::borrow::Cow;
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

    let ct = Connectome::read(&content);

    part1(&ct);
    part2(&ct);
    Ok(())
}

fn part1(ct: &Connectome) {
    let paths = ct.get_paths();
    println!("part 1: There are {} paths.", paths.len());
}

fn part2(ct: &Connectome) {
    todo!()
}

#[derive(Debug, Clone)]
struct Connectome {
    reachable: HashMap<Location, HashSet<Location>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Location {
    label: Cow<'static, str>,
    size: LocationSize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LocationSize {
    Big,
    Small,
}

static START: Location = Location {
    label: Cow::Borrowed("start"),
    size: LocationSize::Small,
};
static END: Location = Location {
    label: Cow::Borrowed("end"),
    size: LocationSize::Small,
};

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

type RawRoute = (Location, Location);

impl Parseable for Connectome {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, routes) = separated_list1(line_ending, RawRoute::parse)(i)?;

        let mut reachable: HashMap<Location, HashSet<Location>> = HashMap::new();
        for route in routes {
            reachable
                .entry(route.0.clone())
                .or_default()
                .insert(route.1.clone());
            reachable.entry(route.1).or_default().insert(route.0);
        }

        Ok((i, Self { reachable }))
    }
}

impl Parseable for RawRoute {
    fn parse(i: &str) -> IResult<&str, Self> {
        separated_pair(Location::parse, char('-'), Location::parse)(i)
    }
}

impl Parseable for Location {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, label) = alpha1(i)?;
        let size = if label.chars().next().unwrap().is_uppercase() {
            LocationSize::Big
        } else {
            LocationSize::Small
        };

        Ok((
            i,
            Self {
                label: Cow::Owned(label.to_owned()),
                size,
            },
        ))
    }
}

impl Connectome {
    fn read(i: &str) -> Self {
        if let Ok((_, parsed)) = Self::parse(i).finish() {
            parsed
        } else {
            panic!("Could not parse input.");
        }
    }

    pub fn get_paths(&self) -> HashSet<Route> {
        self.get_paths_inner(Route::from(START.clone()))
    }

    fn get_paths_inner(&self, current: Route) -> HashSet<Route> {
        let visited = current.visited_by(LocationSize::Small);

        let possible = self.reachable[current.locations.last().unwrap()]
            .iter()
            .filter(|l| !visited.contains(l))
            .cloned()
            .collect::<Vec<_>>();

        let mut routes = HashSet::new();

        for next in possible {
            let new = current.add(next);

            if new.is_complete() {
                routes.insert(new);
            } else {
                routes.extend(self.get_paths_inner(new));
            }
        }
        routes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Route {
    pub locations: Vec<Location>,
}

impl Route {
    fn is_complete(&self) -> bool {
        self.locations.last().map(|l| *l == END).unwrap_or(false)
    }

    fn add(&self, to_add: Location) -> Route {
        let mut locations = self.locations.clone();
        locations.push(to_add);
        Self { locations }
    }

    pub fn visited_by(&self, size: LocationSize) -> HashSet<Location> {
        self.locations
            .iter()
            .filter(|l| l.size == size)
            .cloned()
            .collect()
    }
}

impl From<Location> for Route {
    fn from(location: Location) -> Route {
        Self {
            locations: vec![location],
        }
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The `f` value implements the `Write` trait, which is what the
        // write! macro is expecting. Note that this formatting ignores the
        // various flags provided to format strings.
        if let Some(first) = self.locations.first() {
            write!(f, "{}", first.label)?;
        }
        for location in self.locations.iter().skip(1) {
            write!(f, " -> {}", location.label)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part1() {
        let content = read_to_string(PathBuf::from("debug-10.txt")).unwrap();
        let connections = Connectome::read(&content);
        println!("{:#?}", connections);
        for conn in connections.get_paths() {
            println!("{}", conn);
        }
        assert_eq!(connections.get_paths().len(), 10);

        let content = read_to_string(PathBuf::from("debug-19.txt")).unwrap();
        let connections = Connectome::read(&content);
        assert_eq!(connections.get_paths().len(), 19);

        let content = read_to_string(PathBuf::from("debug-226.txt")).unwrap();
        let connections = Connectome::read(&content);
        assert_eq!(connections.get_paths().len(), 226);
    }

    #[test]
    fn test_part2() {}
}
