<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import { adalahAdmin, adalahAdminGlobal } from '#lib/stores/session.svelte';
	import type { HariLibur, JenisLibur, Satker } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	type FormLibur = {
		id: string | null;
		tanggal: string;
		nama: string;
		jenis: JenisLibur;
		satker_id: string;
		is_active: boolean;
	};

	const LABEL_JENIS: Record<JenisLibur, string> = {
		nasional: 'Libur Nasional',
		cuti_bersama: 'Cuti Bersama',
		khusus: 'Libur Khusus Satker'
	};

	let data = $state<HariLibur[]>([]);
	let satkerList = $state<Satker[]>([]);
	let tahun = $state(new Date().getFullYear());
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);
	let formTerbuka = $state(false);
	let form = $state<FormLibur>(kosong());
	let halaman = $state(1);
	const perPage = 10;

	const admin = $derived(adalahAdmin());
	const globalAdmin = $derived(adalahAdminGlobal());
	const satkerKhusus = $derived(
		satkerList.filter((s) => s.level === 'satker' && s.is_active)
	);
	const bolehTambah = $derived(
		admin && (globalAdmin || satkerKhusus.length > 0)
	);

	function kosong(): FormLibur {
		return {
			id: null,
			tanggal: `${tahun}-01-01`,
			nama: '',
			jenis: 'nasional',
			satker_id: '',
			is_active: true
		};
	}

	const totalHalaman = $derived(Math.max(1, Math.ceil(data.length / perPage)));
	const halamanAman = $derived(Math.min(halaman, totalHalaman));
	const dataHalaman = $derived(data.slice((halamanAman - 1) * perPage, halamanAman * perPage));

	function pesan(err: unknown): string {
		return err instanceof ApiError ? err.message : 'Terjadi kesalahan';
	}

	async function muat() {
		memuat = true;
		error = null;
		halaman = 1;
		try {
			data = await api<HariLibur[]>(`/libur?tahun=${tahun}`);
		} catch (err) {
			error = pesan(err);
		} finally {
			memuat = false;
		}
	}

	async function muatSatker() {
		if (!admin) return;
		satkerList = await api<Satker[]>('/satker');
	}

	onMount(async () => {
		try {
			await Promise.all([muat(), muatSatker()]);
		} catch (err) {
			error = pesan(err);
			memuat = false;
		}
	});

	function bukaTambah() {
		form = kosong();
		if (!globalAdmin) {
			form.jenis = 'khusus';
			form.satker_id = satkerKhusus[0]?.id ?? '';
		}
		formTerbuka = true;
	}

	function bukaEdit(row: HariLibur) {
		form = {
			id: row.id,
			tanggal: row.tanggal,
			nama: row.nama,
			jenis: row.jenis,
			satker_id: row.satker_id ?? '',
			is_active: row.is_active
		};
		formTerbuka = true;
	}

	function gantiCakupan() {
		if (form.satker_id === '') {
			if (form.jenis === 'khusus') form.jenis = 'nasional';
		} else if (form.jenis === 'nasional') {
			form.jenis = 'khusus';
		}
	}

	async function simpan() {
		error = null;
		info = null;
		try {
			if (!form.nama.trim()) throw new Error('Nama hari libur wajib diisi.');
			if (form.id) {
				await api(`/libur/${form.id}`, {
					method: 'PATCH',
					body: {
						tanggal: form.tanggal,
						nama: form.nama.trim(),
						jenis: form.jenis,
						is_active: form.is_active
					}
				});
				info = 'Hari libur diperbarui.';
			} else {
				await api('/libur', {
					method: 'POST',
					body: {
						tanggal: form.tanggal,
						nama: form.nama.trim(),
						jenis: form.jenis,
						satker_id: form.satker_id || null
					}
				});
				info = 'Hari libur ditambahkan.';
			}
			formTerbuka = false;
			await muat();
		} catch (err) {
			error = pesan(err);
		}
	}

	async function hapus(row: HariLibur) {
		if (!confirm(`Hapus hari libur “${row.nama}” (${row.tanggal})?`)) return;
		error = null;
		info = null;
		try {
			await api(`/libur/${row.id}`, { method: 'DELETE' });
			info = 'Hari libur dihapus.';
			await muat();
		} catch (err) {
			error = pesan(err);
		}
	}

	function bolehUbah(row: HariLibur): boolean {
		return admin && (row.satker_id !== null || globalAdmin);
	}

	function cakupan(row: HariLibur): string {
		return row.satker_id ? (row.satker_nama ?? 'Satker') : 'Semua satker';
	}
</script>

