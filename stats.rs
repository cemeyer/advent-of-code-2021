#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use std::cmp::min;

fn main() -> Result<()> {
    let stats = aoc::get_stats(2021)?;

    let days = stats.days();
    let count = stats.count_stars();

    println!("Partial completion on {} days.  Total {} stars collected.", days, count);

    let mut score = 0;
    let mut best_rank = u32::MAX;

    for i in 1..=days {
        let day = stats.day(i).unwrap();
        score += day.part1.score as usize;
        best_rank = min(best_rank, day.part1.rank);
        if let Some(part2) = &day.part2 {
            score += part2.score as usize;
            best_rank = min(best_rank, part2.rank);
        }
    }

    println!("Best-ever rank: {}.  Total score: {}.", best_rank, score);
    Ok(())
}
