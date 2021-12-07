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

fn gencosttable() -> Vec<u64> {
    let mut res = vec![0u64; 2000]; //1949);
    res[1] = 1;
    for i in 2..2000 {
        res[i as usize] = res[i as usize - 1] + i;
    }
    res
}

fn part2(input: &str) -> u64 {
    let xs = input.trim_end()
        .split(',')
        .map(|w| w.parse::<u16>().unwrap())
        .collect::<Vec<_>>();

    let minx = *xs.iter().min().unwrap();
    let maxx = *xs.iter().max().unwrap();
    dbg!(minx, maxx);

    let mut bestcost = u64::MAX;
    let mut bestcentroid = u16::MAX;

    let costs = gencosttable();

    for candidate in minx..=maxx {
        let cost = xs.iter()
            .map(|x| {
                costs[i64::abs((*x as i64) - candidate as i64) as usize]
            })
            .sum();
        if cost < bestcost {
            //dbg!(cost, candidate);
            bestcost = cost;
            bestcentroid = candidate;
        }
    }

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
