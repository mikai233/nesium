use crate::cpu::addressing::AddressingMode as A;
use crate::cpu::instruction::Instruction as I;

pub(crate) type Table = [I; 256];

pub(crate) static LOOKUP_TABLE: [I; 256] = [I::ldx(A::Absolute); 256];
