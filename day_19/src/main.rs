#![allow(unused_imports)]
use anyhow::{bail, Context, Error, Result};
use lazy_static::lazy_static;
use ndarray::{arr1, arr2};
use nom::{
    branch::alt,
    bytes::complete::{is_a, is_not, tag, take_while1},
    character::complete::{
        alpha1, anychar, char, digit1, hex_digit1, line_ending, multispace1, none_of, one_of,
        space0, space1,
    },
    combinator::{map, map_res, not, opt, value},
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
use std::ops::{Add, Neg, Sub};
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
    let scanners = Vec::<Scanner>::parse(&content).finish().unwrap().1;

    let aligned = part1(scanners)?;
    part2(&aligned[..])?;

    Ok(())
}

trait Parseable: Sized {
    fn parse(i: &str) -> IResult<&str, Self>;
}

fn part1(scanners: Vec<Scanner>) -> Result<Vec<Scanner>> {
    let aligned = align(scanners, 12)?;
    let num_beacons = count_beacons(&aligned[..])?;
    println!("part1: there are {} beacons", num_beacons);

    Ok(aligned)
}

fn part2(scanners: &[Scanner]) -> Result<()> {
    let positions = scanners
        .iter()
        .map(|s| s.position.unwrap())
        .collect::<Vec<_>>();

    let manhattan = positions
        .iter()
        .flat_map(|x| positions.iter().map(|y| x.manhattan(y)))
        .max()
        .unwrap();

    println!("part2: max manhattan distance: {}", manhattan);

    Ok(())
}

lazy_static! {
    static ref ROTATIONS: Vec<Matrix> = generate_unique_rotations();
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
struct Position {
    x: i64,
    y: i64,
    z: i64,
}

impl Position {
    fn manhattan(&self, other: &Self) -> u64 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y) + self.z.abs_diff(other.z)
    }
}

impl Add for &Position {
    type Output = Position;

