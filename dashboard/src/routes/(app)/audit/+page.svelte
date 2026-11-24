<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api, ApiError } from '#lib/api/client';
	import { punyaRole } from '#lib/stores/session.svelte';
	import type { AuditLog, ListResponse, Personel } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Combobox from '#lib/components/Combobox.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	const ENTITAS: { value: string; label: string }[] = [
		{ value: '', label: 'Semua entitas' },
		{ value: 'auth', label: 'Autentikasi' },
		{ value: 'admin', label: 'Akun Admin' },
		{ value: 'personel', label: 'Personel' },
		{ value: 'satker', label: 'Satker' },
		{ value: 'absensi', label: 'Absensi' },
		{ value: 'rencana-kerja', label: 'Rencana Kerja' },
		{ value: 'hasil-kerja', label: 'Hasil Kerja' },
		{ value: 'output', label: 'Output' },
		{ value: 'ph', label: 'Strong Point' },
		{ value: 'libur', label: 'Hari Libur' },
		{ value: 'titik-dinas', label: 'Titik Dinas' },
		{ value: 'penilaian', label: 'Penilaian' },
		{ value: 'periode', label: 'Periode' },
		{ value: 'kesiapan', label: 'Kesiapan' },
		{ value: 'notifikasi', label: 'Notifikasi' },
		{ value: 'files', label: 'Berkas' }
	];

	const perPage = 20;

	function tanggalHariIni() {
		const bagian = new Intl.DateTimeFormat('en-CA', {
			timeZone: 'Asia/Makassar',
			year: 'numeric',
			month: '2-digit',
			day: '2-digit'
		}).formatToParts(new Date());
		const ambil = (jenis: string) => bagian.find((item) => item.type === jenis)?.value ?? '';
		return `${ambil('year')}-${ambil('month')}-${ambil('day')}`;
	}

	function awalRentang() {
		const awal = new Date(`${tanggalHariIni()}T00:00:00Z`);
		awal.setUTCDate(awal.getUTCDate() - 6);
		return awal.toISOString().slice(0, 10);
	}

	let dari = $state(awalRentang());
	let sampai = $state(tanggalHariIni());
	let entitas = $state('');
	let actorId = $state('');
	let page = $state(1);
	let total = $state(0);
	let data = $state<AuditLog[]>([]);
	let personelList = $state<Personel[]>([]);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let pilih = $state<AuditLog | null>(null);

	const opsiPelaku = $derived(
		personelList.map((orang) => ({
			value: orang.id,
			label: orang.nama,
			keterangan: orang.nrp
		}))
	);

	function pesan(err: unknown) {
		return err instanceof ApiError ? err.message : 'Gagal memuat audit log';
	}

	async function muat(halaman = page) {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams({
				dari,
				sampai,
				page: String(halaman),
				per_page: String(perPage)
			});
			if (entitas) params.set('entitas', entitas);
			if (actorId) params.set('actor_id', actorId);
			const hasil = await api<ListResponse<AuditLog>>(`/audit?${params}`);
			data = hasil.data;
			total = hasil.meta.total;
			page = hasil.meta.page;
		} catch (err) {
			error = pesan(err);
		} finally {
			memuat = false;
		}
	}

	function terapkanFilter() {
		page = 1;
		void muat(1);
	}

	async function muatPersonel() {
		try {
			const hasil = await api<ListResponse<Personel>>('/personel?per_page=200');
			personelList = hasil.data;
		} catch {
			// Filter pelaku opsional; audit tetap bisa dibuka tanpa daftar personel.
		}
	}

	onMount(() => {
		if (!punyaRole('admin_sistem', 'admin_polda')) {
			void goto('/dashboard', { replaceState: true });
			return;
		}
		void Promise.all([muat(1), muatPersonel()]);
	});

	function waktuAudit(nilai: string) {
		return `${new Date(nilai).toLocaleString('id-ID', {
			timeZone: 'Asia/Makassar',
			day: '2-digit',
			month: 'short',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit'
		})} WITA`;
	}

	function metodeAksi(aksi: string) {
		return aksi.split(' ')[0] ?? aksi;
	}

	function jalurAksi(aksi: string) {
		const pisah = aksi.indexOf(' ');
		return pisah > 0 ? aksi.slice(pisah + 1) : '';
	}

	function toneMetode(metode: string): 'green' | 'blue' | 'amber' | 'red' | 'slate' {
		if (metode === 'POST') return 'green';
		if (metode === 'PUT') return 'blue';
		if (metode === 'PATCH') return 'amber';
		if (metode === 'DELETE') return 'red';
		return 'slate';
	}

	function labelEntitas(nilai: string) {
		return ENTITAS.find((item) => item.value === nilai)?.label ?? nilai;
	}

	function statusAudit(row: AuditLog): number | null {
		if (!row.after_data || typeof row.after_data !== 'object') return null;
		const status = (row.after_data as { status?: unknown }).status;
		return typeof status === 'number' ? status : null;
	}

	function toneStatus(status: number): 'green' | 'blue' | 'red' {
		if (status < 300) return 'green';
		if (status < 400) return 'blue';
		return 'red';
	}

	function jsonTeks(nilai: unknown) {
		if (nilai === null || nilai === undefined) return '—';
		try {
			return JSON.stringify(nilai, null, 2);
		} catch {
			return String(nilai);
		}
	}
