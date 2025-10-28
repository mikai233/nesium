use crate::cpu::addressing::Addressing as A;
use crate::cpu::instruction::InstructionTemplate as I;

pub(crate) type Table = [I; 256];

pub(crate) static LOOKUP_TABLE: [I; 2] = [I::ldx(A::Absolute), I::ldx(A::AbsoluteIndirect)];
