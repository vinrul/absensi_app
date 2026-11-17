<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import type { ListResponse, Rencana } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	let data = $state<Rencana[]>([]);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let page = $state(1);
	let total = $state(0);
	const perPage = 10;

	let dipilih = $state<Rencana | null>(null);
	let catatan = $state('');
	let aksi = $state<'disetujui' | 'perlu_revisi' | 'ditolak'>('disetujui');
	let memproses = $state(false);

	async function muat(p = page) {
		memuat = true;
		error = null;
		try {
			const hasil = await api<ListResponse<Rencana>>(
				`/rencana-kerja/persetujuan?page=${p}&per_page=${perPage}`
			);
			data = hasil.data;
			total = hasil.meta.total;
			page = hasil.meta.page;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat daftar persetujuan';
		} finally {
			memuat = false;
		}
	}

	async function putuskan() {
		if (!dipilih) return;
		memproses = true;
		error = null;
		try {
			await api(`/rencana-kerja/${dipilih.id}/keputusan`, {
				method: 'POST',
				body: { status: aksi, catatan: catatan || null }
			});
			dipilih = null;
			catatan = '';
			await muat(1);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan keputusan';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Persetujuan Rencana Kerja — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card flex items-center justify-between p-4">
		<div>
			<h2 class="text-sm font-semibold text-slate-800">Menunggu Persetujuan</h2>
			<p class="text-xs text-slate-500">Rencana kerja yang diajukan bawahan langsung Anda.</p>
		</div>
		<button class="btn btn-ghost border border-slate-300" onclick={() => muat(page)}>
			<Icon name="refresh" size={16} /> Muat ulang
		</button>
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	<div class="space-y-2">
		{#each data as row}
			<div class="card flex flex-wrap items-center justify-between gap-3 p-4">
				<div class="min-w-0">
					<p class="font-medium text-slate-800">{row.kegiatan}</p>
					<p class="text-xs text-slate-500">
						{row.personel_nama} · {row.personel_nrp} · {row.unit ?? '-'}
					</p>
					<p class="mt-1 text-xs text-slate-500">
						{row.program} · {row.tanggal_mulai} → {row.tanggal_selesai} · target {row.target_kuantitas ?? '-'} {row.satuan ?? ''}
					</p>
				</div>
				<div class="flex items-center gap-2">
					<Badge tone="blue">Diajukan</Badge>
					<a href="/rencana-kerja/{row.id}" class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs">
						<Icon name="eye" size={14} /> Detail
					</a>
					<button
						class="btn btn-primary px-2 py-1 text-xs"
						onclick={() => {
							dipilih = row;
							aksi = 'disetujui';
							catatan = '';
						}}
					>
						Putuskan
					</button>
				</div>
			</div>
		{:else}
			<div class="card p-8 text-center text-sm text-slate-500">
				{memuat ? 'Memuat…' : 'Tidak ada rencana kerja yang menunggu persetujuan.'}
			</div>
		{/each}
	</div>
	{#if total > 0}
		<div class="card overflow-hidden">
			<Pagination page={page} perPage={perPage} total={total} label="rencana" onchange={(p) => muat(p)} />
		</div>
	{/if}
</div>

<Modal open={!!dipilih} title="Keputusan Rencana Kerja" onclose={() => (dipilih = null)}>
	{#if dipilih}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="font-medium text-slate-800">{dipilih.kegiatan}</p>
				<p class="text-xs text-slate-500">{dipilih.personel_nama} · {dipilih.program}</p>
			</div>

			<div>
				<label class="label" for="aksi">Keputusan</label>
				<select id="aksi" class="input" bind:value={aksi}>
					<option value="disetujui">Disetujui</option>
					<option value="perlu_revisi">Perlu Revisi</option>
					<option value="ditolak">Ditolak</option>
				</select>
			</div>

			<div>
				<label class="label" for="catatan">Catatan</label>
				<textarea id="catatan" class="input h-24 resize-none" bind:value={catatan}></textarea>
				{#if aksi === 'perlu_revisi'}
					<p class="mt-1 text-[11px] text-amber-600">Catatan wajib diisi untuk permintaan revisi.</p>
				{/if}
			</div>

			<div class="flex justify-end gap-2">
				<button class="btn btn-ghost" onclick={() => (dipilih = null)}>Batal</button>
				<button class="btn btn-primary" disabled={memproses} onclick={putuskan}>Simpan</button>
			</div>
		</div>
	{/if}
</Modal>
