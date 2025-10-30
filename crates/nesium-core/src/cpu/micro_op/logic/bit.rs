use crate::cpu::{
    addressing::Addressing,
    instruction::{Instruction, Mnemonic},
    micro_op::{MicroOp, ReadFrom},
};

// ================================================================
//  1. Zero Page: BIT $nn      $24    2 bytes, 3 cycles
// ================================================================
pub const fn bit_zero_page() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_zp_addr_lo();
    const OP3: MicroOp = MicroOp::bit(ReadFrom::ZeroPage);

    Instruction {
        opcode: Mnemonic::BIT,
        addressing: Addressing::ZeroPage,
        micro_ops: &[OP1, OP2, OP3],
    }
}

// ================================================================
//  2. Absolute: BIT $nnnn     $2C    3 bytes, 4 cycles
// ================================================================
pub const fn bit_absolute() -> Instruction {
    const OP1: MicroOp = MicroOp::advance_pc_after_opcode();
    const OP2: MicroOp = MicroOp::fetch_abs_addr_lo();
    const OP3: MicroOp = MicroOp::fetch_abs_addr_hi();
    const OP4: MicroOp = MicroOp::bit(ReadFrom::Effective);

    Instruction {
        opcode: Mnemonic::BIT,
        addressing: Addressing::Absolute,
        micro_ops: &[OP1, OP2, OP3, OP4],
    }
}

#[cfg(test)]
mod bit_tests {
    use crate::{
        bus::{BusImpl, mock::MockBus},
        cpu::{Cpu, status::Status},
    };

    use super::*;

    // -----------------------------------------------------------------------
    // Helper: create a fresh CPU + Bus with given initial state
    // -----------------------------------------------------------------------
    fn setup(pc: u16, a: u8, x: u8, y: u8, mem_setup: impl FnOnce(&mut MockBus)) -> (Cpu, BusImpl) {
        let mut mock = MockBus::default();
        mem_setup(&mut mock);

        let mut cpu = Cpu::new();
        cpu.pc = pc;
        cpu.a = a;
        cpu.x = x;
        cpu.y = y;
        cpu.p = Status::empty();

        let bus = BusImpl::Dynamic(Box::new(mock));
        (cpu, bus)
    }

    // -----------------------------------------------------------------------
    // 1. test_bit_zeropage
    //    A & mem = 0 → Z=1, N=mem[7], V=mem[6]
    // -----------------------------------------------------------------------
    #[test]
    fn test_bit_zeropage() {
        let instr = bit_zero_page();
        let (mut cpu, mut bus) = setup(
            0xC000,
            0b0011_0101, // A
            0,
            0,
            |mock| {
                mock.mem[0xC001] = 0x80; // ZP address
                mock.mem[0x0080] = 0b1100_0000; // mem: N=1, V=1, A&mem=0
            },
        );

        let old_a = cpu.a;

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, old_a); // A unchanged
        assert!(cpu.p.contains(Status::ZERO)); // A & mem == 0
        assert!(cpu.p.contains(Status::NEGATIVE)); // mem[7] = 1
        assert!(cpu.p.contains(Status::OVERFLOW)); // mem[6] = 1
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 2. test_bit_zeropage_non_zero_result
    //    A & mem != 0 → Z=0
    // -----------------------------------------------------------------------
    #[test]
    fn test_bit_zeropage_non_zero_result() {
        let instr = bit_zero_page();
        let (mut cpu, mut bus) = setup(0xC000, 0b1111_0000, 0, 0, |mock| {
            mock.mem[0xC001] = 0x34;
            mock.mem[0x0034] = 0b0111_0000; // A & mem = 0b0111_0000 != 0
        });

        let old_a = cpu.a;

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, old_a);
        assert!(!cpu.p.contains(Status::ZERO)); // result != 0
        assert!(!cpu.p.contains(Status::NEGATIVE)); // mem[7] = 0
        assert!(cpu.p.contains(Status::OVERFLOW)); // mem[6] = 1
    }

    // -----------------------------------------------------------------------
    // 3. test_bit_absolute
    //    Full address, same logic
    // -----------------------------------------------------------------------
    #[test]
    fn test_bit_absolute() {
        let instr = bit_absolute();
        let (mut cpu, mut bus) = setup(0xC000, 0b0000_1111, 0, 0, |mock| {
            mock.mem[0xC001] = 0x78;
            mock.mem[0xC002] = 0x9A; // $9A78
            mock.mem[0x9A78] = 0b1100_0011; // N=1, V=1, A&mem=0b0000_0011 != 0
        });

        let old_a = cpu.a;

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC003);
        assert_eq!(cpu.a, old_a);
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(cpu.p.contains(Status::NEGATIVE));
        assert!(cpu.p.contains(Status::OVERFLOW));
        assert!(!cpu.crossed_page);
    }

    // -----------------------------------------------------------------------
    // 4. test_bit_absolute_zero_result
    //    A & mem == 0, N=0, V=0
    // -----------------------------------------------------------------------
    #[test]
    fn test_bit_absolute_zero_result() {
        let instr = bit_absolute();
        let (mut cpu, mut bus) = setup(0xC000, 0b1010_1010, 0, 0, |mock| {
            mock.mem[0xC001] = 0x00;
            mock.mem[0xC002] = 0x20; // $2000
            mock.mem[0x2000] = 0b0101_0101; // A & mem = 0
        });

        let old_a = cpu.a;

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC003);
        assert_eq!(cpu.a, old_a);
        assert!(cpu.p.contains(Status::ZERO));
        assert!(!cpu.p.contains(Status::NEGATIVE)); // mem[7] = 0
        assert!(cpu.p.contains(Status::OVERFLOW)); // mem[6] = 1
    }

    // -----------------------------------------------------------------------
    // 5. test_bit_zeropage_clears_nv
    //    Ensure N/V are taken from memory, not previous state
    // -----------------------------------------------------------------------
    #[test]
    fn test_bit_zeropage_clears_nv() {
        let instr = bit_zero_page();
        let mut cpu = Cpu::new();
        cpu.pc = 0xC000;
        cpu.a = 0xFF;
        cpu.p = Status::NEGATIVE | Status::OVERFLOW; // preset N,V

        let mut mock = MockBus::default();
        mock.mem[0xC001] = 0x50;
        mock.mem[0x0050] = 0b0011_1100; // N=0, V=0, A&mem != 0

        let mut bus = BusImpl::Dynamic(Box::new(mock));

        cpu.test_clock(&mut bus, &instr);

        assert_eq!(cpu.pc, 0xC002);
        assert_eq!(cpu.a, 0xFF);
        assert!(!cpu.p.contains(Status::ZERO));
        assert!(!cpu.p.contains(Status::NEGATIVE)); // cleared
        assert!(!cpu.p.contains(Status::OVERFLOW)); // cleared
    }
}
