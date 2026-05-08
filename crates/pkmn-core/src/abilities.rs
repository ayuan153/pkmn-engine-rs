#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum AbilityId {
    None = 0,
    Intimidate,
    Levitate,
    MoldBreaker,
    Multiscale,
    Sturdy,
    FlameBody,
    Static,
    PoisonPoint,
    RoughSkin,
    IronBarbs,
    NaturalCure,
    Regenerator,
    Unaware,
    MagicBounce,
    MagicGuard,
    Technician,
    Adaptability,
    HugePower,
    PurePower,
    SheerForce,
    Protean,
    Libero,
    ToughClaws,
    IronFist,
    StrongJaw,
    SwiftSwim,
    Chlorophyll,
    SandRush,
    SlushRush,
    SurgeSurfer,
    Drought,
    Drizzle,
    SandStream,
    SnowWarning,
    ElectricSurge,
    GrassySurge,
    MistySurge,
    PsychicSurge,
    Guts,
    MarvelScale,
    Overcoat,
    ThickFat,
    FlashFire,
    VoltAbsorb,
    WaterAbsorb,
    LightningRod,
    StormDrain,
    ClearBody,
    WhiteSmoke,
    FullMetalBody,
    SpeedBoost,
    BeastBoost,
    Moxie,
    TintedLens,
    SwordOfRuin,
    TabletsOfRuin,
    VesselOfRuin,
    BeadsOfRuin,
    Pressure,
    Prankster,
    SupremeOverlord,
    CursedBody,
    SkillLink,
    Unnerve,
    CloudNine,
    Turboblaze,
    Teravolt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbilityTrigger {
    OnSwitchIn,
    OnBeingHit,
    OnTakingDamage,
    OnDealingDamage,
    ModifyDamage,
    ModifySpeed,
    Immunity,
    EndOfTurn,
    OnSwitchOut,
    Passive,
}

pub struct AbilityData {
    pub id: AbilityId,
    pub name: &'static str,
    pub triggers: &'static [AbilityTrigger],
}

