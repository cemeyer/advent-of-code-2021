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

type ParseResult = (Vec<u8>, HashSet<(i32,i32)>);

fn parse(data: &str) -> ParseResult {
    let (algo, input) = data.split_once("\n\n").unwrap();
    let algo = algo.as_bytes().iter().map(|b| {
        match b {
            b'.' => 0,
            b'#' => 1,
            _ => { unreachable!(); }
        }
    }).collect::<Vec<_>>();
    let input = input.split('\n').map(|line| {
        line.as_bytes().iter().map(|b| {
            match b {
                b'.' => 0,
                b'#' => 1,
                _ => { unreachable!(); }
            }
        }).collect::<Vec<_>>()
    }).collect::<Vec<_>>();

    let mut matrix = HashSet::new();
    for (r, line) in input.iter().enumerate() {
        for (c, val) in line.iter().enumerate() {
            if *val != 0 {
                matrix.insert((r as i32, c as i32));
            }
        }
    }
    (algo, matrix)
}

// The key observation here was that we only track pixels in some bounded region (expressed by
// 'range') and then the remaining pixels are all uniformly 'bg'.  In my real input, and I suspect
// everyone else's, the 0 pixel becomes 1 and vice versa (that is, the background flips between off
// and on).  Only the interactions at the edges of the tracked region really matter.
#[inline]
fn get_pixel(input: &HashSet<(i32,i32)>, range: ((i32,i32),(i32,i32)), bg: u8, r: i32, c: i32) -> u16 {
    let (ranger, rangec) = range;
    let (minr, maxr) = ranger;
    let (minc, maxc) = rangec;

    // Inside bounds? Consult the hash.
    if (minr..=maxr).contains(&r) && (minc..=maxc).contains(&c) {
        if input.contains(&(r, c)) { 1 } else { 0 }
    } else {
        // OOB?  Background color.
        bg as u16
    }
}

// Using 'algo', step the infinite 'input' image (hashset covering 'range' indices) with background
// 'bg' once.  Return the new finite portion as a hashset.
fn do_step(algo: &[u8], input: &HashSet<(i32,i32)>, range: ((i32,i32),(i32,i32)), bg: u8) -> HashSet<(i32,i32)> {
    let (ranger, rangec) = range;
    let (minr, maxr) = ranger;
    let (minc, maxc) = rangec;

    let mut res = HashSet::new();

    // As described above, we only care about the bounded region, plus the few pixels it can affect
    // around it.  The background will all do something uniform.
    for r in (minr-3)..=(maxr+3) {
        for c in (minc-3)..=(maxc+3) {
            // I briefly considered keeping some buffer here between points but elected to get
            // something functioning first, and speed was fine.
            let sum = (get_pixel(input, range, bg, r-1, c-1) << 8) |
                (get_pixel(input, range, bg, r-1, c) << 7) |
                (get_pixel(input, range, bg, r-1, c+1) << 6) |
                (get_pixel(input, range, bg, r, c-1) << 5) |
                (get_pixel(input, range, bg, r, c) << 4) |
                (get_pixel(input, range, bg, r, c+1) << 3) |
                (get_pixel(input, range, bg, r+1, c-1) << 2) |
                (get_pixel(input, range, bg, r+1, c) << 1) |
                (get_pixel(input, range, bg, r+1, c+1) << 0);

            // Lookup in the provided 'algorithm' dictionary.
            if algo[sum as usize] == 1 {
                res.insert((r, c));
            }
        }
    }

    res
}

// Debugging aid only.
fn viz(input: &HashSet<(i32,i32)>, range: ((i32,i32),(i32,i32))) {
    let (ranger, rangec) = range;
    let (minr, maxr) = ranger;
    let (minc, maxc) = rangec;

    for r in (minr)..=(maxr) {
        for c in (minc)..=(maxc) {
            if input.contains(&(r, c)) {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }
}

// Step the thing twice.  Have to remember to update our bounded range between steps, or the
// number is too low.  Also, the sample input doesn't have alternating background (change the '1'
// in the second do_step to '0').
fn part1(input: &ParseResult) -> usize {
    let (algo, input) = input.clone();

    let (mut minr, mut maxr) = (i32::MAX, i32::MIN);
    let (mut minc, mut maxc) = (i32::MAX, i32::MIN);
    for (r, c) in input.iter() {
        if *r < minr { minr = *r; }
        if *r > maxr { maxr = *r; }
        if *c < minc { minc = *c; }
        if *c > maxc { maxc = *c; }
    }

    let a = do_step(&algo, &input, ((minr, maxr), (minc, maxc)), 0);
    for (r, c) in a.iter() {
        if *r < minr { minr = *r; }
        if *r > maxr { maxr = *r; }
        if *c < minc { minc = *c; }
        if *c > maxc { maxc = *c; }
    }
    //viz(&a, ((minr, maxr), (minc, maxc)));
    //println!("--");
    let b = do_step(&algo, &a, ((minr, maxr), (minc, maxc)), 1);
    for (r, c) in b.iter() {
        if *r < minr { minr = *r; }
        if *r > maxr { maxr = *r; }
        if *c < minc { minc = *c; }
        if *c > maxc { maxc = *c; }
    }
    //viz(&b, ((minr, maxr), (minc, maxc)));

    b.len()
}

// Step the thing 50 times.  Didn't require any substantial changes from step 1.
fn part2(input: &ParseResult) -> usize {
    let (algo, mut input) = input.clone();

    let mut bg = 0;

    for _ in 0..50 {
        let (mut minr, mut maxr) = (i32::MAX, i32::MIN);
        let (mut minc, mut maxc) = (i32::MAX, i32::MIN);
        for (r, c) in input.iter() {
            if *r < minr { minr = *r; }
            if *r > maxr { maxr = *r; }
            if *c < minc { minc = *c; }
            if *c > maxc { maxc = *c; }
        }

        input = do_step(&algo, &input, ((minr, maxr), (minc, maxc)), bg);

        // This is hardcoded from my input's algorithm, where 0 => 1 and 511 (0b111_111_111) => 0.
        // But I suspect it is the same for everyone.
        bg = if bg != 0 { 0 } else { 1 };
    }

    input.len()
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 20)?;
    let data = puzzle.get_data()?;
//    let data = 
//"..#.#..#####.#.#.#.###.##.....###.##.#..###.####..#####..#....#..#..##..###..######.###...####..#..#####..##..#.#####...##.#.#..#.##..#.#......#.###.######.###.####...#.##.##..#..#..#####.....#.#....###..#.##......#.....#..#..#..##..#...##.######.####.####.#.#...#.......#..#.#.#...####.##.#......#..#...##.#.##..#...##.#.##..###.#......#.#.......#.#.#.####.###.##...#.....####.#..#..#.##.#....##..#.####....##...##..#...#......#.#.......#.......##..####..#...#.#.#...##..#.#..###..#####........#..####......#..#
//
//#..#.
//#....
//##..#
//..#..
//..###";
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 5583);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 19592);
    Ok(())
}
