use crate::types::Type;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveCategory {
    Physical,
    Special,
    Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveFlags {
    bits: u16,
}

impl MoveFlags {
    pub const NONE: Self = Self { bits: 0 };
    pub const CONTACT: u16 = 0x01;
    pub const SOUND: u16 = 0x02;
    pub const BULLET: u16 = 0x04;
    pub const PUNCH: u16 = 0x08;
    pub const BITE: u16 = 0x10;
    pub const PULSE: u16 = 0x20;
    pub const SLICING: u16 = 0x40;

    pub const fn new(bits: u16) -> Self {
        Self { bits }
    }

    pub const fn has(self, flag: u16) -> bool {
        self.bits & flag != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MoveData {
    pub id: u16,
    pub name: &'static str,
    pub move_type: Type,
    pub category: MoveCategory,
    pub base_power: u8,
    pub accuracy: u8, // 0 = always hits
    pub pp: u8,
    pub priority: i8,
    pub flags: MoveFlags,
}

const C: u16 = MoveFlags::CONTACT;
const S: u16 = MoveFlags::SOUND;
const P: u16 = MoveFlags::PUNCH;

pub const MOVES: &[MoveData] = &[
    MoveData { id: 89, name: "Earthquake", move_type: Type::Ground, category: MoveCategory::Physical, base_power: 100, accuracy: 100, pp: 10, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 370, name: "Close Combat", move_type: Type::Fighting, category: MoveCategory::Physical, base_power: 120, accuracy: 100, pp: 5, priority: 0, flags: MoveFlags::new(C) },
    MoveData { id: 282, name: "Knock Off", move_type: Type::Dark, category: MoveCategory::Physical, base_power: 65, accuracy: 100, pp: 20, priority: 0, flags: MoveFlags::new(C) },
    MoveData { id: 369, name: "U-turn", move_type: Type::Bug, category: MoveCategory::Physical, base_power: 70, accuracy: 100, pp: 20, priority: 0, flags: MoveFlags::new(C) },
    MoveData { id: 446, name: "Stealth Rock", move_type: Type::Rock, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 20, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 53, name: "Flamethrower", move_type: Type::Fire, category: MoveCategory::Special, base_power: 90, accuracy: 100, pp: 15, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 58, name: "Ice Beam", move_type: Type::Ice, category: MoveCategory::Special, base_power: 90, accuracy: 100, pp: 10, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 85, name: "Thunderbolt", move_type: Type::Electric, category: MoveCategory::Special, base_power: 90, accuracy: 100, pp: 15, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 57, name: "Surf", move_type: Type::Water, category: MoveCategory::Special, base_power: 90, accuracy: 100, pp: 15, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 94, name: "Psychic", move_type: Type::Psychic, category: MoveCategory::Special, base_power: 90, accuracy: 100, pp: 10, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 247, name: "Shadow Ball", move_type: Type::Ghost, category: MoveCategory::Special, base_power: 80, accuracy: 100, pp: 15, priority: 0, flags: MoveFlags::new(MoveFlags::BULLET) },
    MoveData { id: 399, name: "Dark Pulse", move_type: Type::Dark, category: MoveCategory::Special, base_power: 80, accuracy: 100, pp: 15, priority: 0, flags: MoveFlags::new(MoveFlags::PULSE) },
    MoveData { id: 14, name: "Swords Dance", move_type: Type::Normal, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 20, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 349, name: "Dragon Dance", move_type: Type::Dragon, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 20, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 347, name: "Calm Mind", move_type: Type::Psychic, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 20, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 355, name: "Roost", move_type: Type::Flying, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 5, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 105, name: "Recover", move_type: Type::Normal, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 5, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 92, name: "Toxic", move_type: Type::Poison, category: MoveCategory::Status, base_power: 0, accuracy: 90, pp: 10, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 261, name: "Will-O-Wisp", move_type: Type::Fire, category: MoveCategory::Status, base_power: 0, accuracy: 85, pp: 15, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 86, name: "Thunder Wave", move_type: Type::Electric, category: MoveCategory::Status, base_power: 0, accuracy: 90, pp: 20, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 229, name: "Rapid Spin", move_type: Type::Normal, category: MoveCategory::Physical, base_power: 50, accuracy: 100, pp: 40, priority: 0, flags: MoveFlags::new(C) },
    MoveData { id: 432, name: "Defog", move_type: Type::Flying, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 15, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 413, name: "Brave Bird", move_type: Type::Flying, category: MoveCategory::Physical, base_power: 120, accuracy: 100, pp: 15, priority: 0, flags: MoveFlags::new(C) },
    MoveData { id: 442, name: "Iron Head", move_type: Type::Steel, category: MoveCategory::Physical, base_power: 80, accuracy: 100, pp: 15, priority: 0, flags: MoveFlags::new(C) },
    MoveData { id: 444, name: "Stone Edge", move_type: Type::Rock, category: MoveCategory::Physical, base_power: 100, accuracy: 80, pp: 5, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 56, name: "Hydro Pump", move_type: Type::Water, category: MoveCategory::Special, base_power: 110, accuracy: 80, pp: 5, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 126, name: "Fire Blast", move_type: Type::Fire, category: MoveCategory::Special, base_power: 110, accuracy: 85, pp: 5, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 434, name: "Draco Meteor", move_type: Type::Dragon, category: MoveCategory::Special, base_power: 130, accuracy: 90, pp: 5, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 585, name: "Moonblast", move_type: Type::Fairy, category: MoveCategory::Special, base_power: 95, accuracy: 100, pp: 15, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 583, name: "Play Rough", move_type: Type::Fairy, category: MoveCategory::Physical, base_power: 90, accuracy: 90, pp: 10, priority: 0, flags: MoveFlags::new(C) },
];

pub fn get_move(name: &str) -> Option<&'static MoveData> {
    MOVES.iter().find(|m| m.name.eq_ignore_ascii_case(name))
}

pub fn get_move_by_id(id: u16) -> Option<&'static MoveData> {
    MOVES.iter().find(|m| m.id == id)
}

#[allow(dead_code)]
const _: () = {
    // Suppress unused warnings for constants used only in array initialization
    let _ = S;
    let _ = P;
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_earthquake() {
        let eq = get_move("Earthquake").unwrap();
        assert_eq!(eq.base_power, 100);
        assert_eq!(eq.move_type, Type::Ground);
        assert_eq!(eq.category, MoveCategory::Physical);
    }

    #[test]
    fn test_get_move_case_insensitive() {
        assert!(get_move("earthquake").is_some());
        assert!(get_move("EARTHQUAKE").is_some());
    }

    #[test]
    fn test_status_move_no_power() {
        let sr = get_move("Stealth Rock").unwrap();
        assert_eq!(sr.base_power, 0);
        assert_eq!(sr.category, MoveCategory::Status);
    }

    #[test]
    fn test_contact_flag() {
        let cc = get_move("Close Combat").unwrap();
        assert!(cc.flags.has(MoveFlags::CONTACT));
    }

    #[test]
    fn test_no_contact() {
        let eq = get_move("Earthquake").unwrap();
        assert!(!eq.flags.has(MoveFlags::CONTACT));
    }

    #[test]
    fn test_move_count() {
        assert_eq!(MOVES.len(), 30);
    }

    #[test]
    fn test_nonexistent_move() {
        assert!(get_move("Nonexistent").is_none());
    }
}
