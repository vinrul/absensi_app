<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Avatar from '#lib/components/Avatar.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import {
		angka,
		hariIndo,
		tanggalHariIni,
		tanggalIndo,
		warnaKategori,
		warnaStatus,
		LABEL_KATEGORI,
		LABEL_STATUS_KEHADIRAN
	} from '#lib/format';
	import { bacaAntreanAbsensi, bacaAntreanPh, sinkronkanAntrean } from '#lib/offline';
	import { bisaMenyetujui, peranUtama, session } from '#lib/stores/session.svelte';
	import { keteranganJadwal, statusStrongPoint } from '#lib/strong';
	import type {
		Absensi,
		AbsensiAntrean,
		AbsensiHariIni,
		AbsensiPh,
		AbsensiPhAntrean,
		Hasil,
		JadwalPhSaya,
		ListResponse,
		Output,
		PenilaianDetailResponse,
		Periode,
		Rencana
	} from '#lib/types';

	let memuat = $state(true);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);

	let periode = $state<Periode | null>(null);
	let absensiHariIni = $state<AbsensiHariIni | null>(null);
	let statusOffline = $state(false);
	let riwayat = $state<Absensi[]>([]);
	let rencana = $state<Rencana[]>([]);
	let hasil = $state<Hasil[]>([]);
	let output = $state<Output[]>([]);
	let persetujuan = $state<Rencana[]>([]);
	let verifikasiHasil = $state<Hasil[]>([]);
	let verifikasiOutput = $state<Output[]>([]);
	let jadwalPh = $state<JadwalPhSaya[]>([]);
	let absensiPh = $state<AbsensiPh[]>([]);
	let antrean = $state<AbsensiAntrean[]>([]);
	let antreanPh = $state<AbsensiPhAntrean[]>([]);
	let nilai = $state<PenilaianDetailResponse | null>(null);

	const pengguna = $derived(session.user);
	const bisaSetujui = $derived(bisaMenyetujui());
	const tanggalKlien = $derived(tanggalHariIni());
	const hariIni = $derived(absensiHariIni?.tanggal ?? tanggalKlien);
	const jadwalHariIni = $derived(jadwalPh.find((item) => item.tanggal === tanggalKlien) ?? null);
	const statusStrong = $derived(statusStrongPoint(jadwalHariIni?.id, absensiPh, antreanPh));
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

		const bulan = hariIni.slice(0, 7);
		const hasilMuat = await Promise.allSettled([
			api<Periode[]>('/periode'),
			api<Absensi[]>('/absensi/saya'),
			api<ListResponse<Rencana>>(`/rencana-kerja?per_page=50&personel_id=${user.id}`),
			api<ListResponse<Hasil>>(`/hasil-kerja?per_page=50&personel_id=${user.id}`),
			api<ListResponse<Output>>(`/output?per_page=50&personel_id=${user.id}`),
			api<JadwalPhSaya[]>(`/ph/jadwal/saya?bulan=${bulan}`),
			api<AbsensiPh[]>(`/ph/absensi/saya?bulan=${bulan}`)
		]);

		periode = hasilMuat[0].status === 'fulfilled' ? (hasilMuat[0].value[0] ?? null) : null;
		riwayat = hasilMuat[1].status === 'fulfilled' ? hasilMuat[1].value : [];
		rencana = hasilMuat[2].status === 'fulfilled' ? hasilMuat[2].value.data : [];
		hasil = hasilMuat[3].status === 'fulfilled' ? hasilMuat[3].value.data : [];
		output = hasilMuat[4].status === 'fulfilled' ? hasilMuat[4].value.data : [];
		jadwalPh = hasilMuat[5].status === 'fulfilled' ? hasilMuat[5].value : [];
		absensiPh = hasilMuat[6].status === 'fulfilled' ? hasilMuat[6].value : [];

		if (bisaSetujui) {
			const [p, h, o] = await Promise.allSettled([
				api<ListResponse<Rencana>>('/rencana-kerja/persetujuan?per_page=50'),
				api<ListResponse<Hasil>>('/hasil-kerja/verifikasi?per_page=50'),
				api<ListResponse<Output>>('/output/verifikasi?per_page=50')
			]);
			persetujuan = p.status === 'fulfilled' ? p.value.data : [];
			verifikasiHasil = h.status === 'fulfilled' ? h.value.data : [];
			verifikasiOutput = o.status === 'fulfilled' ? o.value.data : [];
		}

		if (periode) {
			try {
				nilai = await api<PenilaianDetailResponse>(
					`/penilaian/personel/${user.id}?periode_id=${periode.id}`
				);
			} catch {
				nilai = null;
			}
		}

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

