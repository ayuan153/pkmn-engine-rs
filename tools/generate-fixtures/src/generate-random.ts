import { BattleStreams, Dex, RandomPlayerAI, Teams } from '@pkmn/sim';
import * as fs from 'fs';
import * as path from 'path';

const OUTPUT_DIR = path.resolve(import.meta.dirname, '../../../tests/fixtures/full-sim');

const SEMANTIC_PREFIXES = [
  '|turn|', '|move|', '|switch|', '|drag|', '|faint|', '|win|',
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
  if (line === '|tie') return true;
  return SEMANTIC_PREFIXES.some(prefix => line.startsWith(prefix));
}

interface PokemonSet {
  species: string; ability: string; item: string;
  moves: string[]; nature: string;
  evs: { hp: number; atk: number; def: number; spa: number; spd: number; spe: number };
  level: number;
}

function set(s: PokemonSet): PokemonSet { return s; }

const POOL: PokemonSet[] = [
  set({ species: 'Garchomp', ability: 'Rough Skin', item: 'Life Orb', moves: ['Earthquake', 'Dragon Claw', 'Stone Edge', 'Swords Dance'], nature: 'Jolly', evs: {hp:0,atk:252,def:0,spa:0,spd:4,spe:252}, level: 100 }),
  set({ species: 'Dragonite', ability: 'Multiscale', item: 'Choice Band', moves: ['Outrage', 'Extreme Speed', 'Earthquake', 'Fire Punch'], nature: 'Adamant', evs: {hp:0,atk:252,def:0,spa:0,spd:4,spe:252}, level: 100 }),
  set({ species: 'Tyranitar', ability: 'Sand Stream', item: 'Leftovers', moves: ['Stone Edge', 'Crunch', 'Earthquake', 'Stealth Rock'], nature: 'Adamant', evs: {hp:252,atk:252,def:0,spa:0,spd:4,spe:0}, level: 100 }),
  set({ species: 'Gengar', ability: 'Cursed Body', item: 'Choice Specs', moves: ['Shadow Ball', 'Sludge Bomb', 'Focus Blast', 'Thunderbolt'], nature: 'Timid', evs: {hp:0,atk:0,def:0,spa:252,spd:4,spe:252}, level: 100 }),
  set({ species: 'Scizor', ability: 'Technician', item: 'Choice Band', moves: ['Bullet Punch', 'U-turn', 'Knock Off', 'Superpower'], nature: 'Adamant', evs: {hp:248,atk:252,def:0,spa:0,spd:8,spe:0}, level: 100 }),
  set({ species: 'Rotom-Wash', ability: 'Levitate', item: 'Leftovers', moves: ['Hydro Pump', 'Volt Switch', 'Will-O-Wisp', 'Pain Split'], nature: 'Bold', evs: {hp:252,atk:0,def:252,spa:0,spd:4,spe:0}, level: 100 }),
  set({ species: 'Ferrothorn', ability: 'Iron Barbs', item: 'Leftovers', moves: ['Power Whip', 'Knock Off', 'Stealth Rock', 'Leech Seed'], nature: 'Relaxed', evs: {hp:252,atk:0,def:252,spa:0,spd:4,spe:0}, level: 100 }),
  set({ species: 'Blissey', ability: 'Natural Cure', item: 'Leftovers', moves: ['Seismic Toss', 'Soft-Boiled', 'Thunder Wave', 'Toxic'], nature: 'Bold', evs: {hp:252,atk:0,def:252,spa:0,spd:4,spe:0}, level: 100 }),
  set({ species: 'Corviknight', ability: 'Pressure', item: 'Leftovers', moves: ['Brave Bird', 'Body Press', 'Roost', 'Defog'], nature: 'Impish', evs: {hp:252,atk:0,def:252,spa:0,spd:4,spe:0}, level: 100 }),
  set({ species: 'Excadrill', ability: 'Sand Rush', item: 'Choice Scarf', moves: ['Earthquake', 'Iron Head', 'Rock Slide', 'Rapid Spin'], nature: 'Jolly', evs: {hp:0,atk:252,def:0,spa:0,spd:4,spe:252}, level: 100 }),
];

function formatTeamForShowdown(mon: PokemonSet): string {
  const evStr = Object.entries(mon.evs).filter(([,v]) => v > 0).map(([k,v]) => `${v} ${k}`).join(' / ');
  const movesStr = mon.moves.map(m => `- ${m}`).join('\n');
  return `${mon.species} @ ${mon.item}
Ability: ${mon.ability}
Level: ${mon.level}
EVs: ${evStr}
${mon.nature} Nature
${movesStr}`;
}

// Simple seeded PRNG (xorshift32)
function xorshift(state: number): number {
  state ^= state << 13;
  state ^= state >>> 17;
  state ^= state << 5;
  return state >>> 0;
}

