<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import type { Satker, TitikDinas } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';
	import PetaPicker from '#lib/components/PetaPicker.svelte';

	type FormTitik = {
		id: string | null;
		satker_id: string;
		nama: string;
		jenis: 'kantor' | 'luar';
		latitude: string;
		longitude: string;
		radius: string;
		is_active: boolean;
	};

	let satkerList = $state<Satker[]>([]);
	let data = $state<TitikDinas[]>([]);
	let satkerId = $state('');
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);
	let formTerbuka = $state(false);
	let form = $state<FormTitik>(kosong());
	let halaman = $state(1);
	const perPage = 10;

	const totalHalaman = $derived(Math.max(1, Math.ceil(data.length / perPage)));
	const halamanAman = $derived(Math.min(halaman, totalHalaman));
	const dataHalaman = $derived(data.slice((halamanAman - 1) * perPage, halamanAman * perPage));

	const latPeta = $derived(angka(form.latitude));
	const lngPeta = $derived(angka(form.longitude));
	const radiusPeta = $derived(angka(form.radius) ?? 100);

	function angka(nilai: string): number | null {
		const teks = nilai.trim();
		if (!teks) return null;
		const hasil = Number(teks);
		return Number.isFinite(hasil) ? hasil : null;
	}

	function kosong(): FormTitik {
		return {
			id: null,
			satker_id: '',
			nama: '',
			jenis: 'kantor',
			latitude: '',
			longitude: '',
			radius: '100',
			is_active: true
		};
	}

	function pesan(err: unknown): string {
		return err instanceof ApiError ? err.message : 'Terjadi kesalahan';
	}

	async function muat() {
		memuat = true;
		error = null;
		halaman = 1;
		try {
			const params = new URLSearchParams();
			if (satkerId) params.set('satker_id', satkerId);
			data = await api<TitikDinas[]>(`/titik-dinas?${params}`);
		} catch (err) {
			error = pesan(err);
		} finally {
			memuat = false;
		}
	}

	function bukaTambah() {
		error = null;
		info = null;
		form = kosong();
		form.satker_id = satkerId || satkerList[0]?.id || '';
		formTerbuka = true;
	}

	function bukaEdit(row: TitikDinas) {
		error = null;
		info = null;
		form = {
			id: row.id,
			satker_id: row.satker_id,
			nama: row.nama,
			jenis: row.jenis,
			latitude: String(row.latitude),
			longitude: String(row.longitude),
			radius: String(row.radius_m),
			is_active: row.is_active
		};
		formTerbuka = true;
	}

	function pilihDariPeta(lat: number, lng: number) {
		form.latitude = lat.toFixed(6);
		form.longitude = lng.toFixed(6);
	}

	async function simpan() {
		error = null;
		info = null;

		const nama = form.nama.trim();
		const lat = angka(form.latitude);
		const lng = angka(form.longitude);
		const radius = angka(form.radius);

		if (!nama) {
			error = 'Nama titik wajib diisi.';
			return;
		}
		if (lat === null || lng === null) {
			error = 'Pilih titik pada peta atau isi koordinat.';
			return;
		}
		if (lat < -90 || lat > 90 || lng < -180 || lng > 180) {
			error = 'Koordinat tidak valid.';
			return;
		}
		if (radius === null || radius < 10 || radius > 5000) {
			error = 'Radius harus antara 10 dan 5000 meter.';
			return;
		}

		try {
			if (form.id) {
				await api(`/titik-dinas/${form.id}`, {
					method: 'PATCH',
					body: {
						nama,
						jenis: form.jenis,
						latitude: lat,
						longitude: lng,
						radius_m: radius,
						is_active: form.is_active
					}
				});
				info = 'Titik dinas diperbarui.';
			} else {
				await api('/titik-dinas', {
					method: 'POST',
					body: {
						satker_id: form.satker_id,
						nama,
						jenis: form.jenis,
						latitude: lat,
						longitude: lng,
						radius_m: radius
					}
				});
				info = 'Titik dinas ditambahkan.';
			}
			formTerbuka = false;
			await muat();
		} catch (err) {
			error = pesan(err);
		}
	}

	async function nonaktifkan(row: TitikDinas) {
		try {
			await api(`/titik-dinas/${row.id}`, {
				method: 'PATCH',
				body: { is_active: !row.is_active }
			});
			await muat();
		} catch (err) {
			error = pesan(err);
		}
	}

	async function hapus(row: TitikDinas) {
		if (!confirm(`Hapus titik dinas "${row.nama}"?`)) return;
		try {
			await api(`/titik-dinas/${row.id}`, { method: 'DELETE' });
			await muat();
		} catch (err) {
			error = pesan(err);
		}
	}

	onMount(async () => {
		satkerList = await api<Satker[]>('/satker');
		await muat();
	});
</script>

