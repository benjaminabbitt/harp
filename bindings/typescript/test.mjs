import assert from 'node:assert';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __dirname = dirname(fileURLToPath(import.meta.url));

async function main() {
  const wasmPath = join(__dirname, 'pkg', 'harp_wasm_bg.wasm');
  const wasmBuffer = await readFile(wasmPath);

  const { default: init, generate_name, version } = await import('./pkg/harp_wasm.js');
  await init(wasmBuffer);

  // Test generate_name format
  const name = generate_name();
  const parts = name.split('-');
  assert.strictEqual(parts.length, 3, `name should have 3 parts: ${name}`);
  console.log(`✓ generate_name: ${name}`);

  // Test produces different results
  const names = new Set();
  for (let i = 0; i < 10; i++) {
    names.add(generate_name());
  }
  assert.ok(names.size > 1, 'should produce varied names');
  console.log('✓ produces varied names');

  // Test version
  const v = version();
  assert.ok(v.length > 0, 'version should not be empty');
  console.log(`✓ version: ${v}`);

  console.log('\nAll tests passed!');
}

main().catch(console.error);
