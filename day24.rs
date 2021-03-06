#![allow(dead_code, unused_assignments, unused_imports, unused_parens, unused_variables)]

use anyhow::{anyhow, Result};
use bitvec::prelude::*;
use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::Module;
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

fn jit_segment(
    module: &mut JITModule,
    ctx: &mut cranelift_codegen::Context,
    sig: &Signature,
    i: usize,
    segment: &[I],
    brute_force: bool,
    ) -> cranelift_module::FuncId
{
    let func = module.declare_function(
        &format!("seg{}", i),
        cranelift_module::Linkage::Local,
        sig)
        .unwrap();

    ctx.func.signature = sig.clone();
    ctx.func.name = ExternalName::user(0, func.as_u32());

    let mut func_ctx = FunctionBuilderContext::new();
    let mut bcx = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
    let mut block = bcx.create_block();
    let fault_block = if brute_force {
        Some(bcx.create_block())
    } else {
        None
    };

    bcx.switch_to_block(block);
    bcx.append_block_params_for_function_params(block);

    let w = Variable::new(R::W as usize);
    let x = Variable::new(R::X as usize);
    let y = Variable::new(R::Y as usize);
    let z = Variable::new(R::Z as usize);
    let regs = vec![w,x,y,z];
    bcx.declare_var(w, types::I64);
    bcx.declare_var(x, types::I64);
    bcx.declare_var(y, types::I64);
    bcx.declare_var(z, types::I64);

    let zero = bcx.ins().iconst(types::I64, 0);

    let (inp, shift) = if brute_force {
        let inp = Variable::new(4);
        let shift = Variable::new(5);
        bcx.declare_var(inp, types::I64);
        bcx.declare_var(shift, types::I64);
        bcx.def_var(inp, bcx.block_params(block)[0]);
        bcx.def_var(w, zero);
        bcx.def_var(z, zero);
        let shiftstart = bcx.ins().iconst(types::I64, 10000000000000);
        bcx.def_var(shift, shiftstart);
        (Some(inp), Some(shift))
    } else {
        bcx.def_var(w, bcx.block_params(block)[0]);
        bcx.def_var(z, bcx.block_params(block)[1]);
        (None, None)
    };

    bcx.def_var(x, zero);
    bcx.def_var(y, zero);

    for cmd in segment.iter() {
        match *cmd {
            I::Inp(dst) => {
                // In non-brute force mode, we initialized w via param already.  Emit nothing.
                if !brute_force {
                    continue;
                }

                let dst = regs[dst as usize];
                let arg1 = bcx.use_var(inp.unwrap());
                let arg2 = bcx.use_var(shift.unwrap());
                // Next digit:
                let tmp = bcx.ins().udiv(arg1, arg2);
                let tmp = bcx.ins().urem_imm(tmp, 10);
                // Fault if zero
                bcx.ins().brz(tmp, fault_block.unwrap(), &[]);

                // Cranelift doesn't allow basic blocks to continue after a branch, so create a new
                // one and unconditionally "jump" to it.  Hopefully this results in straight-line
                // code...
                block = {
                    let nextblock = bcx.create_block();
                    // Makes no difference:
                    //bcx.insert_block_after(nextblock, block);
                    nextblock
                };
                bcx.ins().jump(block, &[]);
                bcx.switch_to_block(block);

                bcx.def_var(dst, tmp);
                // Adjust shift:
                let tmp2 = bcx.ins().sdiv_imm(arg2, 10);
                bcx.def_var(shift.unwrap(), tmp2);
            }
            I::Add(dst, src) => {
                let dst = regs[dst as usize];
                let arg1 = bcx.use_var(dst);
                let arg2 = bcx.use_var(regs[src as usize]);
                let tmp = bcx.ins().iadd(arg1, arg2);
                bcx.def_var(dst, tmp);
            }
            I::AddI(dst, src) => {
                let dst = regs[dst as usize];
                let arg1 = bcx.use_var(dst);
                let tmp = bcx.ins().iadd_imm(arg1, src);
                bcx.def_var(dst, tmp);
            }
            I::Mul(dst, src) => {
                let dst = regs[dst as usize];
                let arg1 = bcx.use_var(dst);
                let arg2 = bcx.use_var(regs[src as usize]);
                let tmp = bcx.ins().imul(arg1, arg2);
                bcx.def_var(dst, tmp);
            }
            I::MulI(dst, 0) => {
                let dst = regs[dst as usize];
                bcx.def_var(dst, zero);
            }
            I::MulI(dst, _) => { unreachable!(); }
            I::Div(dst, _) => { unreachable!(); }
            I::DivI(dst, 1) => {
                // nop
            }
            I::DivI(dst, src) => {
                let dst = regs[dst as usize];
                let arg1 = bcx.use_var(dst);
                let tmp = bcx.ins().sdiv_imm(arg1, src);
                bcx.def_var(dst, tmp);
            }
            I::Mod(dst, _) => { unreachable!(); }
            I::ModI(dst, src) => {
                let dst = regs[dst as usize];
                let arg1 = bcx.use_var(dst);
                let tmp = bcx.ins().srem_imm(arg1, src);
                bcx.def_var(dst, tmp);
            }
            I::Eql(dst, src) => {
                let dst = regs[dst as usize];
                let arg1 = bcx.use_var(dst);
                let arg2 = bcx.use_var(regs[src as usize]);
                let tmp = bcx.ins().icmp(IntCC::Equal, arg1, arg2);
                let tmp2 = bcx.ins().bint(types::I64, tmp);
                bcx.def_var(dst, tmp2);
            }
            I::EqlI(dst, src) => {
                let dst = regs[dst as usize];
                let arg1 = bcx.use_var(dst);
                let tmp = bcx.ins().icmp_imm(IntCC::Equal, arg1, src);
                let tmp2 = bcx.ins().bint(types::I64, tmp);
                bcx.def_var(dst, tmp2);
            }
        }
    }

    let z_val = bcx.use_var(z);
    bcx.ins().return_(&[z_val]);

    fault_block.map(|fault_block| {
        // No difference:
        //bcx.insert_block_after(fault_block, block);
        bcx.switch_to_block(fault_block);
        let faultval = bcx.ins().iconst(types::I64, -1);
        bcx.ins().return_(&[faultval]);
    });

    bcx.seal_all_blocks();
    bcx.finalize();
    let mut trap_sink = cranelift_codegen::binemit::NullTrapSink {};
    let mut stack_map_sink = cranelift_codegen::binemit::NullStackMapSink {};
    ctx.want_disasm = true;
    module.define_function(func, ctx, &mut trap_sink, &mut stack_map_sink).unwrap();
    std::fs::write("disas.txt", ctx.mach_compile_result.as_ref().unwrap().disasm.as_ref().unwrap()).unwrap();
    module.clear_context(ctx);

    func
}

