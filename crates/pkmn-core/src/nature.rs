use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Nature {
    Hardy,
    Lonely,
    Brave,
    Adamant,
    Naughty,
    Bold,
    Docile,
    Relaxed,
    Impish,
    Lax,
    Timid,
    Hasty,
    Serious,
    Jolly,
    Naive,
    Modest,
    Mild,
    Quiet,
    Bashful,
    Rash,
    Calm,
    Gentle,
    Sassy,
    Careful,
    Quirky,
}

impl Nature {
    /// Returns (atk_mod, def_mod, spa_mod, spd_mod, spe_mod).
    /// Each value is 0.9, 1.0, or 1.1.
    pub fn modifiers(self) -> (f32, f32, f32, f32, f32) {
        match self {
            // Neutral natures
            Nature::Hardy | Nature::Docile | Nature::Serious | Nature::Bashful | Nature::Quirky => {
                (1.0, 1.0, 1.0, 1.0, 1.0)
            }
            // +Atk
            Nature::Lonely => (1.1, 0.9, 1.0, 1.0, 1.0),
            Nature::Brave => (1.1, 1.0, 1.0, 1.0, 0.9),
            Nature::Adamant => (1.1, 1.0, 0.9, 1.0, 1.0),
            Nature::Naughty => (1.1, 1.0, 1.0, 0.9, 1.0),
            // +Def
            Nature::Bold => (0.9, 1.1, 1.0, 1.0, 1.0),
            Nature::Relaxed => (1.0, 1.1, 1.0, 1.0, 0.9),
            Nature::Impish => (1.0, 1.1, 0.9, 1.0, 1.0),
            Nature::Lax => (1.0, 1.1, 1.0, 0.9, 1.0),
            // +Spe
            Nature::Timid => (0.9, 1.0, 1.0, 1.0, 1.1),
            Nature::Hasty => (1.0, 0.9, 1.0, 1.0, 1.1),
            Nature::Jolly => (1.0, 1.0, 0.9, 1.0, 1.1),
            Nature::Naive => (1.0, 1.0, 1.0, 0.9, 1.1),
            // +SpA
            Nature::Modest => (0.9, 1.0, 1.1, 1.0, 1.0),
            Nature::Mild => (1.0, 0.9, 1.1, 1.0, 1.0),
            Nature::Quiet => (1.0, 1.0, 1.1, 1.0, 0.9),
            Nature::Rash => (1.0, 1.0, 1.1, 0.9, 1.0),
            // +SpD
            Nature::Calm => (0.9, 1.0, 1.0, 1.1, 1.0),
            Nature::Gentle => (1.0, 0.9, 1.0, 1.1, 1.0),
            Nature::Sassy => (1.0, 1.0, 1.0, 1.1, 0.9),
            Nature::Careful => (1.0, 1.0, 0.9, 1.1, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adamant() {
        let (atk, def, spa, spd, spe) = Nature::Adamant.modifiers();
        assert_eq!(atk, 1.1);
        assert_eq!(def, 1.0);
        assert_eq!(spa, 0.9);
        assert_eq!(spd, 1.0);
        assert_eq!(spe, 1.0);
    }

    #[test]
    fn test_modest() {
        let (atk, _, spa, _, _) = Nature::Modest.modifiers();
        assert_eq!(atk, 0.9);
        assert_eq!(spa, 1.1);
    }

    #[test]
    fn test_neutral() {
        let mods = Nature::Hardy.modifiers();
        assert_eq!(mods, (1.0, 1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn test_jolly() {
        let (atk, def, spa, spd, spe) = Nature::Jolly.modifiers();
        assert_eq!(atk, 1.0);
        assert_eq!(def, 1.0);
        assert_eq!(spa, 0.9);
        assert_eq!(spd, 1.0);
        assert_eq!(spe, 1.1);
    }
}
