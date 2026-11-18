<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import {
		angka,
		LABEL_STATUS_HASIL,
		statusHasilWarna,
		tanggalIndo
	} from '#lib/format';
	import { bisaMenyetujui, session } from '#lib/stores/session.svelte';
	import type { Hasil, ListResponse, StatusHasil } from '#lib/types';

	let daftar = $state<Hasil[]>([]);
	let jumlahVerifikasi = $state(0);
	let memuat = $state(true);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);
	let filter = $state<'semua' | StatusHasil>('semua');

	const bisaVerifikasi = $derived(bisaMenyetujui());
	const tampil = $derived(
		filter === 'semua' ? daftar : daftar.filter((item) => item.status === filter)
	);

	async function muat() {
		const user = session.user;
		if (!user) return;
		memuat = true;
		galat = null;
		try {
			const hasil = await api<ListResponse<Hasil>>(
				`/hasil-kerja?per_page=50&personel_id=${user.id}`
			);
			daftar = hasil.data;
			if (bisaVerifikasi) {
				try {
					const verifikasi = await api<ListResponse<Hasil>>(
						'/hasil-kerja/verifikasi?per_page=50'
					);
					jumlahVerifikasi = verifikasi.meta.total;
				} catch {
					jumlahVerifikasi = 0;
				}
			}
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Hasil kerja tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	async function ajukan(item: Hasil) {
		if (memproses) return;
		memproses = true;
		pesan = null;
		galat = null;
		try {
			await api(`/hasil-kerja/${item.id}/ajukan`, { method: 'POST' });
			pesan = 'Hasil kerja diajukan untuk verifikasi.';
			await muat();
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Gagal mengajukan hasil kerja.';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Hasil Kerja — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if bisaVerifikasi}
		<a href="/hasil/verifikasi" class="kartu flex items-center gap-3 border-emerald-200 bg-emerald-50/70 p-4">
			<span class="flex size-10 items-center justify-center rounded-xl bg-white text-emerald-700 ring-1 ring-emerald-100">
				<Icon name="checkCircle" size={20} />
			</span>
			<span class="min-w-0 flex-1">
				<span class="block text-sm font-bold text-emerald-900">Verifikasi Hasil Kerja</span>
				<span class="block text-[11px] text-emerald-700">
					{jumlahVerifikasi
						? `${jumlahVerifikasi} hasil menunggu verifikasi`
						: 'Tidak ada hasil yang menunggu'}
				</span>
			</span>
			<Icon name="right" size={18} class="text-emerald-600" />
		</a>
	{/if}

	<div class="flex items-center justify-between gap-2">
		<div>
			<h1 class="text-lg font-bold text-ink-900">Hasil Kerja</h1>
			<p class="text-[11px] text-ink-500">{daftar.length} laporan realisasi</p>
		</div>
		<a href="/hasil/form" class="btn btn-utama px-3 py-2 text-xs">
			<Icon name="plus" size={16} /> Baru
		</a>
	</div>

	<div class="-mx-4 overflow-x-auto px-4">
		<div class="flex w-max gap-1.5">
			{#each [
				{ nilai: 'semua' as const, label: 'Semua' },
				{ nilai: 'draft' as const, label: 'Draft' },
				{ nilai: 'diajukan' as const, label: 'Diajukan' },
				{ nilai: 'terverifikasi' as const, label: 'Terverifikasi' },
				{ nilai: 'perlu_revisi' as const, label: 'Revisi' }
			] as opsi (opsi.nilai)}
				<button
					type="button"
					class="chip whitespace-nowrap border px-3 py-1.5 {filter === opsi.nilai
						? 'border-brand-600 bg-brand-600 text-white'
						: 'border-ink-200 bg-white text-ink-500'}"
					onclick={() => (filter = opsi.nilai)}
				>
					{opsi.label}
				</button>
			{/each}
		</div>
	</div>

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat hasil kerja…</div>
	{:else if tampil.length === 0}
		<div class="kartu p-6 text-center">
			<p class="text-sm font-semibold text-ink-600">Belum ada hasil kerja.</p>
			<p class="mt-1 text-xs text-ink-400">Hasil kerja dilaporkan dari rencana yang sudah disetujui.</p>
		</div>
	{:else}
		{#each tampil as item (item.id)}
			<article class="kartu p-4">
				<div class="flex items-start justify-between gap-3">
					<div class="min-w-0">
						<p class="text-sm font-bold text-ink-900">{item.rencana_kegiatan ?? 'Hasil kerja'}</p>
						<p class="text-[11px] text-ink-500">{tanggalIndo(item.tanggal_realisasi)}</p>
					</div>
					<span class="chip shrink-0 {statusHasilWarna(item.status)}">
						{LABEL_STATUS_HASIL[item.status]}
					</span>
				</div>

				<p class="mt-2 line-clamp-2 text-xs leading-relaxed text-ink-600">
					{item.uraian_pelaksanaan}
				</p>

				<div class="mt-3">
					<div class="flex justify-between text-[11px] text-ink-500">
						<span>Progres</span>
						<span class="font-semibold text-ink-700">{angka(item.progres_persen, 0)}%</span>
					</div>
					<div class="mt-1 h-1.5 overflow-hidden rounded-full bg-ink-100">
						<div
							class="h-full rounded-full bg-brand-600"
							style="width: {Math.min(100, item.progres_persen)}%"
						></div>
					</div>
				</div>

				{#if item.catatan_verifikator}
					<Pesan jenis="info" pesan={`Catatan verifikator: ${item.catatan_verifikator}`} class="mt-2" />
				{/if}

				<div class="mt-3 flex flex-wrap gap-2">
					<a href={`/hasil/${item.id}`} class="btn btn-kedua flex-1 py-2 text-xs">
						<Icon name="file" size={15} /> Detail
					</a>
					{#if item.status === 'draft' || item.status === 'perlu_revisi'}
						<a href={`/hasil/form?id=${item.id}`} class="btn btn-kedua flex-1 py-2 text-xs">
							<Icon name="edit" size={15} />
							{item.status === 'perlu_revisi' ? 'Perbaiki' : 'Ubah'}
						</a>
						<button
							class="btn btn-utama w-full py-2 text-xs"
							disabled={memproses}
							onclick={() => ajukan(item)}
						>
							<Icon name="send" size={15} /> Ajukan
						</button>
					{/if}
				</div>
			</article>
		{/each}
	{/if}
</div>
