<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import {
		LABEL_JENIS_RENCANA,
		LABEL_STATUS_RENCANA,
		tanggalIndo,
		statusRencanaWarna
	} from '#lib/format';
	import { bisaMenyetujui, session } from '#lib/stores/session.svelte';
	import type { ListResponse, Rencana, StatusRencana } from '#lib/types';

	let daftar = $state<Rencana[]>([]);
	let jumlahPersetujuan = $state(0);
	let memuat = $state(true);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);
	let filter = $state<'semua' | StatusRencana>('semua');

	const bisaSetujui = $derived(bisaMenyetujui());
	const tampil = $derived(
		filter === 'semua' ? daftar : daftar.filter((item) => item.status === filter)
	);

	async function muat() {
		const user = session.user;
		if (!user) return;
		memuat = true;
		galat = null;
		try {
			const hasil = await api<ListResponse<Rencana>>(
				`/rencana-kerja?per_page=50&personel_id=${user.id}`
			);
			daftar = hasil.data;
			if (bisaSetujui) {
				try {
					const persetujuan = await api<ListResponse<Rencana>>(
						'/rencana-kerja/persetujuan?per_page=50'
					);
					jumlahPersetujuan = persetujuan.meta.total;
				} catch {
					jumlahPersetujuan = 0;
				}
			}
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Rencana kerja tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	async function ajukan(item: Rencana) {
		if (memproses) return;
		memproses = true;
		pesan = null;
		galat = null;
		try {
			await api(`/rencana-kerja/${item.id}/ajukan`, { method: 'POST' });
			pesan = 'Rencana kerja diajukan ke atasan.';
			await muat();
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Gagal mengajukan rencana.';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Rencana Kerja — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if bisaSetujui}
		<a href="/rencana/persetujuan" class="kartu flex items-center gap-3 border-sky-200 bg-sky-50/70 p-4">
			<span class="flex size-10 items-center justify-center rounded-xl bg-white text-sky-700 ring-1 ring-sky-100">
				<Icon name="list" size={20} />
			</span>
			<span class="min-w-0 flex-1">
				<span class="block text-sm font-bold text-sky-900">Persetujuan Rencana</span>
				<span class="block text-[11px] text-sky-700">
					{jumlahPersetujuan
						? `${jumlahPersetujuan} rencana menunggu keputusan Anda`
						: 'Tidak ada rencana yang menunggu'}
				</span>
			</span>
			<Icon name="right" size={18} class="text-sky-600" />
		</a>
	{/if}

	<div class="flex items-center justify-between gap-2">
		<div>
			<h1 class="text-lg font-bold text-ink-900">Rencana Kerja</h1>
			<p class="text-[11px] text-ink-500">{daftar.length} dokumen</p>
		</div>
		<a href="/rencana/form" class="btn btn-utama px-3 py-2 text-xs">
			<Icon name="plus" size={16} /> Baru
		</a>
	</div>

	<div class="-mx-4 overflow-x-auto px-4">
		<div class="flex w-max gap-1.5">
			{#each [
				{ nilai: 'semua' as const, label: 'Semua' },
				{ nilai: 'draft' as const, label: 'Draft' },
				{ nilai: 'diajukan' as const, label: 'Diajukan' },
				{ nilai: 'disetujui' as const, label: 'Disetujui' },
				{ nilai: 'perlu_revisi' as const, label: 'Revisi' },
				{ nilai: 'ditolak' as const, label: 'Ditolak' }
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
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat rencana kerja…</div>
	{:else if tampil.length === 0}
		<div class="kartu p-6 text-center">
			<p class="text-sm font-semibold text-ink-600">Belum ada rencana kerja.</p>
			<p class="mt-1 text-xs text-ink-400">Ajukan rencana sebelum periode berjalan agar dinilai tepat waktu.</p>
		</div>
	{:else}
		{#each tampil as item (item.id)}
			<article class="kartu p-4">
				<div class="flex items-start justify-between gap-3">
					<div class="min-w-0">
						<p class="text-sm font-bold text-ink-900">{item.kegiatan}</p>
						<p class="text-[11px] text-ink-500">{item.program}</p>
					</div>
					<span class="chip shrink-0 {statusRencanaWarna(item.status)}">
						{LABEL_STATUS_RENCANA[item.status]}
					</span>
				</div>

				<dl class="mt-3">
					<div class="baris-info">
						<dt>Jenis</dt>
						<dd>{LABEL_JENIS_RENCANA[item.jenis] ?? item.jenis}</dd>
					</div>
					<div class="baris-info">
						<dt>Periode</dt>
						<dd>{tanggalIndo(item.tanggal_mulai)} – {tanggalIndo(item.tanggal_selesai)}</dd>
					</div>
					<div class="baris-info">
						<dt>Target</dt>
						<dd>{item.target_kuantitas ?? '-'} {item.satuan ?? ''}</dd>
					</div>
					{#if item.dasar_nomor}
						<div class="baris-info">
							<dt>Dasar tugas</dt>
							<dd>{item.dasar_nomor}</dd>
						</div>
					{/if}
					{#if item.jumlah_revisi > 0}
						<div class="baris-info">
							<dt>Revisi</dt>
							<dd class="!text-amber-700">{item.jumlah_revisi}×</dd>
						</div>
					{/if}
				</dl>

				{#if item.catatan_atasan}
					<Pesan jenis="info" pesan={`Catatan atasan: ${item.catatan_atasan}`} class="mt-2" />
				{/if}

				<div class="mt-3 flex flex-wrap gap-2">
					<a href={`/rencana/${item.id}`} class="btn btn-kedua flex-1 py-2 text-xs">
						<Icon name="file" size={15} /> Detail
					</a>
					{#if item.status === 'draft' || item.status === 'perlu_revisi'}
						<a href={`/rencana/form?id=${item.id}`} class="btn btn-kedua flex-1 py-2 text-xs">
							<Icon name="edit" size={15} />
							{item.status === 'perlu_revisi' ? 'Perbaiki' : 'Ubah'}
						</a>
						<button
							class="btn btn-utama w-full py-2 text-xs"
							disabled={memproses}
							onclick={() => ajukan(item)}
						>
							<Icon name="send" size={15} /> Ajukan ke Atasan
						</button>
					{/if}
				</div>
			</article>
		{/each}
	{/if}
</div>
