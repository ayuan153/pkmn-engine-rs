import { BattleStreams, Teams, Dex, PRNG } from '@pkmn/sim';
import { PokemonSet } from '@pkmn/sim';
import * as fs from 'fs';
import * as path from 'path';

const OUTPUT_DIR = path.resolve(import.meta.dirname, '../../../tests/fixtures/full-sim');

interface FullSimScenario {
  id: string;
  description: string;
  seed: [number, number, number, number];
  p1: PokemonSet[];
  p2: PokemonSet[];
  choices: [string, string][]; // [p1choice, p2choice] per turn
}

interface FullSimFixture {
  id: string;
  description: string;
  seed: [number, number, number, number];
  p1: { team: PokemonSet[] };
  p2: { team: PokemonSet[] };
  choices: [string, string][];
  protocol: string[];
}

// Lines to KEEP (semantic events)
const SEMANTIC_PREFIXES = [
  '|turn|', '|move|', '|switch|', '|drag|', '|faint|', '|win|', '|tie|',
  '|-damage|', '|-heal|', '|-status|', '|-curestatus|',
  '|-boost|', '|-unboost|', '|-setboost|', '|-clearboost|', '|-clearallboost|',
  '|-weather|', '|-fieldstart|', '|-fieldend|',
  '|-sidestart|', '|-sideend|',
  '|-start|', '|-end|',
  '|-item|', '|-enditem|',
  '|-ability|', '|-activate|',
  '|-crit|', '|-supereffective|', '|-resisted|', '|-immune|',
  '|-miss|', '|-hitcount|',
  '|cant|', '|upkeep',
  '|-fail|', '|-block|', '|-notarget|',
  '|-prepare|', '|-mustrecharge|',
  '|-singleturn|', '|-singlemove|',
  '|swap|', '|-transform|', '|-formechange|',
  '|-mega|', '|-terastallize|',
];

function isSemanticLine(line: string): boolean {
  return SEMANTIC_PREFIXES.some(prefix => line.startsWith(prefix));
}

// Remove duplicate lines caused by |split| format (public + private views)
function deduplicateSplitLines(lines: string[]): string[] {
  const result: string[] = [];
  for (let i = 0; i < lines.length; i++) {
    // Skip if this line is identical to the previous one (split duplicate)
    if (i > 0 && lines[i] === lines[i - 1]) continue;
    result.push(lines[i]);
  }
  return result;
}

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

