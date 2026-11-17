// Uji API pada database uji terpisah (kinerja_test).
// Jalankan dari root proyek: bun tools/uji/api-smoke.js
const DATABASE_URL = "postgres://kinerja:kinerja_dev@127.0.0.1:5433/kinerja_test";
const PORT = 8091;
const BASE = `http://127.0.0.1:${PORT}/api/v1`;
const AKHIRAN = process.platform === "win32" ? ".exe" : "";
const BIN_SEED = `./target/debug/seed${AKHIRAN}`;
const BIN_API = `./target/debug/kinerja-api${AKHIRAN}`;

// Selalu mulai dari data seed yang bersih agar jumlah node & jadwal deterministik.
console.log("Menyiapkan database uji (seed --reset)...");
const seed = Bun.spawnSync([BIN_SEED, "--reset"], {
	cwd: "backend",
	env: { ...process.env, APP__DATABASE_URL: DATABASE_URL },
	stdout: "inherit",
	stderr: "inherit"
});
if (seed.exitCode !== 0) {
	console.error("Seed database uji gagal. Jalankan `cargo build` di backend terlebih dahulu.");
	process.exit(1);
}

const server = Bun.spawn([BIN_API], {
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

const hasil = [];
function cek(nama, kondisi, detail = "") {
	hasil.push({ nama, lulus: Boolean(kondisi), detail });
	if (!kondisi) console.error(`GAGAL: ${nama} ${detail}`);
	else console.log(`OK: ${nama}`);
}

async function tungguSehat() {
	for (let i = 0; i < 80; i++) {
		try {
			const r = await fetch(`http://127.0.0.1:${PORT}/health`);
			if (r.ok) return;
		} catch {}
		await Bun.sleep(250);
	}
	throw new Error("API tidak siap");
}

async function login(identitas, password, client = "web") {
	const body =
		client === "android"
			? { nrp: identitas, password, client }
			: { username: identitas, password, client };
	const r = await fetch(`${BASE}/auth/login`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify(body)
	});
	const j = await r.json();
	if (!r.ok) throw new Error(`login ${identitas} (${client}) gagal: ${JSON.stringify(j)}`);
	return j;
}

async function cobaLogin(identitas, password, client = "web") {
	const body =
		client === "android"
			? { nrp: identitas, password, client }
			: { username: identitas, password, client };
	const r = await fetch(`${BASE}/auth/login`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify(body)
	});
	return { status: r.status, data: await r.json() };
}

async function panggil(token, path, { method = "GET", body } = {}) {
	const r = await fetch(`${BASE}${path}`, {
		method,
		headers: {
			authorization: `Bearer ${token}`,
			...(body ? { "content-type": "application/json" } : {})
		},
		body: body ? JSON.stringify(body) : undefined
	});
	const teks = await r.text();
	let data = null;
	try {
		data = teks ? JSON.parse(teks) : null;
	} catch {
		data = teks;
	}
	return { status: r.status, data };
}

