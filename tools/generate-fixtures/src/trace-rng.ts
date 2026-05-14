import { BattleStreams, Teams } from '@pkmn/sim';
import * as fs from 'fs';

// The ESM PRNG is already patched (we modified prng.mjs directly)
// Just enable the global log
(globalThis as any)._prngLog = [];
(globalThis as any)._prngCallCount = 0;

const fixture = JSON.parse(fs.readFileSync('../../tests/fixtures/full-sim/blaze-pinch.json', 'utf8'));

async function run() {
  const stream = new BattleStreams.BattleStream();
  const streams = BattleStreams.getPlayerStreams(stream);

  const allOutput: string[] = [];
  const done = (async () => {
    for await (const chunk of streams.omniscient) {
      for (const line of chunk.split('\n')) if (line.trim()) allOutput.push(line);
    }
  })();

  streams.omniscient.write(`>start {"formatid":"gen9customgame","seed":[${fixture.seed}]}`);
  streams.omniscient.write(`>player p1 {"name":"Player 1","team":"${Teams.pack(fixture.p1.team)}"}`);
  streams.omniscient.write(`>player p2 {"name":"Player 2","team":"${Teams.pack(fixture.p2.team)}"}`);
  await new Promise(r => setTimeout(r, 200));
  streams.omniscient.write('>p1 team 1');
  streams.omniscient.write('>p2 team 1');
  await new Promise(r => setTimeout(r, 200));

  for (const [p1c, p2c] of fixture.choices) {
    streams.omniscient.write(`>p1 ${p1c}`);
    streams.omniscient.write(`>p2 ${p2c}`);
    await new Promise(r => setTimeout(r, 200));
  }

  // Wait for all choices to process, then forcetie
  await new Promise(r => setTimeout(r, 500));
  streams.omniscient.write('>forcetie');
  await done;

  const log = (globalThis as any)._prngLog;
  console.log(`=== PS PRNG LOG (${log.length} calls) ===`);
  for (const entry of log) {
    const frames = entry.stack.split(' | ')
      .filter((l: string) => !l.includes('/prng') && !l.includes('PRNG.'))
      .map((l: string) => {
        const m = l.match(/at (?:(\w+)\.)?(\w+) .*?([^/]+\.mjs):(\d+)/);
        return m ? `${m[1]||''}${m[1]?'.':''}${m[2]}@${m[3]}:${m[4]}` : l.slice(3, 60);
      });
    console.log(`  #${entry.call}: 0x${entry.value.toString(16).padStart(8,'0')} ${frames.slice(0,3).join(' <- ')}`);
  }
}

run();
