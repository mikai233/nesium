use phf::phf_map;

use crate::cpu::instruction::InstructionTemplate;

static LOOKUP: phf::Map<u8, InstructionTemplate> = phf_map! {};