struct JitMachine {
    regs: [i64; 4],
    pc: isize,
    segments: Vec<fn(i64,i64) -> i64>,
    module: JITModule,
}

impl JitMachine {
    fn new(program: &[I]) -> Self {
        let mut flag_builder = cranelift_codegen::settings::builder();
        flag_builder.set("opt_level", "speed").unwrap();
        let isa_builder = cranelift_native::builder().unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder));
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let mut module = JITModule::new(builder);

        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));

        let mut ctx = module.make_context();
        let segments = program.chunks(18).enumerate().map(|(i, segment)| {
            jit_segment(&mut module, &mut ctx, &sig, i, segment, false)
        })
        .collect::<Vec<_>>();

        module.finalize_definitions();
        let segments = segments.into_iter().map(|segment| {
            let code = module.get_finalized_function(segment);
            let ptr = unsafe { std::mem::transmute::<_, fn(i64,i64) -> i64>(code) };
            ptr
        })
        .collect();

        Self {
            regs: [0; 4],
            pc: 0,
            module,
            segments,
        }
    }

    fn new_brute(program: &[I]) -> Self {
        let mut flag_builder = cranelift_codegen::settings::builder();
        flag_builder.set("opt_level", "speed").unwrap();
        let isa_builder = cranelift_native::builder().unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder));
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let mut module = JITModule::new(builder);

        let mut sig = module.make_signature();
        sig.params.push(AbiParam::new(types::I64));
        // 2nd param isn't actually used, we're just preserving ABI with non-brute force.
        sig.params.push(AbiParam::new(types::I64));
        sig.returns.push(AbiParam::new(types::I64));

        let mut ctx = module.make_context();
        let func = jit_segment(&mut module, &mut ctx, &sig, 0, program, true);

        module.finalize_definitions();
        let code = module.get_finalized_function(func);
        let ptr = unsafe { std::mem::transmute::<_, fn(i64,i64) -> i64>(code) };

        Self {
            regs: [0; 4],
            pc: 0,
            module,
            segments: vec![ptr],
        }
    }

    fn run_until(&mut self, _endpc: isize) {
        let idx = (self.pc as usize) / 18;
        self.regs[R::Z as usize] = self.segments[idx](self.regs[R::W as usize], self.regs[R::Z as usize]);
    }
}

fn search(cache: &mut HashMap<(u16, i64), Option<i64>>, emu: &mut JitMachine, pc: u16, z: i64) -> Option<i64> {
    if let Some(&val) = cache.get(&(pc, z)) {
        return val;
    }

    for digit in (1..=9).rev() {
        emu.pc = pc as isize + 1;
        emu.regs = [digit, 0, 0, z];
        emu.run_until(pc as isize + 18);
        let z = emu.regs[R::Z as usize];

        if (pc as usize + 18) == 14 * 18 {
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
    let mut emu = JitMachine::new(program);
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

fn search2(cache: &mut HashMap<(u16, i64), Option<i64>>, emu: &mut JitMachine, pc: u16, z: i64) -> Option<i64> {
    if let Some(&val) = cache.get(&(pc, z)) {
        return val;
    }

    for digit in (1..=9) {
        emu.pc = pc as isize + 1;
        emu.regs = [digit, 0, 0, z];
        emu.run_until(pc as isize + 18);
        let z = emu.regs[R::Z as usize];

        if (pc as usize + 18) == 14 * 18 {
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
    let mut emu = JitMachine::new(program);
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

fn part2_brute(program: &ParseResult) -> i64 {
    let emu = JitMachine::new_brute(program);
    let func = emu.segments[0];
    for i in 11111111111111..=99999999999999 {
    //for i in 11811951311485..=11811951311485 {
    //for i in 11811951311485..=99999999999999 {
        let res = func(i, 0);
        if res == 0 {
            return i;
        }
        if res == -1 {
            //println!("fault: {}", i);
        }
        if i % 10_000_000 == 0 {
            dbg!(i);
        }
    }
    unreachable!();
}

fn main() -> Result<()> {
    let mut puzzle = aoc::Puzzle::new(2021, 24)?;
    let data = puzzle.get_data()?;
    let parsed = parse(data);

    //let answ1 = part1(&parsed);
    //dbg!(&answ1);
    //assert_eq!(answ1, 52926995971999);
    //let answ2 = part2(&parsed);
    let answ2 = part2_brute(&parsed);
    dbg!(&answ2);
    assert_eq!(answ2, 11811951311485);
    Ok(())
}
