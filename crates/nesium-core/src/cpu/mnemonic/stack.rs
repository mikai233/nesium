//! # NES/Ricoh 2A03 CPU Emulation: Cycle-Accurate Stack Operations
//!
//! This module implements the cycle-accurate behavior of stack PUSH (PHA, PHP)
//! and PULL (PLA, PLP) instructions for the NMOS 6502 architecture (used in the NES/Famicom).
//!
//! Due to the 6502's design constraint—requiring a bus access on *every* clock cycle—
//! internal operations (like register setup or pointer arithmetic) are often "filled"
//! with dummy memory reads or writes. This leads to the non-obvious cycle counts.
//!
//! ## 1. PUSH Operations (PHA, PHP) - 3 Cycles Total
//!
//! PUSH operations (Write to Stack) require one extra cycle for internal setup, resulting in 3 total cycles:
//!
//! | Cycle | Bus Action | Address (A) | Data (D) | Purpose                                                      |
//! |-------|------------|-------------|----------|--------------------------------------------------------------|
//! | T1    | Read       | PC          | Opcode   | Fetch the opcode. PC increments.                             |
//! | T2    | Read       | PC + 1      | Junk     | **Internal Setup:** CPU prepares data/address; performs a dummy read from the program counter's next byte (data is discarded). |
//! | T3    | Write      | $01XX       | P/A      | **Execute:** Write data to the Stack; Stack Pointer (SP) decrements. |
//!
//! ## 2. PULL Operations (PLA, PLP) - 4 Cycles Total
//!
//! PULL operations (Read from Stack) require two extra cycles: one for setup and one for Stack Pointer increment, resulting in 4 total cycles:
//!
//! | Cycle | Bus Action | Address (A) | Data (D) | Purpose                                                      |
//! |-------|------------|-------------|----------|--------------------------------------------------------------|
//! | T1    | Read       | PC          | Opcode   | Fetch the opcode. PC increments.                             |
//! | T2    | Read       | PC + 1      | Junk     | **Internal Setup:** CPU prepares to operate. Dummy read from PC+1 (data is discarded). |
//! | T3    | Read       | $01XX       | Junk     | **SP Increment:** CPU increments SP; performs a dummy read from the *old* stack address (data is discarded). |
//! | T4    | Read       | $01XX+1     | Data     | **Execute:** Pull data from the *new* stack address into the target register (A or P). |
//!
//! **Warning:** For cycle-accurate NES emulation, especially when handling Memory-Mapped I/O (MMIO) like the PPU/APU registers, these dummy memory accesses (T2, T3) must be simulated, as they consume crucial clock cycles.

use crate::{
    bus::STACK_ADDR,
    cpu::{micro_op::MicroOp, mnemonic::Mnemonic, status::Status},
};

