#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use std::cmp::{min, max};
use std::convert::TryFrom;
use std::collections::*;
use std::hash::Hash;

/// Reverse the subsequence of `list` starting at `start`, for `seqlen`.
///
/// The sequence to be reversed wraps to the beginning of the list.
fn reverse_subseq(list: &mut[u8], start: usize, seqlen: usize) {
    let resid = min(list.len() - start, seqlen);
    let wresid = seqlen - resid;

    let revseq = list[0..wresid].iter()
        .copied()
        .rev()
        .chain(list[start..(start + resid)].iter().copied().rev())
        .collect::<Vec<_>>();

    for (i, v) in list.iter_mut().enumerate() {
        if i < wresid {
            *v = revseq[i + resid];
        } else if i >= start && i < (start + resid) {
            *v = revseq[i - start];
        }
    }
}

fn apply_round(moves: &[usize], list: &mut[u8], start: usize, mut skip: usize) -> (usize, usize) {
    let mut pos = start;
    let listlen = list.len();

    for &m in moves {
        assert!(m <= 256);

        reverse_subseq(list, pos, m);
        pos = (pos + m + skip) % listlen;
        skip += 1;
    }

    (pos, skip)
}

fn parse(input: &str) -> Vec<usize> {
    input.trim()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect()
}

fn part1(input: &[usize]) -> usize {
    let mut list = (0u8..=255u8).collect::<Vec<_>>();
    apply_round(input, &mut list, 0, 0);
    usize::from(list[0]) * usize::from(list[1])
}

fn apply_64_rounds(moves: &[usize], list: &mut[u8]) {
    let mut pos = 0;
    let mut skip = 0;

    for _ in 0..64 {
        let nposskip = apply_round(moves, list, pos, skip);
        pos = nposskip.0;
        skip = nposskip.1;
    }
}

fn dense_hash(list: &[u8]) -> String {
    let mut res = String::new();

    for i in 0..16 {
        let mut r = 0;
        for x in list[i*16 .. (i*16 + 16)].iter() {
            r ^= x;
        }
        res.push_str(&format!("{:02x}", r));
    }

    res
}

pub fn knothash(data: &[u8]) -> String {
    let mut list = (0u8..=255u8).collect::<Vec<_>>();
    let operations = data.iter()
        .map(|b| usize::from(*b))
        .chain([17, 31, 73, 47, 23])
        .collect::<Vec<_>>();
    apply_64_rounds(&operations, &mut list);
    dense_hash(&list)
}

fn part2(input: &str) -> String {
    knothash(input.trim_end().as_bytes())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_part1() -> Result<()> {
        let mut puzzle = aoc::Puzzle::new(2017, 10)?;
        let data = puzzle.get_data()?;
        let data = parse(data);
        assert_eq!(part1(&data), 2928);
        Ok(())
    }

    #[test]
    fn test_part2() -> Result<()> {
        let mut puzzle = aoc::Puzzle::new(2017, 10)?;
        let data = puzzle.get_data()?;
        assert_eq!(
            part2(&data),
            "0c2f794b2eb555f7830766bf8fb65a16",
            );
        Ok(())
    }
}
