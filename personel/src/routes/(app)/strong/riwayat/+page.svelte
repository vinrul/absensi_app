<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import { awalBulan, jamTeks, tanggalIndo } from '#lib/format';
	import { bacaAntreanPh, sinkronkanAntrean } from '#lib/offline';
	import { session } from '#lib/stores/session.svelte';
	import { keteranganJadwal } from '#lib/strong';
	import type { AbsensiPh, AbsensiPhAntrean, JadwalPhSaya } from '#lib/types';

	let absensi = $state<AbsensiPh[]>([]);
	let jadwal = $state<JadwalPhSaya[]>([]);
	let antrean = $state<AbsensiPhAntrean[]>([]);
	let memuat = $state(true);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);

	async function muat() {
		memuat = true;
		galat = null;
		const bulan = awalBulan().slice(0, 7);
		try {
			[absensi, jadwal] = await Promise.all([
				api<AbsensiPh[]>(`/ph/absensi/saya?bulan=${bulan}`),
				api<JadwalPhSaya[]>(`/ph/jadwal/saya?bulan=${bulan}`)
			]);
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Riwayat strong point tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	async function sinkronkan() {
		const user = session.user;
		if (!user || memproses) return;
		memproses = true;
		pesan = null;
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
							: 'Tidak ada antrean yang dapat dikirim.';
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Gagal mengirim antrean.';
		} finally {
			memproses = false;
		}
	}

	onMount(() => {
		const user = session.user;
		if (user) antrean = bacaAntreanPh(user.id);
		void muat();
	});
</script>

<svelte:head><title>Riwayat Strong Point — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	<a href="/strong" class="btn btn-hantu -ml-2 px-2 py-1"><Icon name="left" size={16} /> Kembali</a>

	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if antrean.length > 0}
		<section class="kartu border-amber-200 bg-amber-50/60 p-4">
			<p class="judul-bagian text-amber-900">Menunggu Dikirim</p>
			<p class="text-xs text-amber-800">{antrean.length} catatan offline</p>
			<div class="mt-2 space-y-1">
				{#each antrean.slice(-5).reverse() as item (item.client_uuid)}
					<p class="text-[11px] text-amber-900">
						{tanggalIndo(item.captured_at.slice(0, 10))} · {item.jenis === 'masuk'
							? 'Check-in'
							: 'Check-out'} · {jamTeks(item.captured_at)}
					</p>
				{/each}
			</div>
			<button
				class="btn btn-kedua mt-3 w-full border-amber-200 bg-amber-100 text-amber-900"
				onclick={sinkronkan}
				disabled={memproses}
			>
				<Icon name="refresh" size={16} /> {memproses ? 'Mengirim…' : 'Kirim Semua Antrean'}
			</button>
		</section>
	{/if}

	<section class="kartu p-4">
		<div class="flex items-center justify-between">
			<div>
				<p class="judul-bagian">Riwayat Strong Point</p>
				<p class="teks-kecil">{absensi.length} catatan bulan ini</p>
			</div>
			<button class="btn btn-hantu px-2 py-1 text-xs" onclick={muat} disabled={memuat}>
				<Icon name="refresh" size={15} /> Muat ulang
			</button>
		</div>

		{#if memuat}
			<p class="mt-4 text-sm text-ink-400">Memuat riwayat…</p>
		{:else if absensi.length === 0}
			<p class="mt-4 text-sm text-ink-400">Belum ada absensi strong point pada bulan ini.</p>
		{:else}
			<ul class="mt-2 divide-y divide-ink-100">
				{#each absensi.slice().sort((a, b) => b.captured_at.localeCompare(a.captured_at)) as item (item.id)}
					<li class="py-3">
						<div class="flex items-start justify-between gap-3">
							<div class="min-w-0">
								<p class="text-sm font-semibold text-ink-800">
									{tanggalIndo(item.captured_at.slice(0, 10))} ·
									{item.jenis === 'masuk' ? 'Check-in' : 'Check-out'}
								</p>
								<p class="truncate text-[11px] text-ink-500">{item.strong_point_nama}</p>
								{#if item.perlu_verifikasi && !item.verified_at}
									<p class="text-[10px] font-semibold text-amber-600">Menunggu verifikasi atasan</p>
								{/if}
							</div>
							<div class="text-right">
								<p class="text-sm font-bold text-ink-900">{item.jam}</p>
								<p class="text-[10px] {!item.dalam_radius ? 'text-rose-600' : 'text-ink-400'}">
									{item.jarak_m !== null ? `${Math.round(item.jarak_m)} m` : '-'}
								</p>
							</div>
						</div>
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	<section class="kartu p-4">
		<p class="judul-bagian">Jadwal Bulan Ini</p>
		<p class="teks-kecil">{jadwal.length} jadwal</p>
		{#if jadwal.length > 0}
			<ul class="mt-2 divide-y divide-ink-100">
				{#each jadwal as item (item.id)}
					<li class="flex items-center justify-between gap-3 py-2.5">
						<p class="text-xs font-semibold text-ink-700">{tanggalIndo(item.tanggal)}</p>
						<p class="text-[10px] text-ink-500">{keteranganJadwal(item.regu, item.strong_point_nama)}</p>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>
