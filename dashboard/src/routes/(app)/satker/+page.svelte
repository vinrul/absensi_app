<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import { adalahAdmin, adalahAdminGlobal, session } from '#lib/stores/session.svelte';
	import type { Satker, SatkerLevel } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	type FormSatker = {
		id: string | null;
		parent_id: string;
		level: SatkerLevel;
		nama: string;
		kode: string;
		alamat: string;
		timezone: string;
		hari_kerja: number[];
		jam_masuk: string;
		jam_pulang: string;
		jam_pulang_sabtu: string;
		is_active: boolean;
	};

	const HARI = [
		{ nilai: 1, label: 'Senin' },
		{ nilai: 2, label: 'Selasa' },
		{ nilai: 3, label: 'Rabu' },
		{ nilai: 4, label: 'Kamis' },
		{ nilai: 5, label: 'Jumat' },
		{ nilai: 6, label: 'Sabtu' },
		{ nilai: 7, label: 'Minggu' }
	];

	const ZONA_WAKTU = ['Asia/Jakarta', 'Asia/Makassar', 'Asia/Jayapura'];

	let data = $state<Satker[]>([]);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);
	let formTerbuka = $state(false);
	let halaman = $state(1);
	const perPage = 10;

	const admin = $derived(adalahAdmin());
	const globalAdmin = $derived(adalahAdminGlobal());
	const scopePolda = $derived(
		session.user?.role_scopes.find((r) => r.role === 'admin_polda')?.scope_satker_id ?? ''
	);
	const scopePolres = $derived(
		session.user?.role_scopes.find((r) => r.role === 'admin_polres')?.scope_satker_id ?? ''
	);
	const bolehTambah = $derived(globalAdmin || Boolean(scopePolda) || Boolean(scopePolres));

	let form = $state<FormSatker>(kosongForm());

	function kosongForm(): FormSatker {
		return {
			id: null,
			parent_id: '',
			level: 'satker',
			nama: '',
			kode: '',
			alamat: '',
			timezone: 'Asia/Makassar',
			hari_kerja: [1, 2, 3, 4, 5],
			jam_masuk: '08:00',
			jam_pulang: '16:00',
			jam_pulang_sabtu: '',
			is_active: true
		};
	}

	function labelLevel(level: SatkerLevel) {
		return level === 'polda' ? 'Polda' : level === 'polres' ? 'Polres' : 'Satker';
	}

	function namaInduk(id: string | null) {
		if (!id) return '—';
		return data.find((s) => s.id === id)?.nama ?? '—';
	}

	const parentOptions = $derived(
		data.filter((s) => {
			if (form.level === 'polda') return false;
			if (form.level === 'polres') return s.level === 'polda';
			return s.level === 'polda' || s.level === 'polres';
		})
	);

	const urut = $derived(
		[...data].sort((a, b) => {
			const rank = { polda: 0, polres: 1, satker: 2 };
			return rank[a.level] - rank[b.level] || a.nama.localeCompare(b.nama);
		})
	);
	const totalHalaman = $derived(Math.max(1, Math.ceil(urut.length / perPage)));
	const halamanAman = $derived(Math.min(halaman, totalHalaman));
	const urutHalaman = $derived(urut.slice((halamanAman - 1) * perPage, halamanAman * perPage));

	async function muat() {
		memuat = true;
		error = null;
		try {
			data = await api<Satker[]>('/satker');
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat satker';
		} finally {
			memuat = false;
		}
	}

	function bukaTambah() {
		form = kosongForm();
		if (!globalAdmin && scopePolda) {
			form.level = 'satker';
			form.parent_id = scopePolda;
		} else if (!globalAdmin && scopePolres) {
			form.level = 'satker';
			form.parent_id = scopePolres;
		} else {
			form.level = 'polda';
		}
		formTerbuka = true;
	}

	function bukaEdit(row: Satker) {
		form = {
			id: row.id,
			parent_id: row.parent_id ?? '',
			level: row.level,
			nama: row.nama,
			kode: row.kode,
			alamat: row.alamat ?? '',
			timezone: row.timezone,
			hari_kerja: [...row.hari_kerja],
			jam_masuk: row.jam_masuk,
			jam_pulang: row.jam_pulang,
			jam_pulang_sabtu: row.jam_pulang_sabtu ?? '',
			is_active: row.is_active
		};
		formTerbuka = true;
	}

	function toggleHari(nilai: number) {
		const aktif = form.hari_kerja.includes(nilai);
		form.hari_kerja = aktif
			? form.hari_kerja.filter((h) => h !== nilai)
			: [...form.hari_kerja, nilai].sort();
		if (nilai === 6 && aktif) form.jam_pulang_sabtu = '';
	}

	async function simpan() {
		error = null;
		info = null;
		try {
			const body = {
				level: form.level,
				nama: form.nama,
				kode: form.kode,
				alamat: form.alamat || null,
				timezone: form.timezone || 'Asia/Makassar',
				hari_kerja: form.hari_kerja,
				jam_masuk: form.jam_masuk,
				jam_pulang: form.jam_pulang,
				jam_pulang_sabtu: form.jam_pulang_sabtu || null,
				is_active: form.is_active
			};
			if (form.id) {
				await api(`/satker/${form.id}`, {
					method: 'PATCH',
					body: { ...body, hapus_jam_pulang_sabtu: !form.jam_pulang_sabtu }
				});
			} else {
				await api('/satker', {
					method: 'POST',
					body: { ...body, parent_id: form.parent_id || null }
				});
			}
			formTerbuka = false;
			info = form.id ? 'Satker diperbarui.' : 'Satker baru dibuat.';
			await muat();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan satker';
		}
	}

	async function toggleAktif(row: Satker) {
		try {
			await api(`/satker/${row.id}`, {
				method: 'PATCH',
				body: { is_active: !row.is_active }
			});
			await muat();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal mengubah status satker';
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Satker & Jam Kerja — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-start justify-between gap-3 p-4">
		<div>
			<h2 class="text-sm font-semibold text-slate-800">Struktur Organisasi & Jam Kerja</h2>
			<p class="mt-1 text-xs text-slate-500">
				Polda → Polres → Satker. Jam kerja dan hari kerja dipakai mesin penilaian.
			</p>
		</div>
		{#if admin && bolehTambah}
			<button class="btn btn-primary" onclick={bukaTambah}>
				<Icon name="plus" size={16} /> Tambah Satker
			</button>
		{/if}
	</div>

	{#if info}<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>{/if}
	{#if error}<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>{/if}

	<div class="card overflow-hidden">
		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">Kode</th>
						<th class="th">Nama</th>
						<th class="th">Level</th>
						<th class="th">Induk</th>
						<th class="th">Jam Kerja</th>
						<th class="th">Hari Kerja</th>
						<th class="th">Status</th>
						<th class="th no-print">Aksi</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each urutHalaman as row}
						<tr class="hover:bg-slate-50/70">
							<td class="td text-slate-600">{row.kode}</td>
							<td class="td font-medium text-slate-800">
								{row.level === 'satker' ? '↳ ' : ''}{row.nama}
							</td>
							<td class="td capitalize text-slate-600">{labelLevel(row.level)}</td>
							<td class="td text-slate-600">{namaInduk(row.parent_id)}</td>
							<td class="td text-slate-600">
								{row.jam_masuk} – {row.jam_pulang}
								{#if row.jam_pulang_sabtu}
									<span class="block text-[11px] text-slate-500">Sabtu s.d. {row.jam_pulang_sabtu}</span>
								{/if}
							</td>
							<td class="td text-xs text-slate-600">
								{row.hari_kerja.map((h) => HARI.find((x) => x.nilai === h)?.label).join(', ')}
							</td>
							<td class="td"><Badge tone={row.is_active ? 'green' : 'slate'}>{row.is_active ? 'Aktif' : 'Nonaktif'}</Badge></td>
							<td class="td no-print">
								{#if admin}
									<div class="flex gap-1">
										<button class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs" onclick={() => bukaEdit(row)}>Edit</button>
										<button class="btn btn-ghost px-2 py-1 text-xs {row.is_active ? 'text-rose-600' : 'text-emerald-600'}" onclick={() => toggleAktif(row)}>
											{row.is_active ? 'Nonaktifkan' : 'Aktifkan'}
										</button>
									</div>
								{/if}
							</td>
						</tr>
					{:else}
						<tr><td class="td text-center text-slate-500" colspan="8">{memuat ? 'Memuat…' : 'Belum ada satker.'}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>
		<Pagination page={halamanAman} perPage={perPage} total={urut.length} label="satker" onchange={(p) => (halaman = p)} />
	</div>
</div>

<Modal open={formTerbuka} title={form.id ? 'Edit Satker' : 'Tambah Satker'} lebar="max-w-2xl" onclose={() => (formTerbuka = false)}>
	<div class="grid gap-3 sm:grid-cols-2">
		{#if form.id}
			<div class="sm:col-span-2 rounded-lg bg-slate-50 px-3 py-2 text-xs text-slate-600">
				Level dan induk tidak dapat dipindah dari halaman ini.
			</div>
		{:else}
			<div>
				<label class="label" for="level">Level</label>
				<select id="level" class="input" bind:value={form.level} disabled={!globalAdmin || Boolean(scopePolda) || Boolean(scopePolres)}>
					{#if globalAdmin}<option value="polda">Polda</option>{/if}
					{#if globalAdmin}<option value="polres">Polres</option>{/if}
					<option value="satker">Satker</option>
				</select>
			</div>
			<div>
				<label class="label" for="induk">Induk</label>
				<select id="induk" class="input" bind:value={form.parent_id} disabled={Boolean(scopePolda) || Boolean(scopePolres)}>
					{#if form.level !== 'polda'}
						<option value="">Pilih induk…</option>
						{#each parentOptions as s}<option value={s.id}>{s.nama} ({labelLevel(s.level)})</option>{/each}
					{/if}
				</select>
			</div>
		{/if}
		<div>
			<label class="label" for="kode">Kode</label>
			<input id="kode" class="input" bind:value={form.kode} placeholder="POLRES-XXX" />
		</div>
		<div>
			<label class="label" for="nama">Nama</label>
			<input id="nama" class="input" bind:value={form.nama} placeholder="Nama satker" />
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="alamat">Alamat</label>
			<input id="alamat" class="input" bind:value={form.alamat} />
		</div>
		<div>
			<label class="label" for="timezone">Zona Waktu</label>
			<select id="timezone" class="input" bind:value={form.timezone}>
				{#each ZONA_WAKTU as zona}
					<option value={zona}>{zona}</option>
				{/each}
				{#if !ZONA_WAKTU.includes(form.timezone)}
					<option value={form.timezone}>{form.timezone}</option>
				{/if}
			</select>
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label class="label" for="masuk">Jam Masuk</label>
				<input id="masuk" type="time" class="input" bind:value={form.jam_masuk} />
			</div>
			<div>
				<label class="label" for="pulang">Jam Pulang</label>
				<input id="pulang" type="time" class="input" bind:value={form.jam_pulang} />
			</div>
		</div>
		{#if form.hari_kerja.includes(6)}
			<div class="sm:col-span-2">
				<label class="label" for="pulang-sabtu">Jam Pulang Sabtu (opsional)</label>
				<input
					id="pulang-sabtu"
					type="time"
					class="input"
					bind:value={form.jam_pulang_sabtu}
				/>
				<p class="mt-1 text-xs text-slate-500">
					Kosongkan bila Sabtu memakai jam pulang biasa. Isi mis. 10:00 atau 12:00
					untuk Sabtu setengah hari.
				</p>
			</div>
		{/if}
		<div class="sm:col-span-2">
			<span class="label">Hari Kerja</span>
			<div class="flex flex-wrap gap-3 rounded-lg border border-slate-200 p-3">
				{#each HARI as hari}
					<label class="flex items-center gap-1 text-xs text-slate-700">
						<input type="checkbox" checked={form.hari_kerja.includes(hari.nilai)} onchange={() => toggleHari(hari.nilai)} />
						{hari.label}
					</label>
				{/each}
			</div>
		</div>
	</div>
	<div class="mt-4 flex justify-end gap-2">
		<button class="btn btn-ghost" onclick={() => (formTerbuka = false)}>Batal</button>
		<button class="btn btn-primary" onclick={simpan}>Simpan</button>
	</div>
</Modal>
