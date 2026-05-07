use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Type {
    Normal = 0,
    Fire,
    Water,
    Electric,
    Grass,
    Ice,
    Fighting,
    Poison,
    Ground,
    Flying,
    Psychic,
    Bug,
    Rock,
    Ghost,
    Dragon,
    Dark,
    Steel,
    Fairy,
}

impl Type {
    /// Get type effectiveness multiplier for an attacking type vs one or more defending types.
    /// Returns 0.0, 0.25, 0.5, 1.0, 2.0, or 4.0.
    pub fn effectiveness(attacking: Type, defending: &[Type]) -> f32 {
        defending
            .iter()
            .map(|d| single_effectiveness(attacking, *d))
            .product()
    }
}

fn single_effectiveness(attacking: Type, defending: Type) -> f32 {
    match TYPE_CHART[attacking as usize][defending as usize] {
        0 => 0.0,
        1 => 0.5,
        2 => 1.0,
        3 => 2.0,
        _ => 1.0,
    }
}

/// Full Gen 6+ type chart (18x18).
/// Rows = attacking type, Cols = defending type.
/// Encoding: 0=immune, 1=0.5x (not very effective), 2=1x (neutral), 3=2x (super effective)
/// Order: Normal, Fire, Water, Electric, Grass, Ice, Fighting, Poison, Ground, Flying, Psychic, Bug, Rock, Ghost, Dragon, Dark, Steel, Fairy
const TYPE_CHART: [[u8; 18]; 18] = [
    //           Nor Fir Wat Ele Gra Ice Fig Poi Gro Fly Psy Bug Roc Gho Dra Dar Ste Fai
    /* Normal  */[2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  1,  0,  2,  2,  1,  2],
    /* Fire    */[2,  1,  1,  2,  3,  3,  2,  2,  2,  2,  2,  3,  1,  2,  1,  2,  3,  2],
    /* Water   */[2,  3,  1,  2,  1,  2,  2,  2,  3,  2,  2,  2,  3,  2,  1,  2,  2,  2],
    /* Electric*/[2,  2,  3,  1,  1,  2,  2,  2,  0,  3,  2,  2,  2,  2,  1,  2,  2,  2],
    /* Grass   */[2,  1,  3,  2,  1,  2,  2,  1,  3,  1,  2,  1,  3,  2,  1,  2,  1,  2],
    /* Ice     */[2,  1,  1,  2,  3,  1,  2,  2,  3,  3,  2,  2,  2,  2,  3,  2,  1,  2],
    /* Fighting*/[3,  2,  2,  2,  2,  3,  2,  1,  2,  1,  1,  1,  3,  0,  2,  3,  3,  1],
    /* Poison  */[2,  2,  2,  2,  3,  2,  2,  1,  1,  2,  2,  2,  1,  1,  2,  2,  0,  3],
    /* Ground  */[2,  3,  2,  3,  1,  2,  2,  3,  2,  0,  2,  1,  3,  2,  2,  2,  3,  2],
    /* Flying  */[2,  2,  2,  1,  3,  2,  3,  2,  2,  2,  2,  3,  1,  2,  2,  2,  1,  2],
    /* Psychic */[2,  2,  2,  2,  2,  2,  3,  3,  2,  2,  1,  2,  2,  2,  2,  0,  1,  2],
    /* Bug     */[2,  1,  2,  2,  3,  2,  1,  1,  2,  1,  3,  2,  2,  1,  2,  3,  1,  1],
    /* Rock    */[2,  3,  2,  2,  2,  3,  1,  2,  1,  3,  2,  3,  2,  2,  2,  2,  1,  2],
    /* Ghost   */[0,  2,  2,  2,  2,  2,  2,  2,  2,  2,  3,  2,  2,  3,  2,  1,  2,  2],
    /* Dragon  */[2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  2,  3,  2,  1,  0],
    /* Dark    */[2,  2,  2,  2,  2,  2,  1,  2,  2,  2,  3,  2,  2,  3,  2,  1,  2,  1],
    /* Steel   */[2,  1,  1,  1,  2,  3,  2,  2,  2,  2,  2,  2,  3,  2,  2,  2,  1,  3],
    /* Fairy   */[2,  1,  2,  2,  2,  2,  3,  1,  2,  2,  2,  2,  2,  2,  3,  3,  1,  2],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fire_vs_grass() {
        assert_eq!(Type::effectiveness(Type::Fire, &[Type::Grass]), 2.0);
    }

    #[test]
    fn test_normal_vs_ghost() {
        assert_eq!(Type::effectiveness(Type::Normal, &[Type::Ghost]), 0.0);
    }

    #[test]
    fn test_fire_vs_grass_steel() {
        assert_eq!(Type::effectiveness(Type::Fire, &[Type::Grass, Type::Steel]), 4.0);
    }

    #[test]
    fn test_ground_vs_flying() {
        assert_eq!(Type::effectiveness(Type::Ground, &[Type::Flying]), 0.0);
    }

    #[test]
    fn test_electric_vs_ground() {
        assert_eq!(Type::effectiveness(Type::Electric, &[Type::Ground]), 0.0);
    }

    #[test]
    fn test_fighting_vs_dark_steel() {
        assert_eq!(Type::effectiveness(Type::Fighting, &[Type::Dark, Type::Steel]), 4.0);
    }

    #[test]
    fn test_dragon_vs_fairy() {
        assert_eq!(Type::effectiveness(Type::Dragon, &[Type::Fairy]), 0.0);
    }

    #[test]
    fn test_fairy_vs_dragon() {
        assert_eq!(Type::effectiveness(Type::Fairy, &[Type::Dragon]), 2.0);
    }

    #[test]
    fn test_neutral_matchup() {
        assert_eq!(Type::effectiveness(Type::Fire, &[Type::Normal]), 1.0);
    }

    #[test]
    fn test_ice_vs_dragon_flying() {
        assert_eq!(Type::effectiveness(Type::Ice, &[Type::Dragon, Type::Flying]), 4.0);
    }
}
