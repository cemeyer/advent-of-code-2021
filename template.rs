#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use graphlib::{Graph, VertexId};
use itertools::{Itertools, iproduct};
//use nalgebra::*;
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use aoc::{dbg2, byte, BitCursor, ByteString};

//type Vec3f = Vector3<f64>;
//type Pt3f = Point3<f64>;

//#[derive(Eq,PartialEq,Clone,Debug,Hash)]

//type ParseResult = Vec<u8>;
//type ParseResult = Vec<Vec<u8>>;
//type ParseResult = Array2<u8>;
//type ParseResult = (Vec<Vec<VertexId>>, Graph<usize>);
type ParseResult<'a> = Vec<&'a str>;

fn parse(data: &str) -> ParseResult {
    data.lines().map(|line| line).collect::<Vec<_>>()
    //let grid = data.lines().map(|line| {
    //    line.chars().map(|c| c.to_digit(10).unwrap() as u8).collect::<Vec<_>>()
    //}).collect::<Vec<_>>();
    //let nrows = grid.len();
    //let ncols = grid[0].len();
    //let mut matrix = Array::zeros((nrows, ncols));
    //for (r, line) in grid.iter().enumerate() {
    //    for (c, val) in line.iter().enumerate() {
    //        matrix[[r, c]] = *val;
    //    }
    //}
}

fn part1(input: &ParseResult) -> i64 {
    0
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 9999)?;
    let data = puzzle.get_data()?;
    //let data = SAMPLE_DATA;
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    //let answ2 = part2(&parsed);
    //dbg!(&answ2);

    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;
    Ok(())
}

const SAMPLE_DATA: &str =
"";
