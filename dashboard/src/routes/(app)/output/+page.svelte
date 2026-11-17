<script lang="ts">
	import { onMount } from 'svelte';
	import { api, apiBlob, ApiError } from '#lib/api/client';
	import { adalahAtasan } from '#lib/stores/session.svelte';
	import type { Hasil, Lampiran, ListResponse, Output, Periode, Personel, Satker } from '#lib/types';
	import { LABEL_JENIS_OUTPUT, LABEL_STATUS_OUTPUT } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	let data = $state<Output[]>([]);
	let meta = $state({ page: 1, per_page: 10, total: 0 });
	let memuat = $state(true);
	let mengekspor = $state(false);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);

	let periodeList = $state<Periode[]>([]);
	let satkerList = $state<Satker[]>([]);
	let personelList = $state<Personel[]>([]);
	let periodeId = $state('');
	let satkerId = $state('');
	let personelId = $state('');
	let status = $state<Output['status'] | ''>('');

	const personelTersedia = $derived(
		satkerId ? personelList.filter((p) => p.satker_id === satkerId) : personelList
	);

	let hasilTerverifikasi = $state<Hasil[]>([]);
	let formTerbuka = $state(false);
	let menyimpan = $state(false);
	let form = $state({
		hasil_kerja_id: '',
		jenis_output: 'laporan',
		judul: '',
		uraian: '',
		tanggal_hasil: new Date().toISOString().slice(0, 10),
		url_eksternal: ''
	});

	let dipilih = $state<Output | null>(null);
	let lampiran = $state<Lampiran[]>([]);
	let mengunggah = $state(false);
	let setuju = $state(true);
	let skorKesesuaian = $state('90');
	let skorKeabsahan = $state('92');
	let catatan = $state('');

	async function muat(page = 1) {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams();
			if (periodeId) params.set('periode_id', periodeId);
			if (satkerId) params.set('satker_id', satkerId);
			if (personelId) params.set('personel_id', personelId);
			if (status) params.set('status', status);
			params.set('page', String(page));
			params.set('per_page', String(meta.per_page));
			const hasil = await api<ListResponse<Output>>(`/output?${params}`);
			data = hasil.data;
			meta = hasil.meta;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat output';
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
			if (satkerId) params.set('satker_id', satkerId);
			if (personelId) params.set('personel_id', personelId);
			if (status) params.set('status', status);

			const blob = await apiBlob(`/output/ekspor?${params}`);
			const url = URL.createObjectURL(blob);
			const tautan = document.createElement('a');
			tautan.href = url;
			tautan.download = `output-kerja-${new Date().toISOString().slice(0, 10)}.pdf`;
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
			const hasil = await api<ListResponse<Hasil>>('/hasil-kerja?status=terverifikasi&per_page=100');
			hasilTerverifikasi = hasil.data;
			form.hasil_kerja_id = hasil.data[0]?.id ?? '';
		} catch {
			hasilTerverifikasi = [];
		}
	}

	async function simpan() {
		menyimpan = true;
		error = null;
		try {
			const dibuat = await api<Output>('/output', {
				method: 'POST',
				body: {
					hasil_kerja_id: form.hasil_kerja_id,
					jenis_output: form.jenis_output,
					judul: form.judul,
					uraian: form.uraian || null,
					tanggal_hasil: form.tanggal_hasil,
					url_eksternal: form.jenis_output === 'link_sistem' ? form.url_eksternal : null
				}
			});
			formTerbuka = false;
			form.judul = '';
			await muat(1);
			await bukaVerifikasi(dibuat, true);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan output';
		} finally {
			menyimpan = false;
		}
	}

	async function bukaVerifikasi(row: Output, langsung = false) {
		dipilih = row;
		setuju = true;
		catatan = '';
		lampiran = [];
		if (!langsung) {
			try {
				const detail = await api<{ lampiran: Lampiran[] }>(`/output/${row.id}`);
				lampiran = detail.lampiran;
			} catch {
				lampiran = [];
			}
		}
	}

	async function unggah(event: Event) {
		const input = event.target as HTMLInputElement;
		const berkas = input.files?.[0];
		if (!berkas || !dipilih) return;

		mengunggah = true;
		error = null;
		try {
			const body = new FormData();
			body.append('owner_type', 'output');
			body.append('owner_id', dipilih.id);
			body.append('kategori', 'bukti');
			body.append('file', berkas);
			await api('/lampiran', { method: 'POST', body });
			input.value = '';
			const detail = await api<{ lampiran: Lampiran[] }>(`/output/${dipilih.id}`);
			lampiran = detail.lampiran;
			info = 'Berkas bukti berhasil diunggah.';
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal mengunggah berkas';
		} finally {
			mengunggah = false;
		}
	}

	async function ajukan() {
		if (!dipilih) return;
		try {
			await api(`/output/${dipilih.id}/ajukan`, { method: 'POST' });
			await muat(meta.page);
			info = 'Output diajukan untuk verifikasi.';
			dipilih = null;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal mengajukan output';
		}
	}

	async function verifikasi() {
		if (!dipilih) return;
		try {
			await api(`/output/${dipilih.id}/verifikasi`, {
				method: 'POST',
				body: {
					setuju,
					skor_kesesuaian: setuju ? Number(skorKesesuaian) : undefined,
					skor_keabsahan: setuju ? Number(skorKeabsahan) : undefined,
					catatan: catatan || null
				}
			});
			dipilih = null;
			await muat(meta.page);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memverifikasi output';
		}
	}

	function nada(status: Output['status']) {
		if (status === 'disetujui') return 'green';
		if (status === 'perlu_revisi') return 'amber';
		if (status === 'dalam_verifikasi') return 'blue';
		return 'slate';
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

<svelte:head><title>Output — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card space-y-3 p-4">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="text-sm font-semibold text-slate-800">Output / Bukti Hasil Kerja</h2>
				<p class="text-xs text-slate-500">
					Laporan, dokumen, foto, video, dan bukti lain dari hasil kerja terverifikasi.
				</p>
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
					<Icon name="plus" size={16} /> Tambah Output
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
			<div class="min-w-44">
				<label class="label" for="status">Status</label>
				<select id="status" class="input" bind:value={status} onchange={() => muat(1)}>
					<option value="">Semua status</option>
					{#each Object.entries(LABEL_STATUS_OUTPUT) as [nilai, label]}
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

	{#if info}
		<p class="rounded-lg bg-sky-50 px-4 py-2 text-sm text-sky-800 ring-1 ring-sky-200">{info}</p>
	{/if}
	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	<div class="card overflow-hidden">
		<table class="w-full">
			<thead class="bg-slate-50">
				<tr>
					<th class="th">Judul Output</th>
					<th class="th">Jenis</th>
					<th class="th">Personel</th>
					<th class="th text-center">Nilai</th>
					<th class="th">Status</th>
					<th class="th no-print">Aksi</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-slate-100">
				{#each data as row}
					<tr class="hover:bg-slate-50/70">
						<td class="td">
							<a class="font-medium text-navy-800 hover:underline" href="/output/{row.id}">{row.judul}</a>
							<p class="text-[11px] text-slate-500">{row.tanggal_hasil}</p>
						</td>
						<td class="td text-slate-600">{LABEL_JENIS_OUTPUT[row.jenis_output] ?? row.jenis_output}</td>
						<td class="td text-slate-600">{row.personel_nama ?? '-'}</td>
						<td class="td text-center font-semibold text-slate-800">
							{row.skor_total != null ? row.skor_total.toFixed(1) : '-'}
						</td>
						<td class="td">
							<Badge tone={nada(row.status)}>{LABEL_STATUS_OUTPUT[row.status]}</Badge>
						</td>
						<td class="td no-print">
							<div class="flex gap-1">
								<a
									class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
									href="/output/{row.id}"
								>
									Detail
								</a>
								{#if adalahAtasan() && row.status !== 'disetujui' && row.status !== 'draft'}
									<button
										class="btn btn-ghost border border-emerald-300 px-2 py-1 text-xs text-emerald-700"
										onclick={async () => {
											await bukaVerifikasi(row);
											setuju = true;
										}}
									>
										Verifikasi
									</button>
								{/if}
							</div>
						</td>
					</tr>
				{:else}
					<tr><td class="td text-center text-slate-500" colspan="6">{memuat ? 'Memuat…' : 'Belum ada output.'}</td></tr>
				{/each}
			</tbody>
		</table>
		<Pagination page={meta.page} perPage={meta.per_page} total={meta.total} label="output" onchange={(p) => muat(p)} />
	</div>
</div>

<Modal open={formTerbuka} title="Tambah Output" lebar="max-w-xl" onclose={() => (formTerbuka = false)}>
	<div class="space-y-3">
		<div>
			<label class="label" for="hasil">Hasil Kerja Terverifikasi</label>
			<select id="hasil" class="input" bind:value={form.hasil_kerja_id}>
				{#each hasilTerverifikasi as h}
					<option value={h.id}>{h.rencana_kegiatan} ({h.tanggal_realisasi})</option>
				{:else}
					<option value="">Belum ada hasil terverifikasi</option>
				{/each}
			</select>
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label class="label" for="jenis">Jenis Output</label>
				<select id="jenis" class="input" bind:value={form.jenis_output}>
					{#each Object.entries(LABEL_JENIS_OUTPUT) as [nilai, label]}
						<option value={nilai}>{label}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="label" for="tanggal">Tanggal Hasil</label>
				<input id="tanggal" type="date" class="input" bind:value={form.tanggal_hasil} />
			</div>
		</div>
		<div>
			<label class="label" for="judul">Judul Output</label>
			<input id="judul" class="input" bind:value={form.judul} />
		</div>
		<div>
			<label class="label" for="uraian">Uraian Singkat</label>
			<textarea id="uraian" class="input h-20 resize-none" bind:value={form.uraian}></textarea>
		</div>
		{#if form.jenis_output === 'link_sistem'}
			<div>
				<label class="label" for="url">URL / Link Sistem</label>
				<input id="url" class="input" placeholder="https://" bind:value={form.url_eksternal} />
			</div>
		{/if}
		<div class="flex justify-end gap-2 pt-2">
			<button class="btn btn-ghost" onclick={() => (formTerbuka = false)}>Batal</button>
			<button class="btn btn-primary" disabled={menyimpan || !form.hasil_kerja_id} onclick={simpan}>
				Simpan &amp; Unggah Berkas
			</button>
		</div>
	</div>
</Modal>

<Modal open={!!dipilih} title="Detail Output" lebar="max-w-xl" onclose={() => (dipilih = null)}>
	{#if dipilih}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="font-medium text-slate-800">{dipilih.judul}</p>
				<p class="text-xs text-slate-500">
					{LABEL_JENIS_OUTPUT[dipilih.jenis_output]} · {dipilih.personel_nama} · {dipilih.tanggal_hasil}
				</p>
				{#if dipilih.uraian}<p class="mt-1 text-xs text-slate-600">{dipilih.uraian}</p>{/if}
				{#if dipilih.url_eksternal}
					<a class="mt-1 block text-xs text-navy-700 underline" href={dipilih.url_eksternal} target="_blank" rel="noreferrer">
						{dipilih.url_eksternal}
					</a>
				{/if}
			</div>

			<div>
				<p class="text-xs font-semibold tracking-wide text-slate-500 uppercase">Berkas Bukti</p>
				<ul class="mt-1 space-y-1">
					{#each lampiran as item}
						<li class="truncate rounded bg-slate-50 px-2 py-1 text-xs text-slate-700">{item.file_name}</li>
					{:else}
						<li class="text-xs text-slate-500">Belum ada berkas.</li>
					{/each}
				</ul>
				{#if dipilih.status !== 'disetujui'}
					<label class="btn btn-ghost mt-2 w-full border border-dashed border-slate-300 text-xs no-print">
						<Icon name="upload" size={14} /> {mengunggah ? 'Mengunggah…' : 'Unggah berkas bukti'}
						<input type="file" class="hidden" accept=".pdf,.jpg,.jpeg,.png,.mp4" onchange={unggah} />
					</label>
				{/if}
			</div>

			{#if adalahAtasan() && dipilih.status !== 'disetujui'}
				<div class="grid gap-3 rounded-lg border border-slate-200 p-3">
					<div>
						<label class="label" for="vsetuju">Keputusan</label>
						<select id="vsetuju" class="input" bind:value={setuju}>
							<option value={true}>Disetujui</option>
							<option value={false}>Perlu Revisi</option>
						</select>
					</div>
					{#if setuju}
						<div class="grid grid-cols-2 gap-2">
							<div>
								<label class="label" for="kesesuaian">Kesesuaian</label>
								<input id="kesesuaian" class="input" bind:value={skorKesesuaian} />
							</div>
							<div>
								<label class="label" for="keabsahan">Keabsahan</label>
								<input id="keabsahan" class="input" bind:value={skorKeabsahan} />
							</div>
						</div>
					{/if}
					<div>
						<label class="label" for="vcatatan">Catatan</label>
						<textarea id="vcatatan" class="input h-16 resize-none" bind:value={catatan}></textarea>
					</div>
					<div class="flex justify-end gap-2">
						<button class="btn btn-ghost" onclick={() => (dipilih = null)}>Tutup</button>
						<button class="btn btn-primary" onclick={verifikasi}>Simpan Verifikasi</button>
					</div>
				</div>
			{:else}
				<div class="flex justify-end gap-2">
					<button class="btn btn-ghost" onclick={() => (dipilih = null)}>Tutup</button>
					{#if dipilih.status === 'menunggu_verifikasi'}
						<button class="btn btn-primary" onclick={ajukan}>Ajukan Verifikasi</button>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</Modal>
