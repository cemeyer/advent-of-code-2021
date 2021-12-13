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
fn do_fold(points: &HashSet<(u16, u16)>, fold: &(char, u16)) -> HashSet<(u16, u16)> {
    let mut res = HashSet::new();
    let (foldaxis, foldval) = fold;
    for (x, y) in points.iter().cloned() {
        match foldaxis {
            'y' => {
                assert_ne!(y, *foldval); // no dots on folds
                if y < *foldval {
                    res.insert((x, y));
                    continue;
                }
                res.insert((x, 2 * foldval - y));
            }
            'x' => {
                assert_ne!(x, *foldval); // no dots on folds
                if x < *foldval {
                    res.insert((x, y));
                    continue;
                }
                res.insert((2 * foldval - x, y));
            }
            _ => { unreachable!(); }
        }
    }

    res
}

fn part1(points: &HashSet<(u16, u16)>, folds: &Vec<(char, u16)>) -> usize {
    let points = do_fold(points, &folds[0]);
    points.len()
}

fn part2(points: &HashSet<(u16, u16)>, folds: &Vec<(char, u16)>) -> Array2<u8> {
    let mut points = points.clone();
    for fold in folds {
        points = do_fold(&points, fold);
    }
    let mut maxx = 0;
    let mut maxy = 0;
    for (x, y) in points.iter() {
        maxx = max(maxx, *x);
        maxy = max(maxy, *y);
    }
    let mut grid = Array::zeros((maxy as usize +1, maxx as usize +1));
    for (x, y) in points.iter().cloned() {
        grid[[y as usize, x as usize]] = 1;
    }

    grid
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
    let lines = data.lines().collect::<Vec<_>>();
    let mut maxx = 0;
    let mut maxy = 0;
    let mut pts = HashSet::new();
    let mut folds = Vec::new();
    let mut line_iter = lines.iter();
    loop {
        let next = line_iter.next().unwrap();
        if next.len() == 0 {
            break;
        }
        let mut nums = next.split(",").map(|x| x.parse::<u16>().unwrap());
        let x = nums.next().unwrap();
        let y = nums.next().unwrap();
        maxx = max(maxx, x);
        maxy = max(maxy, y);
        pts.insert((x, y));
    }
    for fold in line_iter {
        let mut words = fold.split(' ');
        words.next();
        words.next();
        let instr = words.next().unwrap();
        let mut parts = instr.split('=');
        let axis = parts.next().unwrap().chars().next().unwrap();
        let value = parts.next().unwrap().parse::<u16>().unwrap();
        folds.push((axis, value));
    }

    //let answ1 = part1(&pts, &folds);
    //dbg!(&answ1);
    let answ2 = part2(&pts, &folds);
    println!("{:?}", &answ2);

    Ok(())
}
