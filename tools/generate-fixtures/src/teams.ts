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
];
