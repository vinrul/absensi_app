<script lang="ts">
	import { onMount } from 'svelte';
	import { api, apiBlob, ApiError } from '#lib/api/client';
	import { adalahAtasan } from '#lib/stores/session.svelte';
	import type { Hasil, ListResponse, Periode, Personel, Rencana, Satker, StatusHasil } from '#lib/types';
	import { LABEL_STATUS_HASIL } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	let data = $state<Hasil[]>([]);
	let meta = $state({ page: 1, per_page: 10, total: 0 });
	let memuat = $state(true);
	let mengekspor = $state(false);
	let error = $state<string | null>(null);

	let periodeList = $state<Periode[]>([]);
	let satkerList = $state<Satker[]>([]);
	let personelList = $state<Personel[]>([]);
	let periodeId = $state('');
	let satkerId = $state('');
	let personelId = $state('');
	let status = $state<StatusHasil | ''>('');

	const personelTersedia = $derived(
		satkerId ? personelList.filter((p) => p.satker_id === satkerId) : personelList
	);

	let rencanaDisetujui = $state<Rencana[]>([]);
	let formTerbuka = $state(false);
	let menyimpan = $state(false);
	let form = $state({
		rencana_id: '',
		uraian_pelaksanaan: '',
		progres_persen: '100',
		jumlah_realisasi: '',
		kendala: '',
		tanggal_realisasi: new Date().toISOString().slice(0, 10)
	});

	let dipilih = $state<Hasil | null>(null);
	let skorKualitas = $state('85');
	let catatan = $state('');
	let setuju = $state(true);

	async function muat(page = 1) {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams();
			if (periodeId) params.set('periode_id', periodeId);
			if (status) params.set('status', status);
			if (satkerId) params.set('satker_id', satkerId);
			if (personelId) params.set('personel_id', personelId);
			params.set('page', String(page));
			params.set('per_page', String(meta.per_page));
			const hasil = await api<ListResponse<Hasil>>(`/hasil-kerja?${params}`);
			data = hasil.data;
			meta = hasil.meta;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat hasil kerja';
		} finally {
			memuat = false;
		}
	}

	function ubahSatker() {
		const masihCocok = personelList.some(
			(p) => p.id === personelId && (!satkerId || p.satker_id === satkerId)
		);
		if (!masihCocok) personelId = '';
		muat(1);
	}

	async function eksporPdf() {
		mengekspor = true;
		error = null;
		try {
			const params = new URLSearchParams();
			if (periodeId) params.set('periode_id', periodeId);
			if (status) params.set('status', status);
			if (satkerId) params.set('satker_id', satkerId);
			if (personelId) params.set('personel_id', personelId);

			const blob = await apiBlob(`/hasil-kerja/ekspor?${params}`);
			const url = URL.createObjectURL(blob);
			const tautan = document.createElement('a');
			tautan.href = url;
			tautan.download = `hasil-kerja-${new Date().toISOString().slice(0, 10)}.pdf`;
			tautan.click();
			URL.revokeObjectURL(url);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal membuat PDF';
		} finally {
			mengekspor = false;
		}
	}

	async function bukaForm() {
		formTerbuka = true;
		try {
			const hasil = await api<ListResponse<Rencana>>('/rencana-kerja?status=disetujui&per_page=100');
			rencanaDisetujui = hasil.data;
			form.rencana_id = hasil.data[0]?.id ?? '';
		} catch {
			rencanaDisetujui = [];
		}
	}

	async function simpan() {
		menyimpan = true;
		error = null;
		try {
			await api('/hasil-kerja', {
				method: 'POST',
				body: {
					rencana_id: form.rencana_id,
					uraian_pelaksanaan: form.uraian_pelaksanaan,
					progres_persen: Number(form.progres_persen),
					jumlah_realisasi: form.jumlah_realisasi ? Number(form.jumlah_realisasi) : null,
					kendala: form.kendala || null,
					tanggal_realisasi: form.tanggal_realisasi
				}
			});
			formTerbuka = false;
			form.uraian_pelaksanaan = '';
			await muat(1);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan hasil kerja';
		} finally {
			menyimpan = false;
		}
	}

	async function verifikasi() {
		if (!dipilih) return;
		try {
			await api(`/hasil-kerja/${dipilih.id}/verifikasi`, {
				method: 'POST',
				body: {
					setuju,
					skor_kualitas: setuju ? Number(skorKualitas) : undefined,
					catatan: catatan || null
				}
			});
			dipilih = null;
			catatan = '';
			await muat(meta.page);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memverifikasi';
		}
	}

	function nada(status: Hasil['status']) {
		if (status === 'terverifikasi') return 'green';
		if (status === 'perlu_revisi') return 'amber';
		return 'blue';
	}

	onMount(async () => {
		const [p, s, orang] = await Promise.all([
			api<Periode[]>('/periode'),
			api<Satker[]>('/satker'),
			api<ListResponse<Personel>>('/personel?per_page=200')
		]);
		periodeList = p;
		satkerList = s;
		personelList = orang.data;
		periodeId = p[0]?.id ?? '';
		await muat(1);
	});