try {
	await tungguSehat();

	const admin = await login("superuser", "Superuser#2026");
	const adminToken = admin.access_token;
	const adminViaAndroid = await cobaLogin("superuser", "Superuser#2026", "android");
	cek(
		"admin ditolak login lewat Android",
		adminViaAndroid.status === 403,
		`status ${adminViaAndroid.status}`
	);
	const personelViaAndroid = await cobaLogin("85000001", "Polri#2026", "android");
	cek(
		"personel dapat login lewat Android",
		personelViaAndroid.status === 200,
		`status ${personelViaAndroid.status}`
	);
	const semuaSatker = await panggil(adminToken, "/satker");
	cek("admin global melihat 31 node organisasi", semuaSatker.data.length === 31, `dapat ${semuaSatker.data.length}`);
	cek(
		"role_scopes admin global terisi",
		Array.isArray(admin.user.role_scopes) && admin.user.role_scopes.length === 1,
		JSON.stringify(admin.user.role_scopes)
	);
	cek(
		"login web admin memakai username",
		admin.user.jenis_akun === "admin" && admin.user.username === "superuser",
		JSON.stringify({ jenis: admin.user.jenis_akun, username: admin.user.username })
	);
	const daftarPersonel = await panggil(adminToken, "/personel?per_page=500");
	cek(
		"daftar personel tidak memuat akun admin",
		daftarPersonel.data.meta.total === 145,
		`total ${daftarPersonel.data.meta.total}`
	);
	const daftarAdmin = await panggil(adminToken, "/admin");
	cek(
		"halaman akun admin terpisah memuat 5 akun",
		daftarAdmin.status === 200 && daftarAdmin.data.meta.total === 5,
		`status ${daftarAdmin.status} total ${daftarAdmin.data?.meta?.total}`
	);

	const adminPolres = await login("admin_polres", "AdminPolres#2026");
	const periodeAwal = await panggil(adminToken, "/periode");
	const targetPeriode = periodeAwal.data[0];
	const tutup = await panggil(adminToken, `/periode/${targetPeriode.id}/tutup`, {
		method: "POST"
	});
	const setelahTutup = await panggil(adminToken, "/periode");
	const buka = await panggil(adminToken, `/periode/${targetPeriode.id}/buka`, {
		method: "POST",
		body: { alasan: "Uji otomatis" }
	});
	const setelahBuka = await panggil(adminToken, "/periode");
	cek(
		"periode dapat ditutup dan dibuka kembali",
		tutup.status === 200 &&
			buka.status === 200 &&
			setelahTutup.data.find((p) => p.id === targetPeriode.id)?.status === "closed" &&
			setelahBuka.data.find((p) => p.id === targetPeriode.id)?.status === "open" &&
			Boolean(setelahBuka.data.find((p) => p.id === targetPeriode.id)?.dibuka_kembali_at),
		JSON.stringify({ tutup: tutup.status, buka: buka.status })
	);
	const tutupLarang = await panggil(adminPolres.access_token, `/periode/${targetPeriode.id}/tutup`, {
		method: "POST"
	});
	const bukaLarang = await panggil(adminPolres.access_token, `/periode/${targetPeriode.id}/buka`, {
		method: "POST",
		body: { alasan: "x" }
	});
	cek(
		"admin_polres tidak dapat menutup/membuka periode",
		tutupLarang.status === 403 && bukaLarang.status === 403,
		JSON.stringify({ tutup: tutupLarang.status, buka: bukaLarang.status })
	);
	const kesiapanGlobal = await panggil(adminToken, `/periode/${targetPeriode.id}/kesiapan`);
	cek(
		"kesiapan admin global memuat seluruh satker",
		kesiapanGlobal.status === 200 &&
			kesiapanGlobal.data.data.length === 21 &&
			kesiapanGlobal.data.ringkasan.total_satker === 21,
		`status ${kesiapanGlobal.status} jumlah ${kesiapanGlobal.data?.data?.length}`
	);
	const satkerPertama = kesiapanGlobal.data.data[0];
	const tandaiSiap = await panggil(
		adminToken,
		`/periode/${targetPeriode.id}/kesiapan/${satkerPertama.satker_id}`,
		{ method: "POST", body: { siap: true, catatan: "Uji otomatis" } }
	);
	const kesiapanSetelah = await panggil(adminToken, `/periode/${targetPeriode.id}/kesiapan`);
	cek(
		"admin global dapat menandai satker siap",
		tandaiSiap.status === 200 &&
			kesiapanSetelah.data.data.find((s) => s.satker_id === satkerPertama.satker_id)
				?.siap_ditutup === true,
		JSON.stringify(tandaiSiap.data)
	);
	const adminSatker = await login("admin_satker_polda", "AdminSatker#2026");
	const kesiapanSatker = await panggil(
		adminSatker.access_token,
		`/periode/${targetPeriode.id}/kesiapan`
	);
	const satkerSendiri = kesiapanSatker.data?.data?.[0];
	const tandaiSendiri = await panggil(
		adminSatker.access_token,
		`/periode/${targetPeriode.id}/kesiapan/${satkerSendiri.satker_id}`,
		{ method: "POST", body: { siap: true } }
	);
	const satkerLain = kesiapanGlobal.data.data.find(
		(s) => s.satker_id !== satkerSendiri.satker_id
	);
	const tandaiLain = await panggil(
		adminSatker.access_token,
		`/periode/${targetPeriode.id}/kesiapan/${satkerLain.satker_id}`,
		{ method: "POST", body: { siap: true } }
	);
	cek(
		"admin satker hanya melihat & menandai satkernya",
		kesiapanSatker.data?.data?.length === 1 &&
			tandaiSendiri.status === 200 &&
			tandaiLain.status === 403,
		JSON.stringify({
			jumlah: kesiapanSatker.data?.data?.length,
			sendiri: tandaiSendiri.status,
			lain: tandaiLain.status
		})
	);
	const kalender = await panggil(adminToken, "/libur?tahun=2026");
	cek(
		"kalender libur nasional termuat",
		kalender.status === 200 && kalender.data.filter((l) => l.satker_id === null).length >= 5,
		`status ${kalender.status} jumlah ${kalender.data?.length}`
	);
	const liburNasional = await panggil(adminToken, "/libur", {
		method: "POST",
		body: { tanggal: "2026-12-31", nama: "Libur Uji Nasional", jenis: "nasional" }
	});
	const liburGanda = await panggil(adminToken, "/libur", {
		method: "POST",
		body: { tanggal: "2026-12-31", nama: "Libur Uji Ganda", jenis: "nasional" }
	});
	cek(
		"admin global menambah libur nasional dan menolak duplikat",
		liburNasional.status === 200 && liburGanda.status === 409,
		JSON.stringify({ tambah: liburNasional.status, ganda: liburGanda.status })
	);
	const liburPolres = await panggil(adminPolres.access_token, "/libur", {
		method: "POST",
		body: { tanggal: "2026-12-30", nama: "Libur Uji Polres", jenis: "nasional" }
	});
	cek("admin_polres tidak dapat menambah libur nasional", liburPolres.status === 403, `status ${liburPolres.status}`);
	const liburSatker = await panggil(adminSatker.access_token, "/libur", {
		method: "POST",
		body: {
			tanggal: "2026-11-20",
			nama: "Libur Khusus Uji",
			jenis: "khusus",
			satker_id: satkerSendiri.satker_id
		}
	});
	const liburSatkerLain = await panggil(adminSatker.access_token, "/libur", {
		method: "POST",
		body: {
			tanggal: "2026-11-20",
			nama: "Libur Khusus Lain",
			jenis: "khusus",
			satker_id: satkerLain.satker_id
		}
	});
	cek(
		"admin satker menambah libur khusus satkernya & ditolak di satker lain",
		liburSatker.status === 200 && liburSatkerLain.status === 403,
		JSON.stringify({ sendiri: liburSatker.status, lain: liburSatkerLain.status })
	);
	const satkerPolres = await panggil(adminPolres.access_token, "/satker");
	cek(
		"admin_polres hanya melihat Polres + satkernya",
		satkerPolres.data.length === 3,
		JSON.stringify(satkerPolres.data.map((s) => `${s.level}:${s.nama}`))
	);
	const parentPolda = semuaSatker.data.find((s) => s.level === "polda");
	const buatDiPolda = await panggil(adminPolres.access_token, "/satker", {
		method: "POST",
		body: { level: "satker", parent_id: parentPolda.id, nama: "X", kode: "X" }
	});
	cek("admin_polres tidak dapat membuat satker di bawah Polda", buatDiPolda.status === 403, `status ${buatDiPolda.status}`);

	const personel = personelViaAndroid.data;
	const suffix = String(Date.now()).slice(-6);
	const daftarSaya = await panggil(personel.access_token, "/personel");
	cek("personel hanya melihat dirinya sendiri", daftarSaya.status === 200 && daftarSaya.data.data.length === 1);
	const rekapOrang = await panggil(personel.access_token, "/absensi/rekap");
	cek("personel tidak boleh membuka rekap absensi", rekapOrang.status === 403, `status ${rekapOrang.status}`);

	const satkerBaru = await panggil(adminToken, "/satker", {
		method: "POST",
		body: {
			level: "satker",
			parent_id: parentPolda.id,
			nama: "Satker Uji Otomatis",
			kode: `UJI-${suffix}`,
			timezone: "Asia/Jakarta",
			hari_kerja: [1, 2, 3, 4, 5, 6],
			jam_masuk: "07:00",
			jam_pulang: "15:00",
			jam_pulang_sabtu: "12:00"
		}
	});
	cek("admin global membuat satker + jadwal", satkerBaru.status === 200, JSON.stringify(satkerBaru.data));
	cek("hari kerja satker baru tersimpan", satkerBaru.data?.hari_kerja?.length === 6);
	cek(
		"zona waktu dan jam pulang Sabtu tersimpan",
		satkerBaru.data?.timezone === "Asia/Jakarta" && satkerBaru.data?.jam_pulang_sabtu === "12:00",
		JSON.stringify({ tz: satkerBaru.data?.timezone, sabtu: satkerBaru.data?.jam_pulang_sabtu })
	);
	const updateSatker = await panggil(adminToken, `/satker/${satkerBaru.data.id}`, {
		method: "PATCH",
		body: {
			jam_masuk: "06:30",
			jam_pulang: "14:30",
			hari_kerja: [1, 2, 3, 4, 5, 6],
			jam_pulang_sabtu: "10:00"
		}
	});
	cek(
		"jam kerja satker dapat diubah",
		updateSatker.status === 200 &&
			updateSatker.data.jam_masuk === "06:30" &&
			updateSatker.data.jam_pulang_sabtu === "10:00"
	);
	const zonaSalah = await panggil(adminToken, "/satker", {
		method: "POST",
		body: {
			level: "satker",
			parent_id: parentPolda.id,
			nama: "Satker Zona Salah",
			kode: `ZONA-${suffix}`,
			timezone: "Asia/Nowhere"
		}
	});
	cek("zona waktu di luar daftar ditolak", zonaSalah.status === 422, `status ${zonaSalah.status}`);

	const adminBaru = await panggil(adminToken, "/admin", {
		method: "POST",
		body: {
			username: `admin_uji_${suffix}`,
			nama: "Admin Uji Otomatis",
			satker_id: satkerBaru.data.id,
			role: "admin_satker_polda",
			scope_satker_id: satkerBaru.data.id,
			password: "AdminUji#2026"
		}
	});
	cek("admin membuat akun admin satker", adminBaru.status === 200, JSON.stringify(adminBaru.data));
	const adminTerlarang = await panggil(adminPolres.access_token, "/admin", {
		method: "POST",
		body: {
			username: `admin_larang_${suffix}`,
			nama: "Admin Terlarang",
			satker_id: satkerBaru.data.id,
			role: "admin_sistem",
			password: "AdminUji#2026"
		}
	});
	cek("admin_polres tidak dapat membuat akun admin global", adminTerlarang.status === 403, `status ${adminTerlarang.status}`);

	const semuaPersonel = await panggil(adminToken, "/personel?per_page=200");
	const atasanPertama = semuaPersonel.data.data.find((p) => p.nrp === "000100");
	const personelBaru = await panggil(adminToken, "/personel", {
		method: "POST",
		body: {
			nrp: `85${suffix}`,
			nama: "Personel Uji",
			pangkat: "BRIPKA",
			jabatan: "Penyidik",
			satker_id: atasanPertama.satker_id,
			atasan_id: atasanPertama.id,
			password: "Polri#2026",
			roles: [{ role: "personel" }]
		}
	});
	cek("admin membuat personel dengan atasan", personelBaru.status === 200, JSON.stringify(personelBaru.data));
	const rolesBaru = await panggil(adminToken, `/personel/${personelBaru.data.id}/roles`, {
		method: "POST",
		body: { roles: [{ role: "atasan" }, { role: "personel" }] }
	});
	cek("admin dapat mengubah role personel", rolesBaru.status === 200, JSON.stringify(rolesBaru.data));
	const roleTerlarang = await panggil(adminPolres.access_token, `/personel/${personelBaru.data.id}/roles`, {
		method: "POST",
		body: { roles: [{ role: "admin_sistem" }] }
	});
	cek("admin_polres tidak dapat membuat admin global", roleTerlarang.status === 403);

	const jadwalBulan = await panggil(adminToken, "/ph/jadwal?bulan=2026-11");
	cek("jadwal PH admin termuat", jadwalBulan.status === 200 && jadwalBulan.data.length >= 60);
	const strongPoints = await panggil(adminToken, "/ph/strong-point");
	const detailPersonel = await panggil(adminToken, `/personel/${personel.user.id}`);
	const sp = strongPoints.data.find((s) => s.satker_id === detailPersonel.data.satker_id);
	const hariIni = new Date().toISOString().slice(0, 10);
	const jadwalBaru = await panggil(adminToken, "/ph/jadwal", {
		method: "POST",
		body: {
			satker_id: detailPersonel.data.satker_id,
			strong_point_id: sp.id,
			tanggal: hariIni,
			jam_mulai: "09:00",
			jam_selesai: "10:00",
			catatan: "Uji E2E",
			personel: [{ personel_id: personel.user.id, peran: "Anggota" }]
		}
	});
	cek("admin membuat jadwal PH hari ini", jadwalBaru.status === 200, JSON.stringify(jadwalBaru.data));
	const jadwalSaya = await panggil(personel.access_token, "/ph/jadwal/saya?bulan=2026-11");
	cek("personel melihat jadwal PH sendiri", jadwalSaya.status === 200 && jadwalSaya.data.some((j) => j.id === jadwalBaru.data.id));

	const diLuar = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "masuk",
			captured_at: new Date().toISOString(),
			latitude: sp.latitude + 0.1,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-in PH di luar radius ditolak", diLuar.status === 422, `status ${diLuar.status}`);

	const masuk = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "masuk",
			captured_at: new Date().toISOString(),
			latitude: sp.latitude,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-in PH di dalam radius berhasil", masuk.status === 200 && masuk.data.dalam_radius === true, JSON.stringify(masuk.data));
	const ulang = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "masuk",
			captured_at: new Date().toISOString(),
			latitude: sp.latitude,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-in PH ganda ditolak", ulang.status === 409, `status ${ulang.status}`);

	const pulangLuar = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "pulang",
			captured_at: new Date().toISOString(),
			latitude: sp.latitude + 0.1,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-out PH di luar radius ditolak", pulangLuar.status === 422, `status ${pulangLuar.status}`);

	const pulang = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "pulang",
			captured_at: new Date().toISOString(),
			latitude: sp.latitude,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-out PH di dalam radius berhasil", pulang.status === 200, JSON.stringify(pulang.data));
	const rekapPh = await panggil(adminToken, "/ph/rekap?bulan=2026-11");
	cek("rekap PH memuat personel uji", rekapPh.status === 200 && rekapPh.data.data.some((r) => r.personel_id === personel.user.id));
} catch (error) {
	console.error(error);
	hasil.push({ nama: "exception", lulus: false, detail: String(error) });
} finally {
	server.kill();
}

const gagal = hasil.filter((h) => !h.lulus);
console.log(`\n${hasil.length - gagal.length}/${hasil.length} cek lulus`);
if (gagal.length) process.exit(1);