<svelte:head><title>Beranda — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	<section class="relative overflow-hidden rounded-3xl bg-gradient-to-br from-brand-800 via-brand-700 to-brand-500 p-5 text-white shadow-lg">
		<div class="pointer-events-none absolute -top-10 -right-10 size-40 rounded-full bg-white/10 blur-2xl"></div>
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
		<div class="relative mt-4 flex items-center justify-between rounded-2xl bg-white/10 px-3 py-2 ring-1 ring-white/15">
			<span class="text-[11px] text-brand-50">{hariIndo(hariIni)}, {tanggalIndo(hariIni)}</span>
			<span class="text-[11px] font-semibold text-white">{periode?.nama ?? 'Periode berjalan'}</span>
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
					{#if !jadwalHariIni}
						Tidak ada jadwal strong point hari ini
					{:else if statusStrong.sesi.length === 0}
						Belum check-in · {keteranganJadwal(
							jadwalHariIni.regu,
							jadwalHariIni.strong_point_nama
						)}
					{:else if statusStrong.terbuka}
						Check-in pukul {statusStrong.masukTerakhir?.jam}
						{#if statusStrong.perluCheckInBaru}
							· lebih dari 2 jam, belum check-out
						{/if}
					{:else}
						Sudah check-out pukul {statusStrong.pulangTerakhir?.jam}
					{/if}
				</p>
			</div>
			{#if statusStrong.terbuka && !statusStrong.perluCheckInBaru}
				<span class="chip bg-rose-100 text-rose-700">Check-out</span>
			{:else if jadwalHariIni}
				<span class="chip bg-emerald-100 text-emerald-700">Check-in</span>
			{/if}
		</div>
		<a href="/strong" class="btn btn-kedua mt-3 w-full">
			<Icon name="calendar" size={17} />
			Buka Strong Point
		</a>
	</section>

	{#if bisaSetujui}
		<section class="grid gap-3 sm:grid-cols-3">
			<a href="/rencana/persetujuan" class="kartu flex flex-col gap-2 p-4">
				<div class="flex items-center justify-between">
					<span class="flex size-9 items-center justify-center rounded-xl bg-sky-50 text-sky-700">
						<Icon name="list" size={18} />
					</span>
					<span class="chip {persetujuan.length
						? 'bg-sun-400/25 text-sun-600'
						: 'bg-ink-100 text-ink-500'}">{persetujuan.length}</span>
				</div>
				<p class="text-sm font-bold text-ink-800">Persetujuan Rencana</p>
				<p class="text-[11px] text-ink-500">
					{persetujuan.length
						? `${persetujuan.length} rencana menunggu keputusan`
						: 'Tidak ada yang menunggu'}
				</p>
			</a>
			<a href="/hasil/verifikasi" class="kartu flex flex-col gap-2 p-4">
				<div class="flex items-center justify-between">
					<span class="flex size-9 items-center justify-center rounded-xl bg-emerald-50 text-emerald-700">
						<Icon name="checkCircle" size={18} />
					</span>
					<span class="chip {verifikasiHasil.length
						? 'bg-sun-400/25 text-sun-600'
						: 'bg-ink-100 text-ink-500'}">{verifikasiHasil.length}</span>
				</div>
				<p class="text-sm font-bold text-ink-800">Verifikasi Hasil</p>
				<p class="text-[11px] text-ink-500">
					{verifikasiHasil.length
						? `${verifikasiHasil.length} hasil menunggu verifikasi`
						: 'Tidak ada yang menunggu'}
				</p>
			</a>
			<a href="/output/verifikasi" class="kartu flex flex-col gap-2 p-4">
				<div class="flex items-center justify-between">
					<span class="flex size-9 items-center justify-center rounded-xl bg-violet-50 text-violet-700">
						<Icon name="star" size={18} />
					</span>
					<span class="chip {verifikasiOutput.length
						? 'bg-sun-400/25 text-sun-600'
						: 'bg-ink-100 text-ink-500'}">{verifikasiOutput.length}</span>
				</div>
				<p class="text-sm font-bold text-ink-800">Verifikasi Output</p>
				<p class="text-[11px] text-ink-500">
					{verifikasiOutput.length
						? `${verifikasiOutput.length} output menunggu verifikasi`
						: 'Tidak ada yang menunggu'}
				</p>
			</a>
		</section>
	{/if}

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

	<section class="kartu p-4">
		<div class="flex items-start justify-between gap-3">
			<div>
				<p class="judul-bagian">Nilai Kinerja Saya</p>
				<p class="teks-kecil">{periode?.nama ?? 'Periode berjalan'}</p>
			</div>
			{#if nilai}
				<span class="chip {warnaKategori(nilai.data.kategori)}">
					{LABEL_KATEGORI[nilai.data.kategori] ?? nilai.data.kategori}
				</span>
			{/if}
		</div>

		{#if nilai}
			<div class="mt-3 flex items-end gap-2">
				<p class="text-3xl font-black tracking-tight text-ink-900">
					{angka(nilai.data.nilai_akhir, 2)}
				</p>
				<p class="mb-1 text-[11px] text-ink-400">dari 100</p>
			</div>
			<div class="mt-3 space-y-2">
				{#each [
					{ label: 'Absensi', skor: nilai.data.skor_absensi, warna: 'bg-emerald-500' },
					{ label: 'Rencana Kerja', skor: nilai.data.skor_rencana_kerja, warna: 'bg-sky-500' },
					{ label: 'Hasil Kerja', skor: nilai.data.skor_hasil_kerja, warna: 'bg-brand-700' },
					{ label: 'Output', skor: nilai.data.skor_output, warna: 'bg-sun-500' }
				] as bar (bar.label)}
					<div>
						<div class="flex justify-between text-[11px] text-ink-500">
							<span>{bar.label}</span>
							<span class="font-semibold text-ink-700">{angka(bar.skor, 1)}</span>
						</div>
						<div class="mt-1 h-1.5 overflow-hidden rounded-full bg-ink-100">
							<div
								class="h-full rounded-full {bar.warna}"
								style="width: {Math.min(100, bar.skor)}%"
							></div>
						</div>
					</div>
				{/each}
			</div>
		{:else}
			<p class="mt-2 text-xs text-ink-500">Nilai belum dihitung pada periode ini.</p>
		{/if}
	</section>

	<section class="grid grid-cols-3 gap-2">
		{#each [
			{ label: 'Rencana', jumlah: rencana.length, href: '/rencana' },
			{ label: 'Hasil', jumlah: hasil.length, href: '/hasil' },
			{ label: 'Output', jumlah: output.length, href: '/output' }
		] as item (item.label)}
			<a href={item.href} class="kartu flex flex-col items-center gap-0.5 px-2 py-3">
				<span class="text-xl font-bold text-ink-900">{memuat ? '…' : item.jumlah}</span>
				<span class="text-[10px] font-medium text-ink-500">{item.label}</span>
			</a>
		{/each}
	</section>

	<p class="px-1 pb-2 text-center text-[10px] leading-relaxed text-ink-400">
		Absensi utama dilakukan melalui aplikasi ini. Bila tidak ada sinyal, data disimpan di
		perangkat dan dikirim saat Anda menekan Kirim Antrean.
	</p>
</div>