</script>

<svelte:head><title>Audit Log — Absensi & Strong Point</title></svelte:head>

<div class="space-y-4">
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div>
			<h1 class="text-xl font-semibold text-slate-800">Audit Log</h1>
			<p class="mt-1 text-sm text-slate-500">
				Jejak semua permintaan yang mengubah data (POST/PUT/PATCH/DELETE), termasuk pelaku, IP,
				dan status respons. Hanya admin global dan admin Polda yang dapat membuka halaman ini.
			</p>
		</div>
		<span class="text-xs text-slate-500">{total} catatan</span>
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	<div class="card flex flex-wrap items-end gap-3 p-4">
		<div>
			<label class="label" for="audit-dari">Dari Tanggal</label>
			<input
				id="audit-dari"
				type="date"
				class="input"
				bind:value={dari}
				onchange={terapkanFilter}
			/>
		</div>
		<div>
			<label class="label" for="audit-sampai">Sampai Tanggal</label>
			<input
				id="audit-sampai"
				type="date"
				class="input"
				bind:value={sampai}
				onchange={terapkanFilter}
			/>
		</div>
		<div class="min-w-52">
			<label class="label" for="audit-entitas">Entitas</label>
			<select id="audit-entitas" class="input" bind:value={entitas} onchange={terapkanFilter}>
				{#each ENTITAS as item (item.value)}
					<option value={item.value}>{item.label}</option>
				{/each}
			</select>
		</div>
		<div class="min-w-72">
			<label class="label" for="audit-pelaku">Pelaku</label>
			<Combobox
				id="audit-pelaku"
				bind:value={actorId}
				options={opsiPelaku}
				placeholder="Semua Pelaku"
				labelKosong="Semua Pelaku"
				onchange={terapkanFilter}
			/>
		</div>
		<button type="button" class="btn btn-ghost border border-slate-300" onclick={() => muat(page)}>
			<Icon name="refresh" size={16} />
			Muat ulang
		</button>
	</div>

	<div class="card overflow-hidden">
		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">Waktu</th>
						<th class="th">Pelaku</th>
						<th class="th">Aksi</th>
						<th class="th">Entitas</th>
						<th class="th text-center">Status</th>
						<th class="th">IP</th>
						<th class="th text-right">Detail</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each data as row (row.id)}
						{@const metode = metodeAksi(row.aksi)}
						{@const status = statusAudit(row)}
						<tr class="hover:bg-slate-50/70">
							<td class="td whitespace-nowrap text-slate-600">{waktuAudit(row.created_at)}</td>
							<td class="td">
								<p class="font-medium text-slate-800">{row.actor_nama ?? 'Tanpa token'}</p>
								{#if row.actor_id}
									<p class="font-mono text-[10px] text-slate-400">{row.actor_id}</p>
								{/if}
							</td>
							<td class="td">
								<div class="flex flex-wrap items-center gap-2">
									<Badge tone={toneMetode(metode)}>{metode}</Badge>
									<span class="font-mono text-[11px] text-slate-600">{jalurAksi(row.aksi)}</span>
								</div>
							</td>
							<td class="td">
								<p class="font-medium text-slate-700">{labelEntitas(row.entitas)}</p>
								{#if row.entitas_id}
									<p class="font-mono text-[10px] text-slate-400">{row.entitas_id}</p>
								{/if}
							</td>
							<td class="td text-center">
								{#if status !== null}
									<Badge tone={toneStatus(status)}>{status}</Badge>
								{:else}
									<span class="text-slate-300">—</span>
								{/if}
							</td>
							<td class="td font-mono text-[11px] text-slate-500">{row.ip ?? '—'}</td>
							<td class="td text-right">
								<button
									type="button"
									class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
									onclick={() => (pilih = row)}
								>
									Lihat
								</button>
							</td>
						</tr>
					{:else}
						<tr>
							<td class="td text-center text-slate-500" colspan="7">
								{memuat ? 'Memuat…' : 'Belum ada catatan audit pada filter ini.'}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<Pagination page={page} perPage={perPage} total={total} label="catatan" onchange={muat} />
	</div>
</div>

<Modal open={!!pilih} title="Detail Audit" lebar="max-w-3xl" onclose={() => (pilih = null)}>
	{#if pilih}
		{@const status = statusAudit(pilih)}
		<div class="space-y-3 text-sm">
			<div class="grid gap-3 sm:grid-cols-2">
				<div>
					<p class="text-[11px] font-medium tracking-wide text-slate-500 uppercase">Waktu</p>
					<p class="text-slate-800">{waktuAudit(pilih.created_at)}</p>
				</div>
				<div>
					<p class="text-[11px] font-medium tracking-wide text-slate-500 uppercase">Pelaku</p>
					<p class="text-slate-800">{pilih.actor_nama ?? 'Tanpa token'}</p>
					{#if pilih.actor_id}
						<p class="font-mono text-[10px] text-slate-400">{pilih.actor_id}</p>
					{/if}
				</div>
				<div>
					<p class="text-[11px] font-medium tracking-wide text-slate-500 uppercase">Aksi</p>
					<p class="font-mono text-slate-800">{pilih.aksi}</p>
				</div>
				<div>
					<p class="text-[11px] font-medium tracking-wide text-slate-500 uppercase">Entitas</p>
					<p class="text-slate-800">{labelEntitas(pilih.entitas)}</p>
					{#if pilih.entitas_id}
						<p class="font-mono text-[10px] text-slate-400">{pilih.entitas_id}</p>
					{/if}
				</div>
				<div>
					<p class="text-[11px] font-medium tracking-wide text-slate-500 uppercase">IP</p>
					<p class="font-mono text-slate-800">{pilih.ip ?? '—'}</p>
				</div>
				<div>
					<p class="text-[11px] font-medium tracking-wide text-slate-500 uppercase">Status</p>
					{#if status !== null}
						<Badge tone={toneStatus(status)}>{status}</Badge>
					{:else}
						<span class="text-slate-400">—</span>
					{/if}
				</div>
			</div>
			<div>
				<p class="text-[11px] font-medium tracking-wide text-slate-500 uppercase">User agent</p>
				<p class="break-all text-[11px] text-slate-600">{pilih.user_agent ?? '—'}</p>
			</div>
			<div class="grid gap-3 sm:grid-cols-2">
				<div>
					<p class="mb-1 text-[11px] font-medium tracking-wide text-slate-500 uppercase">
						Sebelum
					</p>
					<pre
						class="max-h-64 overflow-auto rounded-lg bg-slate-50 p-3 text-[11px] text-slate-700 ring-1 ring-slate-200">{jsonTeks(
							pilih.before_data
						)}</pre>
				</div>
				<div>
					<p class="mb-1 text-[11px] font-medium tracking-wide text-slate-500 uppercase">
						Sesudah
					</p>
					<pre
						class="max-h-64 overflow-auto rounded-lg bg-slate-50 p-3 text-[11px] text-slate-700 ring-1 ring-slate-200">{jsonTeks(
							pilih.after_data
						)}</pre>
				</div>
			</div>
		</div>
	{/if}
</Modal>
