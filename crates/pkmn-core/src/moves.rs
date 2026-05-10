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
    MoveData { id: 182, name: "Protect", move_type: Type::Normal, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 10, priority: 4, flags: MoveFlags::NONE },
    MoveData { id: 164, name: "Substitute", move_type: Type::Normal, category: MoveCategory::Status, base_power: 0, accuracy: 0, pp: 10, priority: 0, flags: MoveFlags::NONE },
    MoveData { id: 200, name: "Outrage", move_type: Type::Dragon, category: MoveCategory::Physical, base_power: 120, accuracy: 100, pp: 10, priority: 0, flags: MoveFlags::new(C) },
    MoveData { id: 63, name: "Hyper Beam", move_type: Type::Normal, category: MoveCategory::Special, base_power: 150, accuracy: 90, pp: 5, priority: 0, flags: MoveFlags::NONE },
];

pub fn get_move(name: &str) -> Option<&'static MoveData> {
    MOVES.iter().find(|m| m.name.eq_ignore_ascii_case(name))
        .or_else(|| crate::gen::move_data::get_move_by_name(name))
}

pub fn get_move_by_id(id: u16) -> Option<&'static MoveData> {
    MOVES.iter().find(|m| m.id == id)
        .or_else(|| crate::gen::move_data::get_move_by_id(id))
}

