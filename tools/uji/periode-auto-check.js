// Verifikasi periode bulan berjalan otomatis.
// PERINGATAN: skrip ini menghapus rencana_kerja & periode pada database uji
// (kinerja_test), lalu membuka dashboard untuk memicu pembuatan otomatis.
// Jalankan dari root proyek: bun tools/uji/periode-auto-check.js
const DATABASE_URL = "postgres://kinerja:kinerja_dev@127.0.0.1:5433/kinerja_test";
const PORT = 8092;
const BASE = `http://127.0.0.1:${PORT}/api/v1`;

const hapus = Bun.spawnSync([
	"podman",
	"exec",
	"kinerja_db",
	"psql",
	"-U",
	"kinerja",
	"-d",
	"kinerja_test",
	"-c",
	"DELETE FROM rencana_kerja; DELETE FROM periode;"
]);
if (hapus.exitCode !== 0) {
	console.error("Gagal membersihkan periode:", hapus.stderr.toString());
	process.exit(1);
}

const server = Bun.spawn(["./target/debug/kinerja-api.exe"], {
	cwd: "backend",
	env: {
		...process.env,
		APP__DATABASE_URL: DATABASE_URL,
		APP__PORT: String(PORT),
		APP__RUN_MIGRATIONS: "false",
		APP__UPLOAD_DIR: "./data-test/uploads"
	},
	stdout: "inherit",
	stderr: "inherit"
});

try {
	for (let i = 0; i < 80; i++) {
		try {
			if ((await fetch(`http://127.0.0.1:${PORT}/health`)).ok) break;
		} catch {}
		await Bun.sleep(250);
	}
	const login = await fetch(`${BASE}/auth/login`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ username: "superuser", password: "Superuser#2026", client: "web" })
	});
	const sesi = await login.json();
	if (!login.ok) throw new Error(`login gagal: ${JSON.stringify(sesi)}`);

	const dash = await fetch(`${BASE}/dashboard/ringkasan`, {
		headers: { authorization: `Bearer ${sesi.access_token}` }
	});
	const data = await dash.json();
	console.log("dashboard:", dash.status, data.periode_nama ?? data.error);

	const daftar = await fetch(`${BASE}/periode`, {
		headers: { authorization: `Bearer ${sesi.access_token}` }
	});
	const periode = await daftar.json();
	console.log(
		"periode:",
		daftar.status,
		periode.map((p) => `${p.nama} [${p.status}] ${p.tanggal_mulai}..${p.tanggal_selesai}`)
	);
} finally {
	server.kill();
}