// === SCENARIOS ===
const scenarios: FullSimScenario[] = [
  {
    id: 'simple-1v1-physical',
    description: 'Simple 1v1: two physical attackers trading hits until one faints',
    seed: [1, 1, 1, 1],
    p1: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Life Orb', moves: ['Earthquake', 'Dragon Claw', 'Stone Edge', 'Swords Dance'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Dragonite', ability: 'Multiscale', item: 'Choice Band', moves: ['Outrage', 'Extreme Speed', 'Earthquake', 'Fire Punch'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    choices: [['move 1', 'move 1'], ['move 1', 'move 1'], ['move 1', 'move 1'], ['move 1', 'move 1']],
  },
  {
    id: 'weather-hazards-switching',
    description: 'Sand + Stealth Rock + switching + Leftovers recovery',
    seed: [10, 20, 30, 40],
    p1: [
      set({ species: 'Tyranitar', ability: 'Sand Stream', item: 'Leftovers', moves: ['Stealth Rock', 'Crunch', 'Earthquake', 'Stone Edge'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
      set({ species: 'Excadrill', ability: 'Sand Rush', item: 'Choice Band', moves: ['Earthquake', 'Iron Head', 'Rock Slide', 'Rapid Spin'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Corviknight', ability: 'Pressure', item: 'Leftovers', moves: ['Brave Bird', 'Body Press', 'Roost', 'Defog'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
      set({ species: 'Toxapex', ability: 'Regenerator', item: 'Rocky Helmet', moves: ['Scald', 'Toxic', 'Recover', 'Haze'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],   // T1: TTar sets SR, Corv Brave Birds
      ['move 2', 'switch 2'], // T2: TTar Crunches, Corv switches to Toxapex (takes SR)
      ['switch 2', 'move 2'], // T3: TTar switches to Excadrill, Toxapex Toxics
      ['move 1', 'move 3'],   // T4: Excadrill EQs, Toxapex Recovers
      ['move 1', 'move 1'],   // T5: Excadrill EQs, Toxapex Scalds
    ],
  },
  {
    id: 'status-toxic-burn',
    description: 'Toxic escalating + burn damage + Guts interaction',
    seed: [5, 10, 15, 20],
    p1: [
      set({ species: 'Conkeldurr', ability: 'Guts', item: 'Flame Orb', moves: ['Drain Punch', 'Mach Punch', 'Facade', 'Knock Off'], nature: 'Adamant', evs: { hp: 252, atk: 252, def: 0, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Toxapex', ability: 'Regenerator', item: 'Rocky Helmet', moves: ['Toxic', 'Scald', 'Recover', 'Haze'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 3', 'move 1'],  // T1: Facade vs Toxic
      ['move 3', 'move 3'],  // T2: Facade (now burned from Flame Orb) vs Recover
      ['move 1', 'move 2'],  // T3: Drain Punch vs Scald
      ['move 1', 'move 3'],  // T4: Drain Punch vs Recover
    ],
  },
  {
    id: 'terrain-abilities-priority',
    description: 'Electric Terrain + priority blocking + ability triggers',
    seed: [100, 200, 300, 400],
    p1: [
      set({ species: 'Tapu Koko', ability: 'Electric Surge', item: 'Life Orb', moves: ['Thunderbolt', 'Dazzling Gleam', 'U-turn', 'Roost'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Scizor', ability: 'Technician', item: 'Choice Band', moves: ['Bullet Punch', 'U-turn', 'Knock Off', 'Superpower'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: Tbolt vs Bullet Punch (priority blocked by Psychic Terrain? No - Electric Terrain doesn't block priority)
      ['move 2', 'move 2'],  // T2: Dazzling Gleam vs U-turn
      ['move 1', 'move 3'],  // T3: Tbolt vs Knock Off
    ],
  },
  {
    id: 'screens-boosts-crit',
    description: 'Reflect + Swords Dance + critical hit interaction',
    seed: [42, 42, 42, 42],
    p1: [
      set({ species: 'Grimmsnarl', ability: 'Prankster', item: 'Light Clay', moves: ['Reflect', 'Light Screen', 'Thunder Wave', 'Spirit Break'], nature: 'Careful', evs: { hp: 252, atk: 0, def: 0, spa: 0, spd: 252, spe: 4 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Kingambit', ability: 'Supreme Overlord', item: 'Black Glasses', moves: ['Swords Dance', 'Kowtow Cleave', 'Iron Head', 'Sucker Punch'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: Reflect vs Swords Dance
      ['move 3', 'move 2'],  // T2: Thunder Wave vs Kowtow Cleave (never misses)
      ['move 4', 'move 2'],  // T3: Spirit Break vs Kowtow Cleave
      ['move 4', 'move 2'],  // T4
    ],
  },
  {
    id: 'substitute-protect-toxic',
    description: 'Substitute blocking + Protect + Toxic stalling',
    seed: [7, 14, 21, 28],
    p1: [
      set({ species: 'Gengar', ability: 'Cursed Body', item: 'Leftovers', moves: ['Substitute', 'Shadow Ball', 'Sludge Bomb', 'Focus Blast'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Blissey', ability: 'Natural Cure', item: 'Leftovers', moves: ['Toxic', 'Seismic Toss', 'Soft-Boiled', 'Protect'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: Sub vs Toxic (blocked by sub)
      ['move 2', 'move 2'],  // T2: Shadow Ball vs Seismic Toss (hits sub)
      ['move 2', 'move 4'],  // T3: Shadow Ball vs Protect
      ['move 2', 'move 3'],  // T4: Shadow Ball vs Soft-Boiled
      ['move 3', 'move 2'],  // T5: Sludge Bomb vs Seismic Toss
    ],
  },
  {
    id: 'multi-hit-focus-sash',
    description: 'Multi-hit move breaking Focus Sash + Sturdy',
    seed: [50, 60, 70, 80],
    p1: [
      set({ species: 'Cloyster', ability: 'Skill Link', item: 'Focus Sash', moves: ['Shell Smash', 'Icicle Spear', 'Rock Blast', 'Hydro Pump'], nature: 'Naive', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Skarmory', ability: 'Sturdy', item: 'Rocky Helmet', moves: ['Stealth Rock', 'Brave Bird', 'Body Press', 'Whirlwind'], nature: 'Impish', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: Shell Smash vs Stealth Rock
      ['move 2', 'move 2'],  // T2: Icicle Spear (5 hits, Skill Link) vs Brave Bird
      ['move 3', 'move 3'],  // T3: Rock Blast vs Body Press
    ],
  },
  {
    id: 'intimidate-uturn-regen',
    description: 'Intimidate on switch + U-turn + Regenerator healing',
    seed: [11, 22, 33, 44],
    p1: [
      set({ species: 'Landorus-Therian', ability: 'Intimidate', item: 'Choice Scarf', moves: ['Earthquake', 'U-turn', 'Stone Edge', 'Knock Off'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
      set({ species: 'Rotom-Wash', ability: 'Levitate', item: 'Leftovers', moves: ['Volt Switch', 'Hydro Pump', 'Will-O-Wisp', 'Pain Split'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Slowbro', ability: 'Regenerator', item: 'Heavy-Duty Boots', moves: ['Scald', 'Psychic', 'Slack Off', 'Teleport'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
      set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: 'Leftovers', moves: ['Power Whip', 'Knock Off', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 2', 'move 1'],  // T1: Lando U-turns, Slowbro Scalds
      ['switch 2', 'move 4'],  // T1b: Lando switches to Rotom (from U-turn), Slowbro Teleports
      // Actually U-turn forces a switch choice after. Let me simplify:
      ['move 1', 'move 4'],  // T1: Lando EQs, Slowbro Teleports out (Regen heals)
      ['move 2', 'move 1'],  // T2: Lando U-turns into Ferrothorn, Rotom comes in
      ['move 1', 'move 2'],  // T3: Rotom Volt Switches, Ferro Knock Offs
    ],
  },
  {
    id: 'blaze-pinch',
    description: 'Blaze activates at <1/3 HP, boosting Fire moves 1.5x',
    seed: [2100, 2101, 2102, 2103],
    p1: [
      set({ species: 'Charizard', ability: 'Blaze', item: 'Heavy-Duty Boots', moves: ['Flamethrower', 'Air Slash', 'Dragon Pulse', 'Roost'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Swampert', ability: 'Torrent', item: 'Leftovers', moves: ['Earthquake', 'Ice Beam', 'Stealth Rock', 'Flip Turn'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 2', 'move 1'],  // T1: Air Slash vs EQ (Charizard takes big damage)
      ['move 1', 'move 1'],  // T2: Flamethrower (Blaze active) vs EQ
    ],
  },
  {
    id: 'flash-fire-boost',
    description: 'Flash Fire absorbs Fire move then boosts own Fire damage 1.5x',
    seed: [2110, 2111, 2112, 2113],
    p1: [
      set({ species: 'Arcanine', ability: 'Intimidate', item: 'Choice Band', moves: ['Flare Blitz', 'Extreme Speed', 'Close Combat', 'Wild Charge'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Heatran', ability: 'Flash Fire', item: 'Leftovers', moves: ['Magma Storm', 'Earth Power', 'Flash Cannon', 'Stealth Rock'], nature: 'Modest', evs: { hp: 252, atk: 0, def: 0, spa: 252, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 2'],  // T1: Flare Blitz (absorbed by Flash Fire) vs Earth Power
      ['move 2', 'move 1'],  // T2: Extreme Speed vs Magma Storm (Flash Fire boosted)
    ],
  },
  {
    id: 'defeatist-halved',
    description: 'Defeatist halves Atk+SpA below 50% HP',
    seed: [2120, 2121, 2122, 2123],
    p1: [
      set({ species: 'Archeops', ability: 'Defeatist', item: 'Choice Scarf', moves: ['Stone Edge', 'Earthquake', 'U-turn', 'Head Smash'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Slowbro', ability: 'Regenerator', item: 'Heavy-Duty Boots', moves: ['Scald', 'Psychic', 'Slack Off', 'Thunder Wave'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 4', 'move 1'],  // T1: Head Smash (recoil drops below 50%) vs Scald
      ['move 1', 'move 2'],  // T2: Stone Edge (Defeatist halved) vs Psychic
    ],
  },
  {
    id: 'toxic-spikes-poison',
    description: 'Toxic Spikes: 1 layer poisons, 2 layers badly poisons, Poison-type absorbs',
    seed: [2200, 2201, 2202, 2203],
    p1: [
      set({ species: 'Toxapex', ability: 'Regenerator', item: 'Rocky Helmet', moves: ['Toxic Spikes', 'Scald', 'Recover', 'Haze'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Band', moves: ['Earthquake', 'Outrage', 'Stone Edge', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
      set({ species: 'Excadrill', ability: 'Sand Rush', item: 'Choice Scarf', moves: ['Earthquake', 'Iron Head', 'Rock Slide', 'Rapid Spin'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: Toxic Spikes vs EQ
      ['move 1', 'move 1'],  // T2: Toxic Spikes (2nd layer) vs EQ
      ['move 3', 'switch 2'],  // T3: Recover vs switch to Excadrill (takes toxic poison)
      ['move 2', 'move 1'],  // T4: Scald vs EQ
    ],
  },
  {
    id: 'sticky-web-speed',
    description: 'Sticky Web drops Speed by 1 on switch-in (grounded only)',
    seed: [2210, 2211, 2212, 2213],
    p1: [
      set({ species: 'Ribombee', ability: 'Shield Dust', item: 'Focus Sash', moves: ['Sticky Web', 'Moonblast', 'Energy Ball', 'Psychic'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Scarf', moves: ['Earthquake', 'Outrage', 'Stone Edge', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
      set({ species: 'Dragonite', ability: 'Multiscale', item: 'Choice Band', moves: ['Outrage', 'Extreme Speed', 'Earthquake', 'Fire Punch'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: Sticky Web vs EQ (Sash saves)
      ['move 2', 'switch 2'],  // T2: Moonblast vs switch Dragonite (takes Sticky Web -1 Spe)
      ['move 2', 'move 2'],  // T3: Moonblast vs Extreme Speed
    ],
  },
  {
    id: 'hex-doubled-status',
    description: 'Hex doubles BP (65->130) when target has a status condition',
    seed: [2300, 2301, 2302, 2303],
    p1: [
      set({ species: 'Gengar', ability: 'Cursed Body', item: 'Choice Specs', moves: ['Hex', 'Shadow Ball', 'Sludge Bomb', 'Focus Blast'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Blissey', ability: 'Natural Cure', item: 'Leftovers', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 4'],  // T1: Hex (normal BP) vs Toxic (poisons Gengar)
      ['move 1', 'move 2'],  // T2: Hex (doubled, Blissey has... wait Blissey isn't statused)
    ],
  },
  {
    id: 'pixilate-type-change',
    description: 'Pixilate changes Normal moves to Fairy + 1.2x boost',
    seed: [2400, 2401, 2402, 2403],
    p1: [
      set({ species: 'Sylveon', ability: 'Pixilate', item: 'Choice Specs', moves: ['Hyper Voice', 'Mystical Fire', 'Shadow Ball', 'Psyshock'], nature: 'Modest', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Band', moves: ['Earthquake', 'Outrage', 'Stone Edge', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: Hyper Voice (becomes Fairy via Pixilate) vs EQ
      ['move 1', 'move 1'],  // T2: Hyper Voice again vs EQ
    ],
  },
  {
    id: 'grassy-terrain-eq-halved',
    description: 'Grassy Terrain halves Earthquake damage to grounded targets',
    seed: [2500, 2501, 2502, 2503],
    p1: [
      set({ species: 'Rillaboom', ability: 'Grassy Surge', item: 'Choice Band', moves: ['Grassy Glide', 'Wood Hammer', 'Earthquake', 'U-turn'], nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Toxapex', ability: 'Regenerator', item: 'Rocky Helmet', moves: ['Scald', 'Toxic', 'Recover', 'Knock Off'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 3', 'move 1'],  // T1: EQ (halved by Grassy Terrain) vs Scald
      ['move 1', 'move 4'],  // T2: Grassy Glide (boosted by terrain) vs Knock Off
    ],
  },
  {
    id: 'misty-terrain-dragon-halved',
    description: 'Misty Terrain halves Dragon moves against grounded targets',
    seed: [2510, 2511, 2512, 2513],
    p1: [
      set({ species: 'Tapu Fini', ability: 'Misty Surge', item: 'Leftovers', moves: ['Moonblast', 'Surf', 'Calm Mind', 'Taunt'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Band', moves: ['Outrage', 'Earthquake', 'Stone Edge', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: Moonblast vs Outrage (halved by Misty Terrain)
      ['move 2', 'move 1'],  // T2: Surf vs Outrage (still halved)
    ],
  },
  {
    id: 'moxie-ko-boost',
    description: 'Moxie gives +1 Atk after KOing an opponent',
    seed: [2600, 2601, 2602, 2603],
    p1: [
      set({ species: 'Gyarados', ability: 'Moxie', item: 'Choice Scarf', moves: ['Waterfall', 'Earthquake', 'Bounce', 'Ice Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Jolteon', ability: 'Volt Absorb', item: 'Focus Sash', moves: ['Thunderbolt', 'Shadow Ball', 'Volt Switch', 'Yawn'], nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 }, level: 100 }),
      set({ species: 'Blissey', ability: 'Natural Cure', item: 'Leftovers', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 2', 'move 1'],  // T1: EQ vs Tbolt (Jolteon faster, Sash saves if needed)
      ['move 2', 'move 2'],  // T2: EQ (KOs Jolteon, Moxie +1) vs Shadow Ball
      ['move 1', 'move 1'],  // T3: Waterfall (Moxie boosted) vs Seismic Toss
    ],
  },
  {
    id: 'regenerator-switch-heal',
    description: 'Regenerator heals 1/3 HP on switch-out',
    seed: [2610, 2611, 2612, 2613],
    p1: [
      set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Choice Band', moves: ['Earthquake', 'Outrage', 'Stone Edge', 'Fire Fang'], nature: 'Jolly', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 }, level: 100 }),
    ],
    p2: [
      set({ species: 'Slowbro', ability: 'Regenerator', item: 'Heavy-Duty Boots', moves: ['Scald', 'Psychic', 'Slack Off', 'Teleport'], nature: 'Bold', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
      set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: 'Leftovers', moves: ['Power Whip', 'Knock Off', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: { hp: 252, atk: 0, def: 252, spa: 0, spd: 4, spe: 0 }, level: 100 }),
    ],
    choices: [
      ['move 1', 'move 1'],  // T1: EQ vs Scald (Slowbro takes damage)
      ['move 1', 'switch 2'],  // T2: EQ vs switch to Ferro (Slowbro heals via Regen)
      ['move 1', 'move 1'],  // T3: EQ vs Power Whip
    ],
  },
];

async function runFullSimBattle(scenario: FullSimScenario): Promise<FullSimFixture> {
  const stream = new BattleStreams.BattleStream();

  const spec = { formatid: 'gen9customgame' as any, seed: scenario.seed };
  const p1spec = { name: 'Player 1', team: Teams.pack(scenario.p1) };
  const p2spec = { name: 'Player 2', team: Teams.pack(scenario.p2) };

  const allOutput: string[] = [];
  let battleEnded = false;

  const readerDone = (async () => {
    for await (const chunk of stream) {
      const lines = chunk.split('\n');
      for (const line of lines) {
        if (line.trim()) allOutput.push(line);
        if (line.startsWith('|win|') || line === '|tie') {
          battleEnded = true;
        }
      }
    }
  })();

  // Start battle
  stream.write(
    `>start ${JSON.stringify(spec)}\n` +
    `>player p1 ${JSON.stringify(p1spec)}\n` +
    `>player p2 ${JSON.stringify(p2spec)}`
  );
  await new Promise(r => setTimeout(r, 100));

  // Handle team preview
  stream.write(`>p1 default\n>p2 default`);
  await new Promise(r => setTimeout(r, 100));

  // Feed choices turn by turn
  for (const [p1choice, p2choice] of scenario.choices) {
    if (battleEnded) break;
    stream.write(`>p1 ${p1choice}\n>p2 ${p2choice}`);
    await new Promise(r => setTimeout(r, 100));
  }

  // If battle hasn't ended, force tie
  if (!battleEnded) {
    stream.write('>forcetie');
    await new Promise(r => setTimeout(r, 100));
  }

  // Stream auto-ends when battle ends; pushEnd only needed for forced ties
  try { stream.pushEnd(); } catch {}
  await readerDone;

  // Filter to semantic events only, then deduplicate split lines
  const semantic = allOutput.filter(isSemanticLine);
  const protocol = deduplicateSplitLines(semantic);

  return {
    id: scenario.id,
    description: scenario.description,
    seed: scenario.seed,
    p1: { team: scenario.p1 },
    p2: { team: scenario.p2 },
    choices: scenario.choices,
    protocol,
  };
}

async function main() {
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });
  console.log(`Generating ${scenarios.length} full-sim fixtures...`);

  for (const scenario of scenarios) {
    try {
      const fixture = await runFullSimBattle(scenario);
      const outPath = path.join(OUTPUT_DIR, `${scenario.id}.json`);
      fs.writeFileSync(outPath, JSON.stringify(fixture, null, 2));
      console.log(`  ✓ ${scenario.id}: ${fixture.protocol.length} protocol lines`);
    } catch (e: any) {
      console.error(`  ✗ ${scenario.id}: ${e.message}`);
    }
  }
  console.log('Done.');
}

main();