</script>

<svelte:head><title>Hasil Kerja — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card space-y-3 p-4">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="text-sm font-semibold text-slate-800">Hasil Kerja</h2>
				<p class="text-xs text-slate-500">Realisasi capaian dari rencana kerja yang disetujui.</p>
			</div>
			<div class="flex flex-wrap gap-2">
				<button
					class="btn btn-ghost border border-slate-300"
					disabled={mengekspor}
					onclick={eksporPdf}
				>
					<Icon name="download" size={16} />
					{mengekspor ? 'Menyiapkan PDF…' : 'Export PDF'}
				</button>
				<button class="btn btn-primary" onclick={bukaForm}>
					<Icon name="plus" size={16} /> Tambah Hasil
				</button>
			</div>
		</div>

		<div class="flex flex-wrap items-end gap-3">
			<div class="min-w-44">
				<label class="label" for="periode">Periode</label>
				<select id="periode" class="input" bind:value={periodeId} onchange={() => muat(1)}>
					{#each periodeList as p}<option value={p.id}>{p.nama}</option>{/each}
				</select>
			</div>
			<div class="min-w-40">
				<label class="label" for="status">Status</label>
				<select id="status" class="input" bind:value={status} onchange={() => muat(1)}>
					<option value="">Semua status</option>
					{#each Object.entries(LABEL_STATUS_HASIL) as [nilai, label]}
						<option value={nilai}>{label}</option>
					{/each}
				</select>
			</div>
			<div class="min-w-48">
				<label class="label" for="satker">Satker</label>
				<select id="satker" class="input" bind:value={satkerId} onchange={ubahSatker}>
					<option value="">Semua Satker</option>
					{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
				</select>
			</div>
			<div class="min-w-56">
				<label class="label" for="personel">Personel</label>
				<select id="personel" class="input" bind:value={personelId} onchange={() => muat(1)}>
					<option value="">Semua Personel</option>
					{#each personelTersedia as orang}
						<option value={orang.id}>{orang.nama} ({orang.nrp})</option>
					{/each}
				</select>
			</div>
		</div>
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	<div class="card overflow-hidden">
		<table class="w-full">
			<thead class="bg-slate-50">
				<tr>
					<th class="th">Rencana / Kegiatan</th>
					<th class="th">Personel</th>
					<th class="th text-center">Progres</th>
					<th class="th text-center">Realisasi</th>
					<th class="th text-center">Nilai</th>
					<th class="th">Status</th>
					<th class="th no-print">Aksi</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-slate-100">
				{#each data as row}
					<tr class="hover:bg-slate-50/70">
						<td class="td">
							<a class="font-medium text-navy-800 hover:underline" href="/hasil-kerja/{row.id}">
								{row.rencana_kegiatan ?? '-'}
							</a>
							<p class="text-[11px] text-slate-500">{row.tanggal_realisasi}</p>
						</td>
						<td class="td text-slate-600">{row.personel_nama ?? '-'}</td>
						<td class="td text-center text-slate-700">{row.progres_persen.toFixed(0)}%</td>
						<td class="td text-center text-slate-600">{row.jumlah_realisasi ?? '-'}</td>
						<td class="td text-center font-semibold text-slate-800">
							{row.skor_total != null ? row.skor_total.toFixed(1) : '-'}
						</td>
						<td class="td"><Badge tone={nada(row.status)}>{LABEL_STATUS_HASIL[row.status]}</Badge></td>
						<td class="td no-print">
							<div class="flex flex-wrap gap-1">
								<a
									class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
									href="/hasil-kerja/{row.id}"
								>
									Detail
								</a>
								{#if adalahAtasan() && row.status === 'diajukan'}
									<button
										class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
										onclick={() => {
											dipilih = row;
											setuju = true;
											catatan = '';
											skorKualitas = '85';
										}}
									>
										Verifikasi
									</button>
								{/if}
							</div>
						</td>
					</tr>
				{:else}
					<tr><td class="td text-center text-slate-500" colspan="7">{memuat ? 'Memuat…' : 'Belum ada hasil kerja.'}</td></tr>
				{/each}
			</tbody>
		</table>
		<Pagination page={meta.page} perPage={meta.per_page} total={meta.total} label="hasil kerja" onchange={(p) => muat(p)} />
	</div>
</div>

<Modal open={formTerbuka} title="Tambah Hasil Kerja" lebar="max-w-xl" onclose={() => (formTerbuka = false)}>
	<div class="space-y-3">
		<div>
			<label class="label" for="rencana">Rencana Kerja Disetujui</label>
			<select id="rencana" class="input" bind:value={form.rencana_id}>
				{#each rencanaDisetujui as r}
					<option value={r.id}>{r.kegiatan} ({r.tanggal_mulai} → {r.tanggal_selesai})</option>
				{:else}
					<option value="">Belum ada rencana disetujui</option>
				{/each}
			</select>
		</div>
		<div>
			<label class="label" for="uraian">Uraian Pelaksanaan</label>
			<textarea id="uraian" class="input h-24 resize-none" bind:value={form.uraian_pelaksanaan}></textarea>
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label class="label" for="progres">Progres (%)</label>
				<input id="progres" class="input" bind:value={form.progres_persen} />
			</div>
			<div>
				<label class="label" for="realisasi">Jumlah Realisasi</label>
				<input id="realisasi" class="input" bind:value={form.jumlah_realisasi} />
			</div>
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label class="label" for="tanggal">Tanggal Realisasi</label>
				<input id="tanggal" type="date" class="input" bind:value={form.tanggal_realisasi} />
			</div>
			<div>
				<label class="label" for="kendala">Kendala (opsional)</label>
				<input id="kendala" class="input" bind:value={form.kendala} />
			</div>
		</div>
		<div class="flex justify-end gap-2 pt-2">
			<button class="btn btn-ghost" onclick={() => (formTerbuka = false)}>Batal</button>
			<button class="btn btn-primary" disabled={menyimpan || !form.rencana_id} onclick={simpan}>Simpan</button>
		</div>
	</div>
</Modal>

<Modal open={!!dipilih} title="Verifikasi Hasil Kerja" onclose={() => (dipilih = null)}>
	{#if dipilih}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="font-medium text-slate-800">{dipilih.rencana_kegiatan}</p>
				<p class="text-xs text-slate-500">{dipilih.personel_nama} · realisasi {dipilih.jumlah_realisasi ?? '-'}</p>
				<p class="mt-1 text-xs text-slate-600">{dipilih.uraian_pelaksanaan}</p>
				{#if dipilih.kendala}<p class="mt-1 text-xs text-amber-700">Kendala: {dipilih.kendala}</p>{/if}
			</div>

			<div>
				<label class="label" for="setuju">Keputusan</label>
				<select id="setuju" class="input" bind:value={setuju}>
					<option value={true}>Terverifikasi</option>
					<option value={false}>Perlu Revisi</option>
				</select>
			</div>

			{#if setuju}
				<div>
					<label class="label" for="kualitas">Skor Kualitas (0–100)</label>
					<input id="kualitas" class="input" bind:value={skorKualitas} />
				</div>
			{/if}

			<div>
				<label class="label" for="catatan">Catatan Verifikator</label>
				<textarea id="catatan" class="input h-20 resize-none" bind:value={catatan}></textarea>
			</div>

			<div class="flex justify-end gap-2">
				<button class="btn btn-ghost" onclick={() => (dipilih = null)}>Batal</button>
				<button class="btn btn-primary" onclick={verifikasi}>Simpan</button>
			</div>
		</div>
	{/if}
</Modal>
