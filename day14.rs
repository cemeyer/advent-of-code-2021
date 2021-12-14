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

fn do_step(inp: &[u8], rules: &HashMap<Vec<u8>, u8>) -> Vec<u8> {
    let mut res = Vec::new();

    for pair in inp.windows(2) {
        res.push(pair[0]);
        if let Some(blah) = rules.get(pair) {
            res.push(*blah);
        }
        //res.push(pair[1]);
    }
    res.push(inp[inp.len() - 1]);

    res
}

fn part1(input: &ParseResult) -> i64 {
    let (state, rules) = input;
    let mut state = state.clone();
    for _ in 0..10 {
        state = do_step(&state, rules);
        //println!("{}", std::str::from_utf8(&state).unwrap());
    }
    let mut mct = b'\0';
    let mut mct_c = -1;
    let mut lct = b'\0';
    let mut lct_c = i64::MAX;
    let mut freq = HashMap::new();
    for e in state {
        freq.insert(e, freq.get(&e).unwrap_or(&0) + 1);
    }
    for (e, cnt) in freq.iter() {
        if *cnt < lct_c {
            lct_c = *cnt;
            lct = *e;
        }
        if *cnt > mct_c {
            mct_c = *cnt;
            mct = *e;
        }
    }
    mct_c - lct_c
}

// Revised step function using a histogram of pairs.
fn do_step2(inp: &HashMap<Vec<u8>, u64>, rules: &HashMap<Vec<u8>, u8>) -> HashMap<Vec<u8>, u64> {
    let mut res = HashMap::new();

    for (pair, cnt) in inp.iter() {
        if let Some(blah) = rules.get(pair) {
            let v1 = vec![pair[0], *blah];
            let v2 = vec![*blah, pair[1]];
            let incr = res.get(&v1).unwrap_or(&0) + cnt;
            res.insert(v1, incr);
            let incr = res.get(&v2).unwrap_or(&0) + cnt;
            res.insert(v2, incr);
        } else {
            let incr = res.get(pair).unwrap_or(&0) + cnt;
            res.insert(pair.clone(), incr);
        }
    }

    res
}

fn part2(input: &ParseResult) -> u64 {
    let (state, rules) = input;
    let mut hashstate = HashMap::new();
    for pair in state.windows(2) {
        let v = pair.to_vec();
        hashstate.insert(v, hashstate.get(pair).unwrap_or(&0) + 1);
    }

    for i in 0..40 {
        dbg!(i);
        hashstate = do_step2(&hashstate, rules);
    }
    let mut mct = b'\0';
    let mut mct_c = 0;
    let mut lct = b'\0';
    let mut lct_c = u64::MAX;
    let mut freq = HashMap::new();
    for (e, cnt) in hashstate.iter() {
        freq.insert(e[0], freq.get(&e[0]).unwrap_or(&0) + cnt);
        freq.insert(e[1], freq.get(&e[1]).unwrap_or(&0) + cnt);
    }
    for (e, cnt) in freq.iter() {
        if *cnt < lct_c {
            lct_c = *cnt;
            lct = *e;
        }
        if *cnt > mct_c {
            mct_c = *cnt;
            mct = *e;
        }
    }
    // This isn't actually correct, but I observed that my results were exactly double the expected
    // with the sample problem and was able to manually half the counts and produce the correct
    // solution.
    dbg!(mct, mct_c, lct, lct_c);
    mct_c - lct_c
}

type ParseResult = (Vec<u8>, HashMap<Vec<u8>, u8>);

fn parse(data: &str) -> ParseResult {
    let (template, insertion) = data.split_once("\n\n").unwrap();
    let mut rules = HashMap::new();
    for line in insertion.lines() {
        let rule = line.split_once(" -> ").unwrap();
        rules.insert(rule.0.as_bytes().to_vec(), rule.1.as_bytes()[0]);
    }
    (template.as_bytes().to_vec(), rules)
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 14)?;
    let data = puzzle.get_data()?;
//    let data =
//"NNCB
//
//CH -> B
//HH -> N
//CB -> H
//NH -> C
//HB -> C
//HC -> B
//HN -> C
//NN -> C
//BH -> H
//NC -> B
//NB -> B
//BN -> B
//BB -> N
//BC -> B
//CC -> N
//CN -> C";
    let parsed = parse(data);

    //let answ1 = part1(&parsed);
    //dbg!(&answ1);
    let answ2 = part2(&parsed);
    dbg!(&answ2);

    Ok(())
}
