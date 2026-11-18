<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import LampiranList from '#lib/components/LampiranList.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import {
		angka,
		LABEL_JENIS_RENCANA,
		LABEL_STATUS_RENCANA,
		statusRencanaWarna,
		tanggalIndo
	} from '#lib/format';
	import type { RencanaDetail } from '#lib/types';

	let detail = $state<RencanaDetail | null>(null);
	let memuat = $state(true);
	let galat = $state<string | null>(null);

	async function muat() {
		const id = page.params.id;
		if (!id) return;
		memuat = true;
		galat = null;
		try {
			detail = await api<RencanaDetail>(`/rencana-kerja/${id}`);
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Detail rencana tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Detail Rencana — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	<a href="/rencana" class="btn btn-hantu -ml-2 px-2 py-1">
		<Icon name="left" size={16} /> Kembali
	</a>

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat detail rencana…</div>
	{:else if galat}
		<Pesan jenis="galat" pesan={galat} />
	{:else if detail}
		<article class="kartu p-4">
			<div class="flex items-start justify-between gap-3">
				<div>
					<h1 class="text-lg font-bold leading-snug text-ink-900">{detail.kegiatan}</h1>
					<p class="text-xs text-ink-500">{detail.program}</p>
				</div>
				<span class="chip shrink-0 {statusRencanaWarna(detail.status)}">
					{LABEL_STATUS_RENCANA[detail.status]}
				</span>
			</div>

			<dl class="mt-3">
				<div class="baris-info">
					<dt>Jenis</dt>
					<dd>{LABEL_JENIS_RENCANA[detail.jenis] ?? detail.jenis}</dd>
				</div>
				<div class="baris-info">
					<dt>Periode</dt>
					<dd>{tanggalIndo(detail.tanggal_mulai)} – {tanggalIndo(detail.tanggal_selesai)}</dd>
				</div>
				<div class="baris-info">
					<dt>Target</dt>
					<dd>{angka(detail.target_kuantitas)} {detail.satuan ?? ''}</dd>
				</div>
				{#if detail.target_kualitas}
					<div class="baris-info">
						<dt>Kualitas</dt>
						<dd>{detail.target_kualitas}</dd>
					</div>
				{/if}
				{#if detail.dasar_jenis}
					<div class="baris-info">
						<dt>Dasar tugas</dt>
						<dd>
							{detail.dasar_jenis === 'sp'
								? 'Surat Perintah'
								: detail.dasar_jenis === 'tor'
									? 'TOR'
									: 'Perintah Lisan'}
						</dd>
					</div>
				{/if}
				{#if detail.dasar_nomor}
					<div class="baris-info">
						<dt>Nomor</dt>
						<dd>{detail.dasar_nomor}</dd>
					</div>
				{/if}
				{#if detail.dasar_tanggal}
					<div class="baris-info">
						<dt>Tanggal dasar</dt>
						<dd>{tanggalIndo(detail.dasar_tanggal)}</dd>
					</div>
				{/if}
				{#if detail.dasar_perihal}
					<div class="baris-info">
						<dt>Perihal</dt>
						<dd>{detail.dasar_perihal}</dd>
					</div>
				{/if}
			</dl>
		</article>

		{#if detail.uraian}
			<section class="kartu p-4">
				<p class="judul-bagian">Uraian</p>
				<p class="mt-1 whitespace-pre-line text-sm leading-relaxed text-ink-600">{detail.uraian}</p>
				{#if detail.tujuan}
					<p class="mt-3 text-[11px] font-bold uppercase tracking-wider text-ink-400">Tujuan</p>
					<p class="text-sm leading-relaxed text-ink-600">{detail.tujuan}</p>
				{/if}
				{#if detail.indikator_kinerja}
					<p class="mt-3 text-[11px] font-bold uppercase tracking-wider text-ink-400">Indikator</p>
					<p class="text-sm leading-relaxed text-ink-600">{detail.indikator_kinerja}</p>
				{/if}
			</section>
		{/if}

		{#if detail.tahapan.length > 0}
			<section class="kartu p-4">
				<p class="judul-bagian">Rincian Kegiatan</p>
				<div class="mt-2 space-y-3">
					{#each detail.tahapan.slice().sort((a, b) => a.urutan - b.urutan) as tahap, i (tahap.id)}
						<div class="rounded-xl border border-ink-100 p-3">
							<div class="flex items-start gap-2">
								<span class="flex size-6 shrink-0 items-center justify-center rounded-full bg-brand-50 text-[11px] font-bold text-brand-700">
									{i + 1}
								</span>
								<div class="min-w-0">
									<p class="text-sm font-semibold text-ink-800">{tahap.uraian}</p>
									{#if tahap.program || tahap.kegiatan}
										<p class="text-[11px] text-ink-500">
											{[tahap.program, tahap.kegiatan].filter(Boolean).join(' · ')}
										</p>
									{/if}
									{#if tahap.target_kuantitas !== null || tahap.satuan}
										<p class="text-[11px] text-ink-500">
											Target: {angka(tahap.target_kuantitas)} {tahap.satuan ?? ''}
										</p>
									{/if}
									{#if tahap.tanggal_mulai || tahap.tanggal_selesai}
										<p class="text-[11px] text-ink-400">
											{tanggalIndo(tahap.tanggal_mulai)} – {tanggalIndo(tahap.tanggal_selesai)}
										</p>
									{/if}
								</div>
							</div>
						</div>
					{/each}
				</div>
			</section>
		{/if}

		{#if detail.catatan_atasan}
			<Pesan jenis="info" pesan={`Catatan atasan: ${detail.catatan_atasan}`} />
		{/if}

		<section class="kartu p-4">
			<LampiranList lampiran={detail.lampiran} judul="Lampiran Pendukung" />
			{#if detail.lampiran.length === 0}
				<p class="text-xs text-ink-400">Tidak ada lampiran.</p>
			{/if}
		</section>

		{#if detail.status === 'draft' || detail.status === 'perlu_revisi'}
			<a href={`/rencana/form?id=${detail.id}`} class="btn btn-utama w-full py-3">
				<Icon name="edit" size={17} />
				{detail.status === 'perlu_revisi' ? 'Perbaiki Rencana' : 'Ubah Rencana'}
			</a>
		{/if}
	{/if}
</div>
