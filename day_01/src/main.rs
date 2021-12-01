use anyhow::{Context, Result};
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let input = PathBuf::from(
        env::args()
            .nth(1)
            .with_context(|| "No input provided!")?,
    );
    println!("Input: {}", input.display());
    part1(&input)?;
    // part2(&input)?;
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
