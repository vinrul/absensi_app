<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import { geserTanggal, tanggalHariIni, tanggalIndo } from '#lib/format';
	import { bacaAntreanPh, sinkronkanAntrean } from '#lib/offline';
	import { session } from '#lib/stores/session.svelte';
	import { keteranganJadwal, statusStrongPoint } from '#lib/strong';
	import type { AbsensiPh, AbsensiPhAntrean, JadwalPhSaya } from '#lib/types';

	let jadwal = $state<JadwalPhSaya[]>([]);
	let absensi = $state<AbsensiPh[]>([]);
	let antrean = $state<AbsensiPhAntrean[]>([]);
	let memuat = $state(true);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);

	const hariIni = $derived(tanggalHariIni());
	const jadwalHariIni = $derived(jadwal.filter((item) => item.tanggal === hariIni));
	const jadwalMendatang = $derived(
		[...jadwal].filter((item) => item.tanggal >= hariIni).sort((a, b) => a.tanggal.localeCompare(b.tanggal) || a.jam_mulai.localeCompare(b.jam_mulai))
	);

	function statusSesi(item: JadwalPhSaya) {
		return statusStrongPoint(item.id, absensi, antrean);
	}

	function labelSesi(item: JadwalPhSaya) {
		const sesi = item.sesi === 'sore' ? 'Sore' : 'Pagi';
		return `${sesi} ${item.jam_mulai}–${item.jam_selesai}`;
	}

	async function muat() {
		const user = session.user;
		if (!user) return;
		memuat = true;
		galat = null;
		antrean = bacaAntreanPh(user.id);
		const dari = geserTanggal(hariIni, -7);
		const sampai = geserTanggal(hariIni, 30);
		try {
			[jadwal, absensi] = await Promise.all([
				api<JadwalPhSaya[]>(`/ph/jadwal/saya?dari=${dari}&sampai=${sampai}`),
				api<AbsensiPh[]>(`/ph/absensi/saya?dari=${dari}&sampai=${sampai}`)
			]);
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Data strong point tidak dapat dimuat.';
		} finally {
			memuat = false;
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
			antrean = bacaAntreanPh(user.id);
			await muat();
			pesan =
				hasil.ph > 0 && hasil.ditolak > 0
					? `${hasil.ph} strong point terkirim, ${hasil.ditolak} ditolak karena lokasi tiruan.`
					: hasil.ditolak > 0
						? `${hasil.ditolak} strong point ditolak karena lokasi tiruan.`
						: hasil.ph > 0
							? `${hasil.ph} strong point berhasil dikirim.`
							: 'Tidak ada antrean strong point yang dapat dikirim.';
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Gagal mengirim antrean.';
		} finally {
			memproses = false;
		}
	}

	onMount(() => {
		const kilat = page.url.searchParams.get('pesan');
		if (kilat) {
			pesan = kilat;
			void goto('/strong', { replaceState: true });
		}
		void muat();
	});
</script>

<svelte:head><title>Strong Point — Absensi & Strong Point</title></svelte:head>

<div class="space-y-3">
	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	<a
		href="/strong/wizard"
		class="relative block w-full overflow-hidden rounded-3xl bg-gradient-to-br from-brand-800 via-brand-700 to-brand-500 p-5 text-white shadow-lg"
	>
		<div class="pointer-events-none absolute -right-8 -bottom-10 size-40 rounded-full bg-white/10 blur-2xl"></div>
		<div class="relative flex items-center gap-4">
			<span class="flex size-14 items-center justify-center rounded-2xl bg-white/15 ring-1 ring-white/20">
				<Icon name="calendar" size={28} />
			</span>
			<div>
				<p class="text-lg font-bold">Strong Point</p>
				<p class="text-xs text-brand-100">Check-in / check-out dengan validasi lokasi</p>
			</div>
			<Icon name="right" size={20} class="ml-auto text-white/70" />
		</div>
	</a>

	<section class="kartu p-4">
		<p class="judul-bagian">Sesi Hari Ini</p>
		{#if memuat}
			<p class="mt-1 text-xs text-ink-400">Memuat jadwal…</p>
		{:else if jadwalHariIni.length === 0}
			<p class="mt-1 text-xs text-ink-400">Tidak ada jadwal strong point hari ini.</p>
		{:else}
			<ul class="mt-2 divide-y divide-ink-100">
				{#each jadwalHariIni as item (item.id)}
					{@const status = statusSesi(item)}
					<li class="py-3">
						<div class="flex items-start justify-between gap-3">
							<div class="min-w-0">
								<p class="text-xs font-semibold text-ink-800">{labelSesi(item)}</p>
								<p class="truncate text-[10px] text-ink-500">
									{keteranganJadwal(item.regu, item.strong_point_nama)}
								</p>
								<p class="mt-1 text-[11px] text-ink-600">
									{#if status.sesi.length === 0}
										Belum check-in
									{:else if status.terbuka}
										Check-in pukul {status.masukTerakhir?.jam} · belum check-out
									{:else}
										Sudah check-out pukul {status.pulangTerakhir?.jam}
									{/if}
								</p>
							</div>
							<span
								class="chip {status.terbuka
									? 'bg-rose-100 text-rose-700'
									: status.sesi.length
										? 'bg-emerald-100 text-emerald-700'
										: 'bg-brand-50 text-brand-700'}"
							>
								{status.terbuka ? 'Check-out' : status.sesi.length ? 'Selesai' : 'Check-in'}
							</span>
						</div>
						{#if !status.sesi.some((sesi) => sesi.jenis === 'masuk') || status.terbuka}
							<a
								href={`/strong/wizard?jadwal=${item.id}`}
								class="btn btn-utama mt-2 w-full"
							>
								<Icon name="pin" size={17} />
								{status.terbuka ? 'Check-out Sesi Ini' : 'Check-in Sesi Ini'}
							</a>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}

		{#if antrean.length > 0}
			<Pesan
				jenis="info"
				pesan={`${antrean.length} sesi strong point masih menunggu dikirim.`}
				class="mt-2"
			/>
		{/if}
	</section>

	{#if antrean.length > 0}
		<section class="kartu border-amber-200 bg-amber-50/60 p-4">
			<p class="judul-bagian text-amber-900">Antrean Strong Point</p>
			<p class="text-xs text-amber-800">{antrean.length} data menunggu dikirim ke server</p>
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

	<a href="/strong/riwayat" class="btn btn-kedua w-full">
		<Icon name="clock" size={17} /> Riwayat Strong Point
	</a>

	<section class="kartu p-4">
		<p class="judul-bagian">Jadwal Berikutnya</p>
		{#if memuat}
			<p class="mt-1 text-xs text-ink-400">Memuat jadwal…</p>
		{:else if jadwalMendatang.length === 0}
			<p class="mt-1 text-xs text-ink-400">Belum ada jadwal strong point.</p>
		{:else}
			<ul class="mt-2 divide-y divide-ink-100">
				{#each jadwalMendatang as item (item.id)}
					<li class="flex items-center justify-between gap-3 py-2.5">
						<div class="min-w-0">
							<p class="text-xs font-semibold text-ink-800">{tanggalIndo(item.tanggal)}</p>
							<p class="truncate text-[10px] text-ink-500">
								{labelSesi(item)} · {keteranganJadwal(item.regu, item.strong_point_nama)}
							</p>
						</div>
						{#if item.tanggal === hariIni}
							<span class="chip bg-brand-50 text-brand-700">Hari ini</span>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<p class="px-1 text-center text-[10px] leading-relaxed text-ink-400">
		Lokasi saat ini diambil hanya ketika Anda membuka wizard strong point.
	</p>
</div>