static ABILITY_TABLE: &[AbilityData] = &[
    AbilityData { id: AbilityId::None, name: "None", triggers: &[] },
    AbilityData { id: AbilityId::Intimidate, name: "Intimidate", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::Levitate, name: "Levitate", triggers: &[AbilityTrigger::Immunity] },
    AbilityData { id: AbilityId::MoldBreaker, name: "Mold Breaker", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::Multiscale, name: "Multiscale", triggers: &[AbilityTrigger::OnTakingDamage] },
    AbilityData { id: AbilityId::Sturdy, name: "Sturdy", triggers: &[AbilityTrigger::OnTakingDamage] },
    AbilityData { id: AbilityId::FlameBody, name: "Flame Body", triggers: &[AbilityTrigger::OnBeingHit] },
    AbilityData { id: AbilityId::Static, name: "Static", triggers: &[AbilityTrigger::OnBeingHit] },
    AbilityData { id: AbilityId::PoisonPoint, name: "Poison Point", triggers: &[AbilityTrigger::OnBeingHit] },
    AbilityData { id: AbilityId::RoughSkin, name: "Rough Skin", triggers: &[AbilityTrigger::OnBeingHit] },
    AbilityData { id: AbilityId::IronBarbs, name: "Iron Barbs", triggers: &[AbilityTrigger::OnBeingHit] },
    AbilityData { id: AbilityId::NaturalCure, name: "Natural Cure", triggers: &[AbilityTrigger::OnSwitchOut] },
    AbilityData { id: AbilityId::Regenerator, name: "Regenerator", triggers: &[AbilityTrigger::OnSwitchOut] },
    AbilityData { id: AbilityId::Unaware, name: "Unaware", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::MagicBounce, name: "Magic Bounce", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::MagicGuard, name: "Magic Guard", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::Technician, name: "Technician", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::Adaptability, name: "Adaptability", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::HugePower, name: "Huge Power", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::PurePower, name: "Pure Power", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::SheerForce, name: "Sheer Force", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::Protean, name: "Protean", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::Libero, name: "Libero", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::ToughClaws, name: "Tough Claws", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::IronFist, name: "Iron Fist", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::StrongJaw, name: "Strong Jaw", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::SwiftSwim, name: "Swift Swim", triggers: &[AbilityTrigger::ModifySpeed] },
    AbilityData { id: AbilityId::Chlorophyll, name: "Chlorophyll", triggers: &[AbilityTrigger::ModifySpeed] },
    AbilityData { id: AbilityId::SandRush, name: "Sand Rush", triggers: &[AbilityTrigger::ModifySpeed] },
    AbilityData { id: AbilityId::SlushRush, name: "Slush Rush", triggers: &[AbilityTrigger::ModifySpeed] },
    AbilityData { id: AbilityId::SurgeSurfer, name: "Surge Surfer", triggers: &[AbilityTrigger::ModifySpeed] },
    AbilityData { id: AbilityId::Drought, name: "Drought", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::Drizzle, name: "Drizzle", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::SandStream, name: "Sand Stream", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::SnowWarning, name: "Snow Warning", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::ElectricSurge, name: "Electric Surge", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::GrassySurge, name: "Grassy Surge", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::MistySurge, name: "Misty Surge", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::PsychicSurge, name: "Psychic Surge", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::Guts, name: "Guts", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::MarvelScale, name: "Marvel Scale", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::Overcoat, name: "Overcoat", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::ThickFat, name: "Thick Fat", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::FlashFire, name: "Flash Fire", triggers: &[AbilityTrigger::Immunity] },
    AbilityData { id: AbilityId::VoltAbsorb, name: "Volt Absorb", triggers: &[AbilityTrigger::Immunity] },
    AbilityData { id: AbilityId::WaterAbsorb, name: "Water Absorb", triggers: &[AbilityTrigger::Immunity] },
    AbilityData { id: AbilityId::LightningRod, name: "Lightning Rod", triggers: &[AbilityTrigger::Immunity] },
    AbilityData { id: AbilityId::StormDrain, name: "Storm Drain", triggers: &[AbilityTrigger::Immunity] },
    AbilityData { id: AbilityId::ClearBody, name: "Clear Body", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::WhiteSmoke, name: "White Smoke", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::FullMetalBody, name: "Full Metal Body", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::SpeedBoost, name: "Speed Boost", triggers: &[AbilityTrigger::EndOfTurn] },
    AbilityData { id: AbilityId::BeastBoost, name: "Beast Boost", triggers: &[AbilityTrigger::OnDealingDamage] },
    AbilityData { id: AbilityId::Moxie, name: "Moxie", triggers: &[AbilityTrigger::OnDealingDamage] },
    AbilityData { id: AbilityId::TintedLens, name: "Tinted Lens", triggers: &[AbilityTrigger::ModifyDamage] },
    AbilityData { id: AbilityId::SwordOfRuin, name: "Sword of Ruin", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::TabletsOfRuin, name: "Tablets of Ruin", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::VesselOfRuin, name: "Vessel of Ruin", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::BeadsOfRuin, name: "Beads of Ruin", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::Pressure, name: "Pressure", triggers: &[AbilityTrigger::OnSwitchIn] },
    AbilityData { id: AbilityId::Prankster, name: "Prankster", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::SupremeOverlord, name: "Supreme Overlord", triggers: &[AbilityTrigger::Passive] },
    AbilityData { id: AbilityId::CursedBody, name: "Cursed Body", triggers: &[AbilityTrigger::OnBeingHit] },
    AbilityData { id: AbilityId::SkillLink, name: "Skill Link", triggers: &[AbilityTrigger::ModifyDamage] },
];

pub fn get_ability(id: AbilityId) -> &'static AbilityData {
    ABILITY_TABLE.iter().find(|a| a.id == id).unwrap_or(&ABILITY_TABLE[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ability() {
        let data = get_ability(AbilityId::Intimidate);
        assert_eq!(data.name, "Intimidate");
        assert!(data.triggers.contains(&AbilityTrigger::OnSwitchIn));
    }

    #[test]
    fn test_none_ability() {
        let data = get_ability(AbilityId::None);
        assert_eq!(data.name, "None");
        assert!(data.triggers.is_empty());
    }
}
