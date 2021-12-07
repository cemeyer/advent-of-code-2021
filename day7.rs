#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use std::cmp::{min, max};
use std::convert::TryFrom;
use std::collections::*;
use std::hash::Hash;

fn part1(input: &str) -> u64 {
    let histo = aoc::histo(
        input.trim_end()
        .split(',')
        .map(|w| w.parse::<u16>().unwrap()));

    let minx = histo.keys().cloned().next().unwrap();
    let maxx = histo.keys().rev().cloned().next().unwrap();
    dbg!(minx, maxx);

    let (_bestcentroid, bestcost) = aoc::best_meeting_point(minx..=maxx, histo.iter(), |dest, src| {
        let (src, num_crabs) = src;
        let unit_cost = i64::abs((*dest as i64) - (*src as i64)) as u64;
        unit_cost * num_crabs
    });

    assert_eq!(bestcost, 340987);

    bestcost
}

fn part2(input: &str) -> u64 {
    let histo = aoc::histo(
        input.trim_end()
        .split(',')
        .map(|w| w.parse::<u16>().unwrap()));

    let minx = histo.keys().cloned().next().unwrap();
    let maxx = histo.keys().rev().cloned().next().unwrap();
    dbg!(minx, maxx);

    let costs = {
        let max = maxx as usize + 1;
        let mut res = vec![0u64; max];
        res[1] = 1;
        for i in 2..max {
            res[i] = res[i - 1] + i as u64;
        }
        res
    };

    let (_bestcentroid, bestcost) = aoc::best_meeting_point(minx..=maxx, histo.iter(), |dest, src| {
        let (src, num_crabs) = src;
        let unit_cost = costs[i64::abs((*dest as i64) - (*src as i64)) as usize];
        unit_cost * num_crabs
    });

    assert_eq!(bestcost, 96987874);
    bestcost
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 7)?;
    let data = puzzle.get_data()?;

    let answ1 = part1(data);
    dbg!(&answ1);
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    let answ2 = part2(data);
    dbg!(&answ2);
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
