#![allow(dead_code, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use graphlib::{Graph, VertexId};
use std::cmp::{min, max};
use std::convert::TryFrom;
use std::collections::*;
use std::hash::Hash;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum Reg {
    A,B,C,D,E,F,G,H,
}
type Imm = i64;
// Dst, Src
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum Inst {
    Set(Reg, Reg),
    SetImm(Reg, Imm),
    Sub(Reg, Reg),
    SubImm(Reg, Imm),
    Mul(Reg, Reg),
    MulImm(Reg, Imm),
    Jnz(Reg, Reg),
    JnzImm(Reg, Imm),
    JnzImmImm(Imm, Imm),
    JnzImmReg(Imm, Reg),
}

fn parse_reg(input: &str) -> Reg {
    let regc = input.chars().next().unwrap();
    match regc {
        'a' => Reg::A,
        'b' => Reg::B,
        'c' => Reg::C,
        'd' => Reg::D,
        'e' => Reg::E,
        'f' => Reg::F,
        'g' => Reg::G,
        'h' => Reg::H,
        _ => {
            unreachable!("{}", regc);
        }
    }
}

enum RegOrImm {
    Imm(Imm),
    Reg(Reg),
}

fn parse_regimm(input: &str) -> RegOrImm {
    input.parse::<Imm>()
        .map(|i| RegOrImm::Imm(i))
        .unwrap_or_else(|_| RegOrImm::Reg(parse_reg(input)))
}

fn parse(input: &str) -> Vec<Inst> {
    input.lines().map(|line| {
        let mut words = line.split(' ');
        let verb = words.next().unwrap();
        //dbg!(line);
        let dst = parse_regimm(words.next().unwrap());
        let src = parse_regimm(words.next().unwrap());

        match (verb, dst, src) {
            ("set", RegOrImm::Reg(dst), RegOrImm::Reg(r)) => Inst::Set(dst, r),
            ("set", RegOrImm::Reg(dst), RegOrImm::Imm(i)) => Inst::SetImm(dst, i),
            ("sub", RegOrImm::Reg(dst), RegOrImm::Reg(r)) => Inst::Sub(dst, r),
            ("sub", RegOrImm::Reg(dst), RegOrImm::Imm(i)) => Inst::SubImm(dst, i),
            ("mul", RegOrImm::Reg(dst), RegOrImm::Reg(r)) => Inst::Mul(dst, r),
            ("mul", RegOrImm::Reg(dst), RegOrImm::Imm(i)) => Inst::MulImm(dst, i),
            ("jnz", RegOrImm::Reg(dst), RegOrImm::Reg(r)) => Inst::Jnz(dst, r),
            ("jnz", RegOrImm::Reg(dst), RegOrImm::Imm(i)) => Inst::JnzImm(dst, i),
            ("jnz", RegOrImm::Imm(dsti), RegOrImm::Reg(r)) => Inst::JnzImmReg(dsti, r),
            ("jnz", RegOrImm::Imm(dsti), RegOrImm::Imm(i)) => Inst::JnzImmImm(dsti, i),
            _ => {
                unreachable!();
            }
        }
    }).collect()
}

struct Machine {
    regs: [i64; 8],
    pc: isize,
    program: Vec<Inst>,

    muls: u64,
    profile: Vec<u64>,
    insn_lim: u64,
}

impl Machine {
    fn new(program: &[Inst]) -> Self {
        Self {
            regs: [0; 8],
            pc: 0,
            program: program.to_vec(),
            muls: 0,
            profile: Vec::new(),
            insn_lim: u64::MAX,
        }
    }

    fn run(&mut self) {
        let endpc = self.program.len() as isize;
        while self.pc < endpc {
            match self.program[self.pc as usize] {
                Inst::Set(dst, src) => {
                    self.regs[dst as usize] = self.regs[src as usize];
                }
                Inst::SetImm(dst, src) => {
                    self.regs[dst as usize] = src;
                }
                Inst::Sub(dst, src) => {
                    self.regs[dst as usize] -= self.regs[src as usize];
                }
                Inst::SubImm(dst, src) => {
                    self.regs[dst as usize] -= src;
                }
                Inst::Mul(dst, src) => {
                    self.regs[dst as usize] *= self.regs[src as usize];
                    self.muls += 1;
                }
                Inst::MulImm(dst, src) => {
                    self.regs[dst as usize] *= src;
                    self.muls += 1;
                }
                Inst::Jnz(dst, src) => {
                    if self.regs[dst as usize] != 0 {
                        self.pc += (self.regs[src as usize] as isize) - 1;
                        assert!(self.pc >= -1);
                    }
                }
                Inst::JnzImm(dst, src) => {
                    if self.regs[dst as usize] != 0 {
                        self.pc += (src as isize) - 1;
                        assert!(self.pc >= -1);
                    }
                }
                Inst::JnzImmImm(dst, src) => {
                    if dst != 0 {
                        self.pc += (src as isize) - 1;
                        assert!(self.pc >= -1);
                    }
                }
                Inst::JnzImmReg(dst, src) => {
                    if dst != 0 {
                        self.pc += (self.regs[src as usize] as isize) - 1;
                        assert!(self.pc >= -1);
                    }
                }
            };
            self.pc += 1;
        }
    }

