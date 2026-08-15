#!/usr/bin/env node
'use strict';
const fs = require('fs');
const path = require('path');

const SOURCE = path.join(__dirname, '../extdeps/cpu/altra_max_rev_a1_table4_pins.json');
const OUT = path.join(__dirname, '../extdeps/cpu/ampere_altra_package_table4_raw.dag');

function main() {
  const pins = JSON.parse(fs.readFileSync(SOURCE, 'utf8'));
  const entries = Object.entries(pins);
  if (entries.length !== 4926) {
    console.error('Expected 4926 contacts, got', entries.length);
    process.exit(1);
  }
  const sorted = entries.sort((a, b) => a[0].localeCompare(b[0], undefined, { numeric: true }));
  const lines = sorted.map(([designator, signal]) => {
    return `  Table4PinRow { designator: "${designator}", signal_name: "${signal}" },`;
  });
  const body = `module extdeps.cpu.ampere_altra_package_table4_raw

import std.types { NonEmptyStr, List }
import extdeps.cpu.ampere_altra_package { Table4PinRow }

// TRANSCRIBED designator/signal rows from Ampere Altra Max Rev A1 datasheet Issue 1.15, section 5 Table 4.
// Source: extdeps/cpu/altra_max_rev_a1_table4_pins.json — regenerate via node dag/tools/generate_altra_table4_raw.js
data altra_table4_pin_rows: List<Table4PinRow> = [
${lines.join('\n')}
]
`;
  fs.writeFileSync(OUT, body);
  console.log('Wrote', OUT);
}

main();
