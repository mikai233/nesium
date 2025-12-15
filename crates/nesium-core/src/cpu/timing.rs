#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Timing {
    Fixed(u8),
    PageCross(u8),
    Branch(u8),
}

const fn f(cycle: u8) -> Timing {
    Timing::Fixed(cycle)
}

const fn p(cycle: u8) -> Timing {
    Timing::PageCross(cycle)
}

const fn b(cycle: u8) -> Timing {
    Timing::Branch(cycle)
}

impl Timing {
    #[cfg(test)]
    pub(crate) const fn basic_cycles(&self) -> usize {
        match self {
            Timing::Fixed(c) | Timing::PageCross(c) | Timing::Branch(c) => *c as usize,
        }
    }

    #[cfg(test)]
    pub(crate) const fn total_cycles(&self, page_crossed: bool, branch_taken: bool) -> usize {
        let mut total = self.basic_cycles();
        if page_crossed && matches!(self, Timing::PageCross(_)) {
            total += 1;
        }

        if branch_taken && matches!(self, Timing::Branch(_)) {
            total += 1;
            if page_crossed {
                total += 1;
            }
        }
        total
    }
}

#[rustfmt::skip]
pub(crate) static CYCLE_TABLE: [Timing; 256] = [
    f(7), f(6), f(0), f(8), f(3), f(3), f(5), f(5), f(3), f(2), f(2), f(2), f(4), f(4), f(6), f(6),
    b(2), p(5), f(0), f(8), f(4), f(4), f(6), f(6), f(2), p(4), f(2), f(7), p(4), p(4), f(7), f(7),
    f(6), f(6), f(0), f(8), f(3), f(3), f(5), f(5), f(4), f(2), f(2), f(2), f(4), f(4), f(6), f(6),
    b(2), p(5), f(0), f(8), f(4), f(4), f(6), f(6), f(2), p(4), f(2), f(7), p(4), p(4), f(7), f(7),
    f(6), f(6), f(0), f(8), f(3), f(3), f(5), f(5), f(3), f(2), f(2), f(2), f(3), f(4), f(6), f(6),
    b(2), p(5), f(0), f(8), f(4), f(4), f(6), f(6), f(2), p(4), f(2), f(7), p(4), p(4), f(7), f(7),
    f(6), f(6), f(0), f(8), f(3), f(3), f(5), f(5), f(4), f(2), f(2), f(2), f(5), f(4), f(6), f(6),
    b(2), p(5), f(0), f(8), f(4), f(4), f(6), f(6), f(2), p(4), f(2), f(7), p(4), p(4), f(7), f(7),
    f(2), f(6), f(2), f(6), f(3), f(3), f(3), f(3), f(2), f(2), f(2), f(2), f(4), f(4), f(4), f(4),
    b(2), f(6), f(0), f(6), f(4), f(4), f(4), f(4), f(2), f(5), f(2), f(5), f(5), f(5), f(5), f(5),
    f(2), f(6), f(2), f(6), f(3), f(3), f(3), f(3), f(2), f(2), f(2), f(2), f(4), f(4), f(4), f(4),
    b(2), p(5), f(0), p(5), f(4), f(4), f(4), f(4), f(2), p(4), f(2), p(4), p(4), p(4), p(4), p(4),
    f(2), f(6), f(2), f(8), f(3), f(3), f(5), f(5), f(2), f(2), f(2), f(2), f(4), f(4), f(6), f(6),
    b(2), p(5), f(0), f(8), f(4), f(4), f(6), f(6), f(2), p(4), f(2), f(7), p(4), p(4), f(7), f(7),
    f(2), f(6), f(2), f(8), f(3), f(3), f(5), f(5), f(2), f(2), f(2), f(2), f(4), f(4), f(6), f(6),
    b(2), p(5), f(0), f(8), f(4), f(4), f(6), f(6), f(2), p(4), f(2), f(7), p(4), p(4), f(7), f(7),
];
