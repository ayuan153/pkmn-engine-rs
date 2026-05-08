import { PokemonSet } from '@pkmn/sim';

export interface BattleScenario {
  id: string;
  description: string;
  p1: PokemonSet[];
  p2: PokemonSet[];
  seed: [number, number, number, number];
  turns: string[][]; // [p1choice, p2choice] per turn
}

// Helper to create a set with defaults
function set(overrides: Partial<PokemonSet> & { species: string; moves: string[] }): PokemonSet {
  return {
    name: overrides.species,
    species: overrides.species,
    item: overrides.item || '',
    ability: overrides.ability || '',
    moves: overrides.moves,
    nature: overrides.nature || 'Serious',
    evs: overrides.evs || { hp: 0, atk: 0, def: 0, spa: 0, spd: 0, spe: 0 },
    ivs: overrides.ivs || { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 },
    level: overrides.level || 100,
    gender: overrides.gender || '',
    shiny: false,
    happiness: 255,
    pokeball: '',
    hpType: '',
    dynamaxLevel: 10,
    teraType: overrides.teraType || '',
    gigantamax: false,
  };
}

export const scenarios: BattleScenario[] = [
  {
    id: 'basic-physical',
    description: 'Basic physical attack, no modifiers',
    seed: [1, 2, 3, 4],
    p1: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: '', moves: ['Earthquake', 'Dragon Claw'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Tyranitar', ability: 'Sand Stream', item: '', moves: ['Stone Edge', 'Crunch'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 2']],
  },
  {
    id: 'choice-band',
    description: 'Choice Band 1.5x physical boost',
    seed: [10, 20, 30, 40],
    p1: [
      set({ species: 'Dragonite', ability: 'Multiscale', item: 'Choice Band', moves: ['Outrage', 'Earthquake', 'Extreme Speed', 'Ice Punch'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Blissey', ability: 'Natural Cure', item: 'Leftovers', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 3', 'move 1'], ['move 3', 'move 1']],
  },
  {
    id: 'weather-rain',
    description: 'Rain boosting Water, weakening Fire',
    seed: [5, 15, 25, 35],
    p1: [
      set({ species: 'Pelipper', ability: 'Drizzle', item: 'Damp Rock', moves: ['Surf', 'Hurricane', 'U-turn', 'Roost'], nature: 'Modest', evs: { hp: 248, atk: 0, def: 0, spa: 252, spd: 0, spe: 8 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Arcanine', ability: 'Intimidate', item: 'Heavy-Duty Boots', moves: ['Flare Blitz', 'Wild Charge', 'Close Combat', 'Extreme Speed'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 1', 'move 4']],
  },
  {
    id: 'terrain-electric',
    description: 'Electric Terrain 1.3x boost for grounded Electric moves',
    seed: [100, 200, 300, 400],
    p1: [
      set({ species: 'Tapu Koko', ability: 'Electric Surge', item: 'Life Orb', moves: ['Thunderbolt', 'Dazzling Gleam', 'U-turn', 'Roost'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Hippowdon', ability: 'Sand Stream', item: 'Leftovers', moves: ['Earthquake', 'Stone Edge', 'Slack Off', 'Stealth Rock'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 1']],
  },
  {
    id: 'knock-off-item',
    description: 'Knock Off 1.5x when target has item',
    seed: [7, 14, 21, 28],
    p1: [
      set({ species: 'Krookodile', ability: 'Intimidate', item: 'Choice Scarf', moves: ['Knock Off', 'Earthquake', 'Stone Edge', 'U-turn'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Slowbro', ability: 'Regenerator', item: 'Heavy-Duty Boots', moves: ['Scald', 'Psychic', 'Slack Off', 'Thunder Wave'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 1', 'move 1']],
  },
  {
    id: 'adaptability-stab',
    description: 'Adaptability gives 2x STAB instead of 1.5x',
    seed: [42, 84, 126, 168],
    p1: [
      set({ species: 'Crawdaunt', ability: 'Adaptability', item: 'Life Orb', moves: ['Crabhammer', 'Knock Off', 'Aqua Jet', 'Swords Dance'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Tangrowth', ability: 'Regenerator', item: 'Assault Vest', moves: ['Giga Drain', 'Knock Off', 'Earthquake', 'Rock Slide'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 2'], ['move 2', 'move 1']],
  },
  {
    id: 'screens-reflect',
    description: 'Reflect halves physical damage',
    seed: [11, 22, 33, 44],
    p1: [
      set({ species: 'Grimmsnarl', ability: 'Prankster', item: 'Light Clay', moves: ['Reflect', 'Light Screen', 'Spirit Break', 'Thunder Wave'], nature: 'Careful', evs: { hp: 252, atk: 0, def: 0, spa: 0, spd: 252, spe: 4 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Band', moves: ['Earthquake', 'Outrage', 'Stone Edge', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 3', 'move 1']],
  },
  {
    id: 'technician',
    description: 'Technician 1.5x on moves with BP <= 60',
    seed: [55, 66, 77, 88],
    p1: [
      set({ species: 'Scizor', ability: 'Technician', item: 'Choice Band', moves: ['Bullet Punch', 'U-turn', 'Knock Off', 'Superpower'], nature: 'Adamant', evs: { hp: 248, atk: 252, def: 0, spa: 0, spd: 8, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Rotom-Wash', ability: 'Levitate', item: 'Leftovers', moves: ['Volt Switch', 'Hydro Pump', 'Will-O-Wisp', 'Pain Split'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 3'], ['move 1', 'move 1']],
  },
  {
    id: 'burn-physical',
    description: 'Burn halves physical damage',
    seed: [99, 88, 77, 66],
    p1: [
      set({ species: 'Conkeldurr', ability: 'Guts', item: 'Flame Orb', moves: ['Facade', 'Drain Punch', 'Mach Punch', 'Knock Off'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Corviknight', ability: 'Pressure', item: 'Leftovers', moves: ['Body Press', 'Brave Bird', 'Roost', 'Defog'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 2'], ['move 2', 'move 1']],
  },
  {
    id: 'life-orb-special',
    description: 'Life Orb 1.3x boost + recoil',
    seed: [111, 222, 333, 444],
    p1: [
      set({ species: 'Gengar', ability: 'Cursed Body', item: 'Life Orb', moves: ['Shadow Ball', 'Sludge Bomb', 'Focus Blast', 'Thunderbolt'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Chansey', ability: 'Natural Cure', item: 'Eviolite', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Stealth Rock'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 2']],
  },
  // --- Variable/Special BP Moves ---
  {
    id: 'multi-hit-bullet-seed',
    description: 'Multi-hit move: Bullet Seed (2-5 hits, 25 BP each)',
    seed: [200, 201, 202, 203],
    p1: [
      set({ species: 'Breloom', ability: 'Technician', item: 'Choice Band', moves: ['Bullet Seed', 'Mach Punch', 'Spore', 'Swords Dance'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Swampert', ability: 'Torrent', item: 'Leftovers', moves: ['Earthquake', 'Ice Beam', 'Stealth Rock', 'Flip Turn'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 1']],
  },
  {
    id: 'acrobatics-no-item',
    description: 'Acrobatics doubles BP (55->110) when user has no item',
    seed: [210, 211, 212, 213],
    p1: [
      set({ species: 'Hawlucha', ability: 'Unburden', item: '', moves: ['Acrobatics', 'Close Combat', 'Swords Dance', 'Roost'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Toxapex', ability: 'Regenerator', item: 'Rocky Helmet', moves: ['Scald', 'Toxic', 'Recover', 'Haze'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 1']],
  },
  {
    id: 'weather-ball-sun',
    description: 'Weather Ball becomes Fire-type 100BP in Sun',
    seed: [220, 221, 222, 223],
    p1: [
      set({ species: 'Torkoal', ability: 'Drought', item: 'Charcoal', moves: ['Weather Ball', 'Lava Plume', 'Stealth Rock', 'Rapid Spin'], nature: 'Modest', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: 'Leftovers', moves: ['Power Whip', 'Knock Off', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 2']],
  },
  {
    id: 'stored-power-boosts',
    description: 'Stored Power: 20 + 20 per boost stage',
    seed: [230, 231, 232, 233],
    p1: [
      set({ species: 'Espathra', ability: 'Speed Boost', item: 'Leftovers', moves: ['Stored Power', 'Calm Mind', 'Roost', 'Dazzling Gleam'], nature: 'Timid', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Umbreon', ability: 'Synchronize', item: 'Leftovers', moves: ['Foul Play', 'Wish', 'Protect', 'Toxic'], nature: 'Calm', evs: { hp: 252, atk: 0, def: 4, spa: 0, spd: 252, spe: 0 }, level: 100 }),
    ],
    turns: [['move 2', 'move 4'], ['move 1', 'move 1'], ['move 1', 'move 1']],
  },
  // --- Offensive Abilities ---
  {
    id: 'huge-power',
    description: 'Huge Power doubles Attack stat',
    seed: [300, 301, 302, 303],
    p1: [
      set({ species: 'Azumarill', ability: 'Huge Power', item: 'Choice Band', moves: ['Aqua Jet', 'Play Rough', 'Liquidation', 'Knock Off'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Skeledirge', ability: 'Unaware', item: 'Heavy-Duty Boots', moves: ['Torch Song', 'Shadow Ball', 'Slack Off', 'Will-O-Wisp'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 3', 'move 1']],
  },
  {
    id: 'sheer-force',
    description: 'Sheer Force 1.3x on moves with secondary effects, no secondary triggers',
    seed: [310, 311, 312, 313],
    p1: [
      set({ species: 'Nidoking', ability: 'Sheer Force', item: 'Life Orb', moves: ['Earth Power', 'Sludge Wave', 'Ice Beam', 'Thunderbolt'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Blissey', ability: 'Natural Cure', item: 'Leftovers', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 2'], ['move 3', 'move 1']],
  },
  {
    id: 'iron-fist',
    description: 'Iron Fist 1.2x on punching moves',
    seed: [320, 321, 322, 323],
    p1: [
      set({ species: 'Melmetal', ability: 'Iron Fist', item: 'Assault Vest', moves: ['Double Iron Bash', 'Thunder Punch', 'Ice Punch', 'Earthquake'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Landorus-Therian', ability: 'Intimidate', item: 'Choice Scarf', moves: ['Earthquake', 'U-turn', 'Stone Edge', 'Knock Off'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    turns: [['move 2', 'move 1'], ['move 3', 'move 1']],
  },
  {
    id: 'strong-jaw',
    description: 'Strong Jaw 1.5x on biting moves',
    seed: [330, 331, 332, 333],
    p1: [
      set({ species: 'Dracovish', ability: 'Strong Jaw', item: 'Choice Band', moves: ['Fishious Rend', 'Crunch', 'Psychic Fangs', 'Ice Fang'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Corviknight', ability: 'Pressure', item: 'Leftovers', moves: ['Body Press', 'Brave Bird', 'Roost', 'Defog'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 2'], ['move 2', 'move 1']],
  },
  {
    id: 'tinted-lens',
    description: 'Tinted Lens doubles damage on resisted hits',
    seed: [340, 341, 342, 343],
    p1: [
      set({ species: 'Yanmega', ability: 'Tinted Lens', item: 'Choice Specs', moves: ['Bug Buzz', 'Air Slash', 'Psychic', 'Giga Drain'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Heatran', ability: 'Flash Fire', item: 'Leftovers', moves: ['Magma Storm', 'Earth Power', 'Flash Cannon', 'Stealth Rock'], nature: 'Modest', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 3'], ['move 2', 'move 2']],
  },
  // --- Defensive Abilities ---
  {
    id: 'ice-scales',
    description: 'Ice Scales halves special damage taken',
    seed: [400, 401, 402, 403],
    p1: [
      set({ species: 'Volcarona', ability: 'Flame Body', item: 'Heavy-Duty Boots', moves: ['Fire Blast', 'Bug Buzz', 'Psychic', 'Quiver Dance'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Frosmoth', ability: 'Ice Scales', item: 'Heavy-Duty Boots', moves: ['Ice Beam', 'Bug Buzz', 'Hurricane', 'Quiver Dance'], nature: 'Timid', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 3', 'move 2']],
  },
  {
    id: 'fur-coat',
    description: 'Fur Coat halves physical damage taken',
    seed: [410, 411, 412, 413],
    p1: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Band', moves: ['Earthquake', 'Outrage', 'Stone Edge', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Furfrou', ability: 'Fur Coat', item: 'Leftovers', moves: ['Return', 'Wild Charge', 'U-turn', 'Cotton Guard'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 3', 'move 2']],
  },
  {
    id: 'filter-solid-rock',
    description: 'Filter/Solid Rock reduces super effective damage by 25%',
    seed: [420, 421, 422, 423],
    p1: [
      set({ species: 'Weavile', ability: 'Pressure', item: 'Choice Band', moves: ['Triple Axel', 'Knock Off', 'Ice Shard', 'Low Kick'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Rhyperior', ability: 'Solid Rock', item: 'Leftovers', moves: ['Earthquake', 'Stone Edge', 'Megahorn', 'Stealth Rock'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 3', 'move 2']],
  },
  // Weather Interactions
  {
    id: 'weather-sun-fire',
    description: 'Sun boosts Fire 1.5x, weakens Water 0.5x',
    seed: [500, 501, 502, 503],
    p1: [
      set({ species: 'Charizard', ability: 'Blaze', item: 'Choice Specs', moves: ['Fire Blast', 'Air Slash', 'Focus Blast', 'Dragon Pulse'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
      set({ species: 'Torkoal', ability: 'Drought', item: 'Heat Rock', moves: ['Lava Plume', 'Stealth Rock', 'Rapid Spin', 'Yawn'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Vaporeon', ability: 'Water Absorb', item: 'Leftovers', moves: ['Scald', 'Ice Beam', 'Wish', 'Protect'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
      set({ species: 'Tyranitar', ability: 'Sand Stream', item: 'Leftovers', moves: ['Stone Edge', 'Crunch', 'Earthquake', 'Stealth Rock'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['switch 2', 'move 1'], ['move 1', 'move 2']],
  },
  {
    id: 'weather-sand-spd',
    description: 'Sandstorm 1.5x SpD for Rock-types',
    seed: [510, 511, 512, 513],
    p1: [
      set({ species: 'Gengar', ability: 'Cursed Body', item: 'Choice Specs', moves: ['Shadow Ball', 'Sludge Bomb', 'Focus Blast', 'Thunderbolt'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Tyranitar', ability: 'Sand Stream', item: 'Leftovers', moves: ['Stone Edge', 'Crunch', 'Earthquake', 'Stealth Rock'], nature: 'Careful', evs: { hp: 252, atk: 0, def: 4, spa: 0, spd: 252, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 2'], ['move 2', 'move 1']],
  },
  {
    id: 'weather-snow-def',
    description: 'Snow 1.5x Def for Ice-types',
    seed: [520, 521, 522, 523],
    p1: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Band', moves: ['Earthquake', 'Stone Edge', 'Outrage', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Abomasnow', ability: 'Snow Warning', item: 'Assault Vest', moves: ['Blizzard', 'Giga Drain', 'Ice Shard', 'Earthquake'], nature: 'Quiet', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 3']],
  },
  // Terrain Interactions
  {
    id: 'terrain-grassy',
    description: 'Grassy Terrain 1.3x Grass moves for grounded, halves EQ',
    seed: [600, 601, 602, 603],
    p1: [
      set({ species: 'Rillaboom', ability: 'Grassy Surge', item: 'Choice Band', moves: ['Grassy Glide', 'Wood Hammer', 'Knock Off', 'U-turn'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Toxapex', ability: 'Regenerator', item: 'Rocky Helmet', moves: ['Scald', 'Toxic', 'Recover', 'Knock Off'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 4']],
  },
  {
    id: 'terrain-psychic',
    description: 'Psychic Terrain 1.3x Psychic moves for grounded, blocks priority',
    seed: [610, 611, 612, 613],
    p1: [
      set({ species: 'Indeedee-F', ability: 'Psychic Surge', item: 'Choice Specs', moves: ['Psychic', 'Mystical Fire', 'Hyper Voice', 'Healing Wish'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Machamp', ability: 'Guts', item: 'Assault Vest', moves: ['Close Combat', 'Knock Off', 'Stone Edge', 'Bullet Punch'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 3', 'move 2']],
  },
  {
    id: 'terrain-electric-grounded',
    description: 'Electric Terrain 1.3x Electric moves for grounded mons only',
    seed: [620, 621, 622, 623],
    p1: [
      set({ species: 'Tapu Koko', ability: 'Electric Surge', item: 'Choice Specs', moves: ['Thunderbolt', 'Dazzling Gleam', 'Volt Switch', 'Grass Knot'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Swampert', ability: 'Torrent', item: 'Leftovers', moves: ['Earthquake', 'Ice Beam', 'Stealth Rock', 'Flip Turn'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 2']],
  },
  // Screens + Crit Interaction
  {
    id: 'light-screen',
    description: 'Light Screen halves special damage',
    seed: [700, 701, 702, 703],
    p1: [
      set({ species: 'Grimmsnarl', ability: 'Prankster', item: 'Light Clay', moves: ['Light Screen', 'Reflect', 'Spirit Break', 'Thunder Wave'], nature: 'Careful', evs: { hp: 252, atk: 0, def: 0, spa: 0, spd: 252, spe: 4 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Gengar', ability: 'Cursed Body', item: 'Choice Specs', moves: ['Shadow Ball', 'Sludge Bomb', 'Focus Blast', 'Thunderbolt'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 3', 'move 2']],
  },
  {
    id: 'crit-ignores-screens',
    description: 'Critical hits ignore Reflect/Light Screen',
    seed: [710, 711, 712, 713],
    p1: [
      set({ species: 'Kingambit', ability: 'Supreme Overlord', item: 'Choice Band', moves: ['Kowtow Cleave', 'Sucker Punch', 'Iron Head', 'Low Kick'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Grimmsnarl', ability: 'Prankster', item: 'Light Clay', moves: ['Reflect', 'Light Screen', 'Spirit Break', 'Thunder Wave'], nature: 'Careful', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 0, spe: 4 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 1', 'move 3'], ['move 1', 'move 3']],
  },
  // Boost Stacking
  {
    id: 'swords-dance-stacking',
    description: 'Swords Dance +2 Atk, stacks to +4',
    seed: [800, 801, 802, 803],
    p1: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Life Orb', moves: ['Swords Dance', 'Earthquake', 'Scale Shot', 'Stone Edge'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Skarmory', ability: 'Sturdy', item: 'Rocky Helmet', moves: ['Body Press', 'Iron Head', 'Roost', 'Whirlwind'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 3'], ['move 1', 'move 3'], ['move 2', 'move 1']],
  },
  {
    id: 'intimidate-drop',
    description: 'Intimidate -1 Atk on switch-in',
    seed: [810, 811, 812, 813],
    p1: [
      set({ species: 'Dragonite', ability: 'Multiscale', item: 'Choice Band', moves: ['Outrage', 'Extreme Speed', 'Earthquake', 'Fire Punch'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Landorus-Therian', ability: 'Intimidate', item: 'Rocky Helmet', moves: ['Earthquake', 'U-turn', 'Stone Edge', 'Stealth Rock'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 2', 'move 1'], ['move 2', 'move 1']],
  },
  {
    id: 'nasty-plot-special',
    description: 'Nasty Plot +2 SpA then attack',
    seed: [820, 821, 822, 823],
    p1: [
      set({ species: 'Togekiss', ability: 'Serene Grace', item: 'Leftovers', moves: ['Nasty Plot', 'Air Slash', 'Flamethrower', 'Dazzling Gleam'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Blissey', ability: 'Natural Cure', item: 'Leftovers', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 2'], ['move 4', 'move 1']],
  },
  // --- More Items ---
  {
    id: 'type-boosting-items',
    description: 'Type-boosting items give 1.2x (Mystic Water, Charcoal, etc.)',
    seed: [900, 901, 902, 903],
    p1: [
      set({ species: 'Starmie', ability: 'Natural Cure', item: 'Mystic Water', moves: ['Surf', 'Ice Beam', 'Thunderbolt', 'Psychic'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Snorlax', ability: 'Thick Fat', item: 'Leftovers', moves: ['Body Slam', 'Earthquake', 'Rest', 'Sleep Talk'], nature: 'Careful', evs: { hp: 252, atk: 0, def: 4, spa: 0, spd: 252, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 2'], ['move 4', 'move 1']],
  },
  {
    id: 'expert-belt',
    description: 'Expert Belt 1.2x on super effective hits only',
    seed: [910, 911, 912, 913],
    p1: [
      set({ species: 'Lucario', ability: 'Inner Focus', item: 'Expert Belt', moves: ['Close Combat', 'Meteor Mash', 'Ice Punch', 'Extreme Speed'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Tyranitar', ability: 'Sand Stream', item: 'Leftovers', moves: ['Stone Edge', 'Crunch', 'Earthquake', 'Fire Punch'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 4', 'move 3']],
  },
  {
    id: 'assault-vest',
    description: 'Assault Vest 1.5x SpD',
    seed: [920, 921, 922, 923],
    p1: [
      set({ species: 'Gengar', ability: 'Cursed Body', item: 'Choice Specs', moves: ['Shadow Ball', 'Sludge Bomb', 'Focus Blast', 'Thunderbolt'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Conkeldurr', ability: 'Guts', item: 'Assault Vest', moves: ['Drain Punch', 'Mach Punch', 'Knock Off', 'Ice Punch'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 3'], ['move 3', 'move 1']],
  },
  {
    id: 'eviolite',
    description: 'Eviolite 1.5x Def and SpD for NFE Pokemon',
    seed: [930, 931, 932, 933],
    p1: [
      set({ species: 'Alakazam', ability: 'Magic Guard', item: 'Life Orb', moves: ['Psychic', 'Shadow Ball', 'Focus Blast', 'Energy Ball'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Chansey', ability: 'Natural Cure', item: 'Eviolite', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Stealth Rock'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['move 2', 'move 2']],
  },
  // --- Terastallization ---
  {
    id: 'tera-stab-same-type',
    description: 'Tera into same type: STAB becomes 2x (was 1.5x)',
    seed: [1000, 1001, 1002, 1003],
    p1: [
      set({ species: 'Dragonite', ability: 'Multiscale', item: 'Choice Band', moves: ['Outrage', 'Extreme Speed', 'Earthquake', 'Fire Punch'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100, teraType: 'Dragon' }),
    ],
    p2: [
      set({ species: 'Clefable', ability: 'Magic Guard', item: 'Leftovers', moves: ['Moonblast', 'Flamethrower', 'Calm Mind', 'Soft-Boiled'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 3', 'move 1'], ['terastallize move 3', 'move 1']],
  },
  {
    id: 'tera-stab-new-type',
    description: 'Tera into new type: gains 1.5x STAB on new type moves',
    seed: [1010, 1011, 1012, 1013],
    p1: [
      set({ species: 'Dragonite', ability: 'Multiscale', item: 'Choice Band', moves: ['Extreme Speed', 'Outrage', 'Earthquake', 'Fire Punch'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100, teraType: 'Normal' }),
    ],
    p2: [
      set({ species: 'Slowbro', ability: 'Regenerator', item: 'Heavy-Duty Boots', moves: ['Scald', 'Psychic', 'Slack Off', 'Thunder Wave'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 1', 'move 1'], ['terastallize move 1', 'move 1']],
  },
  {
    id: 'tera-type-change',
    description: 'Tera changes defensive type (lose weakness, gain resistance)',
    seed: [1020, 1021, 1022, 1023],
    p1: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Life Orb', moves: ['Earthquake', 'Dragon Claw', 'Stone Edge', 'Swords Dance'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Gardevoir', ability: 'Trace', item: 'Choice Scarf', moves: ['Moonblast', 'Psychic', 'Thunderbolt', 'Mystical Fire'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100, teraType: 'Steel' }),
    ],
    turns: [['move 2', 'terastallize move 1'], ['move 2', 'move 1']],
  },
  // --- Status Interactions ---
  {
    id: 'burn-halves-physical',
    description: 'Burn halves physical damage (non-Guts)',
    seed: [1100, 1101, 1102, 1103],
    p1: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Band', moves: ['Earthquake', 'Outrage', 'Stone Edge', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Rotom-Wash', ability: 'Levitate', item: 'Leftovers', moves: ['Will-O-Wisp', 'Volt Switch', 'Hydro Pump', 'Pain Split'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 3', 'move 1'], ['move 3', 'move 3']],
  },
  {
    id: 'guts-burn-boost',
    description: 'Guts ignores burn penalty AND gets 1.5x Atk',
    seed: [1110, 1111, 1112, 1113],
    p1: [
      set({ species: 'Heracross', ability: 'Guts', item: 'Flame Orb', moves: ['Close Combat', 'Megahorn', 'Facade', 'Knock Off'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Slowbro', ability: 'Regenerator', item: 'Heavy-Duty Boots', moves: ['Scald', 'Psychic', 'Slack Off', 'Thunder Wave'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 3', 'move 4'], ['move 3', 'move 1'], ['move 1', 'move 2']],
  },
  {
    id: 'facade-doubled',
    description: 'Facade 140 BP when user has status',
    seed: [1120, 1121, 1122, 1123],
    p1: [
      set({ species: 'Ursaluna', ability: 'Guts', item: 'Flame Orb', moves: ['Facade', 'Headlong Rush', 'Crunch', 'Swords Dance'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Skarmory', ability: 'Sturdy', item: 'Rocky Helmet', moves: ['Body Press', 'Iron Head', 'Roost', 'Whirlwind'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    turns: [['move 4', 'move 3'], ['move 1', 'move 1'], ['move 1', 'move 2']],
  },
];
