<script lang="ts">
	import { onMount } from 'svelte';
	import { api, apiBlob, ApiError } from '#lib/api/client';
	import type { ListResponse, Periode, Personel, Rencana, Satker, StatusRencana } from '#lib/types';
	import { LABEL_STATUS_RENCANA } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';

	let periodeList = $state<Periode[]>([]);
	let satkerList = $state<Satker[]>([]);
	let personelList = $state<Personel[]>([]);
	let periodeId = $state('');
	let satkerId = $state('');
	let personelId = $state('');
	let status = $state<StatusRencana | ''>('');
	let data = $state<Rencana[]>([]);
	let meta = $state({ page: 1, per_page: 10, total: 0 });
	let memuat = $state(true);
	let mengekspor = $state(false);
	let error = $state<string | null>(null);
	let formTerbuka = $state(false);
	let menyimpan = $state(false);

	const personelTersedia = $derived(
		satkerId ? personelList.filter((p) => p.satker_id === satkerId) : personelList
	);

	let form = $state({
		program: '',
		kegiatan: '',
		uraian: '',
		tujuan: '',
		indikator_kinerja: '',
		jenis: 'bulanan',
		dasar_jenis: 'sp',
		dasar_nomor: '',
		dasar_tanggal: '',
		dasar_perihal: '',
		target_kuantitas: '',
		satuan: 'laporan',
		target_kualitas: 'Sesuai SOP',
		target_waktu: '',
		tanggal_mulai: '',
		tanggal_selesai: '',
		tahapan: ''
	});

	const totalHalaman = $derived(Math.max(1, Math.ceil(meta.total / meta.per_page)));

	function nada(value: StatusRencana) {
		if (value === 'disetujui') return 'green';
		if (value === 'diajukan') return 'blue';
		if (value === 'perlu_revisi') return 'amber';
		if (value === 'ditolak') return 'red';
		return 'slate';
	}

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
			const hasil = await api<ListResponse<Rencana>>(`/rencana-kerja?${params}`);
			data = hasil.data;
			meta = hasil.meta;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat rencana kerja';
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

			const blob = await apiBlob(`/rencana-kerja/ekspor?${params}`);
			const url = URL.createObjectURL(blob);
			const tautan = document.createElement('a');
			tautan.href = url;
			tautan.download = `rencana-kerja-${new Date().toISOString().slice(0, 10)}.pdf`;
			tautan.click();
			URL.revokeObjectURL(url);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal membuat PDF';
		} finally {
			mengekspor = false;
		}
	}

	async function simpan(ajukan: boolean) {
		menyimpan = true;
		error = null;
		try {
			const tahapan = form.tahapan
				.split('\n')
				.map((baris) => baris.trim())
				.filter(Boolean)
				.map((uraian) => ({ uraian }));

			await api('/rencana-kerja', {
				method: 'POST',
				body: {
					periode_id: periodeId,
					program: form.program,
					kegiatan: form.kegiatan,
					uraian: form.uraian,
					tujuan: form.tujuan || null,
					indikator_kinerja: form.indikator_kinerja || null,
					jenis: form.jenis,
					dasar_jenis: form.dasar_jenis,
					dasar_nomor: form.dasar_nomor || null,
					dasar_tanggal: form.dasar_tanggal || null,
					dasar_perihal: form.dasar_perihal || null,
					target_kuantitas: form.target_kuantitas ? Number(form.target_kuantitas) : null,
					satuan: form.satuan || null,
					target_kualitas: form.target_kualitas || null,
					target_waktu: form.target_waktu || null,
					tanggal_mulai: form.tanggal_mulai,
					tanggal_selesai: form.tanggal_selesai,
					tahapan,
					ajukan
				}
			});

			formTerbuka = false;
			await muat(1);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan rencana kerja';
		} finally {
			menyimpan = false;
		}
	}

	async function ajukan(row: Rencana) {
		try {
			await api(`/rencana-kerja/${row.id}/ajukan`, { method: 'POST' });
			await muat(meta.page);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal mengajukan rencana';
		}
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
		const hari = new Date().toISOString().slice(0, 10);
		form.tanggal_mulai = hari;
		form.tanggal_selesai = hari;
		form.target_waktu = hari;
		form.dasar_tanggal = hari;
		await muat(1);
	});
</script>

