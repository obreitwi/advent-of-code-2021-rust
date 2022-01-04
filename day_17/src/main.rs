#![allow(unused_imports)]
use anyhow::{bail, Context, Error, Result};

fn main() -> Result<()> {
    let (ymax, num_valid) = find_ymax((201, 230), (-99, -65));
    println!("part 1: ymax = {}", ymax);
    println!("part 2: num valid velocities = {}", num_valid);
    Ok(())
}

fn find_ymax((x_min, x_max): (i64, i64), (y_min, y_max): (i64, i64)) -> (i64, usize) {
    let mut ymax_observed = 0;
    let mut num_valid_velocities = 0;
    for vx0 in 0..1000 {
        for vy0 in -1000..1000 {
            let mut ymax_local = 0;
            let (mut vx, mut vy) = (vx0, vy0);
            let (mut x, mut y): (i64, i64) = (0, 0);
            while x < x_max && y > y_min {
                x += vx;
                y += vy;
                if y > ymax_local {
                    ymax_local = y;
                }
                if vx > 0 {
                    vx -= 1;
                }
                vy -= 1;
                if x >= x_min && x <= x_max && y >= y_min && y <= y_max {
                    if ymax_observed < ymax_local {
                        ymax_observed = ymax_local;
                    }
                    num_valid_velocities += 1;
                    break;
                }
            }
        }
    }
    (ymax_observed, num_valid_velocities)
}
