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

use aoc::{dbg2, byte, ByteString};

fn do_step(inp: &[u8], rules: &HashMap<ByteString, u8>) -> ByteString {
    let mut res = Vec::new();

    for pair in inp.windows(2) {
        res.push(pair[0]);
        if let Some(blah) = rules.get(pair) {
            res.push(*blah);
        }
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
        *freq.entry(e).or_default() += 1;
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
fn do_step2(inp: &HashMap<ByteString, u64>, rules: &HashMap<ByteString, u8>) -> HashMap<ByteString, u64> {
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
    dbg2!(state);

    // Transform parsed state into histogram of pairs.
    let mut hashstate = HashMap::new();
    for pair in state.windows(2) {
        let v = pair.to_vec();
        *hashstate.entry(v).or_default() += 1;
    }

    // Apply 40 steps.
    for i in 0..40 {
        hashstate = do_step2(&hashstate, rules);
    }

    // Transform pair histogram into frequency of individual elements.
    let mut freq = HashMap::new();
    for (e, cnt) in hashstate.iter() {
        *freq.entry(e[0]).or_default() += cnt;
        *freq.entry(e[1]).or_default() += cnt;
    }

    // Every element in the sequence is double-counted, except first/last.  First/last remain the
    // same from the beginning to the end, so just add them in here.  Now every element is
    // double-counted.
    *freq.entry(state[0]).or_default() += 1;
    *freq.entry(state[state.len()-1]).or_default() += 1;

    // Finally, find the most and least frequent elements in the histogram.
    let mut mct = b'\0';
    let mut mct_c = 0;
    let mut lct = b'\0';
    let mut lct_c = u64::MAX;
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

    // Adjust for double-counting.
    let mct_c_cor = mct_c/2;
    let lct_c_cor = lct_c/2;
    dbg!(mct as char, mct_c_cor, lct as char, lct_c_cor);
    mct_c_cor - lct_c_cor
}

type ParseResult = (ByteString, HashMap<ByteString, u8>);

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

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 3259);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 3459174981021);

    Ok(())
}