<svelte:head><title>Titik Dinas — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-end justify-between gap-3 p-4">
		<div class="min-w-48">
			<label class="label" for="satker">Satker</label>
			<select id="satker" class="input" bind:value={satkerId} onchange={muat}>
				<option value="">Semua Satker</option>
				{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<button class="btn btn-primary" onclick={bukaTambah}>
			<Icon name="plus" size={16} /> Tambah Titik
		</button>
	</div>

	{#if info}
		<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">
			{info}
		</p>
	{/if}
	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">
			{error}
		</p>
	{/if}

	<div class="card overflow-hidden">
		<table class="w-full">
			<thead class="bg-slate-50">
				<tr>
					<th class="th">Nama Titik</th>
					<th class="th">Jenis</th>
					<th class="th">Koordinat</th>
					<th class="th text-center">Radius</th>
					<th class="th">Status</th>
					<th class="th no-print">Aksi</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-slate-100">
				{#each dataHalaman as row}
					<tr class="hover:bg-slate-50/70">
						<td class="td font-medium text-slate-800">{row.nama}</td>
						<td class="td capitalize text-slate-600">{row.jenis}</td>
						<td class="td font-mono text-xs text-slate-600">
							{row.latitude}, {row.longitude}
						</td>
						<td class="td text-center text-slate-600">{row.radius_m} m</td>
						<td class="td">
							<Badge tone={row.is_active ? 'green' : 'slate'}>
								{row.is_active ? 'Aktif' : 'Nonaktif'}
							</Badge>
						</td>
						<td class="td no-print">
							<div class="flex flex-wrap gap-1">
								<button
									class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
									onclick={() => bukaEdit(row)}
								>
									Edit
								</button>
								<button class="btn btn-ghost px-2 py-1 text-xs" onclick={() => nonaktifkan(row)}>
									{row.is_active ? 'Nonaktifkan' : 'Aktifkan'}
								</button>
								<button
									class="btn btn-ghost px-2 py-1 text-xs text-rose-600"
									onclick={() => hapus(row)}
								>
									Hapus
								</button>
							</div>
						</td>
					</tr>
				{:else}
					<tr>
						<td class="td text-center text-slate-500" colspan="6">
							{memuat ? 'Memuat…' : 'Belum ada titik dinas.'}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
		<Pagination
			page={halamanAman}
			perPage={perPage}
			total={data.length}
			label="titik dinas"
			onchange={(p) => (halaman = p)}
		/>
	</div>
</div>

<Modal
	open={formTerbuka}
	title={form.id ? 'Edit Titik Dinas' : 'Tambah Titik Dinas'}
	lebar="max-w-2xl"
	onclose={() => (formTerbuka = false)}
>
	<form
		class="space-y-3"
		onsubmit={(e) => {
			e.preventDefault();
			void simpan();
		}}
	>
		<div>
			<label class="label" for="fsatker">Satker</label>
			<select id="fsatker" class="input" bind:value={form.satker_id} disabled={Boolean(form.id)}>
				{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
			{#if form.id}
				<p class="mt-1 text-[11px] text-slate-500">Satker tidak dapat diubah saat edit.</p>
			{/if}
		</div>
		<div>
			<label class="label" for="fnama">Nama Titik</label>
			<input
				id="fnama"
				class="input"
				placeholder="Markas Polresta Denpasar"
				bind:value={form.nama}
			/>
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label class="label" for="fjenis">Jenis</label>
				<select id="fjenis" class="input" bind:value={form.jenis}>
					<option value="kantor">Kantor</option>
					<option value="luar">Luar</option>
				</select>
			</div>
			<div>
				<label class="label" for="fradius">Radius (meter)</label>
				<input id="fradius" class="input" inputmode="numeric" bind:value={form.radius} />
			</div>
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label class="label" for="flat">Latitude</label>
				<input
					id="flat"
					class="input"
					inputmode="decimal"
					placeholder="-8.65629"
					bind:value={form.latitude}
				/>
			</div>
			<div>
				<label class="label" for="flng">Longitude</label>
				<input
					id="flng"
					class="input"
					inputmode="decimal"
					placeholder="115.22144"
					bind:value={form.longitude}
				/>
			</div>
		</div>

		<PetaPicker
			latitude={latPeta}
			longitude={lngPeta}
			radius={radiusPeta}
			onchange={pilihDariPeta}
		/>

		{#if form.id}
			<label class="flex items-center gap-2 text-sm text-slate-700">
				<input type="checkbox" bind:checked={form.is_active} /> Aktif
			</label>
		{/if}

		<div class="flex justify-end gap-2 pt-2">
			<button type="button" class="btn btn-ghost" onclick={() => (formTerbuka = false)}>
				Batal
			</button>
			<button type="submit" class="btn btn-primary">
				<Icon name="check" size={16} /> Simpan
			</button>
		</div>
	</form>
</Modal>
