<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import { tanggalIndo, warnaStatus, LABEL_STATUS_KEHADIRAN, meter } from '#lib/format';
	import { bacaAntreanAbsensi, sinkronkanAntrean } from '#lib/offline';
	import { session } from '#lib/stores/session.svelte';
	import type { Absensi, AbsensiAntrean } from '#lib/types';

	let riwayat = $state<Absensi[]>([]);
	let antrean = $state<AbsensiAntrean[]>([]);
	let memuat = $state(true);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);

	async function muat() {
		memuat = true;
		galat = null;
		try {
			riwayat = await api<Absensi[]>('/absensi/saya');
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Riwayat tidak dapat dimuat.';
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
			antrean = bacaAntreanAbsensi(user.id);
			await muat();
			pesan = hasil.absensi
				? `${hasil.absensi} absensi berhasil dikirim.`
				: 'Tidak ada antrean yang dapat dikirim.';
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Gagal mengirim antrean.';
		} finally {
			memproses = false;
		}
	}

	onMount(() => {
		const user = session.user;
		if (user) antrean = bacaAntreanAbsensi(user.id);
		void muat();
	});
</script>

<svelte:head><title>Riwayat Absensi — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	<a href="/absensi" class="btn btn-hantu -ml-2 px-2 py-1">
		<Icon name="left" size={16} /> Kembali
	</a>

	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if antrean.length > 0}
		<section class="kartu border-amber-200 bg-amber-50/60 p-4">
			<p class="judul-bagian text-amber-900">Menunggu Dikirim</p>
			<p class="text-xs text-amber-800">{antrean.length} absensi tersimpan offline</p>
			<div class="mt-2 space-y-1">
				{#each antrean.slice(-5).reverse() as item (item.client_uuid)}
					<p class="text-[11px] text-amber-900">
						{tanggalIndo(item.captured_at.slice(0, 10))} · {item.jenis}
					</p>
				{/each}
			</div>
			<button
				class="btn btn-kedua mt-3 w-full border-amber-200 bg-amber-100 text-amber-900"
				onclick={sinkronkan}
				disabled={memproses}
			>
				<Icon name="refresh" size={16} />
				{memproses ? 'Mengirim…' : 'Kirim Semua Antrean'}
			</button>
		</section>
	{/if}

	<section class="kartu p-4">
		<div class="flex items-center justify-between gap-2">
			<div>
				<p class="judul-bagian">Bulan Ini</p>
				<p class="teks-kecil">{riwayat.length} catatan</p>
			</div>
			<button class="btn btn-hantu px-2 py-1 text-xs" onclick={muat} disabled={memuat}>
				<Icon name="refresh" size={15} /> Muat ulang
			</button>
		</div>

		{#if memuat}
			<p class="mt-4 text-sm text-ink-400">Memuat riwayat…</p>
		{:else if riwayat.length === 0}
			<p class="mt-4 text-sm text-ink-400">Belum ada absensi pada bulan ini.</p>
		{:else}
			<ul class="mt-3 divide-y divide-ink-100">
				{#each riwayat as item (item.id)}
					<li class="flex items-start justify-between gap-3 py-3">
						<div class="min-w-0">
							<p class="text-sm font-semibold text-ink-800">
								{tanggalIndo(item.tanggal)} · {item.jenis === 'masuk' ? 'Masuk' : 'Pulang'}
							</p>
							<p class="text-[11px] text-ink-500">
								{LABEL_STATUS_KEHADIRAN[item.status_kehadiran]}
								{#if item.jarak_m !== null}· {meter(item.jarak_m)}{/if}
								{#if item.perlu_verifikasi}· menunggu verifikasi{/if}
							</p>
						</div>
						<span class="chip shrink-0 {warnaStatus(item.status_kehadiran)}">{item.jam}</span>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>
