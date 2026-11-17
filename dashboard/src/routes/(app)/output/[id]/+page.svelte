<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, apiBlobUrl, ApiError } from '#lib/api/client';
	import type { Lampiran, OutputDetail } from '#lib/types';
	import { LABEL_JENIS_OUTPUT, LABEL_STATUS_OUTPUT } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';

	let detail = $state<OutputDetail | null>(null);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let urlLampiran = $state<Record<string, string>>({});

	const id = $derived(page.params.id ?? '');

	function nada(status: string) {
		if (status === 'disetujui') return 'green';
		if (status === 'perlu_revisi') return 'amber';
		if (status === 'draft') return 'slate';
		return 'blue';
	}

	function tanggal(nilai: string | null) {
		if (!nilai) return '-';
		return new Date(`${nilai}T00:00:00`).toLocaleDateString('id-ID', {
			day: '2-digit',
			month: 'long',
			year: 'numeric'
		});
	}

	function angka(nilai: number | null) {
		return nilai == null ? '-' : nilai.toFixed(2);
	}

	function ukuran(bytes: number) {
		const kb = Math.round(bytes / 1024);
		return kb < 1 ? '<1 KB' : `${kb} KB`;
	}

	onMount(async () => {
		try {
			detail = await api<OutputDetail>(`/output/${id}`);
			for (const berkas of detail.lampiran) {
				try {
					urlLampiran[berkas.id] = await apiBlobUrl(berkas.storage_key);
				} catch {
					// abaikan lampiran yang gagal dimuat
				}
			}
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat detail output';
		} finally {
			memuat = false;
		}
	});

	onDestroy(() => {
		for (const url of Object.values(urlLampiran)) URL.revokeObjectURL(url);
	});
</script>

<svelte:head><title>Detail Output — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<a class="btn btn-ghost border border-slate-300" href="/output">
		<Icon name="left" size={16} /> Kembali ke Output
	</a>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	{#if memuat}
		<div class="card p-8 text-center text-sm text-slate-500">Memuat detail…</div>
	{:else if detail}
		<div class="card p-5">
			<div class="flex flex-wrap items-start justify-between gap-3">
				<div>
					<h2 class="text-lg font-semibold text-slate-800">{detail.judul}</h2>
					<p class="text-sm text-slate-500">{LABEL_JENIS_OUTPUT[detail.jenis_output] ?? detail.jenis_output}</p>
					<p class="mt-1 text-xs text-slate-500">
						{detail.personel_nama ?? '-'} · NRP {detail.personel_nrp ?? '-'}
					</p>
				</div>
				<Badge tone={nada(detail.status)}>{LABEL_STATUS_OUTPUT[detail.status]}</Badge>
			</div>
			<div class="mt-4 grid gap-2 text-xs text-slate-600 sm:grid-cols-3">
				<p>Tanggal hasil: <strong class="text-slate-800">{tanggal(detail.tanggal_hasil)}</strong></p>
				<p>Rencana: <strong class="text-slate-800">{detail.rencana_kegiatan ?? '-'}</strong></p>
				<p>Nilai total: <strong class="text-slate-800">{angka(detail.skor_total)}</strong></p>
			</div>
		</div>

		<div class="grid gap-4 lg:grid-cols-2">
			<div class="card p-5">
				<p class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Uraian Output</p>
				<p class="mt-2 text-sm whitespace-pre-line text-slate-700">
					{detail.uraian ?? 'Tidak ada uraian.'}
				</p>
				{#if detail.url_eksternal}
					<a
						class="mt-3 block text-sm text-navy-700 underline"
						href={detail.url_eksternal}
						target="_blank"
						rel="noreferrer"
					>
						{detail.url_eksternal}
					</a>
				{/if}
			</div>

			<div class="card p-5">
				<p class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Penilaian</p>
				{#if detail.skor_total != null}
					<dl class="mt-3 grid gap-3 text-sm sm:grid-cols-2">
						<div>
							<p class="label">Kelengkapan</p>
							<p class="text-slate-700">{angka(detail.skor_kelengkapan)}</p>
						</div>
						<div>
							<p class="label">Kesesuaian</p>
							<p class="text-slate-700">{angka(detail.skor_kesesuaian)}</p>
						</div>
						<div>
							<p class="label">Keabsahan</p>
							<p class="text-slate-700">{angka(detail.skor_keabsahan)}</p>
						</div>
						<div>
							<p class="label">Nilai total</p>
							<p class="font-semibold text-emerald-700">{angka(detail.skor_total)}</p>
						</div>
					</dl>
				{:else}
					<p class="mt-2 text-sm text-slate-500">Belum dinilai atasan.</p>
				{/if}
				{#if detail.catatan_verifikator}
					<div class="mt-3 rounded-lg bg-sky-50 px-3 py-2 text-xs text-sky-800 ring-1 ring-sky-200">
						Catatan atasan: {detail.catatan_verifikator}
					</div>
				{/if}
			</div>
		</div>

		<div class="card p-5">
			<h3 class="text-sm font-semibold text-slate-800">Berkas Bukti</h3>
			<p class="mt-1 text-xs text-slate-500">Dokumen, foto, atau berkas pendukung output.</p>
			<ul class="mt-3 space-y-2">
				{#each detail.lampiran as berkas}
					<li class="flex flex-wrap items-center justify-between gap-2 rounded-lg bg-slate-50 px-3 py-2">
						<span class="text-sm text-slate-700">
							{berkas.file_name}
							<span class="text-xs text-slate-500">· {ukuran(berkas.size_bytes)} · {berkas.mime}</span>
						</span>
						{#if urlLampiran[berkas.id]}
							<a
								class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
								href={urlLampiran[berkas.id]}
								target="_blank"
								rel="noreferrer"
							>
								<Icon name="download" size={14} /> Lihat
							</a>
						{:else}
							<span class="text-xs text-slate-400">Memuat…</span>
						{/if}
					</li>
				{:else}
					<li class="text-sm text-slate-500">Belum ada berkas bukti.</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>
