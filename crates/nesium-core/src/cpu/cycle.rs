#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Cycle {
    Normal(u8),
    Cross(u8),
    Branch(u8),
}

const fn n(cycle: u8) -> Cycle {
    Cycle::Normal(cycle)
}

const fn c(cycle: u8) -> Cycle {
    Cycle::Cross(cycle)
}

const fn b(cycle: u8) -> Cycle {
    Cycle::Branch(cycle)
}

impl Cycle {
    pub(crate) const fn basic_cycle(&self) -> u8 {
        match self {
            Cycle::Normal(cycle) | Cycle::Cross(cycle) | Cycle::Branch(cycle) => *cycle,
        }
    }
}

#[rustfmt::skip]
pub(crate) static CYCLE_TABLE: [Cycle; 256] = [
    n(7), n(6), n(0), n(8), n(3), n(3), n(5), n(5), n(3), n(2), n(2), n(2), n(4), n(4), n(6), n(6), 
    b(2), c(5), n(0), n(8), n(4), n(4), n(6), n(6), n(2), c(4), n(2), n(7), c(4), c(4), n(7), n(7), 
    n(6), n(6), n(0), n(8), n(3), n(3), n(5), n(5), n(4), n(2), n(2), n(2), n(4), n(4), n(6), n(6), 
    b(2), c(5), n(0), n(8), n(4), n(4), n(6), n(6), n(2), c(4), n(2), n(7), c(4), c(4), n(7), n(7), 
    n(6), n(6), n(0), n(8), n(3), n(3), n(5), n(5), n(3), n(2), n(2), n(2), n(3), n(4), n(6), n(6), 
    b(2), c(5), n(0), n(8), n(4), n(4), n(6), n(6), n(2), c(4), n(2), n(7), c(4), c(4), n(7), n(7), 
    n(6), n(6), n(0), n(8), n(3), n(3), n(5), n(5), n(4), n(2), n(2), n(2), n(5), n(4), n(6), n(6), 
    b(2), c(5), n(0), n(8), n(4), n(4), n(6), n(6), n(2), c(4), n(2), n(7), c(4), c(4), n(7), n(7), 
    n(2), n(6), n(2), n(6), n(3), n(3), n(3), n(3), n(2), n(2), n(2), n(2), n(4), n(4), n(4), n(4), 
    b(2), n(6), n(0), n(6), n(4), n(4), n(4), n(4), n(2), n(5), n(2), n(5), n(5), n(5), n(5), n(5), 
    n(2), n(6), n(2), n(6), n(3), n(3), n(3), n(3), n(2), n(2), n(2), n(2), n(4), n(4), n(4), n(4), 
    b(2), c(5), n(0), c(5), n(4), n(4), n(4), n(4), n(2), c(4), n(2), c(4), c(4), c(4), c(4), c(4), 
    n(2), n(6), n(2), n(8), n(3), n(3), n(5), n(5), n(2), n(2), n(2), n(2), n(4), n(4), n(6), n(6), 
    b(2), c(5), n(0), n(8), n(4), n(4), n(6), n(6), n(2), c(4), n(2), n(7), c(4), c(4), n(7), n(7), 
    n(2), n(6), n(2), n(8), n(3), n(3), n(5), n(5), n(2), n(2), n(2), n(2), n(4), n(4), n(6), n(6), 
    b(2), c(5), n(0), n(8), n(4), n(4), n(6), n(6), n(2), c(4), n(2), n(7), c(4), c(4), n(7), n(7), 
];