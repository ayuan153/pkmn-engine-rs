import { BattleStreams, Teams, PRNG, Battle } from '@pkmn/sim';

const seed: [number, number, number, number] = [100, 200, 300, 400];

// Patch Battle.prototype to trace modify calls
const origModify = (Battle.prototype as any).modify;
(Battle.prototype as any).modify = function(value: number, numerator: any, denominator: number = 1) {
  const result = origModify.call(this, value, numerator, denominator);
  if (typeof value === 'number' && value > 50) {
    const num = Array.isArray(numerator) ? numerator[0] : numerator;
    const den = Array.isArray(numerator) ? numerator[1] : denominator;
    console.log(`  modify(${value}, ${num}/${den}) = ${result}`);
  }
  return result;
};

// Patch randomizer
const origRandomizer = (Battle.prototype as any).randomizer;
(Battle.prototype as any).randomizer = function(damage: number) {
  const result = origRandomizer.call(this, damage);
  console.log(`  randomizer(${damage}) = ${result} [roll=${100 - (this as any).lastDamageRoll}]`);
  return result;
};

// Patch random to track damage roll
const origRandom = (Battle.prototype as any).random;
let lastRandom = 0;
(Battle.prototype as any).random = function(m?: number, n?: number) {
  const result = origRandom.call(this, m, n);
  lastRandom = result;
  (this as any).lastDamageRoll = result;
  return result;
};

const stream = new BattleStreams.BattleStream();

const p1team = Teams.pack([{
  name: 'Tapu Koko', species: 'Tapu Koko', item: 'Life Orb', ability: 'Electric Surge',
  moves: ['Thunderbolt', 'Dazzling Gleam', 'U-turn', 'Roost'],
  nature: 'Timid', evs: { hp: 0, atk: 0, def: 0, spa: 252, spd: 4, spe: 252 },
  ivs: { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 }, level: 100, gender: '',
} as any]);

const p2team = Teams.pack([{
  name: 'Scizor', species: 'Scizor', item: 'Choice Band', ability: 'Technician',
  moves: ['Bullet Punch', 'U-turn', 'Knock Off', 'Superpower'],
  nature: 'Adamant', evs: { hp: 0, atk: 252, def: 0, spa: 0, spd: 4, spe: 252 },
  ivs: { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 }, level: 100, gender: 'M',
} as any]);

(async () => {
  const allOutput: string[] = [];
  const readerDone = (async () => {
    for await (const chunk of stream) {
      for (const line of chunk.split('\n')) {
        if (line.trim()) allOutput.push(line);
      }
    }
  })();

  stream.write(`>start {"formatid":"gen9customgame","seed":${JSON.stringify(seed)}}\n>player p1 {"name":"Player 1","team":"${p1team}"}\n>player p2 {"name":"Player 2","team":"${p2team}"}`);
  await new Promise(r => setTimeout(r, 200));
  stream.write('>p1 default\n>p2 default');
  await new Promise(r => setTimeout(r, 200));
  
  console.log('=== Turn 1 ===');
  stream.write('>p1 move 1\n>p2 move 1');
  await new Promise(r => setTimeout(r, 500));
  
  try { stream.pushEnd(); } catch {}
  await readerDone;

  console.log('\n=== Protocol ===');
  for (const line of allOutput) {
    if (line.startsWith('|move|') || line.startsWith('|-damage|') || line.startsWith('|faint|') || line.startsWith('|switch|') || line.startsWith('|-fieldstart|') || line.startsWith('|turn|') || line.startsWith('|win|')) {
      console.log(line);
    }
  }
})();
