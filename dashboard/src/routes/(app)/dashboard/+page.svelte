<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import type {
		AbsensiHarian,
		JadwalPh,
		ListResponse,
		RekapPhHarian,
		Satker,
		StatusJadwalPh,
		StatusKehadiran,
		StrongPoint,
		TitikDinas
	} from '#lib/types';
	import { LABEL_STATUS_KEHADIRAN } from '#lib/types';
	import StatCard from '#lib/components/StatCard.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import PetaDashboard from '#lib/components/PetaDashboard.svelte';

	interface RingkasanKehadiran {
		tanggal: string;
		total_personel: number;
		hadir: number;
		belum_absensi: number;
		telat: number;
		cuti: number;
		izin: number;
		sakit: number;
		persen_hadir: number;
		sesi_strong_point: number;
		personel_strong_point: number;
		hadir_strong_point: number;
	}

	const hariIni = () => {
		const sekarang = new Date();
		const bagian = new Intl.DateTimeFormat('en-CA', {
			timeZone: 'Asia/Makassar',
			year: 'numeric',
			month: '2-digit',
			day: '2-digit'
		}).formatToParts(sekarang);
		const ambil = (jenis: string) => bagian.find((item) => item.type === jenis)?.value ?? '';
		return `${ambil('year')}-${ambil('month')}-${ambil('day')}`;
	};

	let tanggal = $state(hariIni());
	let satkerId = $state('');
	let satkerList = $state<Satker[]>([]);
	let ringkasan = $state<RingkasanKehadiran | null>(null);
	let jadwal = $state<JadwalPh[]>([]);
	let absensi = $state<AbsensiHarian[]>([]);
	let titikDinas = $state<TitikDinas[]>([]);
	let titikStrongPoint = $state<StrongPoint[]>([]);
	let phHarian = $state<RekapPhHarian[]>([]);
	let statusJadwal = $state<StatusJadwalPh | null>(null);
	let memuat = $state(true);
	let error = $state<string | null>(null);

	const satkerMap = $derived(new Map(satkerList.map((row) => [row.id, row.nama])));

	function tanggalPanjang(nilai: string) {
		return new Date(`${nilai}T00:00:00`).toLocaleDateString('id-ID', {
			weekday: 'long',
			day: '2-digit',
			month: 'long',
			year: 'numeric'
		});
	}

	function statusHarian(row: AbsensiHarian): StatusKehadiran | null {
		return row.status_masuk ?? row.status_pulang;
	}

	async function muat() {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams({ tanggal });
			if (satkerId) params.set('satker_id', satkerId);
			const paramsJadwal = new URLSearchParams({ dari: tanggal, sampai: tanggal });
			if (satkerId) paramsJadwal.set('satker_id', satkerId);
			const paramsAbsensi = new URLSearchParams({
				dari: tanggal,
				sampai: tanggal,
				per_page: '500'
			});
			if (satkerId) paramsAbsensi.set('satker_id', satkerId);
			const paramsStatus = new URLSearchParams({ bulan: tanggal.slice(0, 7) });
			if (satkerId) paramsStatus.set('satker_id', satkerId);
			const paramsTitik = new URLSearchParams();
			if (satkerId) paramsTitik.set('satker_id', satkerId);
			const paramsPh = new URLSearchParams({ dari: tanggal, sampai: tanggal });
			if (satkerId) paramsPh.set('satker_id', satkerId);

			const [r, j, a, s, td, sp, ph] = await Promise.all([
				api<RingkasanKehadiran>(`/dashboard/kehadiran?${params}`),
				api<JadwalPh[]>(`/ph/jadwal?${paramsJadwal}`),
				api<ListResponse<AbsensiHarian>>(`/absensi/harian?${paramsAbsensi}`),
				api<StatusJadwalPh>(`/ph/jadwal/status?${paramsStatus}`),
				api<TitikDinas[]>(`/titik-dinas?${paramsTitik}`).catch(() => []),
				api<StrongPoint[]>(`/ph/strong-point?${paramsTitik}`).catch(() => []),
				api<{ rentang: { dari: string; sampai: string }; data: RekapPhHarian[] }>(
					`/ph/rekap/harian?${paramsPh}`
				).catch(() => ({ rentang: { dari: tanggal, sampai: tanggal }, data: [] }))
			]);
			ringkasan = r;
			jadwal = j;
			absensi = a.data;
			statusJadwal = s;
			titikDinas = td;
			titikStrongPoint = sp;
			phHarian = ph.data;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat dashboard';
		} finally {
			memuat = false;
		}
	}

	onMount(async () => {
		try {
			satkerList = await api<Satker[]>('/satker');
		} catch {
			satkerList = [];
		}
		await muat();
	});
