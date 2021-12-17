#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use graphlib::{Graph, VertexId};
use itertools::iproduct;
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

use aoc::{dbg2, byte, BitCursor, ByteString};

fn simulate(idx: i64, idy: i64, tzone: &ParseResult) -> Option<()> {
    let (zonex, zoney) = tzone;
    let zonex = zonex.0..=zonex.1;
    let zoney = zoney.0..=zoney.1;

    let (mut x, mut y) = (0, 0);
    let (mut dx, mut dy) = (idx, idy);
    let mut reached = None;

    loop {
        x = x + dx;
        y = y + dy;

        if zonex.contains(&x) && zoney.contains(&y) {
            reached = Some(());
            break;
        }

        if dx > 0 { dx -= 1; }
        if dx < 0 { dx += 1; }

        if dx == 0 && !zonex.contains(&x) { break; }
        if dx > 0 && x > *zonex.end() { break; }
        if dx < 0 && x < *zonex.start() { break; }
        if dy < 0 && y < *zoney.start() { break; }

        dy -= 1;
    }

    reached
}

fn simulate_y(idy: i64, tzone: &ParseResult) -> Option<i64> {
    let (_, zoney) = tzone;
    let zoney = zoney.0..=zoney.1;

    let mut y = 0;
    let mut dy = idy;
    let mut highy = i64::MIN;
    let mut reached = false;

    loop {
        y = y + dy;

        if y > highy { highy = y; }

        if zoney.contains(&y) {
            reached = true;
            break;
        }

        if dy < 0 && y < *zoney.start() { break; }
        dy -= 1;
    }

    if reached { Some(highy) } else { None }
}

fn simulate_x(idx: i64, tzone: &ParseResult) -> Option<()> {
    let (zonex, _) = tzone;
    let zonex = zonex.0..=zonex.1;

    let mut x = 0;
    let mut dx = idx;
    let mut reached = None;

    loop {
        x = x + dx;

        if zonex.contains(&x) {
            reached = Some(());
            break;
        }

        if dx > 0 { dx -= 1; }
        if dx < 0 { dx += 1; }

        if dx == 0 && !zonex.contains(&x) { break; }
        if dx > 0 && x > *zonex.end() { break; }
        if dx < 0 && x < *zonex.start() { break; }
    }

    reached
}

fn part1(input: &ParseResult) -> i64 {
    let mut besty = i64::MIN;
    for dy in -125..125 {
        let y = simulate_y(dy, &input).unwrap_or(i64::MIN);
        if y > besty {
            besty = y;
        }
    }
    besty
}

fn part2(input: &ParseResult) -> i64 {
    let mut res = 0;
    let mut x_cand = Vec::new();
    let mut y_cand = Vec::new();

    // You can simulate x and y separately to reduce the search space considerably (~184 -> 116 in
    // x for my input, ~5000 -> 234 in y for my input).  You still need to simulate (x, y)
    // candidates to determine actual matches.  This eliminates ~97% of simulations for my initial,
    // large y range guess (-200..5000) or ~30% of simulations with an ideal y range (-125..125) --
    // including the cost of the additional single-dimension simulations.
    for dx in 1..=(input.0.1) {
        if simulate_x(dx, &input).is_some() {
            x_cand.push(dx);
        }
    }
    //for dy in -200..5000 {
    for dy in -125..125 {
        if simulate_y(dy, &input).is_some() {
            y_cand.push(dy);
        }
    }

    for dx in x_cand.iter().copied() {
        for dy in y_cand.iter().copied() {
            if simulate(dx, dy, &input).is_some() {
                res += 1;
            }
        }
    }
    res
}

type ParseResult = ((i64, i64), (i64, i64));

fn parse(_data: &str) -> ParseResult {
    // Eyeball parsing.
    ((138, 184), (-125, -71))
    //((20, 30), (-10, -5))
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 17)?;
    let data = puzzle.get_data()?;
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 7750);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 4120);
    Ok(())
}
