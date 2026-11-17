<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import { adalahAdmin } from '#lib/stores/session.svelte';
	import type {
		AbsensiPh,
		JadwalPh,
		ListResponse,
		Personel,
		RekapPh,
		Satker,
		StrongPoint
	} from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Pagination from '#lib/components/Pagination.svelte';
	import Modal from '#lib/components/Modal.svelte';

	type Tab = 'jadwal' | 'strong' | 'rekap';
	type FormJadwal = {
		id: string | null;
		satker_id: string;
		strong_point_id: string;
		tanggal: string;
		jam_mulai: string;
		jam_selesai: string;
		catatan: string;
		personel_ids: string[];
	};
	type FormStrong = {
		id: string | null;
		satker_id: string;
		nama: string;
		alamat: string;
		latitude: string;
		longitude: string;
		radius_m: string;
		is_active: boolean;
	};

	const sekarang = new Date();
	let bulan = $state(
		`${sekarang.getFullYear()}-${String(sekarang.getMonth() + 1).padStart(2, '0')}`
	);
	let tab = $state<Tab>('jadwal');
	let satkerId = $state('');
	let satkerList = $state<Satker[]>([]);
	let strongPoints = $state<StrongPoint[]>([]);
	let jadwal = $state<JadwalPh[]>([]);
	let rekap = $state<RekapPh[]>([]);
	let personelSatker = $state<Personel[]>([]);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);
	let halamanJadwal = $state(1);
	let halamanStrong = $state(1);
	let halamanRekap = $state(1);
	const perPage = 10;

	let formJadwalTerbuka = $state(false);
	let formStrongTerbuka = $state(false);
	let detail = $state<JadwalPh | null>(null);
	let absensiDetail = $state<AbsensiPh[]>([]);

	let formJadwal = $state<FormJadwal>(kosongJadwal());
	let formStrong = $state<FormStrong>(kosongStrong());

	const admin = $derived(adalahAdmin());
	const satkerNodes = $derived(satkerList.filter((s) => s.level === 'satker' && s.is_active));
	const strongTerfilter = $derived(
		strongPoints.filter((s) => !satkerId || s.satker_id === satkerId)
	);
	const personelPilihan = $derived(
		personelSatker.filter((p) => p.is_active)
	);

	const totalHalamanJadwal = $derived(Math.max(1, Math.ceil(jadwal.length / perPage)));
	const halamanJadwalAman = $derived(Math.min(halamanJadwal, totalHalamanJadwal));
	const jadwalHalaman = $derived(
		jadwal.slice((halamanJadwalAman - 1) * perPage, halamanJadwalAman * perPage)
	);
	const totalHalamanStrong = $derived(
		Math.max(1, Math.ceil(strongTerfilter.length / perPage))
	);
	const halamanStrongAman = $derived(Math.min(halamanStrong, totalHalamanStrong));
	const strongHalaman = $derived(
		strongTerfilter.slice((halamanStrongAman - 1) * perPage, halamanStrongAman * perPage)
	);
	const totalHalamanRekap = $derived(Math.max(1, Math.ceil(rekap.length / perPage)));
	const halamanRekapAman = $derived(Math.min(halamanRekap, totalHalamanRekap));
	const rekapHalaman = $derived(
		rekap.slice((halamanRekapAman - 1) * perPage, halamanRekapAman * perPage)
	);

	function hariIni(): string {
		return new Date().toISOString().slice(0, 10);
	}

	function kosongJadwal(): FormJadwal {
		return {
			id: null,
			satker_id: '',
			strong_point_id: '',
			tanggal: hariIni(),
			jam_mulai: '08:00',
			jam_selesai: '16:00',
			catatan: '',
			personel_ids: []
		};
	}

	function kosongStrong(): FormStrong {
		return {
			id: null,
			satker_id: '',
			nama: '',
			alamat: '',
			latitude: '',
			longitude: '',
			radius_m: '100',
			is_active: true
		};
	}

	function namaSatker(id: string) {
		return satkerList.find((s) => s.id === id)?.nama ?? '-';
	}

	async function muatAwal() {
		try {
			satkerList = await api<Satker[]>('/satker');
			satkerId = satkerId || satkerNodes[0]?.id || '';
			await Promise.all([muatStrong(), muatJadwal(), muatRekap()]);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat data PH';
		} finally {
			memuat = false;
		}
	}

	async function muatStrong() {
		strongPoints = await api<StrongPoint[]>('/ph/strong-point');
	}

	async function muatJadwal() {
		const params = new URLSearchParams({ bulan });
		if (satkerId) params.set('satker_id', satkerId);
		jadwal = await api<JadwalPh[]>(`/ph/jadwal?${params}`);
	}

	async function muatRekap() {
		const params = new URLSearchParams({ bulan });
		if (satkerId) params.set('satker_id', satkerId);
		const hasil = await api<{ data: RekapPh[] }>(`/ph/rekap?${params}`);
		rekap = hasil.data;
	}

	async function muatOrang(satker: string) {
		if (!satker) {
			personelSatker = [];
			return;
		}
		const hasil = await api<ListResponse<Personel>>(
			`/personel?satker_id=${satker}&per_page=200`
		);
		personelSatker = hasil.data;
	}

	async function muatSemua() {
		memuat = true;
		error = null;
		halamanJadwal = 1;
		halamanStrong = 1;
		halamanRekap = 1;
		try {
			await Promise.all([muatStrong(), muatJadwal(), muatRekap()]);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat data PH';
		} finally {
			memuat = false;
		}
	}

	function bukaTambahJadwal() {
		formJadwal = kosongJadwal();
		formJadwal.satker_id = satkerId || satkerNodes[0]?.id || '';
		formJadwal.strong_point_id =
			strongPoints.find((s) => s.satker_id === formJadwal.satker_id)?.id ?? '';
		formJadwalTerbuka = true;
		muatOrang(formJadwal.satker_id);
	}

	function gantiSatkerForm(satker: string) {
		formJadwal.satker_id = satker;
		formJadwal.strong_point_id =
			strongPoints.find((s) => s.satker_id === satker)?.id ?? '';
		formJadwal.personel_ids = [];
		muatOrang(satker);
	}

	function bukaEditJadwal(row: JadwalPh) {
		formJadwal = {
			id: row.id,
			satker_id: row.satker_id,
			strong_point_id: row.strong_point_id,
			tanggal: row.tanggal,
			jam_mulai: row.jam_mulai,
			jam_selesai: row.jam_selesai,
			catatan: row.catatan ?? '',
			personel_ids: row.personel.map((p) => p.personel_id)
		};
		formJadwalTerbuka = true;
		muatOrang(row.satker_id);
	}

	function togglePersonel(id: string) {
		formJadwal.personel_ids = formJadwal.personel_ids.includes(id)
			? formJadwal.personel_ids.filter((x) => x !== id)
			: [...formJadwal.personel_ids, id];
	}

	async function simpanJadwal() {
		error = null;
		info = null;
		try {
			const body = {
				satker_id: formJadwal.satker_id,
				strong_point_id: formJadwal.strong_point_id,
				tanggal: formJadwal.tanggal,
				jam_mulai: formJadwal.jam_mulai,
				jam_selesai: formJadwal.jam_selesai,
				catatan: formJadwal.catatan || null,
				personel: formJadwal.personel_ids.map((personel_id) => ({ personel_id }))
			};
			if (formJadwal.id) {
				await api(`/ph/jadwal/${formJadwal.id}`, { method: 'PUT', body });
			} else {
				await api('/ph/jadwal', { method: 'POST', body });
			}
			formJadwalTerbuka = false;
			info = formJadwal.id ? 'Jadwal PH diperbarui.' : 'Jadwal PH dibuat.';
			await muatSemua();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan jadwal PH';
		}
	}

	async function hapusJadwal(row: JadwalPh) {
		if (!confirm(`Hapus jadwal PH ${row.tanggal} di ${row.strong_point_nama}?`)) return;
		try {
			await api(`/ph/jadwal/${row.id}`, { method: 'DELETE' });
			await muatSemua();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menghapus jadwal';
		}
	}

	async function bukaDetail(row: JadwalPh) {
		detail = row;
		try {
			absensiDetail = await api<AbsensiPh[]>(`/ph/absensi/jadwal/${row.id}`);
		} catch {
			absensiDetail = [];
		}
	}

	function bukaTambahStrong() {
		formStrong = kosongStrong();
		formStrong.satker_id = satkerId || satkerNodes[0]?.id || '';
		formStrongTerbuka = true;
	}

	function bukaEditStrong(row: StrongPoint) {
		formStrong = {
			id: row.id,
			satker_id: row.satker_id,
			nama: row.nama,
			alamat: row.alamat ?? '',
			latitude: String(row.latitude),
			longitude: String(row.longitude),
			radius_m: String(row.radius_m),
			is_active: row.is_active
		};
		formStrongTerbuka = true;
	}

	async function simpanStrong() {
		error = null;
		info = null;
		try {
			const body = {
				satker_id: formStrong.satker_id,
				nama: formStrong.nama,
				alamat: formStrong.alamat || null,
				latitude: Number(formStrong.latitude),
				longitude: Number(formStrong.longitude),
				radius_m: Number(formStrong.radius_m),
				is_active: formStrong.is_active
			};
			if (formStrong.id) {
				await api(`/ph/strong-point/${formStrong.id}`, { method: 'PATCH', body });
			} else {
				await api('/ph/strong-point', { method: 'POST', body });
			}
			formStrongTerbuka = false;
			info = formStrong.id ? 'Strong point diperbarui.' : 'Strong point dibuat.';
			await muatSemua();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan strong point';
		}
	}

	async function hapusStrong(row: StrongPoint) {
		if (!confirm(`Hapus strong point "${row.nama}"?`)) return;
		try {
			await api(`/ph/strong-point/${row.id}`, { method: 'DELETE' });
			await muatSemua();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menghapus strong point';
		}
	}

	function gantiFilterSatker() {
		muatSemua();
	}

	onMount(muatAwal);
