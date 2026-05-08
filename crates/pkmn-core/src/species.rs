use crate::types::Type;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BaseStats {
    pub hp: u8,
    pub atk: u8,
    pub def: u8,
    pub spa: u8,
    pub spd: u8,
    pub spe: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct SpeciesData {
    pub id: u16,
    pub name: &'static str,
    pub types: [Type; 2],
    pub base_stats: BaseStats,
    pub weight_hg: u16, // weight in hectograms (0.1 kg units)
}

pub const SPECIES: &[SpeciesData] = &[
    // Garchomp
    SpeciesData { id: 445, name: "Garchomp", types: [Type::Dragon, Type::Ground], base_stats: BaseStats { hp: 108, atk: 130, def: 95, spa: 80, spd: 85, spe: 102 }, weight_hg: 950 },
    // Dragapult
    SpeciesData { id: 887, name: "Dragapult", types: [Type::Dragon, Type::Ghost], base_stats: BaseStats { hp: 88, atk: 120, def: 75, spa: 100, spd: 75, spe: 142 }, weight_hg: 500 },
    // Iron Valiant
    SpeciesData { id: 1006, name: "Iron Valiant", types: [Type::Fairy, Type::Fighting], base_stats: BaseStats { hp: 74, atk: 130, def: 90, spa: 120, spd: 60, spe: 116 }, weight_hg: 350 },
    // Great Tusk
    SpeciesData { id: 984, name: "Great Tusk", types: [Type::Ground, Type::Fighting], base_stats: BaseStats { hp: 115, atk: 131, def: 131, spa: 53, spd: 53, spe: 87 }, weight_hg: 3200 },
    // Kingambit
    SpeciesData { id: 983, name: "Kingambit", types: [Type::Dark, Type::Steel], base_stats: BaseStats { hp: 100, atk: 135, def: 120, spa: 60, spd: 85, spe: 50 }, weight_hg: 1200 },
    // Gholdengo
    SpeciesData { id: 982, name: "Gholdengo", types: [Type::Steel, Type::Ghost], base_stats: BaseStats { hp: 87, atk: 60, def: 95, spa: 133, spd: 91, spe: 84 }, weight_hg: 300 },
    // Heatran
    SpeciesData { id: 485, name: "Heatran", types: [Type::Fire, Type::Steel], base_stats: BaseStats { hp: 91, atk: 90, def: 106, spa: 130, spd: 106, spe: 77 }, weight_hg: 4300 },
    // Landorus-Therian
    SpeciesData { id: 645, name: "Landorus-Therian", types: [Type::Ground, Type::Flying], base_stats: BaseStats { hp: 89, atk: 145, def: 90, spa: 105, spd: 80, spe: 91 }, weight_hg: 680 },
    // Toxapex
    SpeciesData { id: 748, name: "Toxapex", types: [Type::Poison, Type::Water], base_stats: BaseStats { hp: 50, atk: 63, def: 152, spa: 53, spd: 142, spe: 35 }, weight_hg: 145 },
    // Corviknight
    SpeciesData { id: 823, name: "Corviknight", types: [Type::Flying, Type::Steel], base_stats: BaseStats { hp: 98, atk: 87, def: 105, spa: 53, spd: 85, spe: 67 }, weight_hg: 750 },
    // Skeledirge
    SpeciesData { id: 911, name: "Skeledirge", types: [Type::Fire, Type::Ghost], base_stats: BaseStats { hp: 104, atk: 75, def: 100, spa: 110, spd: 75, spe: 66 }, weight_hg: 3266 },
    // Ceruledge
    SpeciesData { id: 937, name: "Ceruledge", types: [Type::Fire, Type::Ghost], base_stats: BaseStats { hp: 75, atk: 125, def: 80, spa: 60, spd: 100, spe: 85 }, weight_hg: 620 },
    // Annihilape
    SpeciesData { id: 979, name: "Annihilape", types: [Type::Fighting, Type::Ghost], base_stats: BaseStats { hp: 110, atk: 115, def: 80, spa: 50, spd: 90, spe: 90 }, weight_hg: 560 },
    // Iron Moth
    SpeciesData { id: 994, name: "Iron Moth", types: [Type::Fire, Type::Poison], base_stats: BaseStats { hp: 80, atk: 70, def: 60, spa: 140, spd: 110, spe: 110 }, weight_hg: 360 },
    // Breloom
    SpeciesData { id: 286, name: "Breloom", types: [Type::Grass, Type::Fighting], base_stats: BaseStats { hp: 60, atk: 130, def: 80, spa: 60, spd: 60, spe: 70 }, weight_hg: 392 },
    // Volcarona
    SpeciesData { id: 637, name: "Volcarona", types: [Type::Bug, Type::Fire], base_stats: BaseStats { hp: 85, atk: 60, def: 65, spa: 135, spd: 105, spe: 100 }, weight_hg: 460 },
    // Dragonite
    SpeciesData { id: 149, name: "Dragonite", types: [Type::Dragon, Type::Flying], base_stats: BaseStats { hp: 91, atk: 134, def: 95, spa: 100, spd: 100, spe: 80 }, weight_hg: 2100 },
    // Tyranitar
    SpeciesData { id: 248, name: "Tyranitar", types: [Type::Rock, Type::Dark], base_stats: BaseStats { hp: 100, atk: 134, def: 110, spa: 95, spd: 100, spe: 61 }, weight_hg: 2020 },
    // Scizor
    SpeciesData { id: 212, name: "Scizor", types: [Type::Bug, Type::Steel], base_stats: BaseStats { hp: 70, atk: 130, def: 100, spa: 55, spd: 80, spe: 65 }, weight_hg: 1180 },
    // Ferrothorn
    SpeciesData { id: 598, name: "Ferrothorn", types: [Type::Grass, Type::Steel], base_stats: BaseStats { hp: 74, atk: 94, def: 131, spa: 54, spd: 116, spe: 20 }, weight_hg: 1100 },
    // Rotom-Wash
    SpeciesData { id: 479, name: "Rotom-Wash", types: [Type::Electric, Type::Water], base_stats: BaseStats { hp: 50, atk: 65, def: 107, spa: 105, spd: 107, spe: 86 }, weight_hg: 3 },
    // Clefable
    SpeciesData { id: 36, name: "Clefable", types: [Type::Fairy, Type::Fairy], base_stats: BaseStats { hp: 95, atk: 70, def: 73, spa: 95, spd: 90, spe: 60 }, weight_hg: 400 },
    // Slowking
    SpeciesData { id: 199, name: "Slowking", types: [Type::Water, Type::Psychic], base_stats: BaseStats { hp: 95, atk: 75, def: 80, spa: 100, spd: 110, spe: 30 }, weight_hg: 795 },
    // Blissey
    SpeciesData { id: 242, name: "Blissey", types: [Type::Normal, Type::Normal], base_stats: BaseStats { hp: 255, atk: 10, def: 10, spa: 75, spd: 135, spe: 55 }, weight_hg: 468 },
    // Dondozo
    SpeciesData { id: 977, name: "Dondozo", types: [Type::Water, Type::Water], base_stats: BaseStats { hp: 150, atk: 100, def: 115, spa: 65, spd: 65, spe: 35 }, weight_hg: 2200 },
    // Clodsire
    SpeciesData { id: 980, name: "Clodsire", types: [Type::Poison, Type::Ground], base_stats: BaseStats { hp: 130, atk: 75, def: 60, spa: 45, spd: 100, spe: 20 }, weight_hg: 2230 },
    // Glimmora
    SpeciesData { id: 970, name: "Glimmora", types: [Type::Rock, Type::Poison], base_stats: BaseStats { hp: 83, atk: 55, def: 90, spa: 130, spd: 81, spe: 86 }, weight_hg: 450 },
    // Ting-Lu
    SpeciesData { id: 1003, name: "Ting-Lu", types: [Type::Dark, Type::Ground], base_stats: BaseStats { hp: 155, atk: 110, def: 125, spa: 55, spd: 80, spe: 45 }, weight_hg: 6997 },
    // Chi-Yu
    SpeciesData { id: 1004, name: "Chi-Yu", types: [Type::Dark, Type::Fire], base_stats: BaseStats { hp: 55, atk: 80, def: 80, spa: 135, spd: 120, spe: 100 }, weight_hg: 48 },
    // Flutter Mane
    SpeciesData { id: 987, name: "Flutter Mane", types: [Type::Ghost, Type::Fairy], base_stats: BaseStats { hp: 55, atk: 55, def: 55, spa: 135, spd: 135, spe: 135 }, weight_hg: 40 },
    // Iron Bundle
    SpeciesData { id: 991, name: "Iron Bundle", types: [Type::Ice, Type::Water], base_stats: BaseStats { hp: 56, atk: 80, def: 114, spa: 124, spd: 60, spe: 136 }, weight_hg: 110 },
    // Roaring Moon
    SpeciesData { id: 1005, name: "Roaring Moon", types: [Type::Dragon, Type::Dark], base_stats: BaseStats { hp: 105, atk: 139, def: 71, spa: 55, spd: 101, spe: 119 }, weight_hg: 3800 },
    // Iron Hands
    SpeciesData { id: 992, name: "Iron Hands", types: [Type::Fighting, Type::Electric], base_stats: BaseStats { hp: 154, atk: 140, def: 108, spa: 50, spd: 68, spe: 50 }, weight_hg: 3860 },
    // Palafin (Hero form)
    SpeciesData { id: 964, name: "Palafin", types: [Type::Water, Type::Water], base_stats: BaseStats { hp: 100, atk: 160, def: 97, spa: 106, spd: 87, spe: 100 }, weight_hg: 601 },
    // Tinkaton
    SpeciesData { id: 959, name: "Tinkaton", types: [Type::Fairy, Type::Steel], base_stats: BaseStats { hp: 85, atk: 75, def: 77, spa: 70, spd: 105, spe: 94 }, weight_hg: 1129 },
    // Espathra
    SpeciesData { id: 956, name: "Espathra", types: [Type::Psychic, Type::Psychic], base_stats: BaseStats { hp: 95, atk: 60, def: 60, spa: 101, spd: 60, spe: 105 }, weight_hg: 900 },
    // Flamigo
    SpeciesData { id: 973, name: "Flamigo", types: [Type::Flying, Type::Fighting], base_stats: BaseStats { hp: 82, atk: 115, def: 74, spa: 75, spd: 64, spe: 90 }, weight_hg: 370 },
    // Kilowattrel
    SpeciesData { id: 940, name: "Kilowattrel", types: [Type::Electric, Type::Flying], base_stats: BaseStats { hp: 70, atk: 70, def: 60, spa: 105, spd: 60, spe: 125 }, weight_hg: 386 },
    // Grafaiai
    SpeciesData { id: 945, name: "Grafaiai", types: [Type::Poison, Type::Normal], base_stats: BaseStats { hp: 63, atk: 95, def: 65, spa: 80, spd: 72, spe: 110 }, weight_hg: 272 },
    // Arboliva
    SpeciesData { id: 950, name: "Arboliva", types: [Type::Grass, Type::Normal], base_stats: BaseStats { hp: 78, atk: 69, def: 90, spa: 125, spd: 109, spe: 39 }, weight_hg: 482 },
    // Cyclizar
    SpeciesData { id: 967, name: "Cyclizar", types: [Type::Dragon, Type::Normal], base_stats: BaseStats { hp: 70, atk: 95, def: 65, spa: 85, spd: 65, spe: 121 }, weight_hg: 630 },
    // Farigiraf
    SpeciesData { id: 981, name: "Farigiraf", types: [Type::Normal, Type::Psychic], base_stats: BaseStats { hp: 120, atk: 90, def: 70, spa: 110, spd: 70, spe: 60 }, weight_hg: 1600 },
    // Orthworm
    SpeciesData { id: 968, name: "Orthworm", types: [Type::Steel, Type::Steel], base_stats: BaseStats { hp: 70, atk: 85, def: 145, spa: 60, spd: 55, spe: 65 }, weight_hg: 3100 },
    // Garganacl
    SpeciesData { id: 955, name: "Garganacl", types: [Type::Rock, Type::Rock], base_stats: BaseStats { hp: 100, atk: 100, def: 130, spa: 45, spd: 90, spe: 35 }, weight_hg: 2400 },
    // Dachsbun
    SpeciesData { id: 927, name: "Dachsbun", types: [Type::Fairy, Type::Fairy], base_stats: BaseStats { hp: 57, atk: 80, def: 115, spa: 50, spd: 80, spe: 95 }, weight_hg: 149 },
    // Toedscruel
    SpeciesData { id: 949, name: "Toedscruel", types: [Type::Ground, Type::Grass], base_stats: BaseStats { hp: 80, atk: 70, def: 65, spa: 80, spd: 120, spe: 100 }, weight_hg: 580 },
    // Brambleghast
    SpeciesData { id: 947, name: "Brambleghast", types: [Type::Grass, Type::Ghost], base_stats: BaseStats { hp: 55, atk: 115, def: 70, spa: 80, spd: 70, spe: 90 }, weight_hg: 60 },
    // Rabsca
    SpeciesData { id: 954, name: "Rabsca", types: [Type::Bug, Type::Psychic], base_stats: BaseStats { hp: 75, atk: 50, def: 85, spa: 115, spd: 100, spe: 45 }, weight_hg: 35 },
    // Revavroom
    SpeciesData { id: 966, name: "Revavroom", types: [Type::Steel, Type::Poison], base_stats: BaseStats { hp: 80, atk: 119, def: 90, spa: 54, spd: 67, spe: 90 }, weight_hg: 1200 },
    // Maushold (Family of Four)
    SpeciesData { id: 925, name: "Maushold", types: [Type::Normal, Type::Normal], base_stats: BaseStats { hp: 74, atk: 75, def: 70, spa: 65, spd: 75, spe: 111 }, weight_hg: 23 },
];

pub fn get_species(name: &str) -> Option<&'static SpeciesData> {
    SPECIES.iter().find(|s| s.name.eq_ignore_ascii_case(name))
        .or_else(|| crate::gen::species_data::get_species_by_name(name))
        .or_else(|| {
            // Fallback: strip form suffix (e.g. "Vivillon-Continental" -> "Vivillon")
            name.find('-').and_then(|i| {
                let base = &name[..i];
                SPECIES.iter().find(|s| s.name.eq_ignore_ascii_case(base))
                    .or_else(|| crate::gen::species_data::get_species_by_name(base))
            })
        })
}

