<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import type { Absensi, Periode, RekapAbsensi, Satker } from '#lib/types';
	import { LABEL_STATUS_KEHADIRAN } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	let periodeList = $state<Periode[]>([]);
	let satkerList = $state<Satker[]>([]);
	let periodeId = $state('');
	let satkerId = $state('');
	let unit = $state('');

	let rekap = $state<RekapAbsensi[]>([]);
	let periodeNama = $state('');
	let memuat = $state(true);
	let error = $state<string | null>(null);

	let menunggu = $state<Absensi[]>([]);
	let pilihVerifikasi = $state<Absensi | null>(null);
	let catatan = $state('');
	let halaman = $state(1);
	let halamanMenunggu = $state(1);
	const perPage = 10;

	const totalHalaman = $derived(Math.max(1, Math.ceil(rekap.length / perPage)));
	const halamanAman = $derived(Math.min(halaman, totalHalaman));
	const rekapHalaman = $derived(rekap.slice((halamanAman - 1) * perPage, halamanAman * perPage));
	const totalHalamanMenunggu = $derived(Math.max(1, Math.ceil(menunggu.length / perPage)));
	const halamanMenungguAman = $derived(Math.min(halamanMenunggu, totalHalamanMenunggu));
	const menungguHalaman = $derived(
		menunggu.slice((halamanMenungguAman - 1) * perPage, halamanMenungguAman * perPage)
	);

	async function muat() {
		memuat = true;
		error = null;
		halaman = 1;
		halamanMenunggu = 1;
		try {
			const params = new URLSearchParams();
			if (periodeId) params.set('periode_id', periodeId);
			if (satkerId) params.set('satker_id', satkerId);
			if (unit.trim()) params.set('unit', unit.trim());

			const hasil = await api<{ periode: { nama: string }; data: RekapAbsensi[] }>(
				`/absensi/rekap?${params}`
			);
			rekap = hasil.data;
			periodeNama = hasil.periode.nama;

			const daftar = await api<{ data: Absensi[] }>('/absensi?perlu_verifikasi=true&per_page=50');
			menunggu = daftar.data;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat rekap';
		} finally {
			memuat = false;
		}
	}

	async function verifikasi() {
		if (!pilihVerifikasi) return;
		try {
			await api(`/absensi/${pilihVerifikasi.id}/verifikasi`, {
				method: 'POST',
				body: { catatan: catatan || null }
			});
			pilihVerifikasi = null;
			catatan = '';
			await muat();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memverifikasi';
		}
	}

	onMount(async () => {
		const [p, s] = await Promise.all([api<Periode[]>('/periode'), api<Satker[]>('/satker')]);
		periodeList = p;
		satkerList = s;
		periodeId = p[0]?.id ?? '';
		await muat();
	});
</script>

