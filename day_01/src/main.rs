use anyhow::{Context, Result};
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(env::args().nth(1).with_context(|| "No input provided!")?);
    println!("Input: {}", input.display());
    part1(&input)?;
    part2(&input)?;
    Ok(())
}

fn part1(input: &Path) -> Result<()> {
    let mut prev = None;
    let mut count_increases: usize = 0;

    for (i, line) in io::BufReader::new(File::open(input)?).lines().enumerate() {
        let num = line
            .with_context(|| format!("Could not read line {}", i))?
            .parse::<usize>()?;

        if let Some(prev) = prev {
            if prev < num {
                count_increases += 1;
            }
        }

        prev = Some(num);
    }
    println!("Part 1: {} increases", count_increases);
    Ok(())
}

fn part2(input: &Path) -> Result<()> {
    let mut count_increases: usize = 0;

    let mut lines = io::BufReader::new(File::open(input)?).lines().enumerate();

    let parse = |line: String| line.parse::<usize>();
    let first = lines
        .next()
        .with_context(|| "First line missing.")
        .map(|t| t.1)?
        .map(parse)??;
    let second = lines
        .next()
        .with_context(|| "Second line missing")
        .map(|t| t.1)?
        .map(parse)??;
    let third = lines
        .next()
        .with_context(|| "Third line missing")
        .map(|t| t.1)?
        .map(parse)??;

    let mut window = vec![third, second, first];

    for (i, line) in io::BufReader::new(File::open(input)?).lines().enumerate() {
        let num = line
            .with_context(|| format!("Could not read line {}", i))?
            .parse::<usize>()?;

        let sum_prev: usize = window.iter().sum();
        window.pop();
        window.insert(0, num);
        let sum_now: usize = window.iter().sum();

        if sum_prev < sum_now {
            count_increases += 1;
        }
    }
    println!("Part 2: {} increases", count_increases);
    Ok(())
}
