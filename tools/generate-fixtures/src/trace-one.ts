import { BattleStreams, Teams } from '@pkmn/sim';
import * as fs from 'fs';
(globalThis as any)._prngLog = [];
(globalThis as any)._prngCallCount = 0;

const name = process.argv[2] || 'substitute-protect-toxic';
const fixture = JSON.parse(fs.readFileSync(`../../tests/fixtures/full-sim/${name}.json`, 'utf8'));

async function run() {
  const stream = new BattleStreams.BattleStream();
  const streams = BattleStreams.getPlayerStreams(stream);
  const chunks: string[] = [];
  const done = (async () => { for await (const c of streams.omniscient) { chunks.push(c); } })();

  streams.omniscient.write(`>start {"formatid":"gen9customgame","seed":[${fixture.seed}]}`);
  streams.omniscient.write(`>player p1 {"name":"P1","team":"${Teams.pack(fixture.p1.team)}"}`);
  streams.omniscient.write(`>player p2 {"name":"P2","team":"${Teams.pack(fixture.p2.team)}"}`);
  await new Promise(r => setTimeout(r, 500));
  streams.omniscient.write('>p1 team 1');
  streams.omniscient.write('>p2 team 1');
  await new Promise(r => setTimeout(r, 500));

  for (let i = 0; i < fixture.choices.length; i++) {
    const [p1c, p2c] = fixture.choices[i];
    streams.omniscient.write(`>p1 ${p1c}`);
    streams.omniscient.write(`>p2 ${p2c}`);
    await new Promise(r => setTimeout(r, 500));
  }
  streams.omniscient.write('>forcetie');
  await new Promise(r => setTimeout(r, 500));
  await done;

  const log = (globalThis as any)._prngLog;
  console.log(`${name}: ${log.length} total RNG calls`);
  for (const e of log) {
    const frames = e.stack.split(' | ').filter((f:string) => !f.includes('prng')).slice(0,3);
    const ctx = frames.map((f:string) => { const m = f.match(/at (?:\w+\.)?(\w+)/); return m?m[1]:'?'; }).join('<-');
    console.log(`  #${e.call}: 0x${e.value.toString(16).padStart(8,'0')} ${ctx}`);
  }
}
run();
