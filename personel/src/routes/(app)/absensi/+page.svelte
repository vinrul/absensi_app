<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import Peta from '#lib/components/Peta.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import {
		LABEL_STATUS_KEHADIRAN,
		meter,
		warnaStatus
	} from '#lib/format';
	import { ambilLokasi, fotoKeBase64, type Posisi } from '#lib/lokasi';
	import { bacaAntreanAbsensi, sinkronkanAntrean, tambahAntreanAbsensi } from '#lib/offline';
	import { session } from '#lib/stores/session.svelte';
	import type {
		Absensi,
		AbsensiAntrean,
		AbsensiHariIni,
		AbsensiRequest,
		AbsensiResponse,
		JenisAbsen,
		StatusKehadiran,
		TitikDinas,
		TitikTerdekatResponse
	} from '#lib/types';

	const OPSI_STATUS: { nilai: StatusKehadiran; label: string; ikon: string; ket?: string }[] = [
		{ nilai: 'dinas_kantor', label: 'Dinas Kantor', ikon: 'home' },
		{ nilai: 'dinas_luar', label: 'Dinas Luar', ikon: 'pin' },
		{ nilai: 'ph', label: 'PH', ikon: 'user', ket: 'Perintah Hadir' },
		{ nilai: 'piket', label: 'Piket / Shift', ikon: 'lock' },
		{ nilai: 'cuti', label: 'Cuti', ikon: 'calendar' },
		{ nilai: 'izin', label: 'Izin', ikon: 'file' },
		{ nilai: 'sakit', label: 'Sakit', ikon: 'shield' },
		{ nilai: 'tanpa_keterangan', label: 'Tanpa Keterangan', ikon: 'alert' }
	];

	let tahap = $state<'awal' | 'lokasi' | 'status'>('awal');
	let jenis = $state<JenisAbsen>('masuk');
	let statusPilihan = $state<StatusKehadiran | null>(null);
	let posisi = $state<Posisi | null>(null);
	let mengambilLokasi = $state(false);
	let titik = $state<TitikTerdekatResponse | null>(null);
	let galatLokasi = $state<string | null>(null);
	let fotoBase64 = $state<string | null>(null);
	let pratinjauFoto = $state<string | null>(null);
	let namaFoto = $state<string | null>(null);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);
	let absensiHariIni = $state<AbsensiHariIni | null>(null);
	let statusServerOffline = $state(false);
	let offline = $state(false);
	let gagalOnline = $state(false);
	let antrean = $state<AbsensiAntrean[]>([]);
	let riwayat = $state<Absensi[]>([]);
	let permintaanTersimpan = $state<AbsensiRequest | null>(null);

	const dalamRadius = $derived(titik?.terdekat.dalam_radius === true);
	const titikTerdekat = $derived(
		(titik?.titik ?? []).reduce<TitikDinas | null>(
			(terkecil, item) =>
				terkecil === null || (item.jarak_m ?? Infinity) < (terkecil.jarak_m ?? Infinity)
					? item
					: terkecil,
			null
		)
	);
	const butuhRadius = $derived(
		statusPilihan === 'dinas_kantor' || statusPilihan === 'piket'
	);
	const bolehKirim = $derived(
		statusPilihan !== null && !(butuhRadius && !dalamRadius && !offline && !statusServerOffline)
	);

	async function muatAwal() {
		const user = session.user;
		if (!user) return;
		antrean = bacaAntreanAbsensi(user.id);
		try {
			absensiHariIni = await api<AbsensiHariIni>('/absensi/hari-ini');
			statusServerOffline = false;
		} catch {
			absensiHariIni = null;
			statusServerOffline = true;
		}
		try {
			riwayat = await api<Absensi[]>('/absensi/saya');
		} catch {
			riwayat = [];
		}
	}

	async function perbaruiLokasi() {
		mengambilLokasi = true;
		galatLokasi = null;
		try {
			posisi = await ambilLokasi();
			try {
				titik = await api<TitikTerdekatResponse>(
					`/absensi/titik-terdekat?latitude=${posisi.latitude}&longitude=${posisi.longitude}&akurasi_m=${posisi.akurasi ?? 0}`
				);
			} catch {
				titik = null;
			}
		} catch (err) {
			galatLokasi = err instanceof Error ? err.message : 'Gagal mengambil lokasi.';
		} finally {
			mengambilLokasi = false;
		}
	}

	async function mulai(jenisDipilih?: JenisAbsen) {
		pesan = null;
		galat = null;
		gagalOnline = false;
		offline = statusServerOffline;
		tahap = 'lokasi';
		await muatAwal();
		if (!posisi) await perbaruiLokasi();
		if (jenisDipilih) pilihJenis(jenisDipilih);
	}

	function pilihJenis(jenisDipilih: JenisAbsen) {
		jenis = jenisDipilih;
		statusPilihan = dalamRadius ? 'dinas_kantor' : null;
		offline = statusServerOffline;
		galat = null;
		tahap = 'status';
	}

	function kembali() {
		galat = null;
		if (tahap === 'status') tahap = 'lokasi';
		else tahap = 'awal';
	}

	async function pilihFoto(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		galat = null;
		if (!file.type.startsWith('image/')) {
			galat = 'Foto harus berupa gambar JPG atau PNG.';
			input.value = '';
			return;
		}
		if (file.size > 5 * 1024 * 1024) {
			galat = 'Ukuran foto maksimal 5 MB.';
			input.value = '';
			return;
		}
		try {
			if (pratinjauFoto) URL.revokeObjectURL(pratinjauFoto);
			pratinjauFoto = URL.createObjectURL(file);
			namaFoto = file.name;
			fotoBase64 = await fotoKeBase64(file);
		} catch {
			galat = 'Foto tidak dapat diolah. Coba foto lain.';
		}
	}

	function hapusFoto() {
		if (pratinjauFoto) URL.revokeObjectURL(pratinjauFoto);
		pratinjauFoto = null;
		fotoBase64 = null;
		namaFoto = null;
	}

	function bangunPermintaan(): AbsensiRequest {
		return {
			client_uuid: crypto.randomUUID(),
			jenis,
			status_kehadiran: statusPilihan ?? 'dinas_luar',
			captured_at: new Date().toISOString(),
			latitude: posisi?.latitude ?? null,
			longitude: posisi?.longitude ?? null,
			akurasi_m: posisi?.akurasi ?? null,
			is_mock_suspect: false,
			is_offline: offline,
			device_id: 'web-personel',
			foto_base64: fotoBase64
		};
	}

	async function kirim(modeOffline = offline) {
		const user = session.user;
		if (!user || !statusPilihan || memproses) return;
		memproses = true;
		galat = null;
		pesan = null;
		gagalOnline = false;

		const permintaan = bangunPermintaan();
		permintaanTersimpan = permintaan;

		if (modeOffline) {
			tambahAntreanAbsensi(user.id, { ...permintaan, is_offline: true, dibuat_pada: Date.now() });
			antrean = bacaAntreanAbsensi(user.id);
			memproses = false;
			pesan = 'Absensi disimpan di perangkat dan dapat dikirim saat sinyal tersedia.';
			hapusFoto();
			tahap = 'awal';
			return;
		}

		try {
			const hasil = await api<AbsensiResponse>('/absensi', {
				method: 'POST',
				body: {
					...permintaan,
					is_offline: false,
					akurasi_m: permintaan.akurasi_m
				}
			});
			pesan = `Absen ${jenis} tercatat pukul ${hasil.data.jam}${
				hasil.geofence.jarak_m !== null ? ` (${meter(hasil.geofence.jarak_m)} dari titik dinas)` : ''
			}.`;
			hapusFoto();
			tahap = 'awal';
			await muatAwal();
		} catch (err) {
			if (err instanceof ApiError && err.status === 0) {
				gagalOnline = true;
				galat = 'Server tidak terjangkau. Anda dapat menyimpan absensi secara offline.';
			} else {
				galat = err instanceof ApiError ? err.message : 'Gagal mengirim absensi.';
			}
		} finally {
			memproses = false;
		}
	}

	async function sinkronkan() {
		const user = session.user;
		if (!user || memproses) return;
		memproses = true;
		pesan = null;
		galat = null;
		try {
			const hasil = await sinkronkanAntrean(user.id);
			antrean = bacaAntreanAbsensi(user.id);
			await muatAwal();
			pesan =
				hasil.absensi > 0 && hasil.ditolak > 0
					? `${hasil.absensi} absensi terkirim, ${hasil.ditolak} ditolak karena lokasi tiruan.`
					: hasil.ditolak > 0
						? `${hasil.ditolak} absensi ditolak karena lokasi tiruan.`
						: hasil.absensi > 0
							? `${hasil.absensi} absensi berhasil dikirim.`
							: 'Tidak ada antrean absensi yang dapat dikirim.';
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Gagal mengirim antrean.';
		} finally {
			memproses = false;
		}
	}

	onMount(muatAwal);

	onDestroy(() => {
		if (pratinjauFoto) URL.revokeObjectURL(pratinjauFoto);
	});
