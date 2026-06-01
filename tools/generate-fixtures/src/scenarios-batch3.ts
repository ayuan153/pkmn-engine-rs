import { PokemonSet } from '@pkmn/sim';
import { BattleScenario } from './teams.js';

function set(overrides: Partial<PokemonSet> & { species: string; moves: string[] }): PokemonSet {
  return {
    name: overrides.species, species: overrides.species,
    item: overrides.item || '', ability: overrides.ability || '',
    moves: overrides.moves, nature: overrides.nature || 'Serious',
    evs: overrides.evs || { hp: 0, atk: 0, def: 0, spa: 0, spd: 0, spe: 0 },
    ivs: overrides.ivs || { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 },
    level: overrides.level || 100, gender: overrides.gender || '',
    shiny: false, happiness: 255, pokeball: '', hpType: '',
    dynamaxLevel: 10, teraType: overrides.teraType || '', gigantamax: false,
  };
}

export const batch3Scenarios: BattleScenario[] = [
  // === VARIABLE BP ===
  {
    id: 'var-bp-knock-off-item',
    description: 'Knock Off 97 BP when target has item',
    seed: [3300, 3301, 3302, 3303],
    p1: [set({ species: 'Weavile', ability: 'Pressure', item: '', moves: ['Knock Off', 'Ice Punch', 'Ice Shard', 'Low Kick'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Slowbro', ability: 'Regenerator', item: 'Leftovers', moves: ['Scald', 'Psychic', 'Slack Off', 'Thunder Wave'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'var-bp-acrobatics-no-item',
    description: 'Acrobatics 110 BP with no item',
    seed: [3310, 3311, 3312, 3313],
    p1: [set({ species: 'Hawlucha', ability: 'Unburden', item: '', moves: ['Acrobatics', 'Close Combat', 'Swords Dance', 'Roost'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Conkeldurr', ability: 'Guts', item: '', moves: ['Drain Punch', 'Mach Punch', 'Knock Off', 'Ice Punch'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'var-bp-facade-burn',
    description: 'Facade 140 BP when burned (Guts negates burn penalty)',
    seed: [3320, 3321, 3322, 3323],
    p1: [set({ species: 'Conkeldurr', ability: 'Guts', item: 'Flame Orb', moves: ['Facade', 'Drain Punch', 'Mach Punch', 'Knock Off'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 } })],
    p2: [set({ species: 'Slowbro', ability: 'Regenerator', item: '', moves: ['Scald', 'Psychic', 'Slack Off', 'Thunder Wave'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 2', 'move 4'], ['move 1', 'move 1']],
  },
  {
    id: 'var-bp-gyro-ball',
    description: 'Gyro Ball: BP = 25 * target_spe / user_spe (capped 150)',
    seed: [3330, 3331, 3332, 3333],
    p1: [set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: '', moves: ['Gyro Ball', 'Power Whip', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, ivs: { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 0 } })],
    p2: [set({ species: 'Dragapult', ability: 'Infiltrator', item: '', moves: ['Shadow Ball', 'Draco Meteor', 'Fire Blast', 'Thunderbolt'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'var-bp-weather-ball-rain',
    description: 'Weather Ball in Rain becomes Water 100 BP',
    seed: [3340, 3341, 3342, 3343],
    p1: [set({ species: 'Pelipper', ability: 'Drizzle', item: '', moves: ['Weather Ball', 'Hurricane', 'U-turn', 'Roost'], nature: 'Modest', evs: { hp: 248, atk: 0, def: 0, spa: 252, spd: 0, spe: 8 } })],
    p2: [set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: '', moves: ['Power Whip', 'Knock Off', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === OFFENSIVE ABILITIES ===
  {
    id: 'ability-adaptability',
    description: 'Adaptability: STAB becomes 2x',
    seed: [3400, 3401, 3402, 3403],
    p1: [set({ species: 'Porygon-Z', ability: 'Adaptability', item: '', moves: ['Hyper Beam', 'Thunderbolt', 'Ice Beam', 'Shadow Ball'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 2', 'move 2']],
  },
  {
    id: 'ability-technician-bp60',
    description: 'Technician on exactly 60 BP move (Bullet Punch)',
    seed: [3410, 3411, 3412, 3413],
    p1: [set({ species: 'Scizor', ability: 'Technician', item: '', moves: ['Bullet Punch', 'U-turn', 'Knock Off', 'Superpower'], nature: 'Adamant', evs: { hp: 248, atk: 252, def: 0, spa: 0, spd: 8, spe: 0 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'ability-huge-pure-power',
    description: 'Huge Power doubles Atk (Azumarill)',
    seed: [3420, 3421, 3422, 3423],
    p1: [set({ species: 'Azumarill', ability: 'Huge Power', item: '', moves: ['Aqua Jet', 'Play Rough', 'Liquidation', 'Knock Off'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 3', 'move 1']],
  },
  {
    id: 'ability-tinted-lens',
    description: 'Tinted Lens doubles resisted damage',
    seed: [3430, 3431, 3432, 3433],
    p1: [set({ species: 'Yanmega', ability: 'Tinted Lens', item: '', moves: ['Bug Buzz', 'Air Slash', 'Psychic', 'Giga Drain'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Heatran', ability: 'Flash Fire', item: '', moves: ['Magma Storm', 'Earth Power', 'Flash Cannon', 'Stealth Rock'], nature: 'Modest', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 3']],
  },
  {
    id: 'ability-sheer-force-lo',
    description: 'Sheer Force + Life Orb (1.3x SF, 1.3x LO, no LO recoil)',
    seed: [3440, 3441, 3442, 3443],
    p1: [set({ species: 'Nidoking', ability: 'Sheer Force', item: 'Life Orb', moves: ['Earth Power', 'Sludge Wave', 'Ice Beam', 'Thunderbolt'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Slowbro', ability: 'Regenerator', item: '', moves: ['Scald', 'Psychic', 'Slack Off', 'Thunder Wave'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'ability-tough-claws',
    description: 'Tough Claws 1.3x on contact moves',
    seed: [3450, 3451, 3452, 3453],
    p1: [set({ species: 'Charizard', ability: 'Tough Claws', item: '', moves: ['Flare Blitz', 'Dragon Claw', 'Earthquake', 'Roost'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  {
    id: 'ability-sword-of-ruin',
    description: 'Sword of Ruin reduces target Def by 25%',
    seed: [3460, 3461, 3462, 3463],
    p1: [set({ species: 'Chien-Pao', ability: 'Sword of Ruin', item: '', moves: ['Ice Spinner', 'Crunch', 'Sacred Sword', 'Ice Shard'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 2', 'move 1']],
  },
  {
    id: 'ability-tablets-of-ruin',
    description: 'Tablets of Ruin reduces opponent Atk by 25%',
    seed: [3470, 3471, 3472, 3473],
    p1: [set({ species: 'Garchomp', ability: 'Rough Skin', item: '', moves: ['Earthquake', 'Dragon Claw', 'Stone Edge', 'Swords Dance'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Ting-Lu', ability: 'Tablets of Ruin', item: '', moves: ['Earthquake', 'Ruination', 'Stealth Rock', 'Whirlwind'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
];