async function runBattle(id: string, seed: [number, number, number, number], p1Mon: PokemonSet, p2Mon: PokemonSet) {
  const battleStream = new BattleStreams.BattleStream();
  const streams = BattleStreams.getPlayerStreams(battleStream);

  const p1 = new RandomPlayerAI(streams.p1);
  const p2 = new RandomPlayerAI(streams.p2);

  const allOutput: string[] = [];
  const omniscientDone = (async () => {
    for await (const chunk of streams.omniscient) {
      for (const line of chunk.split('\n')) {
        if (line.trim()) allOutput.push(line);
      }
    }
  })();

  void p1.start();
  void p2.start();

  const p1Team = formatTeamForShowdown(p1Mon);
  const p2Team = formatTeamForShowdown(p2Mon);

  streams.omniscient.write(
    `>start ${JSON.stringify({ formatid: 'gen9customgame', seed })}\n` +
    `>player p1 ${JSON.stringify({ name: 'Player 1', team: Teams.pack(Teams.import(p1Team)!) })}\n` +
    `>player p2 ${JSON.stringify({ name: 'Player 2', team: Teams.pack(Teams.import(p2Team)!) })}`
  );

  await omniscientDone;

  const battle = battleStream.battle!;
  const protocol = allOutput.filter(isSemanticLine);

  // With 1v1, choices are simple: just move slots, no switching
  const choices: [string, string][] = [];
  let p1Choice: string | null = null;
  let p2Choice: string | null = null;

  // Build move lookup for each side (inputLog uses toID format like 'ironhead')
  const sides = battle.sides;
  const moveLookup: Map<string, number>[] = [];
  for (let s = 0; s < 2; s++) {
    const map = new Map<string, number>();
    sides[s].team[0].moves.forEach((moveName, i) => {
      const moveId = Dex.moves.get(moveName).id;
      map.set(moveId, i + 1);
    });
    moveLookup.push(map);
  }

  function convertChoice(choice: string, sideIdx: number): string {
    if (choice.startsWith('team ')) return ''; // skip team preview
    if (choice.startsWith('move ')) {
      const rest = choice.slice(5);
      const moveId = rest.split(' ')[0];
      const suffix = rest.slice(moveId.length);
      if (moveId === 'struggle' || moveId === 'recharge') return `move 1${suffix}`;
      const slot = moveLookup[sideIdx].get(moveId);
      if (slot !== undefined) return `move ${slot}${suffix}`;
      return choice;
    }
    return choice;
  }

  for (const line of battle.inputLog) {
    if (line.startsWith('>p1 ')) {
      const c = convertChoice(line.slice(4), 0);
      if (c) p1Choice = c;
    } else if (line.startsWith('>p2 ')) {
      const c = convertChoice(line.slice(4), 1);
      if (c) p2Choice = c;
    }
    if (p1Choice && p2Choice) {
      choices.push([p1Choice, p2Choice]);
      p1Choice = null;
      p2Choice = null;
    }
  }

  const defaultIvs = { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 };
  function buildTeamEntry(mon: PokemonSet) {
    return {
      species: mon.species,
      ability: mon.ability,
      item: mon.item,
      moves: mon.moves,
      nature: mon.nature,
      evs: mon.evs,
      ivs: defaultIvs,
      level: mon.level,
      gender: '',
    };
  }

  return {
    id,
    description: `1v1 random battle: ${p1Mon.species} vs ${p2Mon.species}`,
    seed,
    format: 'gen9customgame',
    p1: { team: [buildTeamEntry(p1Mon)] },
    p2: { team: [buildTeamEntry(p2Mon)] },
    choices,
    protocol,
  };
}

async function main() {
  const count = parseInt(process.argv[2] || '30');
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });

  console.log(`Generating ${count} random 1v1 battle fixtures...`);

  for (let i = 0; i < count; i++) {
    const seed: [number, number, number, number] = [
      5000 + i * 4, 5001 + i * 4, 5002 + i * 4, 5003 + i * 4,
    ];
    // Use seed to pick pokemon - mix multiple times for variety
    let rng = seed[0] + i * 7919; // prime multiplier for spread
    rng = xorshift(xorshift(xorshift(rng >>> 0)));
    const p1Idx = rng % POOL.length;
    rng = xorshift(rng);
    let p2Idx = rng % POOL.length;
    if (p2Idx === p1Idx) p2Idx = (p2Idx + 1) % POOL.length;

    const id = `random-1v1-${String(i + 1).padStart(3, '0')}`;

    try {
      const fixture = await runBattle(id, seed, POOL[p1Idx], POOL[p2Idx]);
      const outPath = path.join(OUTPUT_DIR, `${id}.json`);
      fs.writeFileSync(outPath, JSON.stringify(fixture, null, 2));
      console.log(`  ✓ ${id}: ${fixture.protocol.length} lines, ${fixture.choices.length} turns (${POOL[p1Idx].species} vs ${POOL[p2Idx].species})`);
    } catch (e: any) {
      console.error(`  ✗ ${id}: ${e.message}`);
    }
  }

  console.log('Done.');
}

main();
