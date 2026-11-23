<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError, getAccessToken } from '#lib/api/client';
	import type { Kategori, Penilaian, Periode, Satker } from '#lib/types';
	import { LABEL_KATEGORI } from '#lib/types';
	import Icon from '#lib/components/Icon.svelte';

	interface Laporan {
		periode: { id: string; nama: string; tanggal_mulai: string; tanggal_selesai: string };
		satker: string;
		data: Penilaian[];
		dicetak_oleh: string;
		dicetak_pada: string;
	}

	let periodeList = $state<Periode[]>([]);
	let satkerList = $state<Satker[]>([]);
	let periodeId = $state('');
	let satkerId = $state('');
	let laporan = $state<Laporan | null>(null);
	let memuat = $state(true);
	let error = $state<string | null>(null);

	async function muat() {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams();
			if (periodeId) params.set('periode_id', periodeId);
			if (satkerId) params.set('satker_id', satkerId);
			laporan = await api<Laporan>(`/laporan/penilaian?${params}`);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat laporan';
		} finally {
			memuat = false;
		}
	}

	async function unduhCsv() {
		const params = new URLSearchParams();
		if (periodeId) params.set('periode_id', periodeId);
		if (satkerId) params.set('satker_id', satkerId);
		params.set('format', 'csv');

		const res = await fetch(`/api/v1/laporan/penilaian?${params}`, {
			headers: { authorization: `Bearer ${getAccessToken() ?? ''}` }
		});
		const blob = await res.blob();
		const url = URL.createObjectURL(blob);
		const tautan = document.createElement('a');
		tautan.href = url;
		tautan.download = `laporan-penilaian-${periodeId.slice(0, 8)}.csv`;
		tautan.click();
		URL.revokeObjectURL(url);
	}

	function rata(kolom: (row: Penilaian) => number) {
		if (!laporan?.data.length) return 0;
		return laporan.data.reduce((total, row) => total + kolom(row), 0) / laporan.data.length;
	}

	onMount(async () => {
		const [p, s] = await Promise.all([api<Periode[]>('/periode'), api<Satker[]>('/satker')]);
		periodeList = p;
		satkerList = s;
		periodeId = p[0]?.id ?? '';
		await muat();
	});
</script>

<svelte:head><title>Laporan Penilaian — Polda Bali</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-end justify-between gap-3 p-4 no-print">
		<div class="flex flex-wrap items-end gap-3">
			<div class="min-w-40">
				<label class="label" for="periode">Periode</label>
				<select id="periode" class="input" bind:value={periodeId} onchange={muat}>
					{#each periodeList as p}<option value={p.id}>{p.nama}</option>{/each}
				</select>
			</div>
			<div class="min-w-44">
				<label class="label" for="satker">Satker</label>
				<select id="satker" class="input" bind:value={satkerId} onchange={muat}>
					<option value="">Semua Satker</option>
					{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
				</select>
			</div>
		</div>
		<div class="flex gap-2">
			<button class="btn btn-ghost border border-slate-300" onclick={unduhCsv}>
				<Icon name="download" size={16} /> CSV
			</button>
			<button class="btn btn-gold" onclick={() => window.print()}>
				<Icon name="laporan" size={16} /> Cetak / PDF
			</button>
		</div>
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	{#if laporan}
		<div class="card cetak-area p-8 print:p-0">
			<!-- Kop -->
			<div class="flex items-center gap-4 border-b-2 border-slate-800 pb-4">
				<img
					src="/logo-polda-bali.png"
					alt="Logo SIAP Bali"
					class="h-16 w-auto shrink-0"
				/>
				<div class="text-center">
					<p class="text-sm font-semibold tracking-wide text-slate-700 uppercase">
						Kepolisian Negara Republik Indonesia
					</p>
					<p class="text-sm font-semibold tracking-wide text-slate-700 uppercase">
						Daerah Bali
					</p>
					<h1 class="mt-1 text-lg font-bold text-slate-900">
						Laporan Penilaian Kinerja Personel
					</h1>
					<p class="text-xs text-slate-600">{laporan.satker} · Periode {laporan.periode.nama}</p>
				</div>
			</div>

			<div class="mt-4 grid grid-cols-2 gap-4 text-xs text-slate-600">
				<p>Jumlah personel dinilai: <strong>{laporan.data.length}</strong></p>
				<p class="text-right">
					Dicetak: {new Date(laporan.dicetak_pada).toLocaleString('id-ID')} · oleh {laporan.dicetak_oleh}
				</p>
			</div>

			<table class="mt-4 w-full">
				<thead>
					<tr class="bg-slate-100">
						<th class="th">No</th>
						<th class="th">NRP</th>
						<th class="th">Nama</th>
						<th class="th">Pangkat</th>
						<th class="th">Jabatan / Unit</th>
						<th class="th text-center">Absensi</th>
						<th class="th text-center">Rencana</th>
						<th class="th text-center">Hasil</th>
						<th class="th text-center">Output</th>
						<th class="th text-center">Nilai</th>
						<th class="th">Kategori</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-200">
					{#each laporan.data as row, i}
						<tr>
							<td class="td">{i + 1}</td>
							<td class="td">{row.personel_nrp}</td>
							<td class="td">{row.personel_nama}</td>
							<td class="td">{row.pangkat}</td>
							<td class="td">{row.jabatan}</td>
							<td class="td text-center">{row.skor_absensi.toFixed(1)}</td>
							<td class="td text-center">{row.skor_rencana_kerja.toFixed(1)}</td>
							<td class="td text-center">{row.skor_hasil_kerja.toFixed(1)}</td>
							<td class="td text-center">{row.skor_output.toFixed(1)}</td>
							<td class="td text-center font-semibold">{row.nilai_akhir.toFixed(2)}</td>
							<td class="td">{LABEL_KATEGORI[row.kategori as Kategori]}</td>
						</tr>
					{:else}
						<tr><td class="td text-center text-slate-500" colspan="11">{memuat ? 'Memuat…' : 'Belum ada data.'}</td></tr>
					{/each}
				</tbody>
				{#if laporan.data.length}
					<tfoot>
						<tr class="border-t-2 border-slate-800 bg-slate-50 font-semibold">
							<td class="td" colspan="5">Rata-rata</td>
							<td class="td text-center">{rata((r) => r.skor_absensi).toFixed(1)}</td>
							<td class="td text-center">{rata((r) => r.skor_rencana_kerja).toFixed(1)}</td>
							<td class="td text-center">{rata((r) => r.skor_hasil_kerja).toFixed(1)}</td>
							<td class="td text-center">{rata((r) => r.skor_output).toFixed(1)}</td>
							<td class="td text-center">{rata((r) => r.nilai_akhir).toFixed(2)}</td>
							<td class="td"></td>
						</tr>
					</tfoot>
				{/if}
			</table>

			<div class="mt-10 grid grid-cols-2 gap-8 text-center text-xs text-slate-700">
				<div>
					<p>Mengetahui,</p>
					<p class="font-semibold">Kepala Satuan Kerja</p>
					<div class="h-20"></div>
					<p class="border-t border-slate-400 pt-1">……………………………………</p>
				</div>
				<div>
					<p>Denpasar, {new Date().toLocaleDateString('id-ID')}</p>
					<p class="font-semibold">Yang membuat laporan</p>
					<div class="h-20"></div>
					<p class="border-t border-slate-400 pt-1">{laporan.dicetak_oleh}</p>
				</div>
			</div>
		</div>
	{/if}
</div>
