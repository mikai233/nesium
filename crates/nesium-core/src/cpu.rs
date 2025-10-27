use crate::cpu::{micro_op::MicroOp, status::Status};
mod phase;
mod status;

mod addressing;
mod instruction;
mod lookup;
mod micro_op;

#[derive(Debug)]
struct CPU {
    // Registers
    a: u8,     //Accumulator
    x: u8,     //X Index Register
    y: u8,     //Y Index Register
    s: u8,     //Stack Pointer
    p: Status, //Processor Status
    pc: u16,   //Program Counter

    // Temp
    addr_lo: u8,
    addr_hi: u8,
    eff_addr: u16,
    fetched: u8,
    rel_offset: i8,
    rel_target: u16,

    current_ops: Vec<MicroOp>,
    op_index: usize,
}
