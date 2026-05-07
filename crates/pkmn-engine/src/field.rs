#[derive(Debug, Clone, Copy, Default)]
pub struct Field {
    pub weather: Weather,
    pub weather_turns: u8,
    pub terrain: Terrain,
    pub terrain_turns: u8,
    pub trick_room: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Weather {
    #[default]
    None,
    Sun,
    Rain,
    Sand,
    Snow,
    HarshSun,
    HeavyRain,
    StrongWinds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Terrain {
    #[default]
    None,
    Electric,
    Grassy,
    Misty,
    Psychic,
}
