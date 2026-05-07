/// Calculate HP stat at a given level.
/// Uses the standard Gen 3+ formula. Shedinja (base 1) always returns 1.
pub fn calc_hp(base: u8, iv: u8, ev: u8, level: u8) -> u16 {
    if base == 1 {
        return 1; // Shedinja
    }
    let base = base as u32;
    let iv = iv as u32;
    let ev = ev as u32;
    let level = level as u32;
    ((2 * base + iv + ev / 4) * level / 100 + level + 10) as u16
}

/// Calculate a non-HP stat at a given level with nature modifier.
/// nature_mod should be 0.9, 1.0, or 1.1.
pub fn calc_stat(base: u8, iv: u8, ev: u8, level: u8, nature_mod: f32) -> u16 {
    let base = base as u32;
    let iv = iv as u32;
    let ev = ev as u32;
    let level = level as u32;
    let raw = (2 * base + iv + ev / 4) * level / 100 + 5;
    (raw as f32 * nature_mod) as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_garchomp_hp() {
        // Garchomp: base HP 108, 31 IV, 0 EV, level 100
        let hp = calc_hp(108, 31, 0, 100);
        assert_eq!(hp, 357);
    }

    #[test]
    fn test_garchomp_hp_max() {
        // Garchomp: base HP 108, 31 IV, 252 EV, level 100
        let hp = calc_hp(108, 31, 252, 100);
        assert_eq!(hp, 420);
    }

    #[test]
    fn test_garchomp_atk_adamant() {
        // Garchomp: base Atk 130, 31 IV, 252 EV, level 100, Adamant (+Atk)
        let atk = calc_stat(130, 31, 252, 100, 1.1);
        assert_eq!(atk, 394);
    }

    #[test]
    fn test_shedinja_hp() {
        assert_eq!(calc_hp(1, 31, 252, 100), 1);
    }

    #[test]
    fn test_blissey_hp() {
        // Blissey: base HP 255, 31 IV, 252 EV, level 100
        let hp = calc_hp(255, 31, 252, 100);
        assert_eq!(hp, 714);
    }

    #[test]
    fn test_stat_neutral() {
        // Base 100, 31 IV, 252 EV, level 100, neutral nature
        let stat = calc_stat(100, 31, 252, 100, 1.0);
        assert_eq!(stat, 299);
    }
}
