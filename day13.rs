#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use itertools::iproduct;
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

// up for horizontal 'y' folds, left for vertical 'x' folds
fn do_fold(points: &mut HashSet<(u16, u16)>, fold: &(u8, u16)) {
    let (foldaxis, foldval) = fold;
    // Ideally this would use HashSet::drain_filter(), but the interface is not stabilized yet.
    for (x, y) in points.iter().cloned().collect::<Vec<_>>() {
        match foldaxis {
            b'y' => {
                assert_ne!(y, *foldval); // no dots on folds
                if y < *foldval {
                    continue;
                }
                points.remove(&(x, y));
                points.insert((x, 2 * foldval - y));
            }
            b'x' => {
                assert_ne!(x, *foldval); // no dots on folds
                if x < *foldval {
                    continue;
                }
                points.remove(&(x, y));
                points.insert((2 * foldval - x, y));
            }
            _ => { unreachable!(); }
        }
    }
}

fn part1(points: &HashSet<(u16, u16)>, folds: &Vec<(u8, u16)>) -> usize {
    let mut points = points.clone();
    do_fold(&mut points, &folds[0]);
    points.len()
}

fn part2(points: &HashSet<(u16, u16)>, folds: &Vec<(u8, u16)>) {
    let mut points = points.clone();
    for fold in folds {
        do_fold(&mut points, fold);
    }
    let mut maxx = 0;
    let mut maxy = 0;
    for (x, y) in points.iter() {
        maxx = max(maxx, *x);
        maxy = max(maxy, *y);
    }
    for y in 0..=maxy {
        for x in 0..=maxx {
            print!("{}", if points.contains(&(x as u16, y as u16)) {
                '#'
            } else {
                ' '
            });
        }
        println!("");
    }
}


fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 13)?;
    let data = puzzle.get_data()?;
//let data =
//"6,10
//0,14
//9,10
//0,3
//10,4
//4,11
//6,0
//6,12
//4,1
//0,13
//10,12
//3,4
//3,0
//8,4
//1,10
//2,14
//8,10
//9,0
//
//fold along y=7
//fold along x=5";
    let (data_pts, data_folds) = data.split_once("\n\n").unwrap();
    let mut pts = HashSet::new();
    for line in data_pts.lines() {
        let (x, y) = line.split_once(',').unwrap();
        pts.insert((x.parse::<u16>().unwrap(), y.parse::<u16>().unwrap()));
    }
    let mut folds = Vec::new();
    for fold in data_folds.lines() {
        let instr = fold.trim_start_matches("fold along ");
        let (axis, value) = instr.split_once('=').unwrap();
        folds.push((axis.as_bytes()[0], value.parse::<u16>().unwrap()));
    }

    let answ1 = part1(&pts, &folds);
    dbg!(&answ1);
    part2(&pts, &folds);

    Ok(())
}
