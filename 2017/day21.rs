#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;

fn parse_matrix(mat: &str) -> Array2<u8> {
    let dim = mat.split('/').next().unwrap().len();
    let mut res = Array::zeros((dim, dim));

    for (ii, row) in mat.split('/').enumerate() {
        for (jj, ch) in row.chars().enumerate() {
            if ch == '#' {
                res[[ii, jj]] = 1;
            }
        }
    }
    res
}

fn enhance(patterns: &HashMap<Array2<u8>, ArcArray2<u8>>, start: &Array2<u8>) -> Array2<u8> {
    let size = start.nrows();
    if (size % 2) == 0 {
        let newsize = (size / 2) * 3;
        let mut res = Array::zeros((newsize, newsize));

        azip!((search in start.exact_chunks((2, 2)), mut output in res.exact_chunks_mut((3, 3))) {
            let replace = patterns.get(&search.to_owned()).unwrap();
            output.assign(replace);
        });

        res
    } else if (size % 3) == 0 {
        let newsize = (size / 3) * 4;
        let mut res = Array::zeros((newsize, newsize));

        azip!((search in start.exact_chunks((3, 3)), mut output in res.exact_chunks_mut((4, 4))) {
            let replace = patterns.get(&search.to_owned()).unwrap();
            output.assign(replace);
        });

        res
    } else {
        unreachable!();
    }
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2017, 21)?;
    let data = puzzle.get_data()?;

    let mut patterns = HashMap::new();

    // Parse match / replace rules, and duplicate with each rotation.
    for rule in data.lines() {
        let (pattern, replacement) = rule.split_once(" => ").unwrap();
        let mut pattern = parse_matrix(pattern);
        let replacement = parse_matrix(replacement).into_shared();

        patterns.insert(pattern.to_owned(), replacement.clone());
        // flip
        pattern.invert_axis(Axis(1));
        patterns.insert(pattern.to_owned(), replacement.clone());
        pattern.invert_axis(Axis(1));

        // rotate (and flip)
        for _ in 0..3 {
            let tview = pattern.t();
            patterns.insert(tview.to_owned(), replacement.clone());

            pattern = tview.to_owned();
            // flip
            pattern.invert_axis(Axis(1));

            patterns.insert(pattern.to_owned(), replacement.clone());
        }
    }

    // Play the game.
    let mut state = arr2(&[[0u8,1,0], [0,0,1], [1,1,1]]);

    // Part 1.
    for _i in 0..5 {
        dbg!(_i, state.sum());
        state = enhance(&patterns, &state);
    }

    dbg!(state.sum());
    assert_eq!(state.sum(), 176, "Part 1");

    // Part 2.
    for _i in 5..18 {
        dbg!(_i, state.iter().cloned().map(|a| a as u64).sum::<u64>());
        state = enhance(&patterns, &state);
    }

    dbg!(state.iter().cloned().map(|a| a as u64).sum::<u64>());
    assert_eq!(state.iter().cloned().map(|a| a as u64).sum::<u64>(), 2368161, "Part 2");

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            parse_matrix("../.."),
            arr2(&[[0, 0], [0, 0]]),
            );
        assert_eq!(
            parse_matrix("#./.."),
            arr2(&[[1, 0], [0, 0]]),
            );
        assert_eq!(
            parse_matrix("#.#/##./#.."),
            arr2(&[[1, 0, 1], [1, 1, 0], [1, 0, 0]]),
            );
    }

    #[test]
    fn rotation_test() {
        let input = arr2(&[[1,2,3],
                         [4,5,6],
                         [7,8,9]]);

        let tview = input.t();
        assert_eq!(tview, arr2(&[[1,4,7], [2,5,8], [3,6,9]]));

        let mut rotated = tview.to_owned();
        rotated.invert_axis(Axis(1));
        assert_eq!(rotated, arr2(&[[7,4,1],
                                 [8,5,2],
                                 [9,6,3]]));
    }
}
