<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, apiBlobUrl, ApiError } from '#lib/api/client';
	import { adalahAtasan } from '#lib/stores/session.svelte';
	import type { Lampiran, Rencana, Tahapan } from '#lib/types';
	import { LABEL_STATUS_RENCANA } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';

	interface Detail extends Rencana {
		tahapan: Tahapan[];
		lampiran: Lampiran[];
	}

	let detail = $state<Detail | null>(null);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let catatan = $state('');
	let memproses = $state(false);
	let mengunggah = $state(false);

	const id = $derived(page.params.id);

	async function muat() {
		memuat = true;
		error = null;
		try {
			detail = await api<Detail>(`/rencana-kerja/${id}`);
			catatan = detail.catatan_atasan ?? '';
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat detail';
		} finally {
			memuat = false;
		}
	}

	async function putuskan(status: 'disetujui' | 'perlu_revisi' | 'ditolak') {
		memproses = true;
		error = null;
		try {
			await api(`/rencana-kerja/${id}/keputusan`, {
				method: 'POST',
				body: { status, catatan: catatan || null }
			});
			await muat();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan keputusan';
		} finally {
			memproses = false;
		}
	}

	async function unggah(event: Event) {
		const input = event.target as HTMLInputElement;
		const berkas = input.files?.[0];
		if (!berkas) return;

		mengunggah = true;
		error = null;
		try {
			const data = new FormData();
			data.append('owner_type', 'rencana');
			data.append('owner_id', String(id));
			data.append('kategori', 'lampiran');
			data.append('file', berkas);
			await api('/lampiran', { method: 'POST', body: data });
			input.value = '';
			await muat();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal mengunggah lampiran';
		} finally {
			mengunggah = false;
		}
	}

	async function unduh(item: Lampiran) {
		try {
			const url = await apiBlobUrl(item.storage_key);
			const tautan = document.createElement('a');
			tautan.href = url;
			tautan.download = item.file_name;
			tautan.click();
			URL.revokeObjectURL(url);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal mengunduh';
		}
	}

	async function hapusLampiran(item: Lampiran) {
		if (!confirm(`Hapus lampiran ${item.file_name}?`)) return;
		try {
			await api(`/lampiran/${item.id}`, { method: 'DELETE' });
			await muat();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menghapus lampiran';
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Detail Rencana Kerja — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<a href="/rencana-kerja" class="inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-800 no-print">
		<Icon name="left" size={16} /> Kembali ke daftar
	</a>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	{#if memuat}
		<div class="card p-8 text-center text-sm text-slate-500">Memuat…</div>
	{:else if detail}
		<div class="grid gap-4 lg:grid-cols-3">
			<div class="card p-5 lg:col-span-2">
				<div class="flex flex-wrap items-start justify-between gap-3">
					<div>
						<h2 class="text-lg font-semibold text-slate-800">{detail.kegiatan}</h2>
						<p class="text-sm text-slate-500">{detail.program}</p>
					</div>
					<Badge tone={detail.status === 'disetujui' ? 'green' : detail.status === 'perlu_revisi' ? 'amber' : detail.status === 'ditolak' ? 'red' : 'blue'}>
						{LABEL_STATUS_RENCANA[detail.status]}
					</Badge>
				</div>

				<dl class="mt-5 grid gap-4 text-sm sm:grid-cols-2">
					<div class="sm:col-span-2">
						<dt class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Uraian</dt>
						<dd class="mt-1 text-slate-700">{detail.uraian}</dd>
					</div>
					<div>
						<dt class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Tujuan</dt>
						<dd class="mt-1 text-slate-700">{detail.tujuan ?? '-'}</dd>
					</div>
					<div>
						<dt class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Indikator</dt>
						<dd class="mt-1 text-slate-700">{detail.indikator_kinerja ?? '-'}</dd>
					</div>
					<div>
						<dt class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Dasar Tugas</dt>
						<dd class="mt-1 text-slate-700">
							{detail.dasar_jenis?.toUpperCase() ?? '-'} · {detail.dasar_nomor ?? '-'}
							{#if detail.dasar_perihal}<span class="block text-xs text-slate-500">{detail.dasar_perihal}</span>{/if}
						</dd>
					</div>
					<div>
						<dt class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Target</dt>
						<dd class="mt-1 text-slate-700">
							{detail.target_kuantitas ?? '-'} {detail.satuan ?? ''} · {detail.target_kualitas ?? '-'}
						</dd>
					</div>
					<div>
						<dt class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Periode Kerja</dt>
						<dd class="mt-1 text-slate-700">{detail.tanggal_mulai} → {detail.tanggal_selesai}</dd>
					</div>
					<div>
						<dt class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Target Waktu</dt>
						<dd class="mt-1 text-slate-700">{detail.target_waktu ?? '-'}</dd>
					</div>
				</dl>

				{#if detail.tahapan.length}
					<div class="mt-5">
						<h3 class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Rincian Tahapan</h3>
						<ol class="mt-2 space-y-1 text-sm text-slate-700">
							{#each detail.tahapan as tahap}
								<li>{tahap.urutan}. {tahap.uraian}
									{#if tahap.target_kuantitas}<span class="text-xs text-slate-500">({tahap.target_kuantitas} {tahap.satuan ?? ''})</span>{/if}
								</li>
							{/each}
						</ol>
					</div>
				{/if}

				{#if detail.catatan_atasan}
					<div class="mt-5 rounded-lg bg-amber-50 p-3 text-sm text-amber-800 ring-1 ring-amber-200">
						<p class="text-xs font-semibold uppercase">Catatan Atasan</p>
						<p class="mt-1">{detail.catatan_atasan}</p>
					</div>
				{/if}
			</div>

			<div class="space-y-4">
				<div class="card p-4">
					<h3 class="text-sm font-semibold text-slate-700">Lampiran</h3>
					<ul class="mt-2 space-y-1 text-sm">
						{#each detail.lampiran as item}
							<li class="flex items-center justify-between gap-2 rounded-lg bg-slate-50 px-2 py-1.5">
								<button class="truncate text-left text-navy-800 hover:underline" onclick={() => unduh(item)}>
									{item.file_name}
								</button>
								<button class="text-rose-600 no-print" onclick={() => hapusLampiran(item)} aria-label="Hapus">
									<Icon name="x" size={14} />
								</button>
							</li>
						{:else}
							<li class="text-xs text-slate-500">Belum ada lampiran.</li>
						{/each}
					</ul>

					<label class="btn btn-ghost mt-3 w-full border border-dashed border-slate-300 text-xs no-print">
						<Icon name="upload" size={14} /> {mengunggah ? 'Mengunggah…' : 'Unggah lampiran (PDF/JPG/PNG, maks 10 MB)'}
						<input type="file" class="hidden" accept=".pdf,.jpg,.jpeg,.png" onchange={unggah} />
					</label>
				</div>

				{#if adalahAtasan() && detail.status === 'diajukan'}
					<div class="card p-4 no-print">
						<h3 class="text-sm font-semibold text-slate-700">Keputusan Atasan</h3>
						<label class="label mt-3" for="catatan">Catatan</label>
						<textarea id="catatan" class="input h-24 resize-none" bind:value={catatan}></textarea>
						<div class="mt-3 grid gap-2">
							<button class="btn btn-primary" disabled={memproses} onclick={() => putuskan('disetujui')}>
								<Icon name="check" size={16} /> Setujui
							</button>
							<button class="btn btn-gold" disabled={memproses} onclick={() => putuskan('perlu_revisi')}>
								Minta Revisi
							</button>
							<button class="btn btn-ghost border border-rose-200 text-rose-600" disabled={memproses} onclick={() => putuskan('ditolak')}>
								Tolak
							</button>
						</div>
					</div>
				{/if}
			</div>
		</div>
	{/if}
</div>
