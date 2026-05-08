#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ItemId {
    None = 0,
    ChoiceBand,
    ChoiceSpecs,
    ChoiceScarf,
    LifeOrb,
    Leftovers,
    BlackSludge,
    HeavyDutyBoots,
    AssaultVest,
    RockyHelmet,
    FocusSash,
    WeaknessPolicy,
    BoostingBerry,
    ExpertBelt,
    MysticWater,
    Charcoal,
    Eviolite,
    LightClay,
    HeatRock,
    DampRock,
    FlameOrb,
    ToxicOrb,
    SitrusBerry,
    LumBerry,
    ProtectivePads,
    SafetyGoggles,
    ShedShell,
    RedCard,
    AirBalloon,
    LoadedDice,
    ClearAmulet,
    Magnet,
    MiracleSeed,
    NeverMeltIce,
    BlackBelt,
    PoisonBarb,
    SoftSand,
    SharpBeak,
    TwistedSpoon,
    SilverPowder,
    HardStone,
    SpellTag,
    DragonFang,
    BlackGlasses,
    MetalCoat,
    SilkScarf,
    FairyFeather,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemEffect {
    BoostAttack(u8),
    BoostSpAtk(u8),
    BoostSpeed(u8),
    BoostAllDamage(u8),
    EndOfTurnHeal(u8),
    ReduceHazardDmg,
    BoostSpDef(u8),
    ContactDamage(u8),
    SurviveOneHit,
    BoostOnSuperEff,
    NoEffect,
}

pub struct ItemData {
    pub id: ItemId,
    pub name: &'static str,
    pub effect: ItemEffect,
}

static ITEM_TABLE: &[ItemData] = &[
    ItemData { id: ItemId::None, name: "None", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::ChoiceBand, name: "Choice Band", effect: ItemEffect::BoostAttack(150) },
    ItemData { id: ItemId::ChoiceSpecs, name: "Choice Specs", effect: ItemEffect::BoostSpAtk(150) },
    ItemData { id: ItemId::ChoiceScarf, name: "Choice Scarf", effect: ItemEffect::BoostSpeed(150) },
    ItemData { id: ItemId::LifeOrb, name: "Life Orb", effect: ItemEffect::BoostAllDamage(130) },
    ItemData { id: ItemId::Leftovers, name: "Leftovers", effect: ItemEffect::EndOfTurnHeal(16) },
    ItemData { id: ItemId::BlackSludge, name: "Black Sludge", effect: ItemEffect::EndOfTurnHeal(16) },
    ItemData { id: ItemId::HeavyDutyBoots, name: "Heavy-Duty Boots", effect: ItemEffect::ReduceHazardDmg },
    ItemData { id: ItemId::AssaultVest, name: "Assault Vest", effect: ItemEffect::BoostSpDef(150) },
    ItemData { id: ItemId::RockyHelmet, name: "Rocky Helmet", effect: ItemEffect::ContactDamage(6) },
    ItemData { id: ItemId::FocusSash, name: "Focus Sash", effect: ItemEffect::SurviveOneHit },
    ItemData { id: ItemId::WeaknessPolicy, name: "Weakness Policy", effect: ItemEffect::BoostOnSuperEff },
    ItemData { id: ItemId::BoostingBerry, name: "Boosting Berry", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::ExpertBelt, name: "Expert Belt", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::MysticWater, name: "Mystic Water", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::Charcoal, name: "Charcoal", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::Eviolite, name: "Eviolite", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::LightClay, name: "Light Clay", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::HeatRock, name: "Heat Rock", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::DampRock, name: "Damp Rock", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::FlameOrb, name: "Flame Orb", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::ToxicOrb, name: "Toxic Orb", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::SitrusBerry, name: "Sitrus Berry", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::LumBerry, name: "Lum Berry", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::ProtectivePads, name: "Protective Pads", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::SafetyGoggles, name: "Safety Goggles", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::ShedShell, name: "Shed Shell", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::RedCard, name: "Red Card", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::AirBalloon, name: "Air Balloon", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::LoadedDice, name: "Loaded Dice", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::ClearAmulet, name: "Clear Amulet", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::Magnet, name: "Magnet", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::MiracleSeed, name: "Miracle Seed", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::NeverMeltIce, name: "Never-Melt Ice", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::BlackBelt, name: "Black Belt", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::PoisonBarb, name: "Poison Barb", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::SoftSand, name: "Soft Sand", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::SharpBeak, name: "Sharp Beak", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::TwistedSpoon, name: "Twisted Spoon", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::SilverPowder, name: "Silver Powder", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::HardStone, name: "Hard Stone", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::SpellTag, name: "Spell Tag", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::DragonFang, name: "Dragon Fang", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::BlackGlasses, name: "Black Glasses", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::MetalCoat, name: "Metal Coat", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::SilkScarf, name: "Silk Scarf", effect: ItemEffect::NoEffect },
    ItemData { id: ItemId::FairyFeather, name: "Fairy Feather", effect: ItemEffect::NoEffect },
];

pub fn get_item(id: ItemId) -> &'static ItemData {
    ITEM_TABLE.iter().find(|i| i.id == id).unwrap_or(&ITEM_TABLE[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_item() {
        let data = get_item(ItemId::ChoiceBand);
        assert_eq!(data.name, "Choice Band");
        assert_eq!(data.effect, ItemEffect::BoostAttack(150));
    }

    #[test]
    fn test_none_item() {
        let data = get_item(ItemId::None);
        assert_eq!(data.name, "None");
    }
}
