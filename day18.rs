#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use graphlib::{Graph, VertexId};
use itertools::iproduct;
use ndarray::prelude::*;
use ndarray::{ArcArray2, parallel::par_azip};
use std::cell::RefCell;
use std::cmp::{min, max};
use std::convert::{TryFrom, TryInto};
use std::collections::*;
use std::hash::Hash;
use std::iter::FromIterator;
use std::rc::Rc;

use aoc::{dbg2, byte, BitCursor, ByteString};

type Num = u8;

#[derive(Eq,PartialEq,Clone,Debug)]
enum D {
    Int(Num),
    Pair(Box<D>, Box<D>),
}

impl AsRef<D> for D {
    #[inline]
    fn as_ref(&self) -> &D {
        self
    }
}

#[inline]
fn p<A: AsRef<D>, B: AsRef<D>>(a: A, b: B) -> D {
    D::Pair(Box::new(a.as_ref().clone()), Box::new(b.as_ref().clone()))
}

// Impl largely stolen from benediktwerner's Python solution.
//
// This one is really tough to express and think about in Rust's type/ownership model, IMO.  It's
// easier to think about the Python solution and then translate as a separate step.
fn add_left<DD: AsRef<D>>(x: DD, n: Option<Num>) -> D {
    match (x.as_ref(), n) {
        (y, None) => y.clone(),
        (D::Int(i), Some(n)) => D::Int(i + n),
        (D::Pair(a, b), Some(_)) => p(add_left(a, n), b),
    }
}

fn add_right<DD: AsRef<D>>(x: DD, n: Option<Num>) -> D {
    match (x.as_ref(), n) {
        (y, None) => y.clone(),
        (D::Int(i), Some(n)) => D::Int(i + n),
        (D::Pair(a, b), Some(_)) => p(a, add_right(b, n)),
    }
}

// Explode this subexpression, knowing it is 'n' deep.
//
// Returns (did_explode?, left-exploded-num, replaced_expr, right-exploded-num)
fn explode<DD: AsRef<D>>(x: DD, n: usize) -> (bool, Option<Num>, D, Option<Num>) {
    let (a, b) = match x.as_ref() {
        D::Pair(a, b) => (a, b),
        D::Int(_) => {
            return (false, None, x.as_ref().clone(), None);
        }
    };

    // 4 deep?
    if n == 0 {
        match (a.as_ref(), b.as_ref()) {
            (D::Int(a), D::Int(b)) => {
                return (true, Some(*a), D::Int(0), Some(*b));
            }
            _ => { unreachable!(); }
        }
    }

    // Find the left-most explosion candidate
    let (exp, left, a, right) = explode(a, n - 1);
    if exp { 
        // If there was one in our left subtree, propagate the result to the left-most entry in our
        // right-subtree and bubble up, consuming the 'right' exploded number.
        return (true, left, p(a, add_left(b, right)), None);
    }

    let (exp, left, b, right) = explode(b, n - 1);
    if exp {
        // If there was one in our right subtree, propagate the result to the right-most entry in
        // our left subtree and bubble up, consuming the 'left' exploded number.
        return (true, None, p(add_right(a, left), b), right);
    }
    (false, None, x.as_ref().clone(), None)
}

fn split<DD: AsRef<D>>(x: DD) -> (bool, D) {
    let (a, b) = match x.as_ref() {
        D::Int(i) => {
            if *i >= 10 {
                return (true, p(D::Int(*i / 2), D::Int((*i + 1) / 2)));
            }
            return (false, x.as_ref().clone());
        }
        D::Pair(a, b) => (a, b),
    };

    // Recurse down left subtree
    let (change, a) = split(a);
    if change {
        return (true, p(a, b));
    }
    let (change, b) = split(b);
    (change, p(a, b))
}


fn reduce1<DD: AsRef<D>>(n: DD) -> Option<D> {
    let n = n.as_ref();

    let (didexp, _, x, _) = explode(n, 4);
    if didexp {
        return Some(x);
    }

    let (didsplit, x) = split(n);
    if didsplit {
        return Some(x);
    }

    None
}

fn reduce<DD: AsRef<D>>(n: DD) -> D {
    let mut n = n.as_ref().clone();
    while let Some(m) = reduce1(&n) {
        n = m;
    }
    n
}

fn add<DD: AsRef<D>, DDD: AsRef<D>>(a: DD, b: DDD) -> D {
    let a = a.as_ref();
    let b = b.as_ref();
    let res = p(a, b);
    reduce(&res)
}

fn magnitude(x: &D) -> u64 {
    match x {
        D::Int(i) => *i as u64,
        D::Pair(a, b) => 3 * magnitude(a) + 2 * magnitude(b),
    }
}

fn part1(input: &ParseResult) -> u64 {
    let input = input.clone();
    let mut iter = input.into_iter();
    let mut x: D = iter.next().unwrap();

    for elm in iter {
        x = add(&x, &elm);
    }

    magnitude(&x)
}

fn part2(input: &ParseResult) -> u64 {
    let mut max = 0;

    for i in 0..input.len() {
        for j in 0..input.len() {
            if i == j {
                continue;
            }
            let res = magnitude(&add(&input[i], &input[j]));
            if res > max { max = res; }
        }
    }

    max
}

type ParseResult = Vec<D>;

fn parse_expr(data: &mut std::slice::Iter<u8>) -> D {
    let b = data.next().unwrap();

    // An expr is either a pair or a single-digit integer.
    match b {
        b'[' => {
            let a = parse_expr(data);
            let c = data.next().unwrap();
            assert_eq!(*c, b',');
            let b = parse_expr(data);
            let c = data.next().unwrap();
            assert_eq!(*c, b']');
            p(a, b)
        },
        _ => {
            let i = (*b as char).to_digit(10).unwrap();
            D::Int(i as u8)
        },
    }
}

fn parse(data: &str) -> ParseResult {
    data.lines().map(|line| {
        let bytes = line.as_bytes();
        let mut iter = bytes.iter();

        parse_expr(&mut iter)
    })
    .collect::<Vec<_>>()
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 18)?;
    let data = puzzle.get_data()?;
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 3524);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 4656);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn l(n: Num) -> D {
        D::Int(n)
    }

    #[test]
    fn test() {
        // explode
        let start = p(p(p(p(p(l(9), l(8)), l(1)), l(2)), l(3)), l(4));
        let end = p(p(p(p(l(0), l(9)), l(2)), l(3)), l(4));

        assert_eq!(reduce1(start).unwrap(), end);

        let start = p(l(7), p(l(6), p(l(5), p(l(4), p(l(3), l(2))))));
        let end = p(l(7), p(l(6), p(l(5), p(l(7), l(0)))));

        assert_eq!(reduce1(start).unwrap(), end);
    }
}