impl Mnemonic {
    /// NV-BDIZC
    /// --------
    ///
    /// PHA - Push Accumulator On Stack
    /// Operation: A↓
    ///
    /// This instruction transfers the current value of the accumulator to the next
    /// location on the stack, automatically decrementing the stack to point to the
    /// next empty location.
    ///
    /// The Push A instruction only affects the stack pointer register which is
    /// decremented by 1 as a result of the operation. It affects no flags.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | PHA                      | $48    | 1         | 3
    pub(crate) const fn pha() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "pha_dummy_read",
                micro_fn: |cpu, bus| {
                    // Cycle 1: Dummy read from current PC (internal operation)
                    let _ = bus.read(cpu.pc);
                },
            },
            MicroOp {
                name: "pha_write_stack",
                micro_fn: |cpu, bus| {
                    // Cycle 2: Write accumulator to stack, then decrement S
                    // Hardware writes to [0x0100 + S] using current S, then S--
                    cpu.push(bus, cpu.a);
                },
            },
        ]
    }

    /// NV-BDIZC
    /// --------
    ///
    /// PHP - Push Processor Status On Stack
    /// Operation: P↓
    ///
    /// This instruction transfers the contents of the processor status register
    /// unchanged to the stack, as governed by the stack pointer.
    ///
    /// The PHP instruction affects no registers or flags in the microprocessor.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | PHP                      | $08    | 1         | 3
    pub(crate) const fn php() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "php_dummy_read",
                micro_fn: |cpu, bus| {
                    // Cycle 1: Dummy read from current PC
                    let _ = bus.read(cpu.pc);
                },
            },
            MicroOp {
                name: "php_write_stack",
                micro_fn: |cpu, bus| {
                    // Cycle 2: Hardware forces B flag (bit4) and unused bit5 when pushing
                    let p = cpu.p | Status::BREAK | Status::UNUSED;
                    let p = p.bits();
                    cpu.push(bus, p);
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓-----✓-
    ///
    /// PLA - Pull Accumulator From Stack
    /// Operation: A↑
    ///
    /// This instruction adds 1 to the current value of the stack pointer and uses it
    /// to address the stack and loads the contents of the stack into the A register.
    ///
    /// The PLA instruction does not affect the carry or overflow flags. It sets N if
    /// the bit 7 is on in accumulator A as a result of instructions, otherwise it is
    /// reset. If accumulator A is zero as a result of the PLA, then the Z flag is
    /// set, otherwise it is reset. The PLA instruction changes content of the
    /// accumulator A to the contents of the memory location at stack register plus 1
    /// and also increments the stack register.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | PLA                      | $68    | 1         | 4
    pub(crate) const fn pla() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "pla_dummy_read1",
                micro_fn: |cpu, bus| {
                    // Cycle 1: Dummy read from PC
                    let _ = bus.read(cpu.pc);
                },
            },
            MicroOp {
                name: "pla_dummy_read2",
                micro_fn: |cpu, bus| {
                    // Cycle 2: Dummy read from current stack location (before increment)
                    let _ = bus.read(STACK_ADDR | cpu.s as u16);
                },
            },
            MicroOp {
                name: "pla_pull_value",
                micro_fn: |cpu, bus| {
                    // Cycle 3: Increment S first, then read from new stack pointer
                    let value = cpu.pull(bus);
                    cpu.a = value;
                    cpu.p.set_zn(value); // Update N and Z flags based on pulled value
                },
            },
        ]
    }

    /// NV-BDIZC
    /// ✓✓--✓✓✓✓
    ///
    /// PLP - Pull Processor Status From Stack
    /// Operation: P↑
    ///
    /// This instruction transfers the next value on the stack to the Processor Status
    /// register, thereby changing all of the flags and setting the mode switches to
    /// the values from the stack.
    ///
    /// The PLP instruction affects no registers in the processor other than the
    /// status register. This instruction could affect all flags in the status
    /// register.
    ///
    /// Addressing Mode | Assembly Language Form | Opcode | No. Bytes | No. Cycles
    /// --------------- | ------------------------ | ------ | --------- | ----------
    /// Implied         | PLP                      | $28    | 1         | 4
    pub(crate) const fn plp() -> &'static [MicroOp] {
        &[
            MicroOp {
                name: "plp_dummy_read1",
                micro_fn: |cpu, bus| {
                    // Cycle 1: Dummy read from PC
                    let _ = bus.read(cpu.pc);
                },
            },
            MicroOp {
                name: "plp_dummy_read2",
                micro_fn: |cpu, bus| {
                    // Cycle 2: Dummy read from current stack location
                    let _ = bus.read(STACK_ADDR | cpu.s as u16);
                },
            },
            MicroOp {
                name: "plp_pull_status",
                micro_fn: |cpu, bus| {
                    // Cycle 3: Increment S first
                    let value = cpu.pull(bus);

                    // Hardware behavior:
                    // - Clear B flag (bit 4): & 0xEF
                    // - Force unused bit 5 to 1: | 0x20
                    let mut p = Status::from_bits_truncate(value);
                    p.set_b(false);
                    p.set_u(true);
                    cpu.p = p;
                },
            },
        ]
    }
}

#[cfg(test)]
mod stack_tests {
    use crate::{
        bus::STACK_ADDR,
        cpu::{
            mnemonic::{Mnemonic, tests::InstrTest},
            status::Status,
        },
    };

    #[test]
    fn test_pha() {
        InstrTest::new(Mnemonic::PHA).test(|verify, cpu, bus| {
            let v = verify.cpu.a;
            assert_eq!(verify.cpu.s.wrapping_sub(1), cpu.s);
            let m = bus.read(STACK_ADDR | verify.cpu.s as u16);
            assert_eq!(v, m);
        });
    }

    #[test]
    fn test_php() {
        InstrTest::new(Mnemonic::PHP).test(|verify, cpu, bus| {
            let v = verify.cpu.p | Status::BREAK | Status::UNUSED;
            assert_eq!(verify.cpu.s.wrapping_sub(1), cpu.s);
            let m = bus.read(STACK_ADDR | verify.cpu.s as u16);
            assert_eq!(v.bits(), m);
            assert_eq!(verify.cpu.p, cpu.p);
        });
    }

    #[test]
    fn test_pla() {
        InstrTest::new(Mnemonic::PLA).test(|verify, cpu, bus| {
            assert_eq!(verify.cpu.s.wrapping_add(1), cpu.s);
            let m = bus.read(STACK_ADDR | verify.cpu.s as u16);
            assert_eq!(cpu.a, m);
            verify.check_nz(cpu.p, m);
        });
    }

    #[test]
    fn test_plp() {
        InstrTest::new(Mnemonic::PLP).test(|verify, cpu, bus| {
            assert_eq!(verify.cpu.s.wrapping_add(1), cpu.s);
            let m = bus.read(STACK_ADDR | verify.cpu.s as u16);
            let mut p = Status::from_bits_truncate(m);
            //TODO
            p.remove(Status::BREAK);
            p.insert(Status::UNUSED);
            assert_eq!(cpu.p, p);
        });
    }
}