    fn add(self, other: Self) -> Position {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Add for Position {
    type Output = Position;

    fn add(self, other: Self) -> Position {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for &Position {
    type Output = Position;

    fn sub(self, other: Self) -> Position {
        Position {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Sub for Position {
    type Output = Position;

    fn sub(self, other: Self) -> Position {
        Position {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Neg for Position {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Position {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Neg for &Position {
    type Output = Position;

    fn neg(self) -> Self::Output {
        Position {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

type Vector = ndarray::Array1<i64>;
type Matrix = ndarray::Array2<i64>;
type MatrixView<'a> = ndarray::ArrayView2<'a, i64>;

impl From<Position> for Vector {
    fn from(pos: Position) -> Vector {
        arr1(&[pos.x, pos.y, pos.z])
    }
}

impl From<Vector> for Position {
    fn from(vec: Vector) -> Position {
        Position {
            x: vec[0],
            y: vec[1],
            z: vec[2],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Scanner {
    id: usize,

    // whether or not this scanner has its position determined
    position: Option<Position>,

    // beacons in relative coordinates
    beacons: Vec<Position>,
}

impl Default for Scanner {
    fn default() -> Self {
        Scanner {
            id: usize::MAX,
            position: None,
            beacons: vec![],
        }
    }
}

impl Scanner {
    fn rotate_by(self, rotation: MatrixView) -> Self {
        let rotated = self
            .beacons
            .into_iter()
            .map(|beacon| rotation.dot::<Vector>(&beacon.into()).into())
            .collect();

        Self {
            beacons: rotated,
            ..self
        }
    }

    // check if self and other see the same beacons
    fn check_match(&self, other: &Scanner, min_matches: usize) -> Option<Position> {
        let mut diffs = HashMap::<Position, usize>::new();
        for self_beacon in self.beacons.iter() {
            for other_beacon in other.beacons.iter() {
                // Remember: beacon_self = diff + beacon_in_other_coords
                // -> diff = beacon_self - beacon_in_other_coords
                let diff = self_beacon - other_beacon;
                *diffs.entry(diff).or_insert(0) += 1;
            }
        }
        let counts = {
            let mut counts = diffs.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>();
            counts.sort_by(|l, r| l.1.cmp(&r.1));
            counts
        };
        // hugh voting of differences based on diff
        if let Some((pos, count)) = counts.last() {
            if min_matches <= *count {
                Some(*pos)
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn align(unaligned: Vec<Scanner>, min_matches: usize) -> Result<Vec<Scanner>> {
    let mut origin = unaligned
        .get(0)
        .with_context(|| "must provide at least one element")?
        .clone();
    origin.position = Some(Position { x: 0, y: 0, z: 0 });
    let mut aligned = vec![origin];
    let mut unaligned = unaligned.into_iter().skip(1).collect::<VecDeque<Scanner>>();

    while !unaligned.is_empty() {
        let mut could_align = false;

        'align: for _ in 0..unaligned.len() {
            let current = unaligned.pop_front().unwrap();

            for rot in ROTATIONS.iter() {
                let current = current.clone().rotate_by(rot.into());
                let mut found: Option<Scanner> = None;
                for to_check in aligned.iter() {
                    if let Some(diff) = to_check.check_match(&current, min_matches) {
                        let new_pos = to_check.position.unwrap() + diff;
                        let mut current = current.clone();
                        current.position = Some(new_pos);
                        found = Some(current);
                        break;
                    }
                }
                if let Some(found) = found {
                    aligned.push(found);
                    could_align = true;
                    break 'align;
                }
            }
            unaligned.push_back(current);
        }
        if !could_align {
            bail!("could not align to any aligned scanner");
        }
    }

    Ok(aligned)
}

fn count_beacons(scanners: &[Scanner]) -> Result<usize> {
    let mut beacons = HashSet::<Position>::new();

    for scanner in scanners {
        let pos = scanner
            .position
            .with_context(|| "scanner has no position set")?;
        for beacon in scanner.beacons.iter() {
            beacons.insert(pos + *beacon);
        }
    }

    Ok(beacons.len())
}

impl Parseable for Scanner {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, id) = delimited(tag("--- scanner "), num1, tuple((tag(" ---"), line_ending)))(i)?;
        let position = if id == 0 {
            Some(Position { x: 0, y: 0, z: 0 })
        } else {
            None
        };
        let (i, beacons) = separated_list1(line_ending, Position::parse)(i)?;
        Ok((
            i,
            Self {
                id,
                position,
                beacons,
            },
        ))
    }
}

impl Parseable for Vec<Scanner> {
    fn parse(i: &str) -> IResult<&str, Self> {
        separated_list1(tuple((line_ending, line_ending)), Scanner::parse)(i)
    }
}

impl Parseable for Position {
    fn parse(i: &str) -> IResult<&str, Self> {
        let comma = || char(',');
        let (i, (x, _, y, _, z)) = tuple((signed, comma(), signed, comma(), signed))(i)?;
        Ok((i, Self { x, y, z }))
    }
}

fn num1<T: std::str::FromStr>(i: &str) -> IResult<&str, T> {
    map(digit1, |n: &str| {
        T::from_str(n).unwrap_or_else(|_| panic!("could not parse number, this should not happen"))
    })(i)
}

fn signed(i: &str) -> IResult<&str, i64> {
    let (i, sign) = opt(char('-'))(i)?;
    let (i, num): (&str, i64) = num1(i)?;
    let num = {
        let mut num = num;
        if sign.is_some() {
            num = -num;
        }
        num
    };
    Ok((i, num))
}

fn generate_unique_rotations() -> Vec<Matrix> {
    let mut rotations: HashSet<Matrix> = HashSet::new();
    let rotation = Matrix::eye(3);
    for step_x in 0..4 {
        let rotation = rotation.dot(&rot_x(step_x));
        for step_y in 0..4 {
            let rotation = rotation.dot(&rot_y(step_y));
            for step_z in 0..4 {
                let rotation = rotation.dot(&rot_z(step_z));
                rotations.insert(rotation);
            }
        }
    }
    rotations.into_iter().collect()
}

fn cos(i: usize) -> i64 {
    match i {
        0 => 1,
        1 => 0,
        2 => -1,
        3 => 0,
        _ => panic!("Specified invalid step for cos: {}", i),
    }
}

fn sin(i: usize) -> i64 {
    match i {
        0 => 0,
        1 => 1,
        2 => 0,
        3 => -1,
        _ => panic!("Specified invalid step for cos: {}", i),
    }
}

#[rustfmt::skip]
fn rot_x(i: usize) -> Matrix {
    arr2(&[
        [1,     0,       0],
        [0, cos(i), -sin(i)],
        [0, sin(i),  cos(i)],
    ])
}

#[rustfmt::skip]
fn rot_y(i: usize) -> Matrix {
    arr2(&[
        [ cos(i), 0, sin(i)],
        [      0, 1,     0],
        [-sin(i), 0, cos(i)],
    ])
}

#[rustfmt::skip]
fn rot_z(i: usize) -> Matrix {
    arr2(&[
        [cos(i), -sin(i), 0],
        [sin(i),  cos(i), 0],
        [    0,        0, 1],
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() -> Result<()> {
        let content = read_to_string(PathBuf::from("input.txt"))?;
        let scanners = Vec::<Scanner>::parse(&content).finish().unwrap().1;
        assert_eq!(scanners.len(), 38);
        let mut sum = 0;
        for scanner in scanners.iter() {
            sum += scanner.beacons.len();
        }
        // filler lines + first line
        sum += 37 * 2 + 1;
        assert_eq!(sum, 1060);
        Ok(())
    }

    #[test]
    fn rotations() {
        for rot in ROTATIONS.iter() {
            eprintln!("{:#?}", rot);
        }
        assert_eq!(ROTATIONS.len(), 24);
    }

    #[test]
    fn rotating() -> Result<()> {
        let content = read_to_string(PathBuf::from("debug-rotation.txt"))?;
        let scanners = Vec::<Scanner>::parse(&content).finish().unwrap().1;
        for left in scanners.iter() {
            let mut identity_found = false;
            for right in scanners.iter() {
                match left.check_match(right, 6) {
                    None => {}
                    Some(Position { x: 0, y: 0, z: 0 }) => {
                        identity_found = true;
                    }
                    Some(pos) => {
                        bail!("Invalid position {:#?}", pos);
                    }
                }
            }
            assert!(identity_found, "did not find identity")
        }
        Ok(())
    }

    #[test]
    fn rotate_scanner() -> Result<()> {
        let content = read_to_string(PathBuf::from("debug-rotation.txt"))?;
        let scanners = Vec::<Scanner>::parse(&content).finish().unwrap().1;
        for left in scanners.iter() {
            for right in scanners.iter() {
                let mut found = false;
                for rot in ROTATIONS.iter() {
                    if right.clone().rotate_by(rot.into()) == *left {
                        found = true;
                        break;
                    }
                }
                if !found {
                    bail!("could not rotate to identity")
                }
            }
        }
        Ok(())
    }

    #[test]
    fn single_checks() -> Result<()> {
        let content = read_to_string(PathBuf::from("debug.txt"))?;
        let scanners = Vec::<Scanner>::parse(&content).finish().unwrap().1;

        for (i, scanner) in scanners.iter().enumerate() {
            assert_eq!(scanner.id, i, "Invalid scanner id");
        }

        let pos_scanner_1 = Position {
            x: 68,
            y: -1246,
            z: -43,
        };
        let mut rotated_scanner_1 = Scanner::default();
        for rot in ROTATIONS.iter() {
            rotated_scanner_1 = scanners[1].clone().rotate_by(rot.into());

            if let Some(diff) = scanners[0].check_match(&rotated_scanner_1, 12) {
                assert_eq!(diff, pos_scanner_1, "Invalid difference");
                break;
            }
        }

        let pos_scanner_4 = Position {
            x: -20,
            y: -1133,
            z: 1061,
        };
        for rot in ROTATIONS.iter() {
            let rotated_scanner_4 = scanners[4].clone().rotate_by(rot.into());
            if let Some(diff) = rotated_scanner_1.check_match(&rotated_scanner_4, 12) {
                assert_eq!(pos_scanner_1 + diff, pos_scanner_4);
                break;
            }
        }
        Ok(())
    }

    #[test]
    fn more_parsing() -> Result<()> {
        let content = read_to_string(PathBuf::from("debug.txt"))?;
        let scanners = Vec::<Scanner>::parse(&content).finish().unwrap().1;

        assert_eq!(
            scanners[0],
            Scanner {
                id: 0,
                position: Some(Position { x: 0, y: 0, z: 0 }),
                beacons: vec![
                    Position {
                        x: 404,
                        y: -588,
                        z: -901
                    },
                    Position {
                        x: 528,
                        y: -643,
                        z: 409
                    },
                    Position {
                        x: -838,
                        y: 591,
                        z: 734
                    },
                    Position {
                        x: 390,
                        y: -675,
                        z: -793
                    },
                    Position {
                        x: -537,
                        y: -823,
                        z: -458
                    },
                    Position {
                        x: -485,
                        y: -357,
                        z: 347
                    },
                    Position {
                        x: -345,
                        y: -311,
                        z: 381
                    },
                    Position {
                        x: -661,
                        y: -816,
                        z: -575
                    },
                    Position {
                        x: -876,
                        y: 649,
                        z: 763
                    },
                    Position {
                        x: -618,
                        y: -824,
                        z: -621
                    },
                    Position {
                        x: 553,
                        y: 345,
                        z: -567
                    },
                    Position {
                        x: 474,
                        y: 580,
                        z: 667
                    },
                    Position {
                        x: -447,
                        y: -329,
                        z: 318
                    },
                    Position {
                        x: -584,
                        y: 868,
                        z: -557
                    },
                    Position {
                        x: 544,
                        y: -627,
                        z: -890
                    },
                    Position {
                        x: 564,
                        y: 392,
                        z: -477
                    },
                    Position {
                        x: 455,
                        y: 729,
                        z: 728
                    },
                    Position {
                        x: -892,
                        y: 524,
                        z: 684
                    },
                    Position {
                        x: -689,
                        y: 845,
                        z: -530
                    },
                    Position {
                        x: 423,
                        y: -701,
                        z: 434
                    },
                    Position {
                        x: 7,
                        y: -33,
                        z: -71
                    },
                    Position {
                        x: 630,
                        y: 319,
                        z: -379
                    },
                    Position {
                        x: 443,
                        y: 580,
                        z: 662
                    },
                    Position {
                        x: -789,
                        y: 900,
                        z: -551
                    },
                    Position {
                        x: 459,
                        y: -707,
                        z: 401
                    },
                ],
            },
            "scanner not parsed"
        );
        Ok(())
    }

    #[test]
    fn alignment() -> Result<()> {
        let content = read_to_string(PathBuf::from("debug.txt"))?;
        let scanners = Vec::<Scanner>::parse(&content).finish().unwrap().1;

        let aligned = align(scanners, 12)?;
        assert_eq!(count_beacons(&aligned[..])?, 79, "did not find all beacons");

        Ok(())
    }

    #[test]
    fn check_match() -> Result<()> {
        let content = read_to_string(PathBuf::from("debug.txt"))?;
        let scanners = Vec::<Scanner>::parse(&content).finish().unwrap().1;

        let want_diff = Position { x: 1, y: 2, z: 3 };
        let origin = scanners.first().unwrap().clone();
        let other = Scanner {
            beacons: origin.beacons.iter().map(|b| *b - want_diff).collect(),
            ..origin.clone()
        };

        assert_eq!(origin.check_match(&other, 12), Some(want_diff));

        Ok(())
    }
}