<svelte:head><title>Hari Libur — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div>
			<h1 class="text-xl font-semibold text-slate-800">Hari Libur</h1>
			<p class="mt-1 text-sm text-slate-500">
				Tanggal libur aktif otomatis dikeluarkan dari perhitungan absensi harian.
			</p>
		</div>
		{#if bolehTambah}
			<button class="btn btn-primary" onclick={bukaTambah}>
				<Icon name="plus" size={16} /> Tambah Hari Libur
			</button>
		{/if}
	</div>

	{#if info}
		<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>
	{/if}
	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	<div class="card flex flex-wrap items-end gap-3 p-4">
		<div class="w-40">
			<label class="label" for="tahun">Tahun</label>
			<input id="tahun" class="input" type="number" min="2000" max="2100" bind:value={tahun} />
		</div>
		<button class="btn btn-ghost" onclick={() => muat()}>Tampilkan</button>
	</div>

	<div class="card overflow-hidden">
		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">Tanggal</th>
						<th class="th">Nama</th>
						<th class="th">Jenis</th>
						<th class="th">Cakupan</th>
						<th class="th">Status</th>
						{#if admin}<th class="th text-right">Aksi</th>{/if}
					</tr>
				</thead>
				<tbody>
					{#each dataHalaman as row (row.id)}
						<tr>
							<td class="td font-mono text-xs text-slate-700">{row.tanggal}</td>
							<td class="td font-medium text-slate-800">{row.nama}</td>
							<td class="td text-xs">{LABEL_JENIS[row.jenis]}</td>
							<td class="td text-xs text-slate-600">{cakupan(row)}</td>
							<td class="td">
								<Badge tone={row.is_active ? 'green' : 'slate'}>
									{row.is_active ? 'Aktif' : 'Nonaktif'}
								</Badge>
							</td>
							{#if admin}
								<td class="td text-right">
									{#if bolehUbah(row)}
										<div class="flex justify-end gap-1">
											<button class="btn btn-ghost px-2 py-1 text-xs" onclick={() => bukaEdit(row)}>
												Edit
											</button>
											<button
												class="btn btn-ghost px-2 py-1 text-xs text-rose-600"
												onclick={() => hapus(row)}
											>
												Hapus
											</button>
										</div>
									{:else}
										<span class="text-xs text-slate-400">—</span>
									{/if}
								</td>
							{/if}
						</tr>
					{:else}
						<tr>
							<td class="td text-slate-500" colspan={admin ? 6 : 5}>
								{memuat ? 'Memuat…' : 'Belum ada hari libur pada tahun ini.'}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<Pagination page={halamanAman} perPage={perPage} total={data.length} label="hari libur" onchange={(p) => (halaman = p)} />
	</div>
</div>

<Modal
	open={formTerbuka}
	title={form.id ? 'Edit Hari Libur' : 'Tambah Hari Libur'}
	onclose={() => (formTerbuka = false)}
>
	<form class="space-y-4" onsubmit={(e) => { e.preventDefault(); void simpan(); }}>
		<div class="grid gap-4 sm:grid-cols-2">
			<div>
				<label class="label" for="tanggal">Tanggal</label>
				<input id="tanggal" class="input" type="date" bind:value={form.tanggal} />
			</div>
			<div>
				<label class="label" for="nama">Nama</label>
				<input id="nama" class="input" bind:value={form.nama} placeholder="mis. Hari Raya Nyepi" />
			</div>
			<div>
				<label class="label" for="jenis">Jenis</label>
				<select id="jenis" class="input" bind:value={form.jenis}>
					{#if form.satker_id === ''}
						<option value="nasional">Libur Nasional</option>
						<option value="cuti_bersama">Cuti Bersama</option>
					{:else}
						<option value="khusus">Libur Khusus Satker</option>
						<option value="cuti_bersama">Cuti Bersama</option>
					{/if}
				</select>
			</div>
			<div>
				<label class="label" for="cakupan">Cakupan</label>
				<select
					id="cakupan"
					class="input"
					bind:value={form.satker_id}
					onchange={gantiCakupan}
					disabled={Boolean(form.id)}
				>
					{#if globalAdmin}
						<option value="">Nasional (semua satker)</option>
					{/if}
					{#each satkerKhusus as node}
						<option value={node.id}>{node.nama}</option>
					{/each}
				</select>
			</div>
		</div>
		{#if form.id}
			<label class="flex items-center gap-2 text-sm text-slate-700">
				<input type="checkbox" bind:checked={form.is_active} /> Aktif
			</label>
		{/if}
		<p class="text-xs text-slate-500">
			Libur nasional/berlaku semua satker hanya dapat dikelola admin global. Libur khusus
			satker dikelola admin dengan cakupan satker tersebut.
		</p>
		<div class="flex justify-end gap-2">
			<button type="button" class="btn btn-ghost" onclick={() => (formTerbuka = false)}>Batal</button>
			<button type="submit" class="btn btn-primary">Simpan</button>
		</div>
	</form>
</Modal>
