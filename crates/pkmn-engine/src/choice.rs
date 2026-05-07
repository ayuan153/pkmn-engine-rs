use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Choice {
    Move(u8),
    Switch(u8),
    Tera(u8), // Terastallize + use move index
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BattleResult {
    Ongoing,
    Win(u8),
    Tie,
}
