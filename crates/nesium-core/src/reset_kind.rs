#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetKind {
    PowerOn, // cold boot / power cycle
    Soft,    // regular reset
}
