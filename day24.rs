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

// Register
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum R {
    W,X,Y,Z,
}
type Im = i64;
// Instr dst,src
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum I {
    Inp(R),
    Add(R,R),
    AddI(R,Im),
    Mul(R,R),
    MulI(R,Im),
    Div(R,R),
    DivI(R,Im),
    Mod(R,R),
    ModI(R,Im),
    Eql(R,R),
    EqlI(R,Im),
}

fn parse_reg(input: &str) -> R {
    let regc = input.chars().next().unwrap();
    match regc {
        'w' => R::W,
        'x' => R::X,
        'y' => R::Y,
        'z' => R::Z,
        _ => {
            unreachable!("{}", regc);
        }
    }
}

enum RegOrImm {
    Imm(Im),
    Reg(R),
}

fn parse_regimm(input: &str) -> RegOrImm {
    input.parse::<Im>()
        .map(|i| RegOrImm::Imm(i))
        .unwrap_or_else(|_| RegOrImm::Reg(parse_reg(input)))
}
type ParseResult = Vec<I>;

fn parse(input: &str) -> ParseResult {
    input.lines().map(|line| {
        let mut words = line.split(' ');
        let verb = words.next().unwrap();
        //dbg!(line);
        let dst = parse_regimm(words.next().unwrap());
        let src = words.next().map(|w| parse_regimm(w));

        match (verb, dst, src) {
            ("inp", RegOrImm::Reg(dst), None) => I::Inp(dst),
            ("add", RegOrImm::Reg(dst), Some(RegOrImm::Reg(r))) => I::Add(dst, r),
            ("add", RegOrImm::Reg(dst), Some(RegOrImm::Imm(i))) => I::AddI(dst, i),
            ("mul", RegOrImm::Reg(dst), Some(RegOrImm::Reg(r))) => I::Mul(dst, r),
            ("mul", RegOrImm::Reg(dst), Some(RegOrImm::Imm(i))) => I::MulI(dst, i),
            ("div", RegOrImm::Reg(dst), Some(RegOrImm::Reg(r))) => I::Div(dst, r),
            ("div", RegOrImm::Reg(dst), Some(RegOrImm::Imm(i))) => I::DivI(dst, i),
            ("mod", RegOrImm::Reg(dst), Some(RegOrImm::Reg(r))) => I::Mod(dst, r),
            ("mod", RegOrImm::Reg(dst), Some(RegOrImm::Imm(i))) => I::ModI(dst, i),
            ("eql", RegOrImm::Reg(dst), Some(RegOrImm::Reg(r))) => I::Eql(dst, r),
            ("eql", RegOrImm::Reg(dst), Some(RegOrImm::Imm(i))) => I::EqlI(dst, i),
            _ => {
                unreachable!();
            }
        }
    }).collect()
}

struct Machine {
    regs: [i64; 4],
    pc: isize,
    program: Vec<I>,
    inp: Vec<i64>,
}

impl Machine {
    fn new(program: &[I]) -> Self {
        Self {
            regs: [0; 4],
            pc: 0,
            program: program.to_vec(),
            inp: Vec::new(),
        }
    }

    fn run_until(&mut self, endpc: isize) {
        //let endpc = self.program.len() as isize;
        while self.pc < endpc {
            match self.program[self.pc as usize] {
                I::Inp(dst) => {
                    self.regs[dst as usize] = self.inp.pop().unwrap();
                }
                I::Add(dst, src) => {
                    self.regs[dst as usize] += self.regs[src as usize];
                }
                I::AddI(dst, src) => {
                    self.regs[dst as usize] += src;
                }
                I::Mul(dst, src) => {
                    self.regs[dst as usize] *= self.regs[src as usize];
                }
                I::MulI(dst, src) => {
                    self.regs[dst as usize] *= src;
                }
                I::Div(dst, src) => {
                    self.regs[dst as usize] /= self.regs[src as usize];
                }
                I::DivI(dst, src) => {
                    self.regs[dst as usize] /= src;
                }
                I::Mod(dst, src) => {
                    self.regs[dst as usize] %= self.regs[src as usize];
                }
                I::ModI(dst, src) => {
                    self.regs[dst as usize] %= src;
                }
                I::Eql(dst, src) => {
                    self.regs[dst as usize] = if self.regs[dst as usize] == self.regs[src as usize] { 1 } else { 0 };
                }
                I::EqlI(dst, src) => {
                    self.regs[dst as usize] = if self.regs[dst as usize] == src { 1 } else { 0 };
                }
            };
            self.pc += 1;
        }
    }
}

fn search(cache: &mut HashMap<(u16, i64), Option<i64>>, emu: &mut Machine, pc: u16, z: i64) -> Option<i64> {
    if let Some(&val) = cache.get(&(pc, z)) {
        return val;
    }

    for digit in (1..=9).rev() {
        emu.pc = pc as isize + 1;
        emu.regs = [digit, 0, 0, z];
        emu.run_until(pc as isize + 18);
        let z = emu.regs[R::Z as usize];

        if (pc as usize + 18) == emu.program.len() {
            if z == 0 {
                cache.insert((pc, z), Some(digit));
                return Some(digit);
            }
            continue;
        } else {
            if let Some(best) = search(cache, emu, pc + 18, z) {
                let next = best * 10 + digit;
                cache.insert((pc, z), Some(next));
                return Some(next);
            }
        }
    }
    cache.insert((pc, z), None);
    None
}

fn part1(program: &ParseResult) -> i64 {
    let mut emu = Machine::new(program);
    let mut cache = HashMap::default();

    let mut backwards = search(&mut cache, &mut emu, 0, 0).unwrap();
    let mut forwards = 0;
    while backwards > 0 {
        let last = backwards % 10;
        backwards /= 10;
        forwards = forwards * 10 + last;
    }
    forwards
}

fn search2(cache: &mut HashMap<(u16, i64), Option<i64>>, emu: &mut Machine, pc: u16, z: i64) -> Option<i64> {
    if let Some(&val) = cache.get(&(pc, z)) {
        return val;
    }

    for digit in (1..=9) {
        emu.pc = pc as isize + 1;
        emu.regs = [digit, 0, 0, z];
        emu.run_until(pc as isize + 18);
        let z = emu.regs[R::Z as usize];

        if (pc as usize + 18) == emu.program.len() {
            if z == 0 {
                cache.insert((pc, z), Some(digit));
                return Some(digit);
            }
            continue;
        } else {
            if let Some(best) = search2(cache, emu, pc + 18, z) {
                let next = best * 10 + digit;
                cache.insert((pc, z), Some(next));
                return Some(next);
            }
        }
    }
    cache.insert((pc, z), None);
    None
}

fn part2(program: &ParseResult) -> i64 {
    let mut emu = Machine::new(program);
    let mut cache = HashMap::default();

    let mut backwards = search2(&mut cache, &mut emu, 0, 0).unwrap();
    let mut forwards = 0;
    while backwards > 0 {
        let last = backwards % 10;
        backwards /= 10;
        forwards = forwards * 10 + last;
    }
    forwards
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 24)?;
    let data = puzzle.get_data()?;
    let parsed = parse(data);

    let answ1 = part1(&parsed);
    dbg!(&answ1);
    assert_eq!(answ1, 52926995971999);
    let answ2 = part2(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 11811951311485);
    Ok(())
}
