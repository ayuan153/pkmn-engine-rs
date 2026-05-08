import { BattleStreams, Teams, Dex } from '@pkmn/sim';
import { scenarios } from './teams.js';
import * as fs from 'fs';
import * as path from 'path';

const outputDir = path.resolve(import.meta.dirname, '../../../tests/fixtures/full-info');

interface DamageEvent {
  type: 'damage';
  turn: number;
  source: string;
  target: string;
  move: string;
  damage: number;
  crit: boolean;
  effectiveness: number;
  attacker: {
    species: string; level: number;
    stat_atk: number; stat_spa: number;
    ability: string; item: string;
    boosts: Record<string, number>;
    status: string | null;
  };
  defender: {
    species: string; level: number;
    stat_def: number; stat_spd: number;
    ability: string; item: string;
    boosts: Record<string, number>;
    status: string | null;
    hp_before: number; hp_after: number; max_hp: number;
  };
}

interface Fixture {
  id: string;
  description: string;
  seed: [number, number, number, number];
  format: string;
  p1: { name: string; team: any[] };
  p2: { name: string; team: any[] };
  events: DamageEvent[];
}

const NATURES: Record<string, { plus?: string; minus?: string }> = {
  Adamant: { plus: 'atk', minus: 'spa' }, Jolly: { plus: 'spe', minus: 'spa' },
  Modest: { plus: 'spa', minus: 'atk' }, Timid: { plus: 'spe', minus: 'atk' },
  Bold: { plus: 'def', minus: 'atk' }, Impish: { plus: 'def', minus: 'spa' },
  Calm: { plus: 'spd', minus: 'atk' }, Careful: { plus: 'spd', minus: 'spa' },
  Brave: { plus: 'atk', minus: 'spe' }, Quiet: { plus: 'spa', minus: 'spe' },
  Relaxed: { plus: 'def', minus: 'spe' }, Sassy: { plus: 'spd', minus: 'spe' },
  Naughty: { plus: 'atk', minus: 'spd' }, Lonely: { plus: 'atk', minus: 'def' },
  Mild: { plus: 'spa', minus: 'def' }, Rash: { plus: 'spa', minus: 'spd' },
  Gentle: { plus: 'spd', minus: 'def' }, Hasty: { plus: 'spe', minus: 'def' },
  Lax: { plus: 'def', minus: 'spd' }, Naive: { plus: 'spe', minus: 'spd' },
};

function getNatureMod(nature: string | undefined, stat: string): number {
  if (!nature || !NATURES[nature]) return 1.0;
  const n = NATURES[nature];
  if (n.plus === stat) return 1.1;
  if (n.minus === stat) return 0.9;
  return 1.0;
}

function calcStat(base: number, iv: number, ev: number, level: number, nature: number): number {
  return Math.floor((Math.floor((2 * base + iv + Math.floor(ev / 4)) * level / 100) + 5) * nature);
}

function calcHP(base: number, iv: number, ev: number, level: number): number {
  if (base === 1) return 1; // Shedinja
  return Math.floor((2 * base + iv + Math.floor(ev / 4)) * level / 100) + level + 10;
}

function getStats(teamMon: any, level: number): Record<string, number> {
  const species = Dex.species.get(teamMon.species);
  const baseStats = species.baseStats;
  const ivs = teamMon.ivs || { hp: 31, atk: 31, def: 31, spa: 31, spd: 31, spe: 31 };
  const evs = teamMon.evs || { hp: 0, atk: 0, def: 0, spa: 0, spd: 0, spe: 0 };
  const nature = teamMon.nature;

  return {
    hp: calcHP(baseStats.hp, ivs.hp ?? 31, evs.hp ?? 0, level),
    atk: calcStat(baseStats.atk, ivs.atk ?? 31, evs.atk ?? 0, level, getNatureMod(nature, 'atk')),
    def: calcStat(baseStats.def, ivs.def ?? 31, evs.def ?? 0, level, getNatureMod(nature, 'def')),
    spa: calcStat(baseStats.spa, ivs.spa ?? 31, evs.spa ?? 0, level, getNatureMod(nature, 'spa')),
    spd: calcStat(baseStats.spd, ivs.spd ?? 31, evs.spd ?? 0, level, getNatureMod(nature, 'spd')),
    spe: calcStat(baseStats.spe, ivs.spe ?? 31, evs.spe ?? 0, level, getNatureMod(nature, 'spe')),
  };
}