    fn run_part2(&mut self) {
        let endpc = self.program.len() as isize;
        while self.profile.len() < endpc as usize {
            self.profile.push(0);
        }

        while self.pc < endpc {
            self.profile[self.pc as usize] += 1;
            self.insn_lim -= 1;
            if self.insn_lim == 0 {
                return;
            }

            // Peephole optimizer
            if self.pc == 12 - 1 {
                self.regs[Reg::G as usize] = 0;

                let b = self.regs[Reg::B as usize];
                let d = self.regs[Reg::D as usize];
                if b % d == 0 && b != d {
                    self.regs[Reg::F as usize] = 0;
                }

                self.regs[Reg::E as usize] = b;
                self.pc = 21 - 1;
                continue;
            }

            match self.program[self.pc as usize] {
                Inst::Set(dst, src) => {
                    self.regs[dst as usize] = self.regs[src as usize];
                }
                Inst::SetImm(dst, src) => {
                    self.regs[dst as usize] = src;
                }
                Inst::Sub(dst, src) => {
                    self.regs[dst as usize] -= self.regs[src as usize];
                }
                Inst::SubImm(dst, src) => {
                    self.regs[dst as usize] -= src;
                }
                Inst::Mul(dst, src) => {
                    self.regs[dst as usize] *= self.regs[src as usize];
                    self.muls += 1;
                }
                Inst::MulImm(dst, src) => {
                    self.regs[dst as usize] *= src;
                    self.muls += 1;
                }
                Inst::Jnz(dst, src) => {
                    if self.regs[dst as usize] != 0 {
                        self.pc += (self.regs[src as usize] as isize) - 1;
                        assert!(self.pc >= -1);
                    }
                }
                Inst::JnzImm(dst, src) => {
                    if self.regs[dst as usize] != 0 {
                        self.pc += (src as isize) - 1;
                        assert!(self.pc >= -1);
                    }
                }
                Inst::JnzImmImm(dst, src) => {
                    if dst != 0 {
                        self.pc += (src as isize) - 1;
                        assert!(self.pc >= -1);
                    }
                }
                Inst::JnzImmReg(dst, src) => {
                    if dst != 0 {
                        self.pc += (self.regs[src as usize] as isize) - 1;
                        assert!(self.pc >= -1);
                    }
                }
            };
            self.pc += 1;
        }
    }
}

fn part1(program: &[Inst]) -> u64 {
    let mut emu = Machine::new(program);
    emu.run();
    emu.muls
}

fn part2(program: &[Inst]) -> i64 {
    let mut emu = Machine::new(program);
    emu.regs[Reg::A as usize] = 1;

    // Hotspots identified: L12-15, L17-20.
    // L12:
    // set g d   # g = d;
    // mul g e   # g *= e;
    // sub g b   # g -= b;
    // jnz g 2   # if (g) goto L17;
    //
    // (L16):
    // set f 0
    //
    // L17:
    // sub e -1  # e++;
    // set g e   # g = e;
    // sub g b   # g -= b;
    // jnz g -8  # if (g) goto L12;
    //
    // d = 2;
    // e = 2;
    // while true {
    //   if (d*e == b) {
    //     f = 0;
    //   }
    //   e++;
    //   if (e == b) {
    //     break;
    //   }
    // }
    //
    //emu.insn_lim = 1000_000_000;
    emu.run_part2();
    //dbg!(emu.profile);

    emu.regs[Reg::H as usize]
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2017, 23)?;
    let data = puzzle.get_data()?;

    let data = parse(data);

    let answ1 = part1(&data);
    dbg!(answ1);
    assert_eq!(answ1, 5929);

    let answ2 = part2(&data);
    dbg!(answ2);
    assert_eq!(answ2, 907);

    println!("Ok");
    Ok(())
}