</script>

<svelte:head><title>PH & Strong Point — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-end justify-between gap-3 p-4">
		<div>
			<h2 class="text-sm font-semibold text-slate-800">PH & Strong Point</h2>
			<p class="mt-1 text-xs text-slate-500">
				Jadwal penjagaan terpisah dari absensi harian. Check-in dan check-out divalidasi radius strong point.
			</p>
		</div>
		{#if admin}
			<div class="flex gap-2">
				<button class="btn btn-primary" onclick={bukaTambahJadwal}><Icon name="plus" size={16} /> Jadwal PH</button>
				<button class="btn btn-ghost border border-slate-300" onclick={bukaTambahStrong}><Icon name="pin" size={16} /> Strong Point</button>
			</div>
		{/if}
	</div>

	<div class="flex flex-wrap items-end gap-3">
		<div>
			<label class="label" for="bulan">Bulan</label>
			<input id="bulan" type="month" class="input" bind:value={bulan} onchange={muatSemua} />
		</div>
		<div class="min-w-64">
			<label class="label" for="satker">Satker</label>
			<select id="satker" class="input" bind:value={satkerId} onchange={gantiFilterSatker}>
				<option value="">Semua Satker</option>
				{#each satkerNodes as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<div class="flex gap-2">
			{#each [['jadwal', 'Jadwal PH'], ['strong', 'Strong Point'], ['rekap', 'Rekap PH']] as [nilai, label]}
				<button class="btn {tab === nilai ? 'btn-primary' : 'btn-ghost border border-slate-300'}" onclick={() => (tab = nilai as Tab)}>{label}</button>
			{/each}
		</div>
	</div>

	{#if info}<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>{/if}
	{#if error}<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>{/if}

	{#if tab === 'jadwal'}
		<div class="card overflow-hidden">
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Tanggal</th>
							<th class="th">Jam</th>
							<th class="th">Strong Point</th>
							<th class="th">Satker</th>
							<th class="th text-center">Personel</th>
							<th class="th">Catatan</th>
							<th class="th no-print">Aksi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each jadwalHalaman as row}
							<tr class="hover:bg-slate-50/70">
								<td class="td text-slate-700">{new Date(row.tanggal + 'T00:00:00').toLocaleDateString('id-ID', { weekday: 'short', day: 'numeric', month: 'short' })}</td>
								<td class="td text-slate-600">{row.jam_mulai}–{row.jam_selesai}</td>
								<td class="td font-medium text-slate-800">{row.strong_point_nama}</td>
								<td class="td text-slate-600">{namaSatker(row.satker_id)}</td>
								<td class="td text-center"><Badge tone="blue">{row.jumlah_personel}</Badge></td>
								<td class="td text-slate-600">{row.catatan ?? '-'}</td>
								<td class="td no-print">
									<div class="flex flex-wrap gap-1">
										<button class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs" onclick={() => bukaDetail(row)}>Detail</button>
										{#if admin}
											<button class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs" onclick={() => bukaEditJadwal(row)}>Edit</button>
											<button class="btn btn-ghost px-2 py-1 text-xs text-rose-600" onclick={() => hapusJadwal(row)}>Hapus</button>
										{/if}
									</div>
								</td>
							</tr>
						{:else}
							<tr><td class="td text-center text-slate-500" colspan="7">{memuat ? 'Memuat…' : 'Belum ada jadwal PH.'}</td></tr>
						{/each}
					</tbody>
				</table>
			</div>
			<Pagination page={halamanJadwalAman} perPage={perPage} total={jadwal.length} label="jadwal" onchange={(p) => (halamanJadwal = p)} />
		</div>
	{:else if tab === 'strong'}
		<div class="card overflow-hidden">
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Nama</th>
							<th class="th">Satker</th>
							<th class="th">Koordinat</th>
							<th class="th text-center">Radius</th>
							<th class="th">Status</th>
							<th class="th no-print">Aksi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each strongHalaman as row}
							<tr>
								<td class="td font-medium text-slate-800">{row.nama}</td>
								<td class="td text-slate-600">{namaSatker(row.satker_id)}</td>
								<td class="td text-xs text-slate-600">{row.latitude}, {row.longitude}</td>
								<td class="td text-center text-slate-600">{row.radius_m} m</td>
								<td class="td"><Badge tone={row.is_active ? 'green' : 'slate'}>{row.is_active ? 'Aktif' : 'Nonaktif'}</Badge></td>
								<td class="td no-print">
									{#if admin}
										<div class="flex gap-1">
											<button class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs" onclick={() => bukaEditStrong(row)}>Edit</button>
											<button class="btn btn-ghost px-2 py-1 text-xs text-rose-600" onclick={() => hapusStrong(row)}>Hapus</button>
										</div>
									{/if}
								</td>
							</tr>
						{:else}
							<tr><td class="td text-center text-slate-500" colspan="6">Belum ada strong point.</td></tr>
						{/each}
					</tbody>
				</table>
			</div>
			<Pagination page={halamanStrongAman} perPage={perPage} total={strongTerfilter.length} label="strong point" onchange={(p) => (halamanStrong = p)} />
		</div>
	{:else}
		<div class="card overflow-hidden">
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Personel</th>
							<th class="th">Unit</th>
							<th class="th text-center">Jadwal</th>
							<th class="th text-center">Hadir</th>
							<th class="th text-center">Telat</th>
							<th class="th text-center">Tidak Hadir</th>
							<th class="th text-center">Menunggu</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each rekapHalaman as row}
							<tr>
								<td class="td">
									<p class="font-medium text-slate-800">{row.nama}</p>
									<p class="text-[11px] text-slate-500">{row.pangkat} · {row.nrp}</p>
								</td>
								<td class="td text-slate-600">{row.unit ?? '-'}</td>
								<td class="td text-center text-slate-600">{row.jumlah_jadwal}</td>
								<td class="td text-center font-semibold text-emerald-700">{row.hadir}</td>
								<td class="td text-center {row.telat > 0 ? 'text-amber-700' : 'text-slate-400'}">{row.telat}</td>
								<td class="td text-center {row.tidak_hadir > 0 ? 'font-semibold text-rose-700' : 'text-slate-400'}">{row.tidak_hadir}</td>
								<td class="td text-center">
									{#if row.menunggu_verifikasi > 0}<Badge tone="amber">{row.menunggu_verifikasi}</Badge>{:else}<span class="text-slate-300">-</span>{/if}
								</td>
							</tr>
						{:else}
							<tr><td class="td text-center text-slate-500" colspan="7">{memuat ? 'Memuat…' : 'Belum ada rekap PH.'}</td></tr>
						{/each}
					</tbody>
				</table>
			</div>
			<Pagination page={halamanRekapAman} perPage={perPage} total={rekap.length} label="rekap PH" onchange={(p) => (halamanRekap = p)} />
		</div>
	{/if}
</div>

<Modal open={formJadwalTerbuka} title={formJadwal.id ? 'Edit Jadwal PH' : 'Tambah Jadwal PH'} lebar="max-w-2xl" onclose={() => (formJadwalTerbuka = false)}>
	<div class="grid gap-3 sm:grid-cols-2">
		<div class="sm:col-span-2">
			<label class="label" for="jsatker">Satker</label>
			<select id="jsatker" class="input" value={formJadwal.satker_id} disabled={Boolean(formJadwal.id)} onchange={(e) => gantiSatkerForm((e.target as HTMLSelectElement).value)}>
				{#each satkerNodes as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="jsp">Strong Point</label>
			<select id="jsp" class="input" bind:value={formJadwal.strong_point_id}>
				{#each strongPoints.filter((s) => s.satker_id === formJadwal.satker_id) as s}
					<option value={s.id}>{s.nama} · {s.radius_m} m</option>
				{/each}
			</select>
		</div>
		<div><label class="label" for="jtgl">Tanggal</label><input id="jtgl" type="date" class="input" bind:value={formJadwal.tanggal} /></div>
		<div class="grid grid-cols-2 gap-2">
			<div><label class="label" for="jmasuk">Mulai</label><input id="jmasuk" type="time" class="input" bind:value={formJadwal.jam_mulai} /></div>
			<div><label class="label" for="jpulang">Selesai</label><input id="jpulang" type="time" class="input" bind:value={formJadwal.jam_selesai} /></div>
		</div>
		<div class="sm:col-span-2"><label class="label" for="jcat">Catatan</label><input id="jcat" class="input" bind:value={formJadwal.catatan} /></div>
		<div class="sm:col-span-2">
			<span class="label">Personel Ditugaskan</span>
			<div class="max-h-48 overflow-auto rounded-lg border border-slate-200 p-2">
				{#each personelPilihan as p}
					<label class="flex items-center gap-2 py-1 text-xs text-slate-700">
						<input type="checkbox" checked={formJadwal.personel_ids.includes(p.id)} onchange={() => togglePersonel(p.id)} />
						{p.nama} · {p.nrp} {p.unit ? `· ${p.unit}` : ''}
					</label>
				{:else}
					<p class="text-xs text-slate-500">Pilih satker terlebih dahulu.</p>
				{/each}
			</div>
		</div>
	</div>
	<div class="mt-4 flex justify-end gap-2">
		<button class="btn btn-ghost" onclick={() => (formJadwalTerbuka = false)}>Batal</button>
		<button class="btn btn-primary" onclick={simpanJadwal}>Simpan</button>
	</div>
</Modal>

<Modal open={formStrongTerbuka} title={formStrong.id ? 'Edit Strong Point' : 'Tambah Strong Point'} lebar="max-w-xl" onclose={() => (formStrongTerbuka = false)}>
	<div class="grid gap-3 sm:grid-cols-2">
		<div class="sm:col-span-2">
			<label class="label" for="ssatker">Satker</label>
			<select id="ssatker" class="input" bind:value={formStrong.satker_id} disabled={Boolean(formStrong.id)}>
				{#each satkerNodes as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<div class="sm:col-span-2"><label class="label" for="snama">Nama</label><input id="snama" class="input" bind:value={formStrong.nama} /></div>
		<div class="sm:col-span-2"><label class="label" for="salamat">Alamat</label><input id="salamat" class="input" bind:value={formStrong.alamat} /></div>
		<div><label class="label" for="slat">Latitude</label><input id="slat" class="input" bind:value={formStrong.latitude} /></div>
		<div><label class="label" for="slng">Longitude</label><input id="slng" class="input" bind:value={formStrong.longitude} /></div>
		<div><label class="label" for="srad">Radius (m)</label><input id="srad" class="input" bind:value={formStrong.radius_m} /></div>
		<label class="flex items-center gap-2 pt-5 text-xs text-slate-600"><input type="checkbox" bind:checked={formStrong.is_active} /> Aktif</label>
	</div>
	<div class="mt-4 flex justify-end gap-2">
		<button class="btn btn-ghost" onclick={() => (formStrongTerbuka = false)}>Batal</button>
		<button class="btn btn-primary" onclick={simpanStrong}>Simpan</button>
	</div>
</Modal>

<Modal open={!!detail} title="Detail Jadwal PH" lebar="max-w-2xl" onclose={() => (detail = null)}>
	{#if detail}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="font-semibold text-slate-800">{detail.strong_point_nama}</p>
				<p class="text-xs text-slate-600">{new Date(detail.tanggal + 'T00:00:00').toLocaleDateString('id-ID')} · {detail.jam_mulai}–{detail.jam_selesai}</p>
				<p class="text-xs text-slate-600">{namaSatker(detail.satker_id)}</p>
			</div>
			<div class="overflow-x-auto">
				<table class="w-full text-xs">
					<thead><tr><th class="th">Personel</th><th class="th">Check-in</th><th class="th">Check-out</th><th class="th">Jarak</th></tr></thead>
					<tbody class="divide-y divide-slate-100">
						{#each detail.personel as p}
							{@const masuk = absensiDetail.find((a) => a.personel_id === p.personel_id && a.jenis === 'masuk')}
							{@const pulang = absensiDetail.find((a) => a.personel_id === p.personel_id && a.jenis === 'pulang')}
							<tr>
								<td class="td">{p.nama}<span class="block text-[11px] text-slate-500">{p.nrp}</span></td>
								<td class="td">{masuk ? `${masuk.jam} · ${masuk.dalam_radius ? 'dalam radius' : 'luar radius'}` : '-'}</td>
								<td class="td">{pulang ? `${pulang.jam} · ${pulang.dalam_radius ? 'dalam radius' : 'luar radius'}` : '-'}</td>
								<td class="td">{masuk?.jarak_m != null ? `${masuk.jarak_m.toFixed(0)} m` : '-'}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
</Modal>
