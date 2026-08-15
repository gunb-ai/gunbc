#!/usr/bin/env node
'use strict';
const fs = require('fs');
const path = require('path');

const SOURCE = path.join(__dirname, '../extdeps/cpu/altra_max_rev_a1_table4_pins.json');
const OUT = path.join(__dirname, '../extdeps/cpu/ampere_altra_package_contacts.dag');

function classifyFunction(signal, signals) {
  if (/^RESERVED|^RFU/i.test(signal)) return 'ReservedContact';
  if (signal === 'NC') return 'NoConnectContact';
  if (/^VSS$|^GND$/.test(signal) || /^VSS_/.test(signal)) return 'PowerReturnContact';
  if (/^AGND$|^DGND$/.test(signal)) return 'SignalReferenceContact';
  if (/^VDD|^VCC|^VPP|^VREF|^VTT|^VREG|^PVDD|^AVDD|^DVDD|^VDDQ|^VDD18|^VDDH|^VDDP|^VDD_/.test(signal)) {
    return 'PowerSupplyContact';
  }
  const m = signal.match(/^(.*)_(P|N)$/);
  if (m && signals.has(m[1] + '_P') && signals.has(m[1] + '_N')) {
    return `DifferentialPairMemberContact { pair: "${m[1]}" }`;
  }
  return 'SingleEndedSignalContact';
}

function main() {
  const pins = JSON.parse(fs.readFileSync(SOURCE, 'utf8'));
  const entries = Object.entries(pins);
  if (entries.length !== 4926) {
    console.error('Expected 4926 contacts, got', entries.length);
    process.exit(1);
  }
  const signals = new Set(Object.values(pins));
  const sorted = entries.sort((a, b) => a[0].localeCompare(b[0], undefined, { numeric: true }));
  const lines = sorted.map(([designator, signal]) => {
    const fn = classifyFunction(signal, signals);
    return `  PackageContact { designator: "${designator}", function: ${fn} },`;
  });
  const body = `module extdeps.cpu.ampere_altra_package_contacts

import std.types { NonEmptyStr, List }
import extdeps.cpu.ampere_altra_package { PackageContact }

// TRANSCRIBED from Ampere Altra Max Rev A1 datasheet Issue 1.15, section 5 Table 4 (pin assignment).
// Source rows: extdeps/cpu/altra_max_rev_a1_table4_pins.json
// Regenerate: node dag/tools/generate_altra_contact_map.js
data altra_package_contacts: List<PackageContact> = [
${lines.join('\n')}
]
`;
  fs.writeFileSync(OUT, body);
  console.log('Wrote', OUT);
}

main();