</script>

<svelte:head><title>Absensi — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if tahap === 'awal'}
		<button
			type="button"
			class="relative w-full overflow-hidden rounded-3xl bg-gradient-to-br from-brand-800 via-brand-700 to-brand-500 p-5 text-left text-white shadow-lg"
			onclick={() => mulai()}
		>
			<div class="pointer-events-none absolute -right-8 -bottom-10 size-40 rounded-full bg-white/10 blur-2xl"></div>
			<div class="relative flex items-center gap-4">
				<span class="flex size-14 items-center justify-center rounded-2xl bg-white/15 ring-1 ring-white/20">
					<Icon name="pin" size={28} />
				</span>
				<div>
					<p class="text-lg font-bold">Absensi</p>
					<p class="text-xs text-brand-100">Absen dengan validasi lokasi GPS</p>
				</div>
				<Icon name="right" size={20} class="ml-auto text-white/70" />
			</div>
		</button>

		<section class="kartu p-4">
			<p class="judul-bagian">Status Hari Ini</p>
			<p class="teks-kecil">
				{#if statusServerOffline}
					Status tidak dapat diambil dari server (offline)
				{:else if absensiHariIni?.masuk && absensiHariIni?.pulang}
					Absensi hari ini sudah lengkap
				{:else if absensiHariIni?.masuk}
					Sudah absen masuk pukul {absensiHariIni.masuk.jam}
				{:else}
					Belum absen masuk hari ini
				{/if}
			</p>

			<div class="mt-3 grid grid-cols-2 gap-2">
				{#each [
					{ label: 'Masuk', item: absensiHariIni?.masuk ?? null },
					{ label: 'Pulang', item: absensiHariIni?.pulang ?? null }
				] as kolom (kolom.label)}
					<div class="rounded-xl bg-ink-50 px-3 py-2 ring-1 ring-ink-100">
						<p class="text-[10px] font-bold uppercase tracking-wider text-ink-400">{kolom.label}</p>
						{#if kolom.item}
							<p class="text-base font-bold text-ink-900">{kolom.item.jam}</p>
							<span class="chip {warnaStatus(kolom.item.status_kehadiran)}">
								{LABEL_STATUS_KEHADIRAN[kolom.item.status_kehadiran]}
							</span>
						{:else}
							<p class="text-sm font-semibold text-ink-300">Belum</p>
						{/if}
					</div>
				{/each}
			</div>
		</section>

		<section class="kartu p-4">
			<p class="judul-bagian">Riwayat Bulan Ini</p>
			<p class="teks-kecil">{riwayat.length} catatan absensi</p>
			<a href="/absensi/riwayat" class="btn btn-kedua mt-3 w-full">
				<Icon name="clock" size={17} />
				Riwayat Absensi
			</a>
		</section>

		{#if antrean.length > 0}
			<section class="kartu border-amber-200 bg-amber-50/60 p-4">
				<p class="judul-bagian text-amber-900">Antrean Offline</p>
				<p class="text-xs text-amber-800">{antrean.length} absensi menunggu dikirim ke server</p>
				<button
					class="btn btn-kedua mt-3 w-full border-amber-200 bg-amber-100 text-amber-900"
					onclick={sinkronkan}
					disabled={memproses}
				>
					<Icon name="refresh" size={16} />
					{memproses ? 'Mengirim…' : 'Kirim Antrean Sekarang'}
				</button>
			</section>
		{/if}
	{:else if tahap === 'lokasi'}
		<button type="button" class="btn btn-hantu -ml-2 px-2 py-1" onclick={kembali}>
			<Icon name="left" size={16} /> Kembali
		</button>

		<div>
			<h1 class="text-xl font-bold tracking-tight text-ink-900">Validasi Lokasi (GPS)</h1>
			<p class="text-xs text-ink-500">Lokasi hanya diambil saat Anda membuka halaman ini.</p>
		</div>

		<div class="kartu space-y-3 p-4">
			<Peta
				latitude={posisi?.latitude ?? null}
				longitude={posisi?.longitude ?? null}
				akurasi={posisi?.akurasi ?? null}
				titik={titikTerdekat}
				warna="hijau"
				tinggi="h-60"
			/>

			{#if mengambilLokasi}
				<div class="flex items-center gap-2 text-xs text-ink-500">
					<span class="size-4 animate-spin rounded-full border-2 border-brand-200 border-t-brand-600"></span>
					Mengambil lokasi…
				</div>
			{/if}

			<dl>
				<div class="baris-info">
					<dt>Koordinat Anda</dt>
					<dd>
						{posisi
							? `${posisi.latitude.toFixed(6)}, ${posisi.longitude.toFixed(6)}`
							: '-'}
					</dd>
				</div>
				<div class="baris-info">
					<dt>Akurasi GPS</dt>
					<dd>{posisi?.akurasi ? `± ${Math.round(posisi.akurasi)} m` : '-'}</dd>
				</div>
				<div class="baris-info">
					<dt>Jarak ke Titik Dinas</dt>
					<dd>{titik?.terdekat.jarak_m !== null && titik ? meter(titik.terdekat.jarak_m) : 'Menghitung…'}</dd>
				</div>
				<div class="baris-info">
					<dt>Titik Dinas</dt>
					<dd>{titikTerdekat?.nama ?? titik?.terdekat.titik_nama ?? '-'}</dd>
				</div>
			</dl>

			<button class="btn btn-kedua w-full" onclick={perbaruiLokasi} disabled={mengambilLokasi}>
				<Icon name="refresh" size={16} />
				{posisi ? 'Perbarui Lokasi' : 'Ambil Lokasi'}
			</button>

			{#if galatLokasi}
				<Pesan jenis="galat" pesan={galatLokasi} />
			{/if}
		</div>

		{#if posisi}
			<div
				class="rounded-2xl p-4 ring-1 {dalamRadius
					? 'bg-emerald-50 ring-emerald-200'
					: statusServerOffline
						? 'bg-amber-50 ring-amber-200'
						: 'bg-rose-50 ring-rose-200'}"
			>
				<div class="flex items-center gap-2">
					<Icon
						name={dalamRadius ? 'checkCircle' : 'alert'}
						size={22}
						class={dalamRadius ? 'text-emerald-600' : statusServerOffline ? 'text-amber-600' : 'text-rose-600'}
					/>
					<p class="text-sm font-bold {dalamRadius
						? 'text-emerald-900'
						: statusServerOffline
							? 'text-amber-900'
							: 'text-rose-900'}">
						{dalamRadius
							? 'Lokasi Terverifikasi'
							: statusServerOffline
								? 'Lokasi Belum Terverifikasi'
								: 'Di Luar Radius'}
					</p>
				</div>
				<p class="mt-1 text-xs leading-relaxed {dalamRadius
					? 'text-emerald-800'
					: statusServerOffline
						? 'text-amber-800'
						: 'text-rose-800'}">
					{#if dalamRadius}
						Anda berada dalam radius {titik?.terdekat.radius_m ?? 100} meter dari titik dinas.
					{:else if statusServerOffline}
						Titik dinas belum dapat diverifikasi karena server tidak terjangkau. Absensi akan disimpan offline dan diverifikasi saat sinkronisasi.
					{:else}
						Anda berada di luar radius titik dinas. Status Dinas Kantor/Piket tidak dapat dipilih.
					{/if}
				</p>
			</div>

			{#if statusServerOffline}
				<Pesan
					jenis="info"
					pesan="Status absensi hari ini tidak dapat diambil. Pilih jenis absen yang sesuai; data akan disimpan offline."
				/>
				<div class="grid grid-cols-2 gap-2">
					<button class="btn btn-utama bg-emerald-600 py-4 hover:bg-emerald-700" onclick={() => pilihJenis('masuk')}>
						<Icon name="pin" size={18} /> ABSEN MASUK
					</button>
					<button class="btn btn-bahaya py-4" onclick={() => pilihJenis('pulang')}>
						<Icon name="pin" size={18} /> ABSEN PULANG
					</button>
				</div>
			{:else if !absensiHariIni?.masuk}
				<button class="btn btn-utama w-full bg-emerald-600 py-4 hover:bg-emerald-700" onclick={() => pilihJenis('masuk')}>
					<Icon name="pin" size={18} /> ABSEN MASUK
				</button>
			{:else if !absensiHariIni?.pulang}
				<button class="btn btn-bahaya w-full py-4" onclick={() => pilihJenis('pulang')}>
					<Icon name="pin" size={18} /> ABSEN PULANG
				</button>
			{:else}
				<Pesan jenis="sukses" pesan="Absensi hari ini sudah lengkap (masuk dan pulang)." />
			{/if}
		{:else}
			<Pesan jenis="info" pesan="Ambil lokasi terlebih dahulu untuk melanjutkan absensi." />
		{/if}
	{:else}
		<button type="button" class="btn btn-hantu -ml-2 px-2 py-1" onclick={kembali}>
			<Icon name="left" size={16} /> Kembali
		</button>

		<div>
			<h1 class="text-xl font-bold tracking-tight text-ink-900">Pilih Status Kehadiran</h1>
			<p class="text-xs text-ink-500">
				{jenis === 'masuk' ? 'Absen masuk' : 'Absen pulang'} · pilih alasan kehadiran hari ini.
			</p>
		</div>

		<div class="grid grid-cols-2 gap-2">
			{#each OPSI_STATUS as opsi (opsi.nilai)}
				<button
					type="button"
					class="flex flex-col items-start gap-1 rounded-2xl border p-3 text-left transition-colors {statusPilihan ===
					opsi.nilai
						? 'border-brand-500 bg-brand-50 ring-2 ring-brand-500/20'
						: 'border-ink-200 bg-white hover:border-brand-300'}"
					onclick={() => (statusPilihan = opsi.nilai)}
				>
					<span class="flex size-8 items-center justify-center rounded-lg bg-ink-50 text-ink-600">
						<Icon name={opsi.ikon} size={16} />
					</span>
					<span class="text-xs font-bold text-ink-800">{opsi.label}</span>
					{#if opsi.ket}
						<span class="text-[10px] text-ink-400">{opsi.ket}</span>
					{/if}
				</button>
			{/each}
		</div>

		{#if butuhRadius && !dalamRadius && !offline && !statusServerOffline}
			<Pesan
				jenis="galat"
				pesan="Status Dinas Kantor/Piket membutuhkan lokasi di dalam radius titik dinas. Pilih status lain atau perbarui lokasi."
			/>
		{/if}

		<div class="kartu p-4">
			<div class="flex items-center justify-between gap-3">
				<div>
					<p class="judul-bagian">Dokumentasi</p>
					<p class="teks-kecil">Foto bukti opsional, maksimal 5 MB.</p>
				</div>
				{#if fotoBase64}
					<span class="chip bg-emerald-100 text-emerald-700">Siap</span>
				{/if}
			</div>

			{#if pratinjauFoto}
				<div class="mt-3 overflow-hidden rounded-xl ring-1 ring-ink-100">
					<img src={pratinjauFoto} alt="Pratinjau dokumentasi" class="h-40 w-full object-cover" />
				</div>
				<p class="mt-1 truncate text-[10px] text-ink-400">{namaFoto}</p>
			{/if}

			<div class="mt-3 flex gap-2">
				<label class="btn btn-kedua flex-1 cursor-pointer">
					<Icon name="camera" size={16} />
					{fotoBase64 ? 'Ganti Foto' : 'Ambil Foto'}
					<input
						type="file"
						accept="image/*"
						capture="environment"
						class="sr-only"
						onchange={pilihFoto}
					/>
				</label>
				{#if fotoBase64}
					<button class="btn btn-hantu text-rose-600" type="button" onclick={hapusFoto}>
						<Icon name="trash" size={16} />
					</button>
				{/if}
			</div>
		</div>

		{#if statusServerOffline}
			<label class="kartu flex items-center justify-between gap-3 p-4">
				<span>
					<span class="block text-sm font-bold text-ink-800">Mode offline</span>
					<span class="block text-[11px] text-ink-500">
						Absensi disimpan di perangkat lalu dikirim saat ada sinyal
					</span>
				</span>
				<input type="checkbox" class="size-5 accent-brand-600" bind:checked={offline} />
			</label>
		{/if}

		<button class="btn btn-utama w-full py-3" disabled={!bolehKirim || memproses} onclick={() => kirim()}>
			{#if memproses}
				<span class="size-4 animate-spin rounded-full border-2 border-white/40 border-t-white"></span>
				Mengirim…
			{:else}
				<Icon name="send" size={17} />
				{offline ? 'Simpan Absensi (Offline)' : 'Kirim Absensi'}
			{/if}
		</button>

		{#if gagalOnline && permintaanTersimpan}
			<button class="btn btn-kedua w-full" onclick={() => kirim(true)}>
				<Icon name="wifiOff" size={16} />
				Simpan Secara Offline
			</button>
		{/if}
	{/if}
</div>
