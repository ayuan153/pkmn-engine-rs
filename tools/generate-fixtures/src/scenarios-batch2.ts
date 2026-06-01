import { PokemonSet } from '@pkmn/sim';
import { BattleScenario } from './teams.js';

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

export const batch2Scenarios: BattleScenario[] = [
  // === EFFECTIVENESS COMBOS ===
  {
    id: 'eff-super-effective-4x',
    description: '4x super effective (Ice vs Dragon/Ground)',
    seed: [3000, 3001, 3002, 3003],
    p1: [set({ species: 'Weavile', ability: 'Pressure', item: '', moves: ['Ice Punch', 'Knock Off', 'Ice Shard', 'Low Kick'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Garchomp', ability: 'Rough Skin', item: '', moves: ['Earthquake', 'Dragon Claw', 'Stone Edge', 'Swords Dance'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'eff-double-resist',
    description: '4x resisted (Normal vs Rock/Steel)',
    seed: [3010, 3011, 3012, 3013],
    p1: [set({ species: 'Snorlax', ability: 'Thick Fat', item: '', moves: ['Body Slam', 'Earthquake', 'Rest', 'Sleep Talk'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 } })],
    p2: [set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: '', moves: ['Power Whip', 'Knock Off', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'eff-non-stab-super',
    description: 'Non-STAB super effective (Fighting Lucario vs Tyranitar)',
    seed: [3020, 3021, 3022, 3023],
    p1: [set({ species: 'Lucario', ability: 'Inner Focus', item: '', moves: ['Close Combat', 'Meteor Mash', 'Extreme Speed', 'Swords Dance'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Tyranitar', ability: 'Sand Stream', item: '', moves: ['Stone Edge', 'Crunch', 'Earthquake', 'Fire Punch'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 2']],
  },
  {
    id: 'eff-stab-super-effective',
    description: 'STAB + super effective (1.5x * 2x = 3x)',
    seed: [3030, 3031, 3032, 3033],
    p1: [set({ species: 'Garchomp', ability: 'Rough Skin', item: '', moves: ['Earthquake', 'Dragon Claw', 'Stone Edge', 'Swords Dance'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Heatran', ability: 'Flash Fire', item: '', moves: ['Magma Storm', 'Earth Power', 'Flash Cannon', 'Stealth Rock'], nature: 'Modest', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 3']],
  },
  // === WEATHER ===
  {
    id: 'weather-rain-water-stab',
    description: 'Rain + Water STAB (1.5x weather * 1.5x STAB)',
    seed: [3100, 3101, 3102, 3103],
    p1: [set({ species: 'Pelipper', ability: 'Drizzle', item: '', moves: ['Surf', 'Hurricane', 'U-turn', 'Roost'], nature: 'Modest', evs: { hp: 248, atk: 0, def: 0, spa: 252, spd: 0, spe: 8 } })],
    p2: [set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: '', moves: ['Power Whip', 'Knock Off', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'weather-sun-fire-stab',
    description: 'Sun + Fire STAB move',
    seed: [3110, 3111, 3112, 3113],
    p1: [set({ species: 'Torkoal', ability: 'Drought', item: '', moves: ['Flamethrower', 'Earth Power', 'Stealth Rock', 'Rapid Spin'], nature: 'Modest', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 } })],
    p2: [set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: '', moves: ['Power Whip', 'Knock Off', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'weather-sand-rock-spd',
    description: 'Sand boosts Rock-type SpD by 1.5x',
    seed: [3120, 3121, 3122, 3123],
    p1: [set({ species: 'Alakazam', ability: 'Magic Guard', item: '', moves: ['Psychic', 'Shadow Ball', 'Focus Blast', 'Energy Ball'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Tyranitar', ability: 'Sand Stream', item: '', moves: ['Stone Edge', 'Crunch', 'Earthquake', 'Stealth Rock'], nature: 'Careful', evs: { hp: 252, atk: 0, def: 4, spa: 0, spd: 252, spe: 0 } })],
    turns: [['move 2', 'move 2']],
  },
  // === TERRAIN ===
  {
    id: 'terrain-electric-stab',
    description: 'Electric Terrain + Electric STAB grounded',
    seed: [3200, 3201, 3202, 3203],
    p1: [set({ species: 'Jolteon', ability: 'Volt Absorb', item: '', moves: ['Thunderbolt', 'Shadow Ball', 'Volt Switch', 'Yawn'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [
      set({ species: 'Tapu Koko', ability: 'Electric Surge', item: '', moves: ['Thunderbolt', 'Dazzling Gleam', 'U-turn', 'Roost'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } }),
      set({ species: 'Swampert', ability: 'Torrent', item: '', moves: ['Earthquake', 'Ice Beam', 'Stealth Rock', 'Flip Turn'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } }),
    ],
    turns: [['move 1', 'switch 2'], ['move 1', 'move 1']],
  },
  {
    id: 'terrain-grassy-eq-halved',
    description: 'Grassy Terrain halves Earthquake damage',
    seed: [3210, 3211, 3212, 3213],
    p1: [set({ species: 'Rillaboom', ability: 'Grassy Surge', item: '', moves: ['Earthquake', 'Wood Hammer', 'Knock Off', 'U-turn'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Toxapex', ability: 'Regenerator', item: '', moves: ['Scald', 'Toxic', 'Recover', 'Knock Off'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'terrain-psychic-stab',
    description: 'Psychic Terrain + Psychic STAB grounded',
    seed: [3220, 3221, 3222, 3223],
    p1: [set({ species: 'Tapu Lele', ability: 'Psychic Surge', item: '', moves: ['Psychic', 'Moonblast', 'Focus Blast', 'Psyshock'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Conkeldurr', ability: 'Guts', item: '', moves: ['Drain Punch', 'Mach Punch', 'Knock Off', 'Ice Punch'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
];
