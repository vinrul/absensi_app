<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, apiBlobUrl, ApiError } from '#lib/api/client';
	import type { HasilDetail } from '#lib/types';
	import { LABEL_STATUS_HASIL } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';

	let detail = $state<HasilDetail | null>(null);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let urlLampiran = $state<Record<string, string>>({});

	const id = $derived(page.params.id ?? '');

	function nada(status: string) {
		if (status === 'terverifikasi') return 'green';
		if (status === 'perlu_revisi') return 'amber';
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
			detail = await api<HasilDetail>(`/hasil-kerja/${id}`);
			for (const berkas of detail.lampiran) {
				try {
					urlLampiran[berkas.id] = await apiBlobUrl(berkas.storage_key);
				} catch {
					// abaikan lampiran yang gagal dimuat
				}
			}
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat detail hasil kerja';
		} finally {
			memuat = false;
		}
	});

	onDestroy(() => {
		for (const url of Object.values(urlLampiran)) URL.revokeObjectURL(url);
	});
</script>

<svelte:head><title>Detail Hasil Kerja — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<a class="btn btn-ghost border border-slate-300" href="/hasil-kerja">
		<Icon name="left" size={16} /> Kembali ke Hasil Kerja
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
					<h2 class="text-lg font-semibold text-slate-800">
						{detail.rencana_kegiatan ?? 'Hasil Kerja'}
					</h2>
					<p class="mt-1 text-xs text-slate-500">
						{detail.personel_nama ?? '-'} · NRP {detail.personel_nrp ?? '-'}
					</p>
				</div>
				<Badge tone={nada(detail.status)}>{LABEL_STATUS_HASIL[detail.status]}</Badge>
			</div>
			<div class="mt-4 grid gap-2 text-xs text-slate-600 sm:grid-cols-3">
				<p>Tanggal realisasi: <strong class="text-slate-800">{tanggal(detail.tanggal_realisasi)}</strong></p>
				<p>Progres: <strong class="text-slate-800">{detail.progres_persen.toFixed(0)}%</strong></p>
				<p>Jumlah realisasi: <strong class="text-slate-800">{detail.jumlah_realisasi ?? '-'}</strong></p>
			</div>
		</div>

		<div class="grid gap-4 lg:grid-cols-2">
			<div class="card p-5">
				<p class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Uraian Pelaksanaan</p>
				<p class="mt-2 text-sm whitespace-pre-line text-slate-700">{detail.uraian_pelaksanaan}</p>
				{#if detail.kendala}
					<div class="mt-3 rounded-lg bg-amber-50 px-3 py-2 text-xs text-amber-800 ring-1 ring-amber-200">
						Kendala: {detail.kendala}
					</div>
				{/if}
			</div>

			<div class="card p-5">
				<p class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Penilaian</p>
				{#if detail.skor_total != null}
					<dl class="mt-3 grid gap-3 text-sm sm:grid-cols-2">
						<div>
							<p class="label">Kuantitas</p>
							<p class="text-slate-700">{angka(detail.skor_kuantitas)}</p>
						</div>
						<div>
							<p class="label">Kualitas</p>
							<p class="text-slate-700">{angka(detail.skor_kualitas)}</p>
						</div>
						<div>
							<p class="label">Ketepatan waktu</p>
							<p class="text-slate-700">{angka(detail.skor_waktu)}</p>
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
			<h3 class="text-sm font-semibold text-slate-800">Lampiran Bukti</h3>
			<p class="mt-1 text-xs text-slate-500">Bukti pelaksanaan yang diunggah personel.</p>
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
					<li class="text-sm text-slate-500">Belum ada lampiran.</li>
				{/each}
			</ul>
		</div>
	{/if}
</div>
