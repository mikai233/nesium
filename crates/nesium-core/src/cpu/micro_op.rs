/// A micro-operation represents the smallest atomic CPU action.
/// Each 6502 instruction can be broken down into a sequence of MicroOps.
/// This enum allows precise cycle-by-cycle emulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MicroOp {
    // === Memory access ===
    /// Read a byte from the program counter (usually for opcode or immediate value),
    /// then increment the program counter.
    FetchOpcode,

    /// Read the low byte of an address from memory at PC, increment PC.
    FetchAddrLo,

    /// Read the high byte of an address from memory at PC, increment PC.
    FetchAddrHi,

    /// Read from zero page address.
    ReadZeroPage,

    /// Write to zero page address.
    WriteZeroPage,

    /// Read from absolute address.
    ReadAbs,

    /// Write to absolute address.
    WriteAbs,

    /// Perform a dummy read used in certain addressing modes (e.g., Absolute,X)
    /// to simulate timing and page crossing behavior.
    DummyRead,

    /// Read from memory using (ZeroPage,X) indirect addressing.
    ReadIndirectXLo,
    /// Read from memory using (ZeroPage,X) indirect addressing.
    ReadIndirectXHi,

    /// Read from memory using (ZeroPage),Y indirect addressing.
    ReadIndirectYLo,
    /// Read from memory using (ZeroPage),Y indirect addressing.
    ReadIndirectYHi,

    /// Add index register (X or Y) to the base address (low byte only),
    /// and check if a page boundary is crossed.
    AddIndexToAddrLo,

    /// Fix the high byte of the address if a page boundary was crossed.
    CorrectAddrHiOnPageCross,

    // === Register operations ===
    /// Load accumulator (A) with a value.
    LoadA,

    /// Load X register.
    LoadX,

    /// Load Y register.
    LoadY,

    /// Store accumulator (A) to memory.
    StoreA,

    /// Store X register to memory.
    StoreX,

    /// Store Y register to memory.
    StoreY,

    /// Transfer A to X.
    TransferAToX,

    /// Transfer A to Y.
    TransferAToY,

    /// Transfer X to A.
    TransferXToA,

    /// Transfer Y to A.
    TransferYToA,

    /// Transfer stack pointer to X.
    TransferSPToX,

    /// Transfer X to stack pointer.
    TransferXToSP,

    // === ALU operations ===
    /// Add memory and carry to A.
    ADC,

    /// Subtract memory and borrow from A.
    SBC,

    /// Logical AND with A.
    AND,

    /// Logical OR with A.
    ORA,

    /// Logical XOR with A.
    EOR,

    /// Compare memory with A.
    CMP,

    /// Compare memory with X.
    CPX,

    /// Compare memory with Y.
    CPY,

    /// Shift A or memory left.
    ASL,

    /// Shift A or memory right.
    LSR,

    /// Rotate A or memory left through carry.
    ROL,

    /// Rotate A or memory right through carry.
    ROR,

    /// Increment memory.
    INC,

    /// Decrement memory.
    DEC,

    // === Branching & control flow ===
    /// Branch to a relative address (if condition met).
    BranchIfCond,

    /// Add branch offset to PC low byte.
    AddBranchOffset,

    /// Correct PC high byte if page boundary crossed during branch.
    FixBranchCross,

    /// Push PC high byte onto stack.
    PushPCH,

    /// Push PC low byte onto stack.
    PushPCL,

    /// Push processor status onto stack.
    PushP,

    /// Pull processor status from stack.
    PullP,

    /// Pull PC low byte from stack.
    PullPCL,

    /// Pull PC high byte from stack.
    PullPCH,

    // === Misc operations ===
    /// Increment the program counter.
    IncPC,

    /// No operation (NOP) – consume one cycle without doing anything.
    Nop,

    /// Set a flag (N, V, D, I, Z, C, etc.).
    SetFlag,

    /// Clear a flag.
    ClearFlag,

    /// Wait one cycle (often used for page-cross penalties or internal operations).
    StallCycle,
}
