import { BattleStreams, Dex, RandomPlayerAI, Teams } from '@pkmn/sim';
import { TeamGenerators } from '@pkmn/randoms';
import * as fs from 'fs';
import * as path from 'path';

Teams.setGeneratorFactory(TeamGenerators);

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

async function runRandomBattle(id: string, seed: [number, number, number, number]) {
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

  streams.omniscient.write(
    `>start ${JSON.stringify({ formatid: 'gen9randombattle', seed })}\n` +
    `>player p1 ${JSON.stringify({ name: 'Player 1' })}\n` +
    `>player p2 ${JSON.stringify({ name: 'Player 2' })}`
  );

  await omniscientDone;

  const battle = battleStream.battle!;
  const sides = battle.sides;

  // The team array is the ORIGINAL order (never changes).
  // The pokemon array gets reordered: pokemon[0] is always active.
  // We output the team in the original team[] order, with the lead moved to front.
  // Switch indices need to be converted from pokemon-array-relative to team-array-relative.

  // First, determine the initial pokemon array order by finding which team member was the lead.
  // The lead is identified from the first |switch| protocol line.
  const protocol = allOutput.filter(isSemanticLine);

  function getLeadName(prefix: string): string | null {
    const line = protocol.find(l => l.startsWith(`|switch|${prefix}`));
    if (!line) return null;
    // |switch|p1a: Name|Details|HP
    const match = line.match(/\|switch\|p[12]a: ([^|]+)\|/);
    return match ? match[1] : null;
  }

  // Build team data from the stable team[] array
  function buildTeamData(sideIdx: number) {
    return sides[sideIdx].team.map(mon => ({
      species: mon.species,
      ability: mon.ability ? Dex.abilities.get(mon.ability).name : '',
      item: mon.item ? Dex.items.get(mon.item).name : '',
      moves: mon.moves.map(m => Dex.moves.get(m).name),
      nature: mon.nature || 'Hardy',
      evs: mon.evs,
      ivs: mon.ivs,
      level: mon.level,
      gender: mon.gender || '',
    }));
  }

  // Reorder team so lead is at index 0 (matching what Rust engine expects)
  function reorderWithLead(team: any[], leadName: string | null): { team: any[], leadOrigIdx: number } {
    if (!leadName) return { team, leadOrigIdx: 0 };
    const leadIdx = team.findIndex(t => t.species === leadName);
    if (leadIdx <= 0) return { team, leadOrigIdx: Math.max(0, leadIdx) };
    const reordered = [...team];
    const [lead] = reordered.splice(leadIdx, 1);
    reordered.unshift(lead);
    return { team: reordered, leadOrigIdx: leadIdx };
  }

  const p1TeamRaw = buildTeamData(0);
  const p2TeamRaw = buildTeamData(1);
  const p1Result = reorderWithLead(p1TeamRaw, getLeadName('p1a: '));
  const p2Result = reorderWithLead(p2TeamRaw, getLeadName('p2a: '));

  // Build a mapping from original team index to new (reordered) team index
  // for converting switch targets
  function buildIndexMap(teamRaw: any[], reordered: any[]): Map<number, number> {
    const map = new Map<number, number>();
    for (let newIdx = 0; newIdx < reordered.length; newIdx++) {
      const origIdx = teamRaw.indexOf(reordered[newIdx]);
      map.set(origIdx, newIdx);
    }
    return map;
  }
  const p1IndexMap = buildIndexMap(p1TeamRaw, p1Result.team);
  const p2IndexMap = buildIndexMap(p2TeamRaw, p2Result.team);

  // Now convert choices.
  // In Showdown: pokemon array starts as [lead, ...rest] and gets reordered on switch.
  // "switch N" means switch to pokemon[N-1] in the CURRENT pokemon array state.
  // We need to track the pokemon array state and convert to our fixed team indices.

  // Simulate the Showdown pokemon array state for each side
  // Initial state: pokemon array = [leadOrigIdx, ...otherIndices]
  // We need to figure out the initial pokemon array order.
  // The initial order is: lead first, then the rest in original order.
  function getInitialPokemonOrder(teamSize: number, leadOrigIdx: number): number[] {
    const order = [];
    order.push(leadOrigIdx);
    for (let i = 0; i < teamSize; i++) {
      if (i !== leadOrigIdx) order.push(i);
    }
    return order;
  }

  // pokemonOrder[i] = original team index of the pokemon at position i in Showdown's array
  const pokemonOrder = [
    getInitialPokemonOrder(p1TeamRaw.length, p1Result.leadOrigIdx),
    getInitialPokemonOrder(p2TeamRaw.length, p2Result.leadOrigIdx),
  ];
  const indexMaps = [p1IndexMap, p2IndexMap];

  // Build move lookup: for each side, map (original team index, moveId) -> slot index
  const moveLookup: Map<string, number>[][] = [[], []];
  for (let s = 0; s < 2; s++) {
    for (const mon of sides[s].team) {
      const map = new Map<string, number>();
      mon.moves.forEach((moveId, i) => map.set(moveId, i + 1));
      moveLookup[s].push(map);
    }
  }

  const choices: [string, string][] = [];
  let p1Choice: string | null = null;
  let p2Choice: string | null = null;

  function convertChoice(choice: string, sideIdx: number): string {
    if (choice.startsWith('move ')) {
      const rest = choice.slice(5);
      const moveId = rest.split(' ')[0];
      const suffix = rest.slice(moveId.length);
      if (moveId === 'struggle' || moveId === 'recharge') return `move 1${suffix}`;
      // Active pokemon is at pokemonOrder[sideIdx][0]
      const activeOrigIdx = pokemonOrder[sideIdx][0];
      const map = moveLookup[sideIdx][activeOrigIdx];
      const slot = map?.get(moveId);
      if (slot !== undefined) return `move ${slot}${suffix}`;
      return choice;
    } else if (choice.startsWith('switch ')) {
      const n = parseInt(choice.slice(7));
      if (isNaN(n)) return choice;
      // In Showdown, "switch N" means switch to pokemon at position N-1 in current array
      const targetOrigIdx = pokemonOrder[sideIdx][n - 1];
      // Simulate the swap: Showdown swaps pokemon[0] with pokemon[N-1]
      const tmp = pokemonOrder[sideIdx][0];
      pokemonOrder[sideIdx][0] = pokemonOrder[sideIdx][n - 1];
      pokemonOrder[sideIdx][n - 1] = tmp;
      // Convert to our reordered team index (1-indexed)
      const newTeamIdx = indexMaps[sideIdx].get(targetOrigIdx);
      if (newTeamIdx !== undefined) return `switch ${newTeamIdx + 1}`;
      return choice;
    }
    return choice;
  }

  for (const line of battle.inputLog) {
    if (line.startsWith('>p1 ')) {
      p1Choice = convertChoice(line.slice(4), 0);
    } else if (line.startsWith('>p2 ')) {
      p2Choice = convertChoice(line.slice(4), 1);
    }
    if (p1Choice && p2Choice) {
      choices.push([p1Choice, p2Choice]);
      p1Choice = null;
      p2Choice = null;
    }
  }

  return {
    id,
    description: `Random battle ${id}`,
    seed,
    format: 'gen9randombattle',
    p1: { team: p1Result.team },
    p2: { team: p2Result.team },
    choices,
    protocol,
  };
}

async function main() {
  const count = parseInt(process.argv[2] || '20');
  fs.mkdirSync(OUTPUT_DIR, { recursive: true });

  console.log(`Generating ${count} random battle fixtures...`);

  for (let i = 0; i < count; i++) {
    const seed: [number, number, number, number] = [
      3000 + i * 4, 3001 + i * 4, 3002 + i * 4, 3003 + i * 4,
    ];
    const id = `random-${String(i + 1).padStart(3, '0')}`;

    try {
      const fixture = await runRandomBattle(id, seed);
      const outPath = path.join(OUTPUT_DIR, `${id}.json`);
      fs.writeFileSync(outPath, JSON.stringify(fixture, null, 2));
      console.log(`  ✓ ${id}: ${fixture.protocol.length} lines, ${fixture.choices.length} turns`);
    } catch (e: any) {
      console.error(`  ✗ ${id}: ${e.message}`);
    }
  }

  console.log('Done.');
}

main();
