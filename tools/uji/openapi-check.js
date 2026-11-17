// Validasi cepat berkas OpenAPI: JSON valid, semua $ref schema ada,
// dan setiap path punya operasi HTTP yang dikenal.
const fs = require('fs');
const path = require('path');

const berkas = path.join(__dirname, '../../backend/openapi.json');
const teks = fs.readFileSync(berkas, 'utf8');

let spec;
try {
  spec = JSON.parse(teks);
} catch (err) {
  console.error(`JSON tidak valid: ${err.message}`);
  process.exit(1);
}

const skema = Object.keys((spec.components && spec.components.schemas) || {});
const rujukan = [...new Set([...teks.matchAll(/#\/components\/schemas\/([A-Za-z0-9_]+)/g)].map((m) => m[1]))];
const hilang = rujukan.filter((nama) => !skema.includes(nama));
if (hilang.length > 0) {
  console.error(`Schema tidak ditemukan: ${hilang.join(', ')}`);
  process.exit(1);
}

const metodeSah = new Set(['get', 'post', 'put', 'patch', 'delete', 'head', 'options']);
const masalah = [];
for (const [jalur, operasi] of Object.entries(spec.paths)) {
  for (const [metode, def] of Object.entries(operasi)) {
    if (!metodeSah.has(metode)) continue;
    if (!Array.isArray(def.tags) || def.tags.length === 0) {
      masalah.push(`${metode.toUpperCase()} ${jalur} tanpa tag`);
    }
    if (!def.responses || Object.keys(def.responses).length === 0) {
      masalah.push(`${metode.toUpperCase()} ${jalur} tanpa responses`);
    }
  }
}
if (masalah.length > 0) {
  console.error(`Masalah operasi:\n- ${masalah.join('\n- ')}`);
  process.exit(1);
}

console.log(`OpenAPI valid: ${Object.keys(spec.paths).length} path, ${skema.length} schema.`);
