<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import LampiranList from '#lib/components/LampiranList.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import { angka, LABEL_JENIS_OUTPUT, LABEL_STATUS_OUTPUT, statusOutputWarna, tanggalIndo } from '#lib/format';
	import type { OutputDetail } from '#lib/types';

	let detail = $state<OutputDetail | null>(null);
	let memuat = $state(true);
	let galat = $state<string | null>(null);

	async function muat() {
		const id = page.params.id;
		if (!id) return;
		memuat = true;
		try {
			detail = await api<OutputDetail>(`/output/${id}`);
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Detail output tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Detail Output — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	<a href="/output" class="btn btn-hantu -ml-2 px-2 py-1"><Icon name="left" size={16} /> Kembali</a>

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat output…</div>
	{:else if galat}
		<Pesan jenis="galat" pesan={galat} />
	{:else if detail}
		<article class="kartu p-4">
			<div class="flex items-start justify-between gap-3">
				<div>
					<h1 class="text-lg font-bold leading-snug text-ink-900">{detail.judul}</h1>
					<p class="text-[11px] text-ink-500">
						{LABEL_JENIS_OUTPUT[detail.jenis_output] ?? detail.jenis_output} · {tanggalIndo(detail.tanggal_hasil)}
					</p>
				</div>
				<span class="chip shrink-0 {statusOutputWarna(detail.status)}">
					{LABEL_STATUS_OUTPUT[detail.status]}
				</span>
			</div>

			{#if detail.rencana_kegiatan}
				<p class="mt-2 text-xs text-ink-500">Hasil kerja: {detail.rencana_kegiatan}</p>
			{/if}

			<dl class="mt-3">
				<div class="baris-info">
					<dt>Kelengkapan</dt>
					<dd>{angka(detail.skor_kelengkapan, 1)}</dd>
				</div>
				<div class="baris-info">
					<dt>Kesesuaian</dt>
					<dd>{angka(detail.skor_kesesuaian, 1)}</dd>
				</div>
				<div class="baris-info">
					<dt>Keabsahan</dt>
					<dd>{angka(detail.skor_keabsahan, 1)}</dd>
				</div>
				<div class="baris-info">
					<dt>Skor total</dt>
					<dd>{angka(detail.skor_total, 2)}</dd>
				</div>
			</dl>
		</article>

		{#if detail.uraian}
			<section class="kartu p-4">
				<p class="judul-bagian">Uraian</p>
				<p class="mt-1 whitespace-pre-line text-sm leading-relaxed text-ink-600">{detail.uraian}</p>
			</section>
		{/if}

		{#if detail.url_eksternal}
			<section class="kartu p-4">
				<p class="judul-bagian">Tautan</p>
				<a
					href={detail.url_eksternal}
					target="_blank"
					rel="noopener noreferrer"
					class="mt-1 block break-all text-xs font-semibold text-brand-700 underline"
				>
					{detail.url_eksternal}
				</a>
			</section>
		{/if}

		{#if detail.catatan_verifikator}
			<Pesan jenis="info" pesan={`Catatan verifikator: ${detail.catatan_verifikator}`} />
		{/if}

		<section class="kartu p-4">
			<LampiranList lampiran={detail.lampiran} judul="Berkas Bukti" />
			{#if detail.lampiran.length === 0}
				<p class="text-xs text-ink-400">Tidak ada berkas bukti.</p>
			{/if}
		</section>

		{#if detail.status === 'draft' || detail.status === 'perlu_revisi'}
			<a href={`/output/form?id=${detail.id}`} class="btn btn-utama w-full py-3">
				<Icon name="edit" size={17} />
				{detail.status === 'perlu_revisi' ? 'Perbaiki Output' : 'Ubah Output'}
			</a>
		{/if}
	{/if}
</div>