async function runBattle(scenario: typeof scenarios[0]): Promise<Fixture> {
  const streams = BattleStreams.getPlayerStreams(new BattleStreams.BattleStream());

  const spec = { formatid: 'gen9customgame' as any, seed: scenario.seed };
  const p1spec = { name: 'Player 1', team: Teams.pack(scenario.p1) };
  const p2spec = { name: 'Player 2', team: Teams.pack(scenario.p2) };

  const chunks: string[] = [];
  const omniscientDone = (async () => {
    for await (const chunk of streams.omniscient) { chunks.push(chunk); }
  })();

  streams.omniscient.write(
    `>start ${JSON.stringify(spec)}\n>player p1 ${JSON.stringify(p1spec)}\n>player p2 ${JSON.stringify(p2spec)}`
  );

  // Send team order to get past team preview
  await new Promise(r => setTimeout(r, 50));
  streams.omniscient.write(`>p1 team 1`);
  streams.omniscient.write(`>p2 team 1`);

  for (const [p1choice, p2choice] of scenario.turns) {
    await new Promise(r => setTimeout(r, 50));
    streams.omniscient.write(`>p1 ${p1choice}`);
    streams.omniscient.write(`>p2 ${p2choice}`);
  }

  await new Promise(r => setTimeout(r, 100));
  streams.omniscient.write(`>forcetie`);
  await omniscientDone;

  const events = parseOmniscientLog(chunks.join('\n'), scenario);

  return {
    id: scenario.id, description: scenario.description,
    seed: scenario.seed, format: 'gen9customgame',
    p1: { name: 'Player 1', team: scenario.p1 },
    p2: { name: 'Player 2', team: scenario.p2 },
    events,
  };
}

