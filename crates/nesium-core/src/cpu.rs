use crate::cpu::lookup::Table;
use crate::cpu::status::Status;
mod phase;
mod status;

mod addressing;
mod instruction;
mod lookup;
mod micro_op;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AddrContext {
    effective_addr: Option<u16>,
    data: Option<u8>,
    crossed_page: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CPU {
    // Registers
    a: u8,     //Accumulator
    x: u8,     //X Index Register
    y: u8,     //Y Index Register
    s: u8,     //Stack Pointer
    p: Status, //Processor Status
    pc: u16,   //Program Counter

    lookup: &'static Table,
    context: AddrContext,
    op_index: usize,
}
