/// CPU internal phases to disambiguate micro-operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    // --- Fetch stage ---
    FetchOpcode,
    FetchAddrLo,
    FetchAddrHi,
    FetchImmediate,
    FetchZeroPage,
    FetchOffset, // relative branch offset

    // --- Addressing / effective address calculation ---
    EAZeroPage,       // effective address for zero page
    EAZeroPageX,      // zero page + X
    EAZeroPageY,      // zero page + Y
    EAAbsolute,       // absolute
    EAAbsoluteX,      // absolute + X
    EAAbsoluteY,      // absolute + Y
    EAIndirect,       // JMP ($xxxx)
    EAIndirectX,      // (ZP,X)
    EAIndirectY,      // (ZP),Y
    EACrossPageDummy, // dummy read for page crossing
    EAAddIndex,       // add X/Y index

    // --- Execute / read-modify-write / writeback ---
    Execute,       // general execution (ALU, load/store)
    RMWRead,       // first read for RMW
    RMWAlu,        // ALU modification
    RMWWrite,      // first writeback for RMW
    RMWWriteFinal, // final writeback (if needed)

    // --- Stack operations ---
    PushPCL,
    PushPCH,
    PushP,
    PullPCL,
    PullPCH,
    PullP,

    // --- Branching ---
    BranchCheck,     // check condition flags
    BranchAddOffset, // add relative offset
    BranchFixPage,   // page-cross correction

    // --- Interrupts ---
    InterruptPushSeq,    // composite: push PCH/PCL/Status
    ReturnFromInterrupt, // composite: pull Status/PCL/PCH

    // --- Misc / NOP / stall ---
    StallCycle,
    Nop,
}