/// Secondary effect of a move
#[derive(Debug, Clone, Copy)]
pub enum SecondaryEffect {
    /// Inflict a status on the target
    Status(SecondaryStatus),
    /// Boost/drop a stat on the target
    StatDrop(Stat, i8),
    /// Boost/drop a stat on the user
    SelfStatBoost(Stat, i8),
    /// No-op: just consume the RNG call to stay in sync with PS
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum SecondaryStatus {
    Burn,
    Paralyze,
    Freeze,
    Poison,
    Flinch,
}

#[derive(Debug, Clone, Copy)]
pub enum Stat {
    Atk,
    Def,
    Spa,
    Spd,
    Spe,
}

#[derive(Debug, Clone, Copy)]
pub struct Secondary {
    pub chance: u32,
    pub effect: SecondaryEffect,
}

/// Get secondary effects for a move by ID
pub fn get_secondaries(id: u16) -> &'static [Secondary] {
    match id {
        // 10% burn
        7 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],        // Fire Punch
        53 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],       // Flamethrower
        126 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],      // Fire Blast
        394 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],      // Flare Blitz
        436 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],      // Lava Plume
        // Higher % burn
        221 => &[Secondary { chance: 50, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],      // Sacred Fire
        503 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],      // Scald
        592 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],      // Steam Eruption
        545 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],      // Searing Shot
        551 => &[Secondary { chance: 20, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],      // Blue Flare
        517 => &[Secondary { chance: 100, effect: SecondaryEffect::Status(SecondaryStatus::Burn) }],     // Inferno

        // 10% freeze
        8 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Freeze) }],      // Ice Punch
        58 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Freeze) }],     // Ice Beam
        59 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Freeze) }],     // Blizzard
        573 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Freeze) }],    // Freeze-Dry

        // Paralysis
        9 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Paralyze) }],    // Thunder Punch
        85 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Paralyze) }],   // Thunderbolt
        87 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Paralyze) }],   // Thunder
        435 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Paralyze) }],  // Discharge
        609 => &[Secondary { chance: 100, effect: SecondaryEffect::Status(SecondaryStatus::Paralyze) }], // Nuzzle
        34 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Paralyze) }],   // Body Slam
        340 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Paralyze) }],  // Bounce
        395 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Paralyze) }],  // Force Palm

        // Flinch
        442 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],    // Iron Head
        399 => &[Secondary { chance: 20, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],    // Dark Pulse
        403 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],    // Air Slash
        428 => &[Secondary { chance: 20, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],    // Zen Headbutt
        127 => &[Secondary { chance: 20, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],    // Waterfall
        157 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],    // Rock Slide
        556 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],    // Icicle Crash
        252 => &[Secondary { chance: 100, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],   // Fake Out
        29 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],     // Headbutt
        44 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],     // Bite
        173 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Flinch) }],    // Snore

        // Stat drops: -1 SpA on target
        789 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spa, -1) }],           // Spirit Break
        595 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spa, -1) }],           // Mystical Fire
        585 => &[Secondary { chance: 30, effect: SecondaryEffect::StatDrop(Stat::Spa, -1) }],            // Moonblast
        555 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spa, -1) }],           // Snarl

        // Stat drops: -1 Def on target
        306 => &[Secondary { chance: 50, effect: SecondaryEffect::StatDrop(Stat::Def, -1) }],            // Crush Claw
        708 => &[Secondary { chance: 20, effect: SecondaryEffect::StatDrop(Stat::Def, -1) }],            // Shadow Bone
        242 => &[Secondary { chance: 20, effect: SecondaryEffect::StatDrop(Stat::Def, -1) }],            // Crunch
        710 => &[Secondary { chance: 20, effect: SecondaryEffect::StatDrop(Stat::Def, -1) }],            // Liquidation
        534 => &[Secondary { chance: 50, effect: SecondaryEffect::StatDrop(Stat::Def, -1) }],            // Razor Shell

        // Stat drops: -1 SpD on target
        411 => &[Secondary { chance: 10, effect: SecondaryEffect::StatDrop(Stat::Spd, -1) }],            // Focus Blast
        247 => &[Secondary { chance: 20, effect: SecondaryEffect::StatDrop(Stat::Spd, -1) }],            // Shadow Ball
        94 => &[Secondary { chance: 10, effect: SecondaryEffect::StatDrop(Stat::Spd, -1) }],             // Psychic
        412 => &[Secondary { chance: 10, effect: SecondaryEffect::StatDrop(Stat::Spd, -1) }],            // Energy Ball
        414 => &[Secondary { chance: 10, effect: SecondaryEffect::StatDrop(Stat::Spd, -1) }],            // Earth Power
        430 => &[Secondary { chance: 10, effect: SecondaryEffect::StatDrop(Stat::Spd, -1) }],            // Flash Cannon
        491 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spd, -2) }],           // Acid Spray
        465 => &[Secondary { chance: 40, effect: SecondaryEffect::StatDrop(Stat::Spd, -2) }],            // Seed Flare
        405 => &[Secondary { chance: 10, effect: SecondaryEffect::StatDrop(Stat::Spd, -1) }],            // Bug Buzz

        // Stat drops: -1 Spe on target
        196 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spe, -1) }],           // Icy Wind
        527 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spe, -1) }],           // Electroweb
        523 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spe, -1) }],           // Bulldoze
        317 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spe, -1) }],           // Rock Tomb
        490 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Spe, -1) }],           // Low Sweep

        // Stat drops: -1 Atk on target
        679 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Atk, -1) }],           // Lunge
        583 => &[Secondary { chance: 10, effect: SecondaryEffect::StatDrop(Stat::Atk, -1) }],            // Play Rough
        575 => &[Secondary { chance: 100, effect: SecondaryEffect::StatDrop(Stat::Atk, -1) }],           // Parting Shot (simplified)

        // Poison
        188 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Poison) }],    // Sludge Bomb
        482 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Poison) }],    // Sludge Wave
        398 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Poison) }],    // Poison Jab
        441 => &[Secondary { chance: 30, effect: SecondaryEffect::Status(SecondaryStatus::Poison) }],    // Gunk Shot
        440 => &[Secondary { chance: 10, effect: SecondaryEffect::Status(SecondaryStatus::Poison) }],    // Cross Poison

        // Confusion
        542 => &[Secondary { chance: 30, effect: SecondaryEffect::None }],                               // Hurricane (confusion)
        60 => &[Secondary { chance: 10, effect: SecondaryEffect::None }],                                // Psybeam (confusion)
        93 => &[Secondary { chance: 10, effect: SecondaryEffect::None }],                                // Confusion (confusion)
        223 => &[Secondary { chance: 100, effect: SecondaryEffect::None }],                              // Dynamic Punch (confusion)

        // Sleep
        547 => &[Secondary { chance: 10, effect: SecondaryEffect::None }],                               // Relic Song (sleep)

        // Self stat boosts (Scale Shot)
        799 => &[Secondary { chance: 100, effect: SecondaryEffect::SelfStatBoost(Stat::Def, -1) },
                 Secondary { chance: 100, effect: SecondaryEffect::SelfStatBoost(Stat::Spe, 1) }],       // Scale Shot
        _ => &[],
    }
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
        assert_eq!(MOVES.len(), 34);
    }

    #[test]
    fn test_nonexistent_move() {
        assert!(get_move("Nonexistent").is_none());
    }
}
