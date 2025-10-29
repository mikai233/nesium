use crate::cpu::addressing::Addressing as A;
use crate::cpu::instruction::Instruction as I;

pub(crate) type Table = [I; 256];

pub(crate) static LOOKUP_TABLE: [I; 256] = [I::ldx(A::Absolute); 256];
