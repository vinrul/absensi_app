<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import { session } from '#lib/stores/session.svelte';
	import type { Kategori, ListResponse, Penilaian, Periode, Satker } from '#lib/types';
	import { LABEL_KATEGORI } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';

	interface Komponen {
		indikator: string;
		skor: number;
		komponen: Record<string, unknown>;
	}

	let periodeList = $state<Periode[]>([]);
	let satkerList = $state<Satker[]>([]);
	let periodeId = $state('');
	let satkerId = $state('');
	let kategori = $state<Kategori | ''>('');
	let data = $state<Penilaian[]>([]);
	let meta = $state({ page: 1, per_page: 10, total: 0 });
	let memuat = $state(true);
	let error = $state<string | null>(null);

	let dipilih = $state<{ data: Penilaian; komponen: Komponen[] } | null>(null);

	const totalHalaman = $derived(Math.max(1, Math.ceil(meta.total / meta.per_page)));

	function nada(value: Kategori) {
		if (value === 'sangat_baik') return 'green';
		if (value === 'baik') return 'blue';
		if (value === 'cukup') return 'amber';
		return 'red';
	}

	async function muat(page = 1) {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams();
			if (periodeId) params.set('periode_id', periodeId);
			if (satkerId) params.set('satker_id', satkerId);
			if (kategori) params.set('kategori', kategori);
			params.set('page', String(page));
			params.set('per_page', String(meta.per_page));

			const hasil = await api<ListResponse<Penilaian>>(`/penilaian?${params}`);
			data = hasil.data;
			meta = hasil.meta;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat penilaian';
		} finally {
			memuat = false;
		}
	}

	async function bukaDetail(row: Penilaian) {
		try {
			const detail = await api<{ data: Penilaian; komponen: Komponen[] }>(
				`/penilaian/personel/${row.personel_id}?periode_id=${periodeId}`
			);
			dipilih = detail;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat rincian';
		}
	}

	onMount(async () => {
		const [p, s] = await Promise.all([
			api<Periode[]>('/periode'),
			api<Satker[]>('/satker')
		]);
		periodeList = p;
		satkerList = s;
		periodeId = p[0]?.id ?? '';
		await muat(1);
	});
</script>

