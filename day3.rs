#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use std::collections::HashMap;

fn part1(input: &str) -> String {
    let mut hist = Vec::new();

    for line in input.lines() {
        let mut idx = 0;

        for ch in line.chars() {
            if hist.len() <= idx {
                hist.push(HashMap::new());
            }

            let count = hist[idx].get(&ch).copied().unwrap_or(0);
            hist[idx].insert(ch, count + 1);

            idx += 1;
        }
    }

    let mut gamma = 0u64;
    let mut mask = 0;
    for (i, hist) in hist.iter().enumerate() {
        dbg!(i, hist);

        gamma <<= 1;
        if hist[&'1'] > hist[&'0'] {
            gamma |= 0x1;
        }

        mask <<= 1;
        mask |= 1;
    }

    let epsilon = (!gamma & mask);

    dbg!(gamma, epsilon);

    format!("{}", gamma * epsilon)
}

fn histo(lines: &Vec<&str>) -> Vec<HashMap<char, u64>> {
    let mut hist = Vec::new();

    for line in lines {
        let mut idx = 0;

        for ch in line.chars() {
            if hist.len() <= idx {
                hist.push(HashMap::new());
            }

            let count = hist[idx].get(&ch).copied().unwrap_or(0);
            hist[idx].insert(ch, count + 1);

            idx += 1;
        }
    }

    return hist;
}

fn bit_criteria_o2<'a>(lines: &Vec<&'a str>, idx: usize) -> Vec<&'a str> {
    let hist_o2 = histo(lines);
    let hist = &hist_o2[idx];
    let desired_bit = if hist[&'1'] >= hist[&'0'] {
        '1'
    } else {
        '0'
    };

    lines.iter().filter(|line| {
        line.chars().nth(idx).unwrap() == desired_bit
    })
    .map(|s| *s)
    .collect()
}

fn bit_criteria_co2<'a>(lines: &Vec<&'a str>, idx: usize) -> Vec<&'a str> {
    let hist_co2 = histo(lines);
    let hist = &hist_co2[idx];
    let desired_bit = if hist[&'1'] >= hist[&'0'] {
        '0'
    } else {
        '1'
    };

    lines.iter().filter(|line| {
        line.chars().nth(idx).unwrap() == desired_bit
    })
    .map(|s| *s)
    .collect()
}

fn part2(input: &str) -> String {
    let mut lines_o2 = input.lines().collect::<Vec<_>>();
    let mut lines_co2 = lines_o2.clone();
    let mut idx = 0;
    while lines_o2.len() > 1 {
        lines_o2 = bit_criteria_o2(&lines_o2, idx);
        idx += 1;
    }
    let o2_gen = usize::from_str_radix(lines_o2[0], 2).unwrap();
    dbg!(o2_gen);

    let mut idx = 0;
    while lines_co2.len() > 1 {
        lines_co2 = bit_criteria_co2(&lines_co2, idx);
        idx += 1;
    }
    let co2_scrub = usize::from_str_radix(lines_co2[0], 2).unwrap();
    dbg!(co2_scrub);

    let lfs = o2_gen * co2_scrub;
    format!("{}", lfs)
}

fn submit(puzzle: &mut aoc::Puzzle, part: aoc::Part, answ: &str) -> Result<()> {
    println!("Submitting: {} for part {:?}", answ, part);
    puzzle.submit_answer(part, answ)
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new2021(3)?;
    let data = puzzle.get_data()?;

    //let answ1 = part1(data);
    //submit(&mut puzzle, aoc::Part::One, &answ1)?;
    let answ2 = part2(data);
    submit(&mut puzzle, aoc::Part::Two, &answ2)?;

    Ok(())
}