<svelte:head><title>Rencana Kerja — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card space-y-3 p-4">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="text-sm font-semibold text-slate-800">Rencana Kerja</h2>
				<p class="text-xs text-slate-500">
					Rencana kerja personel, target, dan status persetujuan atasan.
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
				<button class="btn btn-primary" onclick={() => (formTerbuka = true)}>
					<Icon name="plus" size={16} /> Buat Rencana
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
					{#each Object.entries(LABEL_STATUS_RENCANA) as [nilai, label]}
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
		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">Kegiatan</th>
						<th class="th">Personel</th>
						<th class="th">Periode Kerja</th>
						<th class="th text-center">Target</th>
						<th class="th">Status</th>
						<th class="th no-print">Aksi</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each data as row}
						<tr class="hover:bg-slate-50/70">
							<td class="td">
								<a class="font-medium text-navy-800 hover:underline" href="/rencana-kerja/{row.id}">
									{row.kegiatan}
								</a>
								<p class="text-[11px] text-slate-500">{row.program}</p>
							</td>
							<td class="td text-slate-600">
								{row.personel_nama ?? '-'}
								<p class="text-[11px] text-slate-500">{row.unit ?? '-'}</p>
							</td>
							<td class="td text-xs text-slate-600">{row.tanggal_mulai} → {row.tanggal_selesai}</td>
							<td class="td text-center text-slate-600">
								{row.target_kuantitas ?? '-'} {row.satuan ?? ''}
							</td>
							<td class="td">
								<Badge tone={nada(row.status)}>{LABEL_STATUS_RENCANA[row.status]}</Badge>
								{#if row.jumlah_revisi > 0}
									<p class="mt-1 text-[10px] text-amber-600">{row.jumlah_revisi}× revisi</p>
								{/if}
							</td>
							<td class="td no-print">
								{#if row.status === 'draft' || row.status === 'perlu_revisi'}
									<button class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs" onclick={() => ajukan(row)}>
										Ajukan
									</button>
								{/if}
							</td>
						</tr>
					{:else}
						<tr><td class="td text-center text-slate-500" colspan="6">{memuat ? 'Memuat…' : 'Belum ada rencana kerja.'}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="flex items-center justify-between border-t border-slate-200 px-4 py-3">
			<p class="text-xs text-slate-500">Halaman {meta.page} dari {totalHalaman} · {meta.total} rencana</p>
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

{#snippet aksiFormRencana()}
	<div class="flex flex-wrap justify-end gap-2">
		<button class="btn btn-ghost" onclick={() => (formTerbuka = false)}>Batal</button>
		<button class="btn btn-ghost border border-slate-300" disabled={menyimpan} onclick={() => simpan(false)}>
			Simpan Draft
		</button>
		<button class="btn btn-primary" disabled={menyimpan} onclick={() => simpan(true)}>
			<Icon name="check" size={16} /> Ajukan ke Atasan
		</button>
	</div>
{/snippet}

<Modal
	open={formTerbuka}
	title="Buat Rencana Kerja"
	lebar="max-w-2xl"
	footer={aksiFormRencana}
	onclose={() => (formTerbuka = false)}
>
	<div class="grid gap-3 sm:grid-cols-2">
		<div class="sm:col-span-2">
			<label class="label" for="program">Program</label>
			<input id="program" class="input" placeholder="Penegakan Hukum" bind:value={form.program} />
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="kegiatan">Kegiatan Utama</label>
			<input id="kegiatan" class="input" placeholder="Penyelidikan kasus cyber" bind:value={form.kegiatan} />
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="uraian">Uraian Tugas</label>
			<textarea id="uraian" class="input h-20 resize-none" bind:value={form.uraian}></textarea>
		</div>
		<div>
			<label class="label" for="tujuan">Tujuan</label>
			<input id="tujuan" class="input" bind:value={form.tujuan} />
		</div>
		<div>
			<label class="label" for="indikator">Indikator Kinerja</label>
			<input id="indikator" class="input" bind:value={form.indikator_kinerja} />
		</div>
		<div>
			<label class="label" for="jenis">Jenis Rencana</label>
			<select id="jenis" class="input" bind:value={form.jenis}>
				<option value="harian">Harian</option>
				<option value="mingguan">Mingguan</option>
				<option value="bulanan">Bulanan</option>
				<option value="triwulan">Triwulan</option>
				<option value="penugasan_khusus">Penugasan Khusus</option>
			</select>
		</div>
		<div>
			<label class="label" for="dasar">Dasar Tugas</label>
			<select id="dasar" class="input" bind:value={form.dasar_jenis}>
				<option value="sp">Surat Perintah (SP)</option>
				<option value="tor">TOR</option>
				<option value="perintah_lisan">Perintah Lisan</option>
			</select>
		</div>
		<div>
			<label class="label" for="nomor">Nomor SP/TOR</label>
			<input id="nomor" class="input" placeholder="SP/123/XI/2026" bind:value={form.dasar_nomor} />
		</div>
		<div>
			<label class="label" for="dasartgl">Tanggal Dasar</label>
			<input id="dasartgl" type="date" class="input" bind:value={form.dasar_tanggal} />
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="perihal">Perihal Dasar</label>
			<input id="perihal" class="input" bind:value={form.dasar_perihal} />
		</div>
		<div>
			<label class="label" for="kuantitas">Target Kuantitas</label>
			<input id="kuantitas" class="input" placeholder="4" bind:value={form.target_kuantitas} />
		</div>
		<div>
			<label class="label" for="satuan">Satuan</label>
			<input id="satuan" class="input" bind:value={form.satuan} />
		</div>
		<div>
			<label class="label" for="kualitas">Target Kualitas</label>
			<input id="kualitas" class="input" bind:value={form.target_kualitas} />
		</div>
		<div>
			<label class="label" for="twaktu">Target Waktu</label>
			<input id="twaktu" type="date" class="input" bind:value={form.target_waktu} />
		</div>
		<div>
			<label class="label" for="mulai">Tanggal Mulai</label>
			<input id="mulai" type="date" class="input" bind:value={form.tanggal_mulai} />
		</div>
		<div>
			<label class="label" for="selesai">Tanggal Selesai</label>
			<input id="selesai" type="date" class="input" bind:value={form.tanggal_selesai} />
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="tahapan">Rincian Tahapan (satu per baris)</label>
			<textarea id="tahapan" class="input h-20 resize-none" bind:value={form.tahapan}></textarea>
		</div>
	</div>
</Modal>
