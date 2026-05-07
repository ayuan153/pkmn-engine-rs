#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Move(u8),
    Switch(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleResult {
    Ongoing,
    Win(u8),
    Tie,
}
