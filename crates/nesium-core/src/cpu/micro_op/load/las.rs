use crate::cpu::{
    addressing::AddressingMode,
    instruction::{Instruction, Mnemonic},
    micro_op::MicroFn,
};

pub const fn las_absolute_y() -> Instruction {
    struct C1;

    impl MicroFn for C1 {
        fn exec<B>(&self, cpu: &mut crate::cpu::Cpu, _: &mut B)
        where
            B: crate::bus::Bus,
        {
            cpu.incr_pc();
        }
    }

    Instruction {
        opcode: Mnemonic::LAS,
        addressing: AddressingMode::Absolute,
        micro_ops: todo!(),
    }
}
