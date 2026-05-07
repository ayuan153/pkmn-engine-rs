#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Move(u8),
    Switch(u8),
    Tera(u8), // Terastallize + use move index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleResult {
    Ongoing,
    Win(u8),
    Tie,
}
