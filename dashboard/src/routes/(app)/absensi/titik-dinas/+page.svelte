<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import type { Satker, TitikDinas } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	let satkerList = $state<Satker[]>([]);
	let data = $state<TitikDinas[]>([]);
	let satkerId = $state('');
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let formTerbuka = $state(false);

	let nama = $state('');
	let jenis = $state<'kantor' | 'luar'>('kantor');
	let latitude = $state('');
	let longitude = $state('');
	let radius = $state('100');
	let formSatker = $state('');
	let halaman = $state(1);
	const perPage = 10;

	const totalHalaman = $derived(Math.max(1, Math.ceil(data.length / perPage)));
	const halamanAman = $derived(Math.min(halaman, totalHalaman));
	const dataHalaman = $derived(data.slice((halamanAman - 1) * perPage, halamanAman * perPage));

	async function muat() {
		memuat = true;
		error = null;
		halaman = 1;
		try {
			const params = new URLSearchParams();
			if (satkerId) params.set('satker_id', satkerId);
			data = await api<TitikDinas[]>(`/titik-dinas?${params}`);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat titik dinas';
		} finally {
			memuat = false;
		}
	}

	async function simpan() {
		error = null;
		try {
			await api('/titik-dinas', {
				method: 'POST',
				body: {
					satker_id: formSatker,
					nama,
					jenis,
					latitude: Number(latitude),
					longitude: Number(longitude),
					radius_m: Number(radius)
				}
			});
			formTerbuka = false;
			nama = '';
			latitude = '';
			longitude = '';
			radius = '100';
			await muat();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan titik dinas';
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
			error = err instanceof ApiError ? err.message : 'Gagal memperbarui';
		}
	}

	async function hapus(row: TitikDinas) {
		if (!confirm(`Hapus titik dinas "${row.nama}"?`)) return;
		try {
			await api(`/titik-dinas/${row.id}`, { method: 'DELETE' });
			await muat();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menghapus';
		}
	}

	onMount(async () => {
		satkerList = await api<Satker[]>('/satker');
		formSatker = satkerList[0]?.id ?? '';
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
		<button class="btn btn-primary" onclick={() => (formTerbuka = true)}>
			<Icon name="plus" size={16} /> Tambah Titik
		</button>
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
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
						<td class="td text-xs text-slate-600">{row.latitude}, {row.longitude}</td>
						<td class="td text-center text-slate-600">{row.radius_m} m</td>
						<td class="td">
							<Badge tone={row.is_active ? 'green' : 'slate'}>{row.is_active ? 'Aktif' : 'Nonaktif'}</Badge>
						</td>
						<td class="td no-print">
							<div class="flex gap-1">
								<button class="btn btn-ghost px-2 py-1 text-xs" onclick={() => nonaktifkan(row)}>
									{row.is_active ? 'Nonaktifkan' : 'Aktifkan'}
								</button>
								<button class="btn btn-ghost px-2 py-1 text-xs text-rose-600" onclick={() => hapus(row)}>
									Hapus
								</button>
							</div>
						</td>
					</tr>
				{:else}
					<tr><td class="td text-center text-slate-500" colspan="6">{memuat ? 'Memuat…' : 'Belum ada titik dinas.'}</td></tr>
				{/each}
			</tbody>
		</table>
		<Pagination page={halamanAman} perPage={perPage} total={data.length} label="titik dinas" onchange={(p) => (halaman = p)} />
	</div>
</div>

<Modal open={formTerbuka} title="Tambah Titik Dinas" onclose={() => (formTerbuka = false)}>
	<div class="space-y-3">
		<div>
			<label class="label" for="fsatker">Satker</label>
			<select id="fsatker" class="input" bind:value={formSatker}>
				{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<div>
			<label class="label" for="fnama">Nama Titik</label>
			<input id="fnama" class="input" placeholder="Markas Polresta Denpasar" bind:value={nama} />
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label class="label" for="flat">Latitude</label>
				<input id="flat" class="input" placeholder="-8.65629" bind:value={latitude} />
			</div>
			<div>
				<label class="label" for="flng">Longitude</label>
				<input id="flng" class="input" placeholder="115.22144" bind:value={longitude} />
			</div>
		</div>
		<div class="grid grid-cols-2 gap-3">
			<div>
				<label class="label" for="fjenis">Jenis</label>
				<select id="fjenis" class="input" bind:value={jenis}>
					<option value="kantor">Kantor</option>
					<option value="luar">Luar</option>
				</select>
			</div>
			<div>
				<label class="label" for="fradius">Radius (meter)</label>
				<input id="fradius" class="input" bind:value={radius} />
			</div>
		</div>
		<div class="flex justify-end gap-2 pt-2">
			<button class="btn btn-ghost" onclick={() => (formTerbuka = false)}>Batal</button>
			<button class="btn btn-primary" onclick={simpan}><Icon name="check" size={16} /> Simpan</button>
		</div>
	</div>
</Modal>
