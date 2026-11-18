<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import LampiranList from '#lib/components/LampiranList.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import { angka, LABEL_STATUS_HASIL, statusHasilWarna, tanggalIndo } from '#lib/format';
	import type { HasilDetail } from '#lib/types';

	let detail = $state<HasilDetail | null>(null);
	let memuat = $state(true);
	let galat = $state<string | null>(null);

	async function muat() {
		const id = page.params.id;
		if (!id) return;
		memuat = true;
		try {
			detail = await api<HasilDetail>(`/hasil-kerja/${id}`);
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Detail hasil kerja tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Detail Hasil — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	<a href="/hasil" class="btn btn-hantu -ml-2 px-2 py-1"><Icon name="left" size={16} /> Kembali</a>

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat hasil kerja…</div>
	{:else if galat}
		<Pesan jenis="galat" pesan={galat} />
	{:else if detail}
		<article class="kartu p-4">
			<div class="flex items-start justify-between gap-3">
				<div>
					<h1 class="text-lg font-bold leading-snug text-ink-900">
						{detail.rencana_kegiatan ?? 'Hasil kerja'}
					</h1>
					<p class="text-[11px] text-ink-500">{tanggalIndo(detail.tanggal_realisasi)}</p>
				</div>
				<span class="chip shrink-0 {statusHasilWarna(detail.status)}">
					{LABEL_STATUS_HASIL[detail.status]}
				</span>
			</div>

			<div class="mt-4">
				<div class="flex justify-between text-[11px] text-ink-500">
					<span>Progres</span>
					<span class="font-bold text-ink-700">{angka(detail.progres_persen, 0)}%</span>
				</div>
				<div class="mt-1 h-2 overflow-hidden rounded-full bg-ink-100">
					<div class="h-full rounded-full bg-brand-600" style="width: {Math.min(100, detail.progres_persen)}%"></div>
				</div>
			</div>

			<dl class="mt-3">
				<div class="baris-info">
					<dt>Jumlah realisasi</dt>
					<dd>{detail.jumlah_realisasi ?? '-'}</dd>
				</div>
				<div class="baris-info">
					<dt>Skor kuantitas</dt>
					<dd>{angka(detail.skor_kuantitas, 1)}</dd>
				</div>
				<div class="baris-info">
					<dt>Skor kualitas</dt>
					<dd>{angka(detail.skor_kualitas, 1)}</dd>
				</div>
				<div class="baris-info">
					<dt>Skor waktu</dt>
					<dd>{angka(detail.skor_waktu, 1)}</dd>
				</div>
				<div class="baris-info">
					<dt>Skor total</dt>
					<dd>{angka(detail.skor_total, 2)}</dd>
				</div>
			</dl>
		</article>

		<section class="kartu p-4">
			<p class="judul-bagian">Uraian Pelaksanaan</p>
			<p class="mt-1 whitespace-pre-line text-sm leading-relaxed text-ink-600">{detail.uraian_pelaksanaan}</p>
			{#if detail.kendala}
				<p class="mt-3 text-[11px] font-bold uppercase tracking-wider text-ink-400">Kendala</p>
				<p class="text-sm leading-relaxed text-ink-600">{detail.kendala}</p>
			{/if}
		</section>

		{#if detail.catatan_verifikator}
			<Pesan jenis="info" pesan={`Catatan verifikator: ${detail.catatan_verifikator}`} />
		{/if}

		<section class="kartu p-4">
			<LampiranList lampiran={detail.lampiran} judul="Bukti Pendukung" />
			{#if detail.lampiran.length === 0}
				<p class="text-xs text-ink-400">Tidak ada bukti pendukung.</p>
			{/if}
		</section>

		{#if detail.status === 'draft' || detail.status === 'perlu_revisi'}
			<a href={`/hasil/form?id=${detail.id}`} class="btn btn-utama w-full py-3">
				<Icon name="edit" size={17} />
				{detail.status === 'perlu_revisi' ? 'Perbaiki Hasil' : 'Ubah Hasil'}
			</a>
		{/if}
	{/if}
</div>
