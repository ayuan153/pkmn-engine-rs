/// Context for calculating damage in the Gen 9 formula.
#[derive(Debug, Clone, Copy)]
pub struct DamageContext {
    pub attacker_level: u8,
    pub attacker_stat: u16,
    pub defender_stat: u16,
    pub base_power: u16,
    pub stab: bool,
    pub type_effectiveness: f32,
    pub critical: bool,
    pub weather_boost: f32,
    pub other_modifiers: f32,
    pub random_factor: u8, // 85-100
}

/// Gen 9 damage formula.
/// damage = ((2*level/5 + 2) * power * atk/def) / 50 + 2) * modifiers
/// Modifiers applied as: floor(damage * modifier / 4096) per PS convention.
pub fn calculate_damage(ctx: &DamageContext) -> u16 {
    let level = ctx.attacker_level as u32;
    let power = ctx.base_power as u32;
    let atk = ctx.attacker_stat as u32;
    let def = ctx.defender_stat as u32;

    // Base damage
    let mut damage = ((2 * level / 5 + 2) * power * atk / def) / 50 + 2;

    // Weather: convert f32 to 4096-based
    let weather_mod = (ctx.weather_boost * 4096.0) as u32;
    damage = damage * weather_mod / 4096;

    // Critical hit: 6144/4096 = 1.5x
    if ctx.critical {
        damage = damage * 6144 / 4096;
    }

    // Random factor (85-100)
    damage = damage * ctx.random_factor as u32 / 100;

    // STAB: 6144/4096 = 1.5x
    if ctx.stab {
        damage = damage * 6144 / 4096;
    }

    // Type effectiveness: convert to integer (2.0 -> 8192/4096, 4.0 -> 16384/4096, 0.5 -> 2048/4096)
    let type_mod = (ctx.type_effectiveness * 4096.0) as u32;
    damage = damage * type_mod / 4096;

    // Other modifiers: convert f32 to 4096-based
    let other_mod = (ctx.other_modifiers * 4096.0) as u32;
    damage = damage * other_mod / 4096;

    damage.max(1) as u16
}

/// Calculate all 16 damage rolls (random factor 85-100).
pub fn damage_roll(ctx: &DamageContext) -> [u16; 16] {
    let mut rolls = [0u16; 16];
    for i in 0..16 {
        let mut c = *ctx;
        c.random_factor = 85 + i as u8;
        rolls[i as usize] = calculate_damage(&c);
    }
    rolls
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ctx() -> DamageContext {
        DamageContext {
            attacker_level: 100,
            attacker_stat: 394, // Garchomp Adamant max Atk
            defender_stat: 299, // Neutral base 100 max Def
            base_power: 100,    // Earthquake
            stab: true,
            type_effectiveness: 1.0,
            critical: false,
            weather_boost: 1.0,
            other_modifiers: 1.0,
            random_factor: 100,
        }
    }

    #[test]
    fn test_basic_damage() {
        let ctx = base_ctx();
        let dmg = calculate_damage(&ctx);
        assert!(dmg > 0);
    }

    #[test]
    fn test_minimum_damage() {
        let ctx = DamageContext {
            attacker_level: 1,
            attacker_stat: 1,
            defender_stat: 500,
            base_power: 1,
            stab: false,
            type_effectiveness: 1.0,
            critical: false,
            weather_boost: 1.0,
            other_modifiers: 1.0,
            random_factor: 85,
        };
        assert_eq!(calculate_damage(&ctx), 1);
    }

    #[test]
    fn test_critical_increases_damage() {
        let mut ctx = base_ctx();
        let normal = calculate_damage(&ctx);
        ctx.critical = true;
        let crit = calculate_damage(&ctx);
        assert!(crit > normal);
    }

    #[test]
    fn test_stab_increases_damage() {
        let mut ctx = base_ctx();
        ctx.stab = false;
        let no_stab = calculate_damage(&ctx);
        ctx.stab = true;
        let with_stab = calculate_damage(&ctx);
        assert!(with_stab > no_stab);
    }

    #[test]
    fn test_super_effective() {
        let mut ctx = base_ctx();
        ctx.type_effectiveness = 2.0;
        let se = calculate_damage(&ctx);
        ctx.type_effectiveness = 1.0;
        let neutral = calculate_damage(&ctx);
        assert!(se > neutral);
    }

    #[test]
    fn test_damage_roll_16_values() {
        let ctx = base_ctx();
        let rolls = damage_roll(&ctx);
        // Rolls should be ascending
        for i in 1..16 {
            assert!(rolls[i] >= rolls[i - 1]);
        }
        // Min roll < max roll
        assert!(rolls[0] < rolls[15]);
    }

    #[test]
    fn test_immune_does_zero_but_min_1() {
        let mut ctx = base_ctx();
        ctx.type_effectiveness = 0.0;
        // With 0.0 effectiveness, the cast to u32 gives 0, but max(1) applies
        // Actually (damage as f32 * 0.0) = 0, then subsequent multiplications stay 0
        // max(1) at the end ensures minimum 1
        let dmg = calculate_damage(&ctx);
        assert_eq!(dmg, 1);
    }
}
