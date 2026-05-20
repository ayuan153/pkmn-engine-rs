import { BattleStreams, Teams } from '@pkmn/sim';
import * as fs from 'fs';

const fixtures = [
  'substitute-protect-toxic',
  'random-1v1-002', 'random-1v1-003', 'random-1v1-005',
  'random-1v1-015', 'random-1v1-019', 'random-1v1-029'
];

async function traceFixture(name: string) {
  (globalThis as any)._prngLog = [];
  (globalThis as any)._prngCallCount = 0;

  const fixture = JSON.parse(fs.readFileSync(`../../tests/fixtures/full-sim/${name}.json`, 'utf8'));
  const stream = new BattleStreams.BattleStream();
  const streams = BattleStreams.getPlayerStreams(stream);
  const done = (async () => { for await (const c of streams.omniscient) {} })();

  streams.omniscient.write(`>start {"formatid":"gen9customgame","seed":[${fixture.seed}]}`);
  streams.omniscient.write(`>player p1 {"name":"Player 1","team":"${Teams.pack(fixture.p1.team)}"}`);
  streams.omniscient.write(`>player p2 {"name":"Player 2","team":"${Teams.pack(fixture.p2.team)}"}`);
  await new Promise(r => setTimeout(r, 100));
  streams.omniscient.write('>p1 team 1');
  streams.omniscient.write('>p2 team 1');
  await new Promise(r => setTimeout(r, 100));

  for (const [p1c, p2c] of fixture.choices) {
    streams.omniscient.write(`>p1 ${p1c}`);
    streams.omniscient.write(`>p2 ${p2c}`);
    await new Promise(r => setTimeout(r, 50));
  }
  streams.omniscient.write('>forcetie');
  await done;

  const log = (globalThis as any)._prngLog;
  // Categorize calls
  const categories: Record<string, number> = {};
  for (const entry of log) {
    const frames = entry.stack.split(' | ');
    let cat = 'unknown';
    for (const f of frames) {
      if (f.includes('hitStepAccuracy')) { cat = 'accuracy'; break; }
      if (f.includes('getDamage')) { cat = 'crit'; break; }
      if (f.includes('randomizer')) { cat = 'damage_roll'; break; }
      if (f.includes('secondaries')) { cat = 'secondary'; break; }
      if (f.includes('speedSort')) { cat = 'speedSort'; break; }
      if (f.includes('Pokemon') || f.includes('sample')) { cat = 'gender'; break; }
      if (f.includes('flinch') || f.includes('paralyz')) { cat = 'status_check'; break; }
    }
    categories[cat] = (categories[cat] || 0) + 1;
  }
  console.log(`${name}: ${log.length} calls | ${JSON.stringify(categories)}`);
}

async function main() {
  for (const f of fixtures) {
    await traceFixture(f);
  }
}
main();
