<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Avatar from '#lib/components/Avatar.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import {
		hariIndo,
		tanggalHariIni,
		tanggalIndo,
		warnaStatus,
		LABEL_STATUS_KEHADIRAN
	} from '#lib/format';
	import { bacaAntreanAbsensi, bacaAntreanPh, sinkronkanAntrean } from '#lib/offline';
	import { peranUtama, session } from '#lib/stores/session.svelte';
	import { statusStrongPoint } from '#lib/strong';
	import type {
		Absensi,
		AbsensiAntrean,
		AbsensiHariIni,
		AbsensiPh,
		AbsensiPhAntrean,
		JadwalPhSaya
	} from '#lib/types';

	let memuat = $state(true);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);

	let absensiHariIni = $state<AbsensiHariIni | null>(null);
	let statusOffline = $state(false);
	let riwayat = $state<Absensi[]>([]);
	let jadwalPh = $state<JadwalPhSaya[]>([]);
	let absensiPh = $state<AbsensiPh[]>([]);
	let antrean = $state<AbsensiAntrean[]>([]);
	let antreanPh = $state<AbsensiPhAntrean[]>([]);

	const pengguna = $derived(session.user);
	const hariIni = $derived(absensiHariIni?.tanggal ?? tanggalHariIni());
	const jadwalHariIni = $derived(jadwalPh.filter((item) => item.tanggal === hariIni));
	const masuk = $derived(
		absensiHariIni?.masuk ??
			riwayat.find((item) => item.tanggal === hariIni && item.jenis === 'masuk') ??
			null
	);
	const pulang = $derived(
		absensiHariIni?.pulang ??
			riwayat.find((item) => item.tanggal === hariIni && item.jenis === 'pulang') ??
			null
	);
	const jumlahAntrean = $derived(antrean.length + antreanPh.length);

	function statusSesi(jadwal: JadwalPhSaya) {
		return statusStrongPoint(jadwal.id, absensiPh, antreanPh);
	}

	async function muat() {
		const user = session.user;
		if (!user) return;
		memuat = true;
		galat = null;

		antrean = bacaAntreanAbsensi(user.id);
		antreanPh = bacaAntreanPh(user.id);

		try {
			absensiHariIni = await api<AbsensiHariIni>('/absensi/hari-ini');
			statusOffline = false;
		} catch {
			absensiHariIni = null;
			statusOffline = true;
		}

		const hasilMuat = await Promise.allSettled([
			api<Absensi[]>('/absensi/saya'),
			api<JadwalPhSaya[]>(`/ph/jadwal/saya?dari=${hariIni}&sampai=${hariIni}`),
			api<AbsensiPh[]>(`/ph/absensi/saya?dari=${hariIni}&sampai=${hariIni}`)
		]);

		riwayat = hasilMuat[0].status === 'fulfilled' ? hasilMuat[0].value : [];
		jadwalPh = hasilMuat[1].status === 'fulfilled' ? hasilMuat[1].value : [];
		absensiPh = hasilMuat[2].status === 'fulfilled' ? hasilMuat[2].value : [];

		memuat = false;
	}

	async function sinkronkan() {
		const user = session.user;
		if (!user || memproses) return;
		memproses = true;
		pesan = null;
		galat = null;
		try {
			const hasilSinkron = await sinkronkanAntrean(user.id);
			antrean = bacaAntreanAbsensi(user.id);
			antreanPh = bacaAntreanPh(user.id);
			await muat();
			const total = hasilSinkron.absensi + hasilSinkron.ph;
			const ditolak = hasilSinkron.ditolak;
			pesan =
				total > 0 && ditolak > 0
					? `${total} data terkirim, ${ditolak} ditolak karena lokasi tiruan (mock GPS).`
					: ditolak > 0
						? `${ditolak} data ditolak karena lokasi tiruan (mock GPS).`
						: total > 0
							? `${total} data berhasil dikirim.`
							: 'Tidak ada antrean yang dapat dikirim.';
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Gagal mengirim antrean.';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Beranda — Absensi & Strong Point</title></svelte:head>

<div class="space-y-3">
	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	<section
		class="relative overflow-hidden rounded-3xl bg-gradient-to-br from-brand-800 via-brand-700 to-brand-500 p-5 text-white shadow-lg"
	>
		<div
			class="pointer-events-none absolute -top-10 -right-10 size-40 rounded-full bg-white/10 blur-2xl"
		></div>
		<div class="relative flex items-center gap-3">
			<Avatar pengguna={pengguna} size={54} class="ring-2 ring-white/40" />
			<div class="min-w-0">
				<p class="text-xs text-brand-100">Selamat bertugas,</p>
				<p class="truncate text-lg font-bold">{pengguna?.nama ?? '-'}</p>
				<p class="truncate text-[11px] text-brand-100">
					{pengguna?.pangkat} · {pengguna?.jabatan}
				</p>
				<p class="truncate text-[11px] font-semibold text-sun-400">
					{pengguna?.satker_nama} · {peranUtama()}
				</p>
			</div>
		</div>
		<div
			class="relative mt-4 flex items-center justify-between rounded-2xl bg-white/10 px-3 py-2 ring-1 ring-white/15"
		>
			<span class="text-[11px] text-brand-50">{hariIndo(hariIni)}, {tanggalIndo(hariIni)}</span>
			<span class="text-[11px] font-semibold text-white">Absensi & Strong Point</span>
		</div>
	</section>

	<section class="kartu p-4">
		<div class="flex items-center justify-between gap-3">
			<div>
				<p class="judul-bagian">Kehadiran Hari Ini</p>
				<p class="teks-kecil">
					{#if statusOffline}
						Status server tidak dapat diambil (offline)
					{:else if memuat}
						Memeriksa status…
					{:else}
						Absensi tercatat sesuai waktu server
					{/if}
				</p>
			</div>
			{#if statusOffline}
				<span class="chip bg-amber-100 text-amber-800">Offline</span>
			{/if}
		</div>

		<div class="mt-4 grid grid-cols-2 gap-3">
			<div class="rounded-2xl bg-emerald-50 p-3 ring-1 ring-emerald-100">
				<p class="text-[10px] font-bold uppercase tracking-wider text-emerald-700">Masuk</p>
				{#if masuk}
					<p class="mt-1 text-xl font-bold text-emerald-900">{masuk.jam}</p>
					<span class="chip mt-1 {warnaStatus(masuk.status_kehadiran)}">
						{LABEL_STATUS_KEHADIRAN[masuk.status_kehadiran]}
					</span>
				{:else}
					<p class="mt-1 text-lg font-bold text-emerald-900/40">Belum</p>
					<p class="text-[10px] text-emerald-700/70">Absen masuk</p>
				{/if}
			</div>
			<div class="rounded-2xl bg-rose-50 p-3 ring-1 ring-rose-100">
				<p class="text-[10px] font-bold uppercase tracking-wider text-rose-700">Pulang</p>
				{#if pulang}
					<p class="mt-1 text-xl font-bold text-rose-900">{pulang.jam}</p>
					<span class="chip mt-1 {warnaStatus(pulang.status_kehadiran)}">
						{LABEL_STATUS_KEHADIRAN[pulang.status_kehadiran]}
					</span>
				{:else}
					<p class="mt-1 text-lg font-bold text-rose-900/40">Belum</p>
					<p class="text-[10px] text-rose-700/70">Absen pulang</p>
				{/if}
			</div>
		</div>

		<a href="/absensi" class="btn btn-utama mt-3 w-full py-3">
			<Icon name="pin" size={18} />
			{masuk ? (pulang ? 'Lihat Absensi' : 'Absen Pulang') : 'Absen Sekarang'}
		</a>
	</section>

	<section class="kartu p-4">
		<div class="flex items-start justify-between gap-3">
			<div>
				<p class="judul-bagian">Strong Point Hari Ini</p>
				<p class="teks-kecil">
					{#if memuat}
						Memuat jadwal…
					{:else if jadwalHariIni.length === 0}
						Tidak ada jadwal strong point hari ini
					{:else}
						{jadwalHariIni.length} sesi dijadwalkan
					{/if}
				</p>
			</div>
			{#if jadwalHariIni.length > 0}
				<span class="chip bg-emerald-100 text-emerald-700">Ada jadwal</span>
			{/if}
		</div>
		<a href="/strong" class="btn btn-kedua mt-3 w-full">
			<Icon name="calendar" size={17} />
			Buka Strong Point
		</a>
	</section>

	{#if jumlahAntrean > 0}
		<section class="kartu border-amber-200 bg-amber-50/60 p-4">
			<div class="flex items-center justify-between gap-3">
				<div>
					<p class="judul-bagian text-amber-900">Antrean Offline</p>
					<p class="text-xs text-amber-800">
						{antrean.length} absensi · {antreanPh.length} strong point menunggu dikirim
					</p>
				</div>
				<Icon name="wifiOff" size={22} class="text-amber-700" />
			</div>
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

	<p class="px-1 pb-2 text-center text-[10px] leading-relaxed text-ink-400">
		Absensi utama dilakukan melalui aplikasi ini. Bila tidak ada sinyal, data disimpan di
		perangkat dan dikirim saat Anda menekan Kirim Antrean.
	</p>
</div>
