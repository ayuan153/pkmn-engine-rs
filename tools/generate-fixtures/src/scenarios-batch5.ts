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

export const batch5Scenarios: BattleScenario[] = [
  // === SHARPNESS: 1.5x on slicing moves ===
  {
    id: 'ability-sharpness',
    description: 'Sharpness 1.5x on Leaf Blade (slicing)',
    seed: [5000, 5001, 5002, 5003],
    p1: [set({ species: 'Gallade', ability: 'Sharpness', item: '', moves: ['Leaf Blade', 'Psycho Cut', 'Sacred Sword', 'Swords Dance'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === RECKLESS: 1.2x on recoil moves ===
  {
    id: 'ability-reckless',
    description: 'Reckless 1.2x on Brave Bird (recoil)',
    seed: [5010, 5011, 5012, 5013],
    p1: [set({ species: 'Staraptor', ability: 'Reckless', item: '', moves: ['Brave Bird', 'Double-Edge', 'Close Combat', 'U-turn'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === MEGA LAUNCHER: 1.5x on pulse/aura moves ===
  {
    id: 'ability-mega-launcher',
    description: 'Mega Launcher 1.5x on Dark Pulse (pulse)',
    seed: [5020, 5021, 5022, 5023],
    p1: [set({ species: 'Clawitzer', ability: 'Mega Launcher', item: '', moves: ['Dark Pulse', 'Aura Sphere', 'Dragon Pulse', 'Water Pulse'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === PUNK ROCK: 1.3x on sound moves (attacker) ===
  {
    id: 'ability-punk-rock-offense',
    description: 'Punk Rock 1.3x on Boomburst (sound, attacker)',
    seed: [5030, 5031, 5032, 5033],
    p1: [set({ species: 'Toxtricity', ability: 'Punk Rock', item: '', moves: ['Boomburst', 'Overdrive', 'Sludge Bomb', 'Volt Switch'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === PUNK ROCK: halve sound damage taken (defender) ===
  {
    id: 'ability-punk-rock-defense',
    description: 'Punk Rock halves sound damage taken by defender',
    seed: [5040, 5041, 5042, 5043],
    p1: [set({ species: 'Exploud', ability: 'Scrappy', item: '', moves: ['Boomburst', 'Hyper Voice', 'Fire Blast', 'Surf'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Toxtricity', ability: 'Punk Rock', item: '', moves: ['Overdrive', 'Sludge Bomb', 'Volt Switch', 'Boomburst'], nature: 'Modest', evs: { hp: 252, atk: 0, def: 4, spa: 0, spd: 252, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === STEELWORKER: 1.5x on Steel moves ===
  {
    id: 'ability-steelworker',
    description: 'Steelworker 1.5x on Flash Cannon',
    seed: [5050, 5051, 5052, 5053],
    p1: [set({ species: 'Dhelmise', ability: 'Steelworker', item: '', moves: ['Flash Cannon', 'Shadow Ball', 'Earthquake', 'Rapid Spin'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === WATER BUBBLE: 2x on Water moves ===
  {
    id: 'ability-water-bubble-offense',
    description: 'Water Bubble 2x on Liquidation',
    seed: [5060, 5061, 5062, 5063],
    p1: [set({ species: 'Araquanid', ability: 'Water Bubble', item: '', moves: ['Liquidation', 'Leech Life', 'Ice Beam', 'Sticky Web'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === WATER BUBBLE: halve Fire damage taken ===
  {
    id: 'ability-water-bubble-defense',
    description: 'Water Bubble halves Fire damage taken',
    seed: [5070, 5071, 5072, 5073],
    p1: [set({ species: 'Volcarona', ability: 'Flame Body', item: '', moves: ['Fire Blast', 'Bug Buzz', 'Psychic', 'Quiver Dance'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Araquanid', ability: 'Water Bubble', item: '', moves: ['Liquidation', 'Leech Life', 'Ice Beam', 'Sticky Web'], nature: 'Careful', evs: { hp: 252, atk: 0, def: 4, spa: 0, spd: 252, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === NEUROFORCE: 1.25x on super effective ===
  {
    id: 'ability-neuroforce',
    description: 'Neuroforce 1.25x on super effective Psychic vs Fighting',
    seed: [5080, 5081, 5082, 5083],
    p1: [set({ species: 'Necrozma', ability: 'Neuroforce', item: '', moves: ['Psychic', 'Dark Pulse', 'Flash Cannon', 'Earth Power'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Lucario', ability: 'Inner Focus', item: '', moves: ['Close Combat', 'Meteor Mash', 'Extreme Speed', 'Swords Dance'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    turns: [['move 1', 'move 1']],
  },
  // === PIXILATE: Normal -> Fairy + 1.2x ===
  {
    id: 'ability-pixilate',
    description: 'Pixilate converts Hyper Voice Normal->Fairy + 1.2x',
    seed: [5090, 5091, 5092, 5093],
    p1: [set({ species: 'Sylveon', ability: 'Pixilate', item: '', moves: ['Hyper Voice', 'Shadow Ball', 'Mystical Fire', 'Calm Mind'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Garchomp', ability: 'Rough Skin', item: '', moves: ['Earthquake', 'Dragon Claw', 'Stone Edge', 'Swords Dance'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    turns: [['move 1', 'move 1']],
  },
  // === REFRIGERATE: Normal -> Ice + 1.2x ===
  {
    id: 'ability-refrigerate',
    description: 'Refrigerate converts Return Normal->Ice + 1.2x',
    seed: [5100, 5101, 5102, 5103],
    p1: [set({ species: 'Aurorus', ability: 'Refrigerate', item: '', moves: ['Hyper Voice', 'Earth Power', 'Flash Cannon', 'Thunderbolt'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Garchomp', ability: 'Rough Skin', item: '', moves: ['Earthquake', 'Dragon Claw', 'Stone Edge', 'Swords Dance'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    turns: [['move 1', 'move 1']],
  },
  // === AERILATE: Normal -> Flying + 1.2x ===
  {
    id: 'ability-aerilate',
    description: 'Aerilate converts Return Normal->Flying + 1.2x',
    seed: [5110, 5111, 5112, 5113],
    p1: [set({ species: 'Salamence', ability: 'Aerilate', item: '', moves: ['Hyper Voice', 'Earthquake', 'Fire Blast', 'Dragon Dance'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === GALVANIZE: Normal -> Electric + 1.2x ===
  {
    id: 'ability-galvanize',
    description: 'Galvanize converts Hyper Voice Normal->Electric + 1.2x',
    seed: [5120, 5121, 5122, 5123],
    p1: [set({ species: 'Golem-Alola', ability: 'Galvanize', item: '', moves: ['Hyper Voice', 'Earthquake', 'Stone Edge', 'Stealth Rock'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 } })],
    p2: [set({ species: 'Blissey', ability: 'Natural Cure', item: '', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 } })],
    turns: [['move 1', 'move 1']],
  },
  // === ANALYTIC: 1.3x when moving last ===
  {
    id: 'ability-analytic',
    description: 'Analytic 1.3x when moving last',
    seed: [5130, 5131, 5132, 5133],
    p1: [set({ species: 'Porygon-Z', ability: 'Analytic', item: '', moves: ['Thunderbolt', 'Ice Beam', 'Dark Pulse', 'Tri Attack'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 } })],
    p2: [set({ species: 'Weavile', ability: 'Pressure', item: '', moves: ['Ice Punch', 'Knock Off', 'Ice Shard', 'Low Kick'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 } })],
    turns: [['move 1', 'move 1']],
  },
];