</script>

<svelte:head>
	<title>Dashboard Kehadiran — Polda Bali</title>
</svelte:head>

<div class="space-y-4">
	{#if statusJadwal}
		{#if statusJadwal.disalin_dari_bulan}
			<p class="rounded-lg bg-amber-50 px-4 py-2 text-sm text-amber-800 ring-1 ring-amber-200">
				Jadwal strong point bulan ini belum disusun; sistem memakai salinan jadwal sebelumnya.
				Segera susun jadwal bulan ini di menu Strong Point.
			</p>
		{:else if statusJadwal.perlu_susun_bulan_depan && statusJadwal.bulan.slice(0, 7) === hariIni().slice(0, 7)}
			<p class="rounded-lg bg-sky-50 px-4 py-2 text-sm text-sky-800 ring-1 ring-sky-200">
				Jadwal strong point bulan depan belum disusun. Segera susun sebelum tanggal 1 agar tidak
				memakai jadwal bulan sebelumnya.
			</p>
		{/if}
	{/if}
	<div class="card flex flex-wrap items-end gap-3 p-4 no-print">
		<div class="min-w-40 flex-1">
			<label class="label" for="tanggal">Tanggal</label>
			<input id="tanggal" type="date" class="input" bind:value={tanggal} onchange={muat} />
		</div>
		<div class="min-w-52 flex-1">
			<label class="label" for="satker">Satuan Kerja</label>
			<select id="satker" class="input" bind:value={satkerId} onchange={muat}>
				<option value="">Semua Satker</option>
				{#each satkerList as s}
					<option value={s.id}>{s.nama}</option>
				{/each}
			</select>
		</div>
		<div class="flex gap-2">
			<button class="btn btn-ghost border border-slate-300" onclick={muat}>
				<Icon name="refresh" size={16} /> Muat ulang
			</button>
			<a class="btn btn-primary" href="/absensi/rekap?tab=rekap">
				<Icon name="laporan" size={16} /> Buka Rekap
			</a>
		</div>
	</div>

	{#if error}
		<div class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">
			{error}
		</div>
	{/if}

	<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-6">
		<StatCard
			label="Total Personel"
			nilai={ringkasan?.total_personel ?? 0}
			sub="personel aktif"
			icon="users"
			warna="bg-navy-800"
		/>
		<StatCard
			label="Hadir"
			nilai={ringkasan?.hadir ?? 0}
			sub={`${(ringkasan?.persen_hadir ?? 0).toFixed(1)}% kehadiran`}
			icon="check"
			warna="bg-emerald-600"
		/>
		<StatCard
			label="Belum Absen"
			nilai={ringkasan?.belum_absensi ?? 0}
			sub="belum ada absensi masuk"
			icon="clock"
			warna="bg-rose-600"
		/>
		<StatCard
			label="Telat"
			nilai={ringkasan?.telat ?? 0}
			sub="melewati jam masuk"
			icon="alert"
			warna="bg-amber-500"
		/>
		<StatCard
			label="Sesi Strong Point"
			nilai={ringkasan?.sesi_strong_point ?? 0}
			sub={`${ringkasan?.hadir_strong_point ?? 0} dari ${ringkasan?.personel_strong_point ?? 0} anggota check-in`}
			icon="pin"
			warna="bg-sky-700"
		/>
		<StatCard
			label="Cuti / Izin / Sakit"
			nilai={(ringkasan?.cuti ?? 0) + (ringkasan?.izin ?? 0) + (ringkasan?.sakit ?? 0)}
			sub={`${ringkasan?.cuti ?? 0} cuti · ${ringkasan?.izin ?? 0} izin · ${ringkasan?.sakit ?? 0} sakit`}
			icon="clock"
			warna="bg-violet-600"
		/>
	</div>

	<p class="text-xs text-slate-500">Ringkasan untuk {tanggalPanjang(tanggal)}.</p>

	<PetaDashboard
		{absensi}
		titikDinas={titikDinas}
		strongPoint={titikStrongPoint}
		{phHarian}
	/>

	<div class="grid gap-4 xl:grid-cols-2">
		<div class="card overflow-hidden">
			<div class="flex items-center justify-between gap-2 border-b border-slate-200 px-4 py-3">
				<h2 class="text-sm font-semibold text-slate-700">Strong Point Hari Ini</h2>
				<span class="text-xs text-slate-500">{jadwal.length} sesi</span>
			</div>
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Sesi / Jam</th>
							<th class="th">Titik</th>
							<th class="th">Satker</th>
							<th class="th text-center">Anggota</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each jadwal as row (row.id)}
							<tr class="hover:bg-slate-50/70">
								<td class="td">
									<p class="font-medium capitalize text-slate-800">{row.sesi ?? 'pagi'}</p>
									<p class="text-[11px] text-slate-500">
										{row.jam_mulai}–{row.jam_selesai}
									</p>
								</td>
								<td class="td text-slate-600">{row.strong_point_nama}</td>
								<td class="td text-xs text-slate-500">
									{satkerMap.get(row.satker_id) ?? '-'}
								</td>
								<td class="td text-center text-slate-600">{row.jumlah_personel}</td>
							</tr>
						{:else}
							<tr>
								<td class="td text-center text-slate-500" colspan="4">
									{memuat ? 'Memuat…' : 'Tidak ada jadwal strong point pada tanggal ini.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>

		<div class="card overflow-hidden">
			<div class="flex items-center justify-between gap-2 border-b border-slate-200 px-4 py-3">
				<h2 class="text-sm font-semibold text-slate-700">Absensi Masuk / Pulang</h2>
				<a class="text-xs font-medium text-navy-700 underline" href="/absensi/rekap?tab=detail">
					Detail rekap
				</a>
			</div>
			<div class="max-h-96 overflow-auto">
				<table class="w-full">
					<thead class="sticky top-0 bg-slate-50">
						<tr>
							<th class="th">Personel</th>
							<th class="th">Masuk</th>
							<th class="th">Pulang</th>
							<th class="th">Status</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each absensi as row (row.personel_id + row.tanggal)}
							<tr class="hover:bg-slate-50/70">
								<td class="td">
									<p class="font-medium text-slate-800">{row.personel_nama}</p>
									<p class="text-[11px] text-slate-500">
										{row.personel_nrp} · {satkerMap.get(row.satker_id) ?? row.unit ?? '-'}
									</p>
								</td>
								<td class="td text-slate-600">
									{row.masuk_jam ?? '-'}
									{#if row.menit_telat > 0}
										<span class="ml-1 text-[11px] font-medium text-amber-700">
											+{row.menit_telat} mnt
										</span>
									{/if}
								</td>
								<td class="td text-slate-600">{row.pulang_jam ?? '-'}</td>
								<td class="td text-xs text-slate-600">
									{LABEL_STATUS_KEHADIRAN[statusHarian(row) ?? 'tanpa_keterangan']}
								</td>
							</tr>
						{:else}
							<tr>
								<td class="td text-center text-slate-500" colspan="4">
									{memuat ? 'Memuat…' : 'Belum ada absensi pada tanggal ini.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	</div>
</div>
