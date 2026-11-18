<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, apiBlob, apiBlobUrl, ApiError } from '#lib/api/client';
	import { adalahAdmin } from '#lib/stores/session.svelte';
	import type {
		AbsensiPh,
		FotoPh,
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
		regu: string;
		tanggal: string;
		tanggal_selesai: string;
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

	function pilihTab(nilai: Tab) {
		tab = nilai;
		if (page.url.searchParams.get('tab') !== nilai) {
			goto(`/ph?tab=${nilai}`, { replace: true, reset: false });
		}
	}

	// Sidebar dapat berpindah submenu pada route yang sama; ikuti nilai `?tab=`.
	// Tab master strong point disembunyikan sementara (strong point belum punya titik tetap).
	$effect(() => {
		const nilai = page.url.searchParams.get('tab');
		if (nilai === 'jadwal' || nilai === 'rekap') {
			tab = nilai;
		} else if (nilai === 'strong') {
			tab = 'jadwal';
			goto('/ph?tab=jadwal', { replace: true, reset: false });
		}
	});

	function daftarTanggal(daftar: string[]) {
		return daftar
			.map((tanggal) => {
				const [, bulan, hari] = tanggal.split('-');
				return `${hari}/${bulan}`;
			})
			.join(', ');
	}
	let satkerId = $state('');
	let satkerList = $state<Satker[]>([]);
	let strongPoints = $state<StrongPoint[]>([]);
	let jadwal = $state<JadwalPh[]>([]);
	let rekap = $state<RekapPh[]>([]);
	let personelSatker = $state<Personel[]>([]);
	let memuat = $state(true);
	let mengekspor = $state(false);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);
	type ItemGaleri = { url: string; caption: string };
	let galeri = $state<{ nama: string; memuat: boolean; item: ItemGaleri[] } | null>(null);
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
			regu: 'Regu 1',
			tanggal: hariIni(),
			tanggal_selesai: hariIni(),
			catatan: '',
			personel_ids: []
		};
	}

	type AnggotaRoster = { id: string; nama: string; pangkat: string; nrp: string };
	type BlokRoster = {
		satker_id: string;
		regu: string;
		anggota: AnggotaRoster[];
		jadwalPerHari: Set<string>[];
		lengkap: boolean[];
	};

	const jumlahHari = $derived.by(() => {
		const [tahun, bulanKe] = bulan.split('-').map(Number);
		return tahun && bulanKe ? new Date(tahun, bulanKe, 0).getDate() : 31;
	});

	const hariBulan = $derived(Array.from({ length: jumlahHari }, (_, i) => i + 1));

	function jumlahJadwalRegu(blok: BlokRoster) {
		return blok.jadwalPerHari.filter((set) => set.size > 0).length;
	}

	function tanggalPanjang(tanggal: string) {
		return new Date(tanggal + 'T00:00:00').toLocaleDateString('id-ID', {
			weekday: 'long',
			day: 'numeric',
			month: 'long',
			year: 'numeric'
		});
	}

	const roster = $derived.by<BlokRoster[]>(() => {
		const peta = new Map<
			string,
			{
				satker_id: string;
				regu: string;
				anggota: Map<string, AnggotaRoster>;
				perHari: Map<number, Set<string>>;
			}
		>();

		for (const row of jadwal) {
			const regu = row.regu?.trim() || 'Tanpa Regu';
			const kunci = `${row.satker_id}|${regu}`;
			const grup = peta.get(kunci) ?? {
				satker_id: row.satker_id,
				regu,
				anggota: new Map<string, AnggotaRoster>(),
				perHari: new Map<number, Set<string>>()
			};
			const hari = Number(row.tanggal.slice(8, 10));
			const set = grup.perHari.get(hari) ?? new Set<string>();
			for (const orang of row.personel) {
				grup.anggota.set(orang.personel_id, {
					id: orang.personel_id,
					nama: orang.nama,
					pangkat: orang.pangkat,
					nrp: orang.nrp
				});
				set.add(orang.personel_id);
			}
			grup.perHari.set(hari, set);
			peta.set(kunci, grup);
		}

		return [...peta.values()].map((grup) => {
			const anggota = [...grup.anggota.values()].sort((a, b) => a.nama.localeCompare(b.nama));
			const jadwalPerHari = Array.from(
				{ length: jumlahHari },
				(_, i) => grup.perHari.get(i + 1) ?? new Set<string>()
			);
			const lengkap = jadwalPerHari.map(
				(set) => anggota.length > 0 && set.size === anggota.length
			);
			return { satker_id: grup.satker_id, regu: grup.regu, anggota, jadwalPerHari, lengkap };
		});
	});

	function awalBlok(lengkap: boolean[], i: number) {
		return lengkap[i] && (i === 0 || !lengkap[i - 1]);
	}

	function panjangBlok(lengkap: boolean[], i: number) {
		let jumlah = 0;
		for (let k = i; k < lengkap.length && lengkap[k]; k++) jumlah += 1;
		return jumlah;
	}

	function akhirPekan(hari: number) {
		const [tahun, bulanKe] = bulan.split('-').map(Number);
		const hariKe = new Date(tahun, bulanKe - 1, hari).getDay();
		return hariKe === 0 || hariKe === 6;
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

	function tutupGaleri() {
		galeri?.item.forEach((item) => URL.revokeObjectURL(item.url));
		galeri = null;
	}

	async function bukaGaleri(row: RekapPh) {
		tutupGaleri();
		galeri = { nama: row.nama, memuat: true, item: [] };
		try {
			const params = new URLSearchParams({ bulan, personel_id: row.personel_id });
			if (satkerId) params.set('satker_id', satkerId);
			const hasil = await api<{ data: FotoPh[] }>(`/ph/foto?${params}`);
			const item = await Promise.all(
				hasil.data.map(async (foto) => ({
					url: await apiBlobUrl(foto.foto_url),
					caption: `${tanggalPanjang(foto.tanggal)} · ${foto.jenis === 'masuk' ? 'Check-in' : 'Check-out'} ${foto.jam}`
				}))
			);
			if (galeri) galeri = { ...galeri, memuat: false, item };
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat dokumentasi strong point';
			if (galeri) galeri = { ...galeri, memuat: false };
		}
	}

	async function eksporPdf() {
		mengekspor = true;
		error = null;
		try {
			const params = new URLSearchParams({ bulan });
			if (satkerId) params.set('satker_id', satkerId);
			const jenis = tab === 'rekap' ? 'rekap' : 'jadwal';

			const blob = await apiBlob(`/ph/${jenis}/ekspor?${params}`);
			const url = URL.createObjectURL(blob);
			const tautan = document.createElement('a');
			tautan.href = url;
			tautan.download = `strong-point-${jenis}-${bulan}.pdf`;
			tautan.click();
			URL.revokeObjectURL(url);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal membuat PDF';
		} finally {
			mengekspor = false;
		}
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
		formJadwalTerbuka = true;
		muatOrang(formJadwal.satker_id);
	}

	function gantiSatkerForm(satker: string) {
		formJadwal.satker_id = satker;
		formJadwal.personel_ids = [];
		muatOrang(satker);
	}

	function bukaEditJadwal(row: JadwalPh) {
		formJadwal = {
			id: row.id,
			satker_id: row.satker_id,
			regu: row.regu ?? 'Regu 1',
			tanggal: row.tanggal,
			tanggal_selesai: row.tanggal,
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
				tanggal: formJadwal.tanggal,
				tanggal_selesai: formJadwal.id ? null : formJadwal.tanggal_selesai || null,
				regu: formJadwal.regu.trim() || null,
				catatan: formJadwal.catatan || null,
				personel: formJadwal.personel_ids.map((personel_id) => ({ personel_id }))
			};
			if (formJadwal.id) {
				await api(`/ph/jadwal/${formJadwal.id}`, { method: 'PUT', body });
			} else {
				await api('/ph/jadwal', { method: 'POST', body });
			}
			formJadwalTerbuka = false;
			info = formJadwal.id ? 'Jadwal strong point diperbarui.' : 'Jadwal strong point dibuat.';
			await muatSemua();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan jadwal strong point';
		}
	}

	async function hapusJadwal(row: JadwalPh) {
		if (!confirm(`Hapus jadwal strong point tanggal ${row.tanggal}?`)) return;
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

<svelte:head><title>Strong Point — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-end justify-between gap-3 p-4">
		<div>
			<h2 class="text-sm font-semibold text-slate-800">Strong Point</h2>
			<p class="mt-1 text-xs text-slate-500">
				Jadwal penjagaan terpisah dari absensi harian. Check-in dan check-out divalidasi radius strong point.
			</p>
		</div>
		{#if admin}
			<div class="flex gap-2">
				<button class="btn btn-ghost border border-slate-300" disabled={mengekspor} onclick={eksporPdf}>
					<Icon name="download" size={16} />
					{mengekspor ? 'Menyiapkan PDF…' : 'Export PDF'}
				</button>
				<button class="btn btn-primary" onclick={bukaTambahJadwal}><Icon name="plus" size={16} /> Jadwal Strong Point</button>
				<!-- Tab master strong point disembunyikan sementara (belum ada titik tetap). -->
			</div>
		{:else}
			<button class="btn btn-ghost border border-slate-300" disabled={mengekspor} onclick={eksporPdf}>
				<Icon name="download" size={16} />
				{mengekspor ? 'Menyiapkan PDF…' : 'Export PDF'}
			</button>
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
			{#each [['jadwal', 'Jadwal Strong Point'], ['rekap', 'Rekap Strong Point']] as [nilai, label]}
				<button class="btn {tab === nilai ? 'btn-primary' : 'btn-ghost border border-slate-300'}" onclick={() => pilihTab(nilai as Tab)}>{label}</button>
			{/each}
		</div>
	</div>

	{#if info}<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>{/if}
	{#if error}<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>{/if}

	{#if tab === 'jadwal'}
		{#if jadwal.length === 0}
			<div class="card px-4 py-10 text-center text-sm text-slate-500">
				{memuat ? 'Memuat…' : 'Belum ada jadwal strong point pada bulan ini.'}
			</div>
		{:else}
			{#each roster as blok (blok.satker_id + '|' + blok.regu)}
				<div class="card overflow-hidden">
					<div class="flex flex-wrap items-center justify-between gap-2 border-b border-slate-200 px-4 py-3">
						<div>
							<h3 class="text-sm font-semibold text-slate-800">{blok.regu}</h3>
							<p class="text-xs text-slate-500">
								{namaSatker(blok.satker_id)} · {blok.anggota.length} anggota
							</p>
						</div>
						<span class="text-xs text-slate-500">
							{jumlahJadwalRegu(blok)} jadwal pada {bulan}
						</span>
					</div>

					<div class="overflow-x-auto">
						<table class="w-full border-collapse text-xs">
							<thead class="bg-slate-50">
								<tr>
									<th class="th w-10 px-2 text-center">No</th>
									<th class="th min-w-56">Anggota</th>
									<th class="th w-28">Pangkat</th>
									{#each hariBulan as hari}
										<th class="th w-8 px-1 text-center {akhirPekan(hari) ? 'text-rose-600' : ''}">
											{hari}
										</th>
									{/each}
								</tr>
							</thead>
							<tbody>
								{#each blok.anggota as orang, baris}
									<tr class="border-t border-slate-200">
										<td class="td px-2 text-center">{baris + 1}</td>
										<td class="td whitespace-nowrap font-medium text-slate-800">{orang.nama}</td>
										<td class="td text-slate-600">{orang.pangkat}</td>
										{#each hariBulan as hari}
											{@const i = hari - 1}
											{@const dijadwalkan = blok.jadwalPerHari[i].has(orang.id)}
											{#if blok.lengkap[i]}
												{#if baris === 0 && awalBlok(blok.lengkap, i)}
													<td
														rowspan={blok.anggota.length}
														colspan={panjangBlok(blok.lengkap, i)}
														class="border border-slate-300 bg-sky-200 px-1 py-2 text-center align-middle font-semibold text-sky-950"
													>
														{blok.regu}
													</td>
												{/if}
											{:else}
												<td
													class="border border-slate-200 px-1 py-1 text-center {dijadwalkan
														? 'bg-sky-200 font-semibold text-sky-900'
														: ''}"
													title={dijadwalkan ? `${blok.regu} · ${hari}/${bulan}` : ''}
												>
													{dijadwalkan ? '•' : ''}
												</td>
											{/if}
										{/each}
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</div>
			{/each}
		{/if}

		<details class="card overflow-hidden">
			<summary class="cursor-pointer px-4 py-3 text-sm font-semibold text-slate-700">
				Kelola jadwal bulan ini ({jadwal.length})
			</summary>
			<div class="overflow-x-auto border-t border-slate-200">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Tanggal</th>
							<th class="th">Regu</th>
							<th class="th text-center">Personel</th>
							<th class="th">Catatan</th>
							<th class="th no-print">Aksi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each jadwal as row}
							<tr class="hover:bg-slate-50/70">
								<td class="td text-slate-700">{tanggalPanjang(row.tanggal)}</td>
								<td class="td text-slate-600">{row.regu ?? 'Tanpa Regu'}</td>
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
							<tr><td class="td text-center text-slate-500" colspan="5">Belum ada jadwal strong point.</td></tr>
						{/each}
					</tbody>
				</table>
			</div>
		</details>
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
								<th class="th text-center">Check-in</th>
								<th class="th text-center">Hanya Check-in</th>
								<th class="th text-center">Tidak Strong Point</th>
								<th class="th text-center">Menunggu</th>
								<th class="th text-center">Dokumentasi</th>
							</tr>
						</thead>
						<tbody class="divide-y divide-slate-100">
							{#each rekapHalaman as row}
								<tr>
									<td class="td">
										<p class="font-medium text-slate-800">{row.nama}</p>
										<p class="text-[11px] text-slate-500">{row.pangkat} · {row.nrp}</p>
										<div class="mt-1 flex flex-wrap gap-1">
											{#if row.tidak_hadir > 0}
												<Badge tone="red">Tidak strong point</Badge>
											{/if}
											{#if row.hanya_check_in > 0}
												<Badge tone="amber">Hanya check-in</Badge>
											{/if}
											{#if row.jumlah_jadwal > 0 && row.tidak_hadir === 0 && row.hanya_check_in === 0}
												<Badge tone="green">Lengkap</Badge>
											{/if}
										</div>
									</td>
									<td class="td text-slate-600">{row.unit ?? '-'}</td>
									<td class="td text-center text-slate-600">{row.jumlah_jadwal}</td>
									<td class="td text-center font-semibold text-emerald-700">{row.hadir}</td>
									<td class="td text-center">
										{#if row.hanya_check_in > 0}
											<p class="font-semibold text-amber-700">{row.hanya_check_in}</p>
											<p class="text-[10px] text-amber-700">
												{daftarTanggal(row.tanggal_hanya_check_in)}
											</p>
										{:else}
											<span class="text-slate-300">-</span>
										{/if}
									</td>
									<td class="td text-center">
										{#if row.tidak_hadir > 0}
											<p class="font-semibold text-rose-700">{row.tidak_hadir}</p>
											<p class="text-[10px] text-rose-700">
												{daftarTanggal(row.tanggal_tidak_hadir)}
											</p>
										{:else}
											<span class="text-slate-300">-</span>
										{/if}
									</td>
									<td class="td text-center">
										{#if row.menunggu_verifikasi > 0}<Badge tone="amber">{row.menunggu_verifikasi}</Badge>{:else}<span class="text-slate-300">-</span>{/if}
									</td>
									<td class="td text-center">
										<button
											class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
											onclick={() => bukaGaleri(row)}
										>
											<Icon name="image" size={14} /> Lihat
										</button>
									</td>
								</tr>
							{:else}
								<tr><td class="td text-center text-slate-500" colspan="8">{memuat ? 'Memuat…' : 'Belum ada rekap PH.'}</td></tr>
							{/each}
					</tbody>
				</table>
			</div>
			<Pagination page={halamanRekapAman} perPage={perPage} total={rekap.length} label="rekap PH" onchange={(p) => (halamanRekap = p)} />
		</div>
	{/if}
</div>

<Modal open={formJadwalTerbuka} title={formJadwal.id ? 'Edit Jadwal Strong Point' : 'Tambah Jadwal Strong Point'} lebar="max-w-2xl" onclose={() => (formJadwalTerbuka = false)}>
	<div class="grid gap-3 sm:grid-cols-2">
		<div class="sm:col-span-2">
			<label class="label" for="jsatker">Satker</label>
			<select id="jsatker" class="input" value={formJadwal.satker_id} disabled={Boolean(formJadwal.id)} onchange={(e) => gantiSatkerForm((e.target as HTMLSelectElement).value)}>
				{#each satkerNodes as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<div><label class="label" for="jregu">Regu</label><input id="jregu" class="input" placeholder="Regu 1" bind:value={formJadwal.regu} /></div>
		<div>
			<label class="label" for="jtgl">{formJadwal.id ? 'Tanggal' : 'Tanggal Mulai'}</label>
			<input id="jtgl" type="date" class="input" bind:value={formJadwal.tanggal} />
		</div>
		{#if !formJadwal.id}
			<div>
				<label class="label" for="jselesai">Tanggal Selesai</label>
				<input
					id="jselesai"
					type="date"
					class="input"
					min={formJadwal.tanggal}
					bind:value={formJadwal.tanggal_selesai}
				/>
				<p class="mt-1 text-[11px] text-slate-500">
					Satu jadwal dibuat untuk setiap tanggal dalam rentang.
				</p>
			</div>
		{/if}
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
				<p class="font-semibold text-slate-800">Jadwal Strong Point</p>
				<p class="text-xs text-slate-600">{new Date(detail.tanggal + 'T00:00:00').toLocaleDateString('id-ID', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' })}</p>
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

	<Modal
		open={!!galeri}
		title={galeri ? `Dokumentasi Strong Point — ${galeri.nama}` : 'Dokumentasi Strong Point'}
		lebar="max-w-4xl"
		onclose={tutupGaleri}
	>
		{#if galeri}
			{#if galeri.memuat}
				<p class="py-8 text-center text-sm text-slate-500">Memuat dokumentasi…</p>
			{:else if galeri.item.length === 0}
				<p class="py-8 text-center text-sm text-slate-500">
					Belum ada foto dokumentasi strong point pada bulan ini.
				</p>
			{:else}
				<div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
					{#each galeri.item as foto}
						<figure class="overflow-hidden rounded-lg ring-1 ring-slate-200">
							<img
								src={foto.url}
								alt={foto.caption}
								class="h-40 w-full object-cover"
							/>
							<figcaption class="bg-slate-50 px-2 py-1 text-[11px] text-slate-600">
								{foto.caption}
							</figcaption>
						</figure>
					{/each}
				</div>
			{/if}
		{/if}
	</Modal>
