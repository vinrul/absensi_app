// Uji API pada database uji terpisah (kinerja_test).
// Jalankan dari root proyek: bun tools/uji/api-smoke.js
const DATABASE_URL = "postgres://kinerja:kinerja_dev@127.0.0.1:5433/kinerja_test";
const PORT = 8091;
const BASE = `http://127.0.0.1:${PORT}/api/v1`;
const FOTO_PNG =
	"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";
const AKHIRAN = process.platform === "win32" ? ".exe" : "";
const BIN_SEED = `./target/debug/seed${AKHIRAN}`;
const BIN_API = `./target/debug/kinerja-api${AKHIRAN}`;

// Selalu mulai dari data seed yang bersih agar jumlah node & jadwal deterministik.
console.log("Menyiapkan database uji (seed --reset)...");
const seed = Bun.spawnSync([BIN_SEED, "--reset"], {
	cwd: "backend",
	env: {
		...process.env,
		APP__DATABASE_URL: DATABASE_URL,
		APP__UPLOAD_DIR: "./data-test/uploads",
		// Database uji memakai contoh jadwal 3–5 agar skenario jadwal hari ini tidak bentrok.
		APP__SEED_FULL_ROSTER: "false"
	},
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
		APP__RUN_MIGRATIONS: "true",
		APP__UPLOAD_DIR: "./data-test/uploads",
		APP__ALLOW_FAKE_TIME: "true"
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

async function kirimLogin(body) {
	let r = await fetch(`${BASE}/auth/login`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify(body)
	});
	if (r.status === 429) {
		const detik = Number(r.headers.get("retry-after") ?? "61");
		await Bun.sleep((detik + 1) * 1000);
		r = await fetch(`${BASE}/auth/login`, {
			method: "POST",
			headers: { "content-type": "application/json" },
			body: JSON.stringify(body)
		});
	}
	return r;
}

async function login(identitas, password, client = "web") {
	const body =
		client === "android" || client === "personel_web"
			? { nrp: identitas, password, client }
			: { username: identitas, password, client };
	const r = await kirimLogin(body);
	const j = await r.json();
	if (!r.ok) throw new Error(`login ${identitas} (${client}) gagal: ${JSON.stringify(j)}`);
	return j;
}

async function cobaLogin(identitas, password, client = "web") {
	const body =
		client === "android" || client === "personel_web"
			? { nrp: identitas, password, client }
			: { username: identitas, password, client };
	const r = await kirimLogin(body);
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

	const waktuRes = await fetch(`${BASE}/waktu`);
	const waktu = await waktuRes.json();
	cek(
		"jam server tersedia tanpa token",
		waktuRes.status === 200 &&
			typeof waktu.epoch_ms === "number" &&
			waktu.zona === "Asia/Makassar",
		`status ${waktuRes.status}`
	);

	const admin = await login("superuser", "Superuser#2026");
	const adminToken = admin.access_token;

	const matriks = await panggil(
		adminToken,
		"/absensi/matriks?dari=2026-11-01&sampai=2026-11-30"
	);
	const adaLokasi = (matriks.data?.data ?? []).some((p) =>
		Object.values(p.hari ?? {}).some((h) => h.masuk_latitude !== null)
	);
	cek(
		"matriks kehadiran 30 hari + koordinat",
		matriks.status === 200 &&
			matriks.data?.tanggal?.length === 30 &&
			Array.isArray(matriks.data?.data) &&
			adaLokasi,
		`status ${matriks.status}`
	);
	const selMatriks = (matriks.data?.data ?? []).flatMap((p) =>
		Object.entries(p.hari ?? {}).map(([tanggal, h]) => ({ tanggal, ...h }))
	);
	const selTa = selMatriks.filter((h) => h.tidak_absen === true);
	const taRapi = selTa.every((h) => {
		const hari = new Date(`${h.tanggal}T00:00:00`).getDay();
		return (
			hari !== 0 &&
			hari !== 6 &&
			h.tanggal <= waktu.tanggal &&
			h.masuk_jam === null &&
			h.pulang_jam === null
		);
	});
	cek(
		"matriks memuat flag TA yang konsisten",
		selMatriks.length > 0 &&
			selMatriks.every((h) => typeof h.tidak_absen === "boolean") &&
			taRapi,
		`${selTa.length} sel TA dari ${selMatriks.length} sel`
	);
	const matriksPanjang = await panggil(
		adminToken,
		"/absensi/matriks?dari=2026-10-01&sampai=2026-11-30"
	);
	cek(
		"matriks lebih dari 31 hari ditolak",
		matriksPanjang.status === 422,
		`status ${matriksPanjang.status}`
	);
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
	const atasanViaWeb = await cobaLogin("000100", "Atasan#2026", "web");
	cek(
		"atasan langsung ditolak login dashboard",
		atasanViaWeb.status === 403,
		`status ${atasanViaWeb.status}`
	);
	const personelViaWeb = await cobaLogin("85000001", "Polri#2026", "web");
	cek(
		"personel ditolak login dashboard",
		personelViaWeb.status === 403,
		`status ${personelViaWeb.status}`
	);
	const personelViaWebPersonel = await cobaLogin("85000001", "Polri#2026", "personel_web");
	cek(
		"personel dapat login lewat web personel",
		personelViaWebPersonel.status === 200,
		`status ${personelViaWebPersonel.status}`
	);
	const atasanViaWebPersonel = await cobaLogin("000100", "Atasan#2026", "personel_web");
	cek(
		"atasan dapat login lewat web personel",
		atasanViaWebPersonel.status === 200,
		`status ${atasanViaWebPersonel.status}`
	);
	const adminViaWebPersonel = await cobaLogin("superuser", "Superuser#2026", "personel_web");
	cek(
		"admin ditolak login lewat web personel",
		adminViaWebPersonel.status === 403,
		`status ${adminViaWebPersonel.status}`
	);
	const pimpinanViaWeb = await cobaLogin("000002", "Pimpinan#2026", "web");
	cek(
		"pimpinan dapat login dashboard",
		pimpinanViaWeb.status === 200 &&
			Array.isArray(pimpinanViaWeb.data.user?.roles) &&
			pimpinanViaWeb.data.user.roles.some((role) => String(role).includes("pimpinan")),
		`status ${pimpinanViaWeb.status}`
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

	const titikBaru = await panggil(adminToken, "/titik-dinas", {
		method: "POST",
		body: {
			satker_id: satkerBaru.data.id,
			nama: "Titik Uji Peta",
			jenis: "kantor",
			latitude: -8.65,
			longitude: 115.22,
			radius_m: 150
		}
	});
	cek(
		"admin membuat titik dinas",
		titikBaru.status === 200 && titikBaru.data?.radius_m === 150,
		JSON.stringify(titikBaru.data)
	);
	const titikUbah = await panggil(adminToken, `/titik-dinas/${titikBaru.data.id}`, {
		method: "PATCH",
		body: {
			nama: "Titik Uji Peta Baru",
			jenis: "luar",
			latitude: -8.66,
			longitude: 115.23,
			radius_m: 250,
			is_active: false
		}
	});
	cek(
		"titik dinas dapat diubah termasuk jenis & koordinat",
		titikUbah.status === 200 &&
			titikUbah.data?.nama === "Titik Uji Peta Baru" &&
			titikUbah.data?.jenis === "luar" &&
			titikUbah.data?.latitude === -8.66 &&
			titikUbah.data?.longitude === 115.23 &&
			titikUbah.data?.radius_m === 250 &&
			titikUbah.data?.is_active === false,
		JSON.stringify(titikUbah.data)
	);
	const titikHapus = await panggil(adminToken, `/titik-dinas/${titikBaru.data.id}`, {
		method: "DELETE"
	});
	cek("titik dinas dapat dihapus", titikHapus.status === 200, `status ${titikHapus.status}`);
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

	// Personel baru belum punya absensi: hari kerja lampau harus ditandai TA.
	const matriksBaru = await panggil(
		adminToken,
		`/absensi/matriks?dari=2026-11-01&sampai=2026-11-30&personel_id=${personelBaru.data.id}`
	);
	const hariBaru = Object.entries(matriksBaru.data?.data?.[0]?.hari ?? {});
	const taBaru = hariBaru.filter(([, h]) => h.tidak_absen === true);
	cek(
		"personel tanpa absensi ditandai TA pada hari kerja lampau",
		matriksBaru.status === 200 &&
			hariBaru.length > 0 &&
			taBaru.length > 0 &&
			taBaru.every(([tanggal, h]) => {
				const hari = new Date(`${tanggal}T00:00:00`).getDay();
				return (
					hari !== 0 &&
					hari !== 6 &&
					tanggal <= waktu.tanggal &&
					h.masuk_jam === null &&
					h.pulang_jam === null
				);
			}),
		`${taBaru.length} hari TA dari ${hariBaru.length} sel`
	);

	// Penempatan tetap: satu anggota = satu titik, tanpa tanggal.
	const satkerPh = personelBaru.data.satker_id;
	const titikInduk = await panggil(adminToken, `/ph/strong-point?satker_id=${satkerPh}`);
	cek("satker memakai titik induk polda/polres", titikInduk.status === 200 && titikInduk.data.length > 0);
	const tempatPh = await panggil(adminToken, "/ph/penempatan", {
		method: "POST",
		body: {
			satker_id: satkerPh,
			items: [{ personel_id: personelBaru.data.id, strong_point_id: titikInduk.data[0].id }]
		}
	});
	cek("admin menempatkan anggota ke titik", tempatPh.status === 200, JSON.stringify(tempatPh.data));
	const tempatBaca = await panggil(adminToken, `/ph/penempatan?satker_id=${satkerPh}`);
	const barisTempat = tempatBaca.data.filter((p) => p.personel_id === personelBaru.data.id);
	cek(
		"penempatan anggota terbaca tepat satu baris",
		barisTempat.length === 1 && barisTempat[0].strong_point_id === titikInduk.data[0].id,
		JSON.stringify(barisTempat)
	);
	const tempatUlang = await panggil(adminToken, "/ph/penempatan", {
		method: "POST",
		body: {
			satker_id: satkerPh,
			items: [{ personel_id: personelBaru.data.id, strong_point_id: titikInduk.data[0].id }]
		}
	});
	const tempatBacaUlang = await panggil(adminToken, `/ph/penempatan?satker_id=${satkerPh}`);
	cek(
		"simpan ulang penempatan tidak menggandakan baris",
		tempatUlang.status === 200 &&
			tempatBacaUlang.data.filter((p) => p.personel_id === personelBaru.data.id).length === 1
	);

	const statusBulan = await panggil(
		adminToken,
		`/ph/jadwal/status?bulan=2026-11&satker_id=${satkerPh}`
	);
	cek(
		"status jadwal memberi peringatan susun bulan depan",
		statusBulan.status === 200 &&
			statusBulan.data.jumlah_jadwal > 0 &&
			statusBulan.data.perlu_susun_bulan_depan === true,
		JSON.stringify(statusBulan.data)
	);

	const salin = await panggil(adminToken, "/ph/jadwal/salin", {
		method: "POST",
		body: { satker_id: satkerPh, bulan: "2026-12" }
	});
	cek(
		"salin jadwal bulan sebelumnya",
		salin.status === 200 && salin.data.dibuat > 0,
		JSON.stringify(salin.data)
	);
	const salinUlang = await panggil(adminToken, "/ph/jadwal/salin", {
		method: "POST",
		body: { satker_id: satkerPh, bulan: "2026-12" }
	});
	cek("salin ulang tanpa timpa ditolak", salinUlang.status === 409, `status ${salinUlang.status}`);
	const jadwalDes = await panggil(
		adminToken,
		`/ph/jadwal?dari=2026-12-01&sampai=2026-12-31&satker_id=${satkerPh}`
	);
	cek(
		"jadwal bulan baru hasil salinan tidak ditandai fallback",
		jadwalDes.status === 200 &&
			jadwalDes.data.length > 0 &&
			jadwalDes.data.every((j) => j.disalin_dari_bulan === null),
		JSON.stringify(jadwalDes.data?.slice(0, 2))
	);
	const statusDes = await panggil(
		adminToken,
		`/ph/jadwal/status?bulan=2026-12&satker_id=${satkerPh}`
	);
	cek(
		"status bulan baru tersusun tanpa peringatan salinan",
		statusDes.status === 200 &&
			statusDes.data.jumlah_jadwal > 0 &&
			statusDes.data.disalin_dari_bulan === null,
		JSON.stringify(statusDes.data)
	);
	const matriksDepan = await panggil(
		adminToken,
		`/absensi/matriks?dari=2026-12-01&sampai=2026-12-31&satker_id=${satkerPh}`
	);
	const selDepan = (matriksDepan.data?.data ?? []).flatMap((p) =>
		Object.values(p.hari ?? {})
	);
	cek(
		"jadwal strong point mendatang tidak ditandai TA",
		matriksDepan.status === 200 &&
			selDepan.length > 0 &&
			selDepan.every((h) => h.jadwal_strong === true && h.tidak_absen === false),
		`${selDepan.length} sel jadwal mendatang`
	);

	const jadwalRentang = await panggil(adminToken, "/ph/jadwal?dari=2026-11-01&sampai=2026-11-30");
	cek("jadwal PH admin termuat", jadwalRentang.status === 200 && jadwalRentang.data.length >= 60);
	const detailPersonel = await panggil(adminToken, `/personel/${personel.user.id}`);
	const hariIni = new Date().toISOString().slice(0, 10);
	const jadwalBaru = await panggil(adminToken, "/ph/jadwal", {
		method: "POST",
		body: {
			satker_id: detailPersonel.data.satker_id,
			tanggal: hariIni,
			jam_mulai: "09:00",
			jam_selesai: "10:00",
			catatan: "Uji E2E",
			personel: [{ personel_id: personel.user.id, peran: "Anggota" }]
		}
	});
	cek("admin membuat jadwal PH hari ini", jadwalBaru.status === 200, JSON.stringify(jadwalBaru.data));
	const tempatPersonel = await panggil(
		adminToken,
		`/ph/penempatan?satker_id=${detailPersonel.data.satker_id}`
	);
	const titikTerpasang = tempatPersonel.data.find((p) => p.personel_id === personel.user.id)?.strong_point_id;
	cek(
		"jadwal memakai titik dari penempatan anggota",
		Boolean(titikTerpasang) && jadwalBaru.data?.strong_point_id === titikTerpasang,
		`${jadwalBaru.data?.strong_point_id} vs ${titikTerpasang}`
	);
	const jadwalSaya = await panggil(personel.access_token, "/ph/jadwal/saya?dari=2026-11-01&sampai=2026-11-30");
	cek("personel melihat jadwal PH sendiri", jadwalSaya.status === 200 && jadwalSaya.data.some((j) => j.id === jadwalBaru.data.id));
	// Koordinat check-in harus mengikuti titik pada jadwal yang dibuat server.
	const sp = jadwalSaya.data.find((j) => j.id === jadwalBaru.data.id) ?? { latitude: 0, longitude: 0 };

	const diLuar = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "masuk",
			captured_at: `${hariIni}T01:05:00Z`,
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
			captured_at: `${hariIni}T01:10:00Z`,
			latitude: sp.latitude,
			longitude: sp.longitude,
			akurasi_m: 5,
			foto_base64: FOTO_PNG
		}
	});
	cek("check-in PH di dalam radius berhasil", masuk.status === 200 && masuk.data.dalam_radius === true, JSON.stringify(masuk.data));
	cek("check-in PH mencatat telat 10 menit", masuk.data?.menit_telat === 10, JSON.stringify(masuk.data?.menit_telat));
	const ulang = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "masuk",
			captured_at: `${hariIni}T01:15:00Z`,
			latitude: sp.latitude,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-in PH ganda ditolak", ulang.status === 409, `status ${ulang.status}`);

	const masukKedua = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "masuk",
			captured_at: `${hariIni}T02:15:00Z`,
			latitude: sp.latitude,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-in sesi baru pada jadwal yang sama ditolak", masukKedua.status === 409, JSON.stringify(masukKedua.data));

	const pulangLuar = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "pulang",
			captured_at: `${hariIni}T01:25:00Z`,
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
			captured_at: `${hariIni}T01:30:00Z`,
			latitude: sp.latitude,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-out PH di dalam radius berhasil", pulang.status === 200, JSON.stringify(pulang.data));
	cek("check-out PH mencatat pulang cepat 30 menit", pulang.data?.menit_pulang_cepat === 30, JSON.stringify(pulang.data?.menit_pulang_cepat));
	const matriksPh = await panggil(
		adminToken,
		`/absensi/matriks?dari=${hariIni}&sampai=${hariIni}&personel_id=${personel.user.id}`
	);
	const selPh = matriksPh.data?.data?.find((p) => p.personel_id === personel.user.id)
		?.hari?.[hariIni];
	cek(
		"matriks menampilkan sesi strong point dan realisasinya",
		matriksPh.status === 200 &&
			selPh?.jadwal_strong === true &&
			selPh?.ph_absen === true &&
			selPh?.sesi_strong?.length === 1 &&
			selPh.sesi_strong[0].sesi === "pagi" &&
			selPh.sesi_strong[0].masuk_jam === "09:10" &&
			selPh.sesi_strong[0].pulang_jam === "09:30",
		JSON.stringify(selPh?.sesi_strong)
	);
	const pulangUlang = await panggil(personel.access_token, "/ph/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jadwal_ph_id: jadwalBaru.data.id,
			jenis: "pulang",
			captured_at: `${hariIni}T01:35:00Z`,
			latitude: sp.latitude,
			longitude: sp.longitude,
			akurasi_m: 5
		}
	});
	cek("check-out ganda ditolak", pulangUlang.status === 409, `status ${pulangUlang.status}`);

	const fotoPh = await panggil(adminToken, `/ph/foto?dari=${hariIni}&sampai=${hariIni}&personel_id=${personel.user.id}`);
	cek(
		"galeri foto strong point memuat dokumentasi",
		fotoPh.status === 200 && fotoPh.data.length > 0,
		`status ${fotoPh.status}`
	);
	const berkasPh = await fetch(`http://127.0.0.1:${PORT}${fotoPh.data[0].foto_url}`, {
		headers: { authorization: `Bearer ${adminToken}` }
	});
	cek(
		"berkas foto strong point dapat diunduh",
		berkasPh.status === 200 && (berkasPh.headers.get("content-type") ?? "").startsWith("image/")
	);
	const tanpaToken = await fetch(`http://127.0.0.1:${PORT}${fotoPh.data[0].foto_url}`);
	cek("foto tanpa token ditolak", tanpaToken.status === 401, `status ${tanpaToken.status}`);
	const rekapPh = await panggil(adminToken, `/ph/rekap?dari=2026-11-01&sampai=${hariIni}`);
	cek("rekap PH memuat personel uji", rekapPh.status === 200 && rekapPh.data.data.some((r) => r.personel_id === personel.user.id));

	// ------------------------------------------------------------ jam palsu dev
	const fakeSet = await fetch(`${BASE}/dev/waktu`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ tanggal: "2026-11-01", jam: "07:45" })
	});
	const fakeData = await fakeSet.json();
	cek(
		"jam server palsu development dapat diaktifkan",
		fakeSet.status === 200 && fakeData.aktif === true && fakeData.tanggal === "2026-11-01",
		JSON.stringify(fakeData)
	);

	const hariFake = await panggil(personel.access_token, "/absensi/hari-ini");
	cek(
		"hari ini mengikuti tanggal server palsu",
		hariFake.status === 200 && hariFake.data.tanggal === "2026-11-01" && hariFake.data.masuk === null,
		JSON.stringify(hariFake.data)
	);

	const daftarTitik = await panggil(adminToken, "/titik-dinas");
	const titikPersonel = daftarTitik.data.find((t) => t.satker_id === detailPersonel.data.satker_id);
	const absenFake = await panggil(personel.access_token, "/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jenis: "masuk",
			status_kehadiran: "dinas_kantor",
			captured_at: new Date().toISOString(),
			latitude: titikPersonel.latitude,
			longitude: titikPersonel.longitude,
			akurasi_m: 5,
			is_offline: false,
			is_mock_suspect: false,
			device_id: "uji-fake-clock"
		}
	});
	cek(
		"absen masuk memakai tanggal dan jam palsu",
		absenFake.status === 200 &&
			absenFake.data.data.tanggal === "2026-11-01" &&
			absenFake.data.data.jam === "07:45",
		JSON.stringify(absenFake.data?.data)
	);

	const hariSetelahAbsen = await panggil(personel.access_token, "/absensi/hari-ini");
	cek(
		"status hari ini mendeteksi absen masuk",
		hariSetelahAbsen.status === 200 && hariSetelahAbsen.data.masuk?.jenis === "masuk",
		JSON.stringify(hariSetelahAbsen.data)
	);

	const konteksCheckout = await panggil(personel.access_token, "/absensi/konteks");
	cek(
		"konteks menawarkan check-out setelah check-in",
		konteksCheckout.status === 200 &&
			konteksCheckout.data.aksi === "checkout" &&
			konteksCheckout.data.status_terkunci === "dinas_kantor",
		JSON.stringify(konteksCheckout.data)
	);
	const absenPulangAuto = await panggil(personel.access_token, "/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jenis: "pulang",
			status_kehadiran: "dinas_kantor",
			captured_at: new Date().toISOString(),
			latitude: titikPersonel.latitude,
			longitude: titikPersonel.longitude,
			akurasi_m: 5
		}
	});
	cek(
		"scan kedua otomatis menjadi check-out",
		absenPulangAuto.status === 200 &&
			absenPulangAuto.data.aksi === "checkout" &&
			absenPulangAuto.data.data.jenis === "pulang",
		JSON.stringify(absenPulangAuto.data)
	);
	const konteksSelesai = await panggil(personel.access_token, "/absensi/konteks");
	cek(
		"konteks selesai setelah check-in dan check-out",
		konteksSelesai.status === 200 && konteksSelesai.data.aksi === "selesai",
		JSON.stringify(konteksSelesai.data)
	);

	await fetch(`${BASE}/dev/waktu`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ tanggal: "2026-11-21", jam: "08:00" })
	});
	const absenPh = await panggil(personel.access_token, "/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jenis: "masuk",
			status_kehadiran: "ph",
			captured_at: new Date().toISOString(),
			latitude: titikPersonel.latitude,
			longitude: titikPersonel.longitude,
			akurasi_m: 5,
			is_offline: false,
			is_mock_suspect: false,
			device_id: "uji-status-ph"
		}
	});
	cek(
		"status PH tidak dapat dikirim lewat absensi harian",
		absenPh.status === 422 &&
			String(absenPh.data?.error?.message ?? "").includes("strong point"),
		JSON.stringify(absenPh.data)
	);

	// Check-in tanpa check-out: matriks harus menandai TC, bukan status biasa.
	const masukTc = await panggil(personel.access_token, "/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jenis: "masuk",
			status_kehadiran: "dinas_kantor",
			captured_at: new Date().toISOString(),
			latitude: titikPersonel.latitude,
			longitude: titikPersonel.longitude,
			akurasi_m: 5,
			is_offline: false,
			is_mock_suspect: false,
			device_id: "uji-tanpa-checkout"
		}
	});
	cek(
		"check-in tanpa check-out tercatat",
		masukTc.status === 200 && masukTc.data.aksi === "checkin",
		JSON.stringify(masukTc.data)
	);
	const matriksTc = await panggil(
		adminToken,
		`/absensi/matriks?dari=2026-11-01&sampai=2026-11-21&personel_id=${personel.user.id}`
	);
	const selTc = matriksTc.data?.data?.find((p) => p.personel_id === personel.user.id)
		?.hari?.["2026-11-21"];
	cek(
		"matriks memuat check-in tanpa check-out (TC)",
		matriksTc.status === 200 &&
			selTc?.masuk_jam === "08:00" &&
			selTc?.pulang_jam === null &&
			selTc?.tidak_absen === false,
		JSON.stringify(selTc)
	);
	// Tutup kembali catatan TC agar tidak mengganggu skenario piket lintas hari.
	await fetch(`${BASE}/dev/waktu`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ tanggal: "2026-11-21", jam: "17:00" })
	});
	const keluarTc = await panggil(personel.access_token, "/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jenis: "pulang",
			status_kehadiran: "dinas_kantor",
			captured_at: new Date().toISOString(),
			latitude: titikPersonel.latitude,
			longitude: titikPersonel.longitude,
			akurasi_m: 5
		}
	});
	cek(
		"check-out penutup uji TC tercatat",
		keluarTc.status === 200 && keluarTc.data.aksi === "checkout",
		JSON.stringify(keluarTc.data)
	);

	// Piket lintas hari: check-in 23:00, scan besok 22:00 (23 jam) = check-out.
	await fetch(`${BASE}/dev/waktu`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ tanggal: "2026-11-01", jam: "23:00" })
	});
	const piketMasuk = await panggil(personel.access_token, "/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jenis: "masuk",
			status_kehadiran: "piket",
			captured_at: new Date().toISOString(),
			latitude: titikPersonel.latitude,
			longitude: titikPersonel.longitude,
			akurasi_m: 5
		}
	});
	cek(
		"check-in piket lintas hari tercatat",
		piketMasuk.status === 200 && piketMasuk.data.aksi === "checkin",
		JSON.stringify(piketMasuk.data)
	);
	await fetch(`${BASE}/dev/waktu`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ tanggal: "2026-11-02", jam: "22:00" })
	});
	const piketPulang = await panggil(personel.access_token, "/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jenis: "masuk",
			status_kehadiran: "piket",
			captured_at: new Date().toISOString(),
			latitude: titikPersonel.latitude,
			longitude: titikPersonel.longitude,
			akurasi_m: 5
		}
	});
	cek(
		"scan piket 23 jam kemudian menjadi check-out",
		piketPulang.status === 200 &&
			piketPulang.data.aksi === "checkout" &&
			piketPulang.data.data.tanggal === "2026-11-01",
		JSON.stringify(piketPulang.data)
	);

	const fakeReset = await fetch(`${BASE}/dev/waktu/reset`, { method: "POST" });
	const resetData = await fakeReset.json();
	cek("jam server palsu dapat dikembalikan", fakeReset.status === 200 && resetData.aktif === false);

	// ------------------------------------------- fallback jadwal bulan sebelumnya
	const satkerFallback = semuaSatker.data.find(
		(s) => s.level === "satker" && s.id !== satkerPh
	);
	await fetch(`${BASE}/dev/waktu`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ tanggal: "2026-12-01", jam: "08:00" })
	});
	const jadwalFallback = await panggil(
		adminToken,
		`/ph/jadwal?dari=2026-12-01&sampai=2026-12-31&satker_id=${satkerFallback.id}`
	);
	cek(
		"jadwal bulan berjalan memakai jadwal bulan sebelumnya bila kosong",
		jadwalFallback.status === 200 &&
			jadwalFallback.data.length > 0 &&
			jadwalFallback.data.every((j) => j.disalin_dari_bulan === "2026-11-01"),
		JSON.stringify(jadwalFallback.data?.slice(0, 2))
	);
	const statusFallback = await panggil(
		adminToken,
		`/ph/jadwal/status?bulan=2026-12&satker_id=${satkerFallback.id}`
	);
	cek(
		"status menandai jadwal sebagai salinan bulan sebelumnya",
		statusFallback.status === 200 && statusFallback.data.disalin_dari_bulan === "2026-11-01",
		JSON.stringify(statusFallback.data)
	);
	await fetch(`${BASE}/dev/waktu/reset`, { method: "POST" });

	// ------------------------------------------------- kebijakan mock GPS (flag)
	await fetch(`${BASE}/dev/waktu`, {
		method: "POST",
		headers: { "content-type": "application/json" },
		body: JSON.stringify({ tanggal: "2026-11-22", jam: "08:00" })
	});
	const absenMock = await panggil(personel.access_token, "/absensi", {
		method: "POST",
		body: {
			client_uuid: crypto.randomUUID(),
			jenis: "masuk",
			status_kehadiran: "dinas_luar",
			captured_at: "2026-11-22T08:00:00Z",
			latitude: titikPersonel.latitude,
			longitude: titikPersonel.longitude,
			akurasi_m: 5,
			is_mock_suspect: true
		}
	});
	cek(
		"mock GPS ditandai perlu verifikasi (policy flag)",
		absenMock.status === 200 &&
			absenMock.data.data.is_mock_suspect === true &&
			absenMock.data.data.perlu_verifikasi === true,
		JSON.stringify(absenMock.data?.data)
	);
	await fetch(`${BASE}/dev/waktu/reset`, { method: "POST" });
} catch (error) {
	console.error(error);
	hasil.push({ nama: "exception", lulus: false, detail: String(error) });
} finally {
	server.kill();
}

const gagal = hasil.filter((h) => !h.lulus);
console.log(`\n${hasil.length - gagal.length}/${hasil.length} cek lulus`);
if (gagal.length) process.exit(1);