<svelte:head><title>Rekap Kehadiran — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-end gap-3 p-4">
		<div class="min-w-40 flex-1">
			<label class="label" for="periode">Periode</label>
			<select id="periode" class="input" bind:value={periodeId} onchange={muat}>
				{#each periodeList as p}<option value={p.id}>{p.nama}</option>{/each}
			</select>
		</div>
		<div class="min-w-40 flex-1">
			<label class="label" for="satker">Satker</label>
			<select id="satker" class="input" bind:value={satkerId} onchange={muat}>
				<option value="">Semua Satker</option>
				{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<div class="min-w-32 flex-1">
			<label class="label" for="unit">Subdit / Bagian</label>
			<input id="unit" class="input" placeholder="Semua unit" bind:value={unit} onchange={muat} />
		</div>
		<button class="btn btn-ghost border border-slate-300" onclick={muat}>
			<Icon name="refresh" size={16} /> Muat ulang
		</button>
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	{#if menunggu.length}
		<div class="card overflow-hidden">
			<div class="flex items-center gap-2 border-b border-amber-200 bg-amber-50 px-4 py-2">
				<Icon name="alert" size={16} class="text-amber-600" />
				<h2 class="text-sm font-semibold text-amber-800">
					{menunggu.length} absensi menunggu verifikasi
				</h2>
			</div>
			<div class="divide-y divide-slate-100">
				{#each menungguHalaman as row}
					<div class="flex flex-wrap items-center justify-between gap-2 px-4 py-2 text-sm">
						<div>
							<p class="font-medium text-slate-800">{row.personel_nama}</p>
							<p class="text-xs text-slate-500">
								{new Date(row.tanggal + 'T00:00:00').toLocaleDateString('id-ID')} · {row.jenis} {row.jam}
								{#if row.is_offline}<span class="ml-1 text-amber-600">offline</span>{/if}
								{#if row.is_mock_suspect}<span class="ml-1 text-rose-600">mock GPS</span>{/if}
							</p>
						</div>
						<button
							class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
							onclick={() => {
								pilihVerifikasi = row;
								catatan = '';
							}}
						>
							Verifikasi
						</button>
					</div>
				{/each}
			</div>
			<Pagination page={halamanMenungguAman} perPage={perPage} total={menunggu.length} label="absensi" onchange={(p) => (halamanMenunggu = p)} />
		</div>
	{/if}

	<div class="card overflow-hidden">
		<div class="flex items-center justify-between border-b border-slate-200 px-4 py-3">
			<h2 class="text-sm font-semibold text-slate-700">Rekap Kehadiran {periodeNama}</h2>
			<span class="text-xs text-slate-500">{rekap.length} personel</span>
		</div>

		<div class="max-h-[70vh] overflow-auto">
			<table class="w-full">
				<thead class="sticky top-0 bg-slate-50">
					<tr>
						<th class="th">Personel</th>
						<th class="th">Unit</th>
						<th class="th text-center">Hadir</th>
						<th class="th text-center">Telat</th>
						<th class="th text-center">DL</th>
						<th class="th text-center">Piket</th>
						<th class="th text-center">Cuti</th>
						<th class="th text-center">Izin</th>
						<th class="th text-center">Sakit</th>
						<th class="th text-center">PH</th>
						<th class="th text-center">TK</th>
						<th class="th text-center">Menunggu</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each rekapHalaman as row}
						<tr class="hover:bg-slate-50/70">
							<td class="td">
								<p class="font-medium text-slate-800">{row.nama}</p>
								<p class="text-[11px] text-slate-500">{row.pangkat} · {row.nrp}</p>
							</td>
							<td class="td text-slate-600">{row.unit ?? '-'}</td>
							<td class="td text-center font-semibold text-emerald-700">{row.hadir}</td>
							<td class="td text-center {row.telat > 0 ? 'text-amber-700' : 'text-slate-400'}">{row.telat}</td>
							<td class="td text-center text-slate-600">{row.dinas_luar}</td>
							<td class="td text-center text-slate-600">{row.piket}</td>
							<td class="td text-center text-slate-600">{row.cuti}</td>
							<td class="td text-center text-slate-600">{row.izin}</td>
							<td class="td text-center text-slate-600">{row.sakit}</td>
							<td class="td text-center text-slate-600">{row.libur_nasional}</td>
							<td class="td text-center {row.tanpa_keterangan > 0 ? 'font-semibold text-rose-700' : 'text-slate-400'}">
								{row.tanpa_keterangan}
							</td>
							<td class="td text-center">
								{#if row.menunggu_verifikasi > 0}
									<Badge tone="amber">{row.menunggu_verifikasi}</Badge>
								{:else}
									<span class="text-slate-300">-</span>
								{/if}
							</td>
						</tr>
					{:else}
						<tr><td class="td text-center text-slate-500" colspan="12">{memuat ? 'Memuat…' : 'Belum ada data.'}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>
		<Pagination page={halamanAman} perPage={perPage} total={rekap.length} label="personel" onchange={(p) => (halaman = p)} />
	</div>
</div>

<Modal open={!!pilihVerifikasi} title="Verifikasi Absensi" onclose={() => (pilihVerifikasi = null)}>
	{#if pilihVerifikasi}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="font-medium text-slate-800">{pilihVerifikasi.personel_nama}</p>
				<p class="text-xs text-slate-500">
					{new Date(pilihVerifikasi.tanggal + 'T00:00:00').toLocaleDateString('id-ID')} · {pilihVerifikasi.jenis} {pilihVerifikasi.jam}
				</p>
				<p class="mt-1 text-xs text-slate-600">
					Status: {LABEL_STATUS_KEHADIRAN[pilihVerifikasi.status_kehadiran]} · Jarak:
					{pilihVerifikasi.jarak_m != null ? `${pilihVerifikasi.jarak_m.toFixed(0)} m` : '-'}
				</p>
				<p class="mt-1 text-xs text-slate-600">
					{JSON.stringify({
						offline: pilihVerifikasi.is_offline,
						mock: pilihVerifikasi.is_mock_suspect
					})}
				</p>
			</div>

			<div>
				<label class="label" for="catatan">Catatan Verifikator</label>
				<textarea id="catatan" class="input h-24 resize-none" bind:value={catatan}></textarea>
			</div>

			<div class="flex justify-end gap-2">
				<button class="btn btn-ghost" onclick={() => (pilihVerifikasi = null)}>Batal</button>
				<button class="btn btn-primary" onclick={verifikasi}><Icon name="check" size={16} /> Setujui</button>
			</div>
		</div>
	{/if}
</Modal>