function parseOmniscientLog(log: string, scenario: typeof scenarios[0]): DamageEvent[] {
  const lines = log.split('\n');
  const events: DamageEvent[] = [];
  let currentTurn = 0;
  let lastMove: { source: string; move: string } | null = null;
  let nextCrit = false;
  let nextEffectiveness = 1.0;

  const pokemon: Record<string, {
    species: string; level: number;
    ability: string; item: string;
    boosts: Record<string, number>;
    status: string | null;
    hp: number; maxHp: number;
    stats: Record<string, number>;
  }> = {};

  for (const line of lines) {
    const parts = line.split('|').slice(1);
    if (!parts.length) continue;
    const cmd = parts[0];

    if (cmd === 'turn') {
      currentTurn = parseInt(parts[1]);
    } else if (cmd === 'switch' || cmd === 'drag') {
      const slot = parts[1].split(':')[0].trim();
      const details = parts[2];
      const hpStr = parts[3];
      const [hp, maxHp] = hpStr.split('/').map(s => parseInt(s));
      const species = details.split(',')[0].trim();
      const levelMatch = details.match(/L(\d+)/);
      const level = levelMatch ? parseInt(levelMatch[1]) : 100;

      const player = slot.startsWith('p1') ? 'p1' : 'p2';
      const team = player === 'p1' ? scenario.p1 : scenario.p2;
      const teamMon = team.find(p => p.species === species);
      const stats = teamMon ? getStats(teamMon, level) : {};

      pokemon[slot] = {
        species, level,
        ability: teamMon?.ability || '',
        item: teamMon?.item || '',
        boosts: { atk: 0, def: 0, spa: 0, spd: 0, spe: 0 },
        status: null, hp, maxHp, stats,
      };
    } else if (cmd === 'move') {
      const source = parts[1].split(':')[0].trim();
      lastMove = { source, move: parts[2] };
    } else if (cmd === '-crit') {
      nextCrit = true;
    } else if (cmd === '-supereffective') {
      nextEffectiveness = 2.0;
    } else if (cmd === '-resisted') {
      nextEffectiveness = 0.5;
    } else if (cmd === '-damage') {
      if (!lastMove) continue;
      const targetSlot = parts[1].split(':')[0].trim();
      const hpStr = parts[2].split(' ')[0];
      const hpAfter = hpStr === '0' || hpStr.startsWith('0 ') ? 0 : parseInt(hpStr.split('/')[0]);

      const target = pokemon[targetSlot];
      const attacker = pokemon[lastMove.source];
      if (!target || !attacker) continue;

      const hpBefore = target.hp;
      const damage = hpBefore - hpAfter;

      if (damage > 0) {
        events.push({
          type: 'damage', turn: currentTurn,
          source: lastMove.source, target: targetSlot,
          move: lastMove.move, damage, crit: nextCrit,
          effectiveness: nextEffectiveness,
          attacker: {
            species: attacker.species, level: attacker.level,
            stat_atk: attacker.stats.atk || 0,
            stat_spa: attacker.stats.spa || 0,
            ability: attacker.ability, item: attacker.item,
            boosts: { ...attacker.boosts }, status: attacker.status,
          },
          defender: {
            species: target.species, level: target.level,
            stat_def: target.stats.def || 0,
            stat_spd: target.stats.spd || 0,
            ability: target.ability, item: target.item,
            boosts: { ...target.boosts }, status: target.status,
            hp_before: hpBefore, hp_after: hpAfter, max_hp: target.maxHp,
          },
        });
      }

      target.hp = hpAfter;
      nextCrit = false;
      nextEffectiveness = 1.0;
    } else if (cmd === '-boost') {
      const slot = parts[1].split(':')[0].trim();
      if (pokemon[slot]) pokemon[slot].boosts[parts[2]] = (pokemon[slot].boosts[parts[2]] || 0) + parseInt(parts[3]);
    } else if (cmd === '-unboost') {
      const slot = parts[1].split(':')[0].trim();
      if (pokemon[slot]) pokemon[slot].boosts[parts[2]] = (pokemon[slot].boosts[parts[2]] || 0) - parseInt(parts[3]);
    } else if (cmd === '-status') {
      const slot = parts[1].split(':')[0].trim();
      if (pokemon[slot]) pokemon[slot].status = parts[2];
    } else if (cmd === '-curestatus') {
      const slot = parts[1].split(':')[0].trim();
      if (pokemon[slot]) pokemon[slot].status = null;
    } else if (cmd === '-heal') {
      const slot = parts[1].split(':')[0].trim();
      if (pokemon[slot]) {
        const hpStr = parts[2].split(' ')[0];
        const hpNow = parseInt(hpStr.split('/')[0]);
        pokemon[slot].hp = hpNow;
      }
    } else if (cmd === '-enditem') {
      const slot = parts[1].split(':')[0].trim();
      if (pokemon[slot]) pokemon[slot].item = '';
    } else if (cmd === '-ability') {
      const slot = parts[1].split(':')[0].trim();
      if (pokemon[slot]) pokemon[slot].ability = parts[2];
    }
  }

  return events;
}

async function main() {
  fs.mkdirSync(outputDir, { recursive: true });
  console.log(`Generating ${scenarios.length} battle fixtures...`);

  for (const scenario of scenarios) {
    try {
      const fixture = await runBattle(scenario);
      const outPath = path.join(outputDir, `${scenario.id}.json`);
      fs.writeFileSync(outPath, JSON.stringify(fixture, null, 2));
      console.log(`  ✓ ${scenario.id}: ${fixture.events.length} damage events`);
    } catch (e) {
      console.error(`  ✗ ${scenario.id}: ${e}`);
    }
  }

  console.log('Done.');
}

main();