<svelte:head><title>Penilaian Kinerja — Polda Bali</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-end gap-3 p-4 no-print">
		<div class="min-w-40 flex-1">
			<label class="label" for="periode">Periode</label>
			<select id="periode" class="input" bind:value={periodeId} onchange={() => muat(1)}>
				{#each periodeList as p}<option value={p.id}>{p.nama}</option>{/each}
			</select>
		</div>
		<div class="min-w-40 flex-1">
			<label class="label" for="satker">Satker</label>
			<select id="satker" class="input" bind:value={satkerId} onchange={() => muat(1)}>
				<option value="">Semua Satker</option>
				{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<div class="min-w-36 flex-1">
			<label class="label" for="kategori">Kategori</label>
			<select id="kategori" class="input" bind:value={kategori} onchange={() => muat(1)}>
				<option value="">Semua</option>
				{#each Object.entries(LABEL_KATEGORI) as [nilai, label]}<option value={nilai}>{label}</option>{/each}
			</select>
		</div>
		<button class="btn btn-ghost border border-slate-300" onclick={() => muat(meta.page)}>
			<Icon name="refresh" size={16} /> Muat ulang
		</button>
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	<div class="card overflow-hidden">
		<div class="flex items-center justify-between border-b border-slate-200 px-4 py-3">
			<h2 class="text-sm font-semibold text-slate-700">
				Penilaian {session.user?.satker_nama ?? ''}
			</h2>
			<span class="text-xs text-slate-500">{meta.total} personel</span>
		</div>

		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">Personel</th>
						<th class="th">Jabatan / Unit</th>
						<th class="th text-center">Absensi</th>
						<th class="th text-center">Rencana</th>
						<th class="th text-center">Hasil</th>
						<th class="th text-center">Output</th>
						<th class="th text-center">Nilai</th>
						<th class="th">Kategori</th>
						<th class="th no-print"></th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each data as row}
						<tr class="hover:bg-slate-50/70">
							<td class="td">
								<p class="font-medium text-slate-800">{row.personel_nama}</p>
								<p class="text-[11px] text-slate-500">{row.pangkat} · {row.personel_nrp}</p>
							</td>
							<td class="td text-slate-600">{row.jabatan}</td>
							<td class="td text-center">{row.skor_absensi.toFixed(1)}</td>
							<td class="td text-center">{row.skor_rencana_kerja.toFixed(1)}</td>
							<td class="td text-center">{row.skor_hasil_kerja.toFixed(1)}</td>
							<td class="td text-center">{row.skor_output.toFixed(1)}</td>
							<td class="td text-center font-semibold text-slate-800">{row.nilai_akhir.toFixed(2)}</td>
							<td class="td"><Badge tone={nada(row.kategori)}>{LABEL_KATEGORI[row.kategori]}</Badge></td>
							<td class="td no-print">
								<button class="btn btn-ghost px-2 py-1 text-xs" onclick={() => bukaDetail(row)}>
									<Icon name="eye" size={14} /> Rincian
								</button>
							</td>
						</tr>
					{:else}
						<tr><td class="td text-center text-slate-500" colspan="9">
							{memuat ? 'Memuat…' : 'Belum ada penilaian. Jalankan perhitungan dari halaman Dashboard.'}
						</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="flex items-center justify-between border-t border-slate-200 px-4 py-3 no-print">
			<p class="text-xs text-slate-500">Halaman {meta.page} dari {totalHalaman}</p>
			<div class="flex gap-2">
				<button class="btn btn-ghost border border-slate-300 px-2 py-1" disabled={meta.page <= 1} onclick={() => muat(meta.page - 1)}>
					<Icon name="left" size={16} />
				</button>
				<button class="btn btn-ghost border border-slate-300 px-2 py-1" disabled={meta.page >= totalHalaman} onclick={() => muat(meta.page + 1)}>
					<Icon name="right" size={16} />
				</button>
			</div>
		</div>
	</div>
</div>

<Modal open={!!dipilih} title="Rincian Perhitungan Nilai" lebar="max-w-xl" onclose={() => (dipilih = null)}>
	{#if dipilih}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="font-semibold text-slate-800">{dipilih.data.personel_nama}</p>
				<p class="text-xs text-slate-500">{dipilih.data.pangkat} · {dipilih.data.jabatan}</p>
				<p class="mt-2 text-sm text-slate-700">
					Nilai akhir <strong class="text-lg">{dipilih.data.nilai_akhir.toFixed(2)}</strong> ·
					<Badge tone={nada(dipilih.data.kategori)}>{LABEL_KATEGORI[dipilih.data.kategori]}</Badge>
				</p>
			</div>

			<table class="w-full text-sm">
				<thead class="bg-slate-50">
					<tr><th class="th">Indikator</th><th class="th text-center">Skor</th><th class="th">Komponen</th></tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each dipilih.komponen as k}
						<tr>
							<td class="td capitalize">{k.indikator.replace('_', ' ')}</td>
							<td class="td text-center font-semibold">{k.skor.toFixed(2)}</td>
							<td class="td text-[11px] text-slate-500">
								{Object.entries(k.komponen)
									.slice(0, 4)
									.map(([kunci, nilai]) => `${kunci}: ${typeof nilai === 'number' ? nilai.toFixed(2) : nilai}`)
									.join(' · ')}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>

			{#if dipilih.data.catatan_pimpinan}
				<div class="rounded-lg bg-amber-50 p-3 text-xs text-amber-800 ring-1 ring-amber-200">
					<p class="font-semibold uppercase">Catatan Pimpinan</p>
					<p class="mt-1">{dipilih.data.catatan_pimpinan}</p>
				</div>
			{/if}
		</div>
	{/if}
</Modal>
