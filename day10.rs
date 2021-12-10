#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;

fn parse(line: &str) -> Result<Vec<char>, (usize, char)> {
    let mut stack = Vec::new();

    for (i, chr) in line.chars().enumerate() {
        match chr {
            '(' => { stack.push('('); }
            '{' => { stack.push('{'); }
            '[' => { stack.push('['); }
            '<' => { stack.push('<'); }


            ')' => {
                if stack.pop() != Some('(') {
                    return Err((i, chr));
                }
            }
            ']' => {
                if stack.pop() != Some('[') {
                    return Err((i, chr));
                }
            }
            '}' => {
                if stack.pop() != Some('{') {
                    return Err((i, chr));
                }
            }
            '>' => {
                if stack.pop() != Some('<') {
                    return Err((i, chr));
                }
            }

            _ => { unreachable!(); }
        }
    }

    Ok(stack)
}

fn part1(lines: &Vec<&str>) -> i64 {
    let scores = {
        let mut scores = HashMap::new();
        scores.insert(')', 3);
        scores.insert(']', 57);
        scores.insert('}', 1197);
        scores.insert('>', 25137);
        scores
    };
    let mut total = 0;
    for line in lines.iter() {
        if let Err((_pos, chr)) = parse(line) {
            total += scores.get(&chr).unwrap();
        }
    }
    total
}

fn part2(lines: &Vec<&str>) -> u64 {
    let scores = {
        let mut scores = HashMap::new();
        scores.insert('(', 1);
        scores.insert('[', 2);
        scores.insert('{', 3);
        scores.insert('<', 4);
        scores
    };
    let mut res = Vec::new();
    for line in lines.iter() {
        if let Ok(stk) = parse(line) {
            let mut sco = 0u64;
            for chr in stk.iter().rev() {
                sco = (5 * sco) + scores[chr];
            }
            res.push(sco);
        }
    }
    res.sort();
    dbg!(&res);
    let i = res.len() / 2;
    dbg!(i, res.len());
    res[i]
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 10)?;
    let data = puzzle.get_data()?;
    let lines = data.lines().collect::<Vec<_>>();

    //let answ1 = part1(&lines);
    //dbg!(&answ1);
    //puzzle.submit_answer(aoc::Part::One, &format!("{}", answ1))?;

    let answ2 = part2(&lines);
    dbg!(&answ2);
    //puzzle.submit_answer(aoc::Part::Two, &format!("{}", answ2))?;

    Ok(())
}
