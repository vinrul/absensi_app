<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { page } from '$app/state';
	import { api, apiBlobUrl, ApiError } from '#lib/api/client';
	import { adalahAtasan } from '#lib/stores/session.svelte';
	import type { Lampiran, RencanaDetail } from '#lib/types';
	import { LABEL_JENIS_RENCANA, LABEL_STATUS_RENCANA } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';

	let detail = $state<RencanaDetail | null>(null);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let urlLampiran = $state<Record<string, string>>({});
	let catatan = $state('');
	let memproses = $state(false);
	let mengunggah = $state(false);

	const id = $derived(page.params.id ?? '');

	function nada(status: string) {
		if (status === 'disetujui') return 'green';
		if (status === 'diajukan') return 'blue';
		if (status === 'perlu_revisi') return 'amber';
		if (status === 'ditolak') return 'red';
		return 'slate';
	}

	function tanggal(nilai: string | null | undefined) {
		if (!nilai) return '-';
		return new Date(`${nilai}T00:00:00`).toLocaleDateString('id-ID', {
			day: '2-digit',
			month: 'long',
			year: 'numeric'
		});
	}

	function ukuran(bytes: number) {
		const kb = Math.round(bytes / 1024);
		return kb < 1 ? '<1 KB' : `${kb} KB`;
	}

	async function siapkanLampiran(daftar: Lampiran[]) {
		for (const url of Object.values(urlLampiran)) URL.revokeObjectURL(url);
		urlLampiran = {};
		for (const item of daftar) {
			try {
				urlLampiran[item.id] = await apiBlobUrl(item.storage_key);
			} catch {
				// biarkan tautan tidak tampil bila gagal dimuat
			}
		}
	}

	async function muat() {
		memuat = true;
		error = null;
		try {
			detail = await api<RencanaDetail>(`/rencana-kerja/${id}`);
			catatan = detail.catatan_atasan ?? '';
			await siapkanLampiran(detail.lampiran);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat detail rencana';
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

	onDestroy(() => {
		for (const url of Object.values(urlLampiran)) URL.revokeObjectURL(url);
	});
</script>

<svelte:head><title>Detail Rencana Kerja — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<a class="btn btn-ghost border border-slate-300" href="/rencana-kerja">
		<Icon name="left" size={16} /> Kembali ke Rencana Kerja
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
					<h2 class="text-lg font-semibold text-slate-800">{detail.kegiatan}</h2>
					<p class="text-sm text-slate-500">{detail.program}</p>
					<p class="mt-1 text-xs text-slate-500">
						{detail.personel_nama ?? '-'} · NRP {detail.personel_nrp ?? '-'}
					</p>
				</div>
				<Badge tone={nada(detail.status)}>{LABEL_STATUS_RENCANA[detail.status]}</Badge>
			</div>
			<div class="mt-4 grid gap-2 text-xs text-slate-600 sm:grid-cols-3">
				<p>Jenis: <strong class="text-slate-800">{LABEL_JENIS_RENCANA[detail.jenis] ?? detail.jenis}</strong></p>
				<p>Mulai: <strong class="text-slate-800">{tanggal(detail.tanggal_mulai)}</strong></p>
				<p>Selesai: <strong class="text-slate-800">{tanggal(detail.tanggal_selesai)}</strong></p>
				<p>Target: <strong class="text-slate-800">{detail.target_kuantitas ?? '-'} {detail.satuan ?? ''}</strong></p>
				<p>Kualitas: <strong class="text-slate-800">{detail.target_kualitas ?? '-'}</strong></p>
				<p>Target waktu: <strong class="text-slate-800">{tanggal(detail.target_waktu)}</strong></p>
			</div>
			{#if detail.jumlah_revisi > 0}
				<p class="mt-3 text-xs text-amber-700">Sudah {detail.jumlah_revisi}× diminta revisi.</p>
			{/if}
		</div>

		<div class="grid gap-4 lg:grid-cols-2">
			<div class="card space-y-3 p-5">
				<p class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Uraian Tugas</p>
				<p class="text-sm whitespace-pre-line text-slate-700">{detail.uraian}</p>
				{#if detail.tujuan}
					<div>
						<p class="label">Tujuan</p>
						<p class="text-sm text-slate-700">{detail.tujuan}</p>
					</div>
				{/if}
				{#if detail.indikator_kinerja}
					<div>
						<p class="label">Indikator Kinerja</p>
						<p class="text-sm text-slate-700">{detail.indikator_kinerja}</p>
					</div>
				{/if}
			</div>

			<div class="card space-y-3 p-5">
				<p class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Dasar Tugas</p>
				<dl class="grid gap-3 text-sm sm:grid-cols-2">
					<div>
						<p class="label">Jenis</p>
						<p class="text-slate-700">{detail.dasar_jenis ?? '-'}</p>
					</div>
					<div>
						<p class="label">Nomor</p>
						<p class="text-slate-700">{detail.dasar_nomor ?? '-'}</p>
					</div>
					<div>
						<p class="label">Tanggal</p>
						<p class="text-slate-700">{tanggal(detail.dasar_tanggal)}</p>
					</div>
					<div>
						<p class="label">Perihal</p>
						<p class="text-slate-700">{detail.dasar_perihal ?? '-'}</p>
					</div>
				</dl>
				{#if detail.catatan_atasan}
					<div class="rounded-lg bg-amber-50 px-3 py-2 text-xs text-amber-800 ring-1 ring-amber-200">
						Catatan atasan: {detail.catatan_atasan}
					</div>
				{/if}
			</div>
		</div>

		<div class="card overflow-hidden">
			<div class="border-b border-slate-200 px-5 py-4">
				<h3 class="text-sm font-semibold text-slate-800">Rincian Kegiatan</h3>
				<p class="text-xs text-slate-500">Tahapan yang tercatat pada rencana ini.</p>
			</div>
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">No</th>
							<th class="th">Uraian</th>
							<th class="th">Program / Kegiatan</th>
							<th class="th text-center">Target</th>
							<th class="th text-center">Bobot</th>
							<th class="th">Tanggal</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each detail.tahapan as tahap}
							<tr>
								<td class="td">{tahap.urutan}</td>
								<td class="td text-slate-700">{tahap.uraian}</td>
								<td class="td text-slate-600">
									{tahap.program ?? '-'}
									{#if tahap.kegiatan}<p class="text-[11px] text-slate-500">{tahap.kegiatan}</p>{/if}
								</td>
								<td class="td text-center text-slate-600">
									{tahap.target_kuantitas ?? '-'} {tahap.satuan ?? ''}
									{#if tahap.target_kualitas}
										<p class="text-[11px] text-slate-500">{tahap.target_kualitas}</p>
									{/if}
								</td>
								<td class="td text-center text-slate-600">{tahap.bobot ?? '-'}</td>
								<td class="td text-xs text-slate-600">
									{tanggal(tahap.tanggal_mulai)} → {tanggal(tahap.tanggal_selesai)}
								</td>
							</tr>
						{:else}
							<tr><td class="td text-center text-slate-500" colspan="6">Tidak ada rincian kegiatan.</td></tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>

		<div class="card p-5">
			<h3 class="text-sm font-semibold text-slate-800">Lampiran</h3>
			<p class="mt-1 text-xs text-slate-500">Berkas pendukung yang diunggah pada rencana kerja ini.</p>
			<ul class="mt-3 space-y-2">
				{#each detail.lampiran as berkas}
					<li class="flex flex-wrap items-center justify-between gap-2 rounded-lg bg-slate-50 px-3 py-2">
						<span class="text-sm text-slate-700">
							{berkas.file_name}
							<span class="text-xs text-slate-500">· {ukuran(berkas.size_bytes)} · {berkas.mime}</span>
						</span>
						{#if urlLampiran[berkas.id]}
							<div class="flex gap-1">
								<a
									class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
									href={urlLampiran[berkas.id]}
									target="_blank"
									rel="noreferrer"
								>
									<Icon name="download" size={14} /> Lihat
								</a>
								<button
									class="btn btn-ghost border border-rose-200 px-2 py-1 text-xs text-rose-600"
									onclick={() => hapusLampiran(berkas)}
								>
									<Icon name="x" size={14} /> Hapus
								</button>
							</div>
						{:else}
							<span class="text-xs text-slate-400">Memuat…</span>
						{/if}
					</li>
				{:else}
					<li class="text-sm text-slate-500">Belum ada lampiran.</li>
				{/each}
			</ul>

			<label class="btn btn-ghost mt-3 w-full border border-dashed border-slate-300 text-xs">
				<Icon name="upload" size={14} />
				{mengunggah ? 'Mengunggah…' : 'Unggah lampiran (PDF/JPG/PNG, maks 10 MB)'}
				<input type="file" class="hidden" accept=".pdf,.jpg,.jpeg,.png" onchange={unggah} />
			</label>
		</div>

		{#if adalahAtasan() && detail.status === 'diajukan'}
			<div class="card p-5">
				<h3 class="text-sm font-semibold text-slate-800">Keputusan Atasan</h3>
				<p class="mt-1 text-xs text-slate-500">
					Setujui, minta revisi, atau tolak rencana ini. Catatan wajib untuk revisi.
				</p>
				<label class="label mt-3" for="catatan">Catatan</label>
				<textarea id="catatan" class="input h-24 resize-none" bind:value={catatan}></textarea>
				<div class="mt-3 flex flex-wrap gap-2">
					<button class="btn btn-primary" disabled={memproses} onclick={() => putuskan('disetujui')}>
						<Icon name="check" size={16} /> Setujui
					</button>
					<button class="btn btn-gold" disabled={memproses} onclick={() => putuskan('perlu_revisi')}>
						Minta Revisi
					</button>
					<button
						class="btn btn-ghost border border-rose-200 text-rose-600"
						disabled={memproses}
						onclick={() => putuskan('ditolak')}
					>
						Tolak
					</button>
				</div>
			</div>
		{/if}
	{/if}
</div>