pub fn get_species_by_id(id: u16) -> Option<&'static SpeciesData> {
    SPECIES.iter().find(|s| s.id == id)
        .or_else(|| crate::gen::species_data::get_species_by_id(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_species_count() {
        assert_eq!(SPECIES.len(), 50);
    }

    #[test]
    fn test_get_garchomp() {
        let pokemon = get_species("Garchomp").unwrap();
        assert_eq!(pokemon.base_stats.hp, 108);
        assert_eq!(pokemon.base_stats.atk, 130);
        assert_eq!(pokemon.base_stats.def, 95);
        assert_eq!(pokemon.base_stats.spa, 80);
        assert_eq!(pokemon.base_stats.spd, 85);
        assert_eq!(pokemon.base_stats.spe, 102);
        assert_eq!(pokemon.types, [Type::Dragon, Type::Ground]);
    }

    #[test]
    fn test_get_species_case_insensitive() {
        assert!(get_species("garchomp").is_some());
        assert!(get_species("GARCHOMP").is_some());
    }

    #[test]
    fn test_blissey_hp() {
        let pokemon = get_species("Blissey").unwrap();
        assert_eq!(pokemon.base_stats.hp, 255);
    }

    #[test]
    fn test_dragapult_speed() {
        let pokemon = get_species("Dragapult").unwrap();
        assert_eq!(pokemon.base_stats.spe, 142);
    }

    #[test]
    fn test_nonexistent_species() {
        assert!(get_species("MissingNo").is_none());
    }

    #[test]
    fn test_iron_valiant_types() {
        let pokemon = get_species("Iron Valiant").unwrap();
        assert_eq!(pokemon.types, [Type::Fairy, Type::Fighting]);
    }

    #[test]
    fn test_toxapex_defenses() {
        let pokemon = get_species("Toxapex").unwrap();
        assert_eq!(pokemon.base_stats.def, 152);
        assert_eq!(pokemon.base_stats.spd, 142);
    }
}
