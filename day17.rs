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

fn simulate(idx: i64, idy: i64, tzone: &ParseResult) -> i64 {
    let (zonex, zoney) = tzone;
    let zonex = zonex.0..=zonex.1;
    let zoney = zoney.0..=zoney.1;

    let (mut x, mut y) = (0, 0);
    let (mut dx, mut dy) = (idx, idy);
    let mut highy = i64::MIN;
    let mut reached = false;

    loop {
        x = x + dx;
        y = y + dy;

        if y > highy { highy = y; }

        if zonex.contains(&x) && zoney.contains(&y) {
            reached = true;
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

    if reached { highy } else { i64::MIN }
}

fn part1(input: &ParseResult) -> i64 {
    let mut besty = i64::MIN;
    for dx in 1..=(input.0.1) {
        for dy in -200..5000 {
            let y = simulate(dx, dy, &input);
            if y > besty {
                besty = y;
            }
        }
    }
    besty
}

fn part2(input: &ParseResult) -> i64 {
    let mut res = 0;
    for dx in 1..=(input.0.1) {
        for dy in -200..5000 {
            let y = simulate(dx, dy, &input);
            if y != i64::MIN {
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
