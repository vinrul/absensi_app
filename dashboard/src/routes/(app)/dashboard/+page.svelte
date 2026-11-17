<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import { adalahPimpinan, session } from '#lib/stores/session.svelte';
	import type {
		Kategori,
		Penilaian,
		Periode,
		RingkasanDashboard,
		Satker
	} from '#lib/types';
	import { LABEL_KATEGORI, LABEL_STATUS_KEHADIRAN } from '#lib/types';
	import StatCard from '#lib/components/StatCard.svelte';
	import Donut from '#lib/components/Donut.svelte';
	import TrendChart from '#lib/components/TrendChart.svelte';
	import CategoryPie from '#lib/components/CategoryPie.svelte';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';

	interface Capaian {
		absensi: number;
		rencana_kerja: number;
		hasil_kerja: number;
		output: number;
		nilai_akhir: number;
	}

	interface SebaranItem {
		kategori: Kategori;
		jumlah: number;
		persen: number;
	}

	let periodeList = $state<Periode[]>([]);
	let satkerList = $state<Satker[]>([]);

	let periodeId = $state('');
	let satkerId = $state('');
	let unit = $state('');

	let ringkasan = $state<RingkasanDashboard | null>(null);
	let capaian = $state<Capaian | null>(null);
	let tren = $state<{ nama: string; rata_rata: number }[]>([]);
	let sebaran = $state<SebaranItem[]>([]);
	let rekap = $state<Penilaian[]>([]);
	let meta = $state({ page: 1, per_page: 10, total: 0 });

	let memuat = $state(true);
	let error = $state<string | null>(null);
	let infoHitung = $state<string | null>(null);

	let panelPimpinan = $state(false);
	let dipilih = $state<Penilaian | null>(null);
	let catatanPimpinan = $state('');
	let keputusan = $state<'disetujui' | 'perlu_perbaikan'>('disetujui');
	let menyimpan = $state(false);

	const bisaNilai = $derived(adalahPimpinan() || session.user?.roles.includes('admin_sistem'));
	const totalHalaman = $derived(Math.max(1, Math.ceil(meta.total / meta.per_page)));

	function query(page = 1) {
		const params = new URLSearchParams();
		if (periodeId) params.set('periode_id', periodeId);
		if (satkerId) params.set('satker_id', satkerId);
		if (unit.trim()) params.set('unit', unit.trim());
		params.set('page', String(page));
		params.set('per_page', String(meta.per_page));
		return params.toString();
	}

	async function muatSemua(page = 1) {
		memuat = true;
		error = null;
		try {
			const dasar = new URLSearchParams();
			if (periodeId) dasar.set('periode_id', periodeId);
			if (satkerId) dasar.set('satker_id', satkerId);
			if (unit.trim()) dasar.set('unit', unit.trim());

			const [r, c, t, s, k] = await Promise.all([
				api<RingkasanDashboard>(`/dashboard/ringkasan?${dasar}`),
				api<Capaian>(`/dashboard/capaian-indikator?${dasar}`),
				api<{ nama: string; rata_rata: number }[]>(`/dashboard/tren?bulan=6${satkerId ? `&satker_id=${satkerId}` : ''}`),
				api<SebaranItem[]>(`/dashboard/sebaran-kategori?${dasar}`),
				api<{ data: Penilaian[]; meta: { page: number; per_page: number; total: number } }>(
					`/dashboard/rekap?${query(page)}`
				)
			]);

			ringkasan = r;
			capaian = c;
			tren = t;
			sebaran = s;
			rekap = k.data;
			meta = k.meta;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat dashboard';
		} finally {
			memuat = false;
		}
	}

	async function hitungUlang() {
		if (!periodeId) return;
		infoHitung = 'Menghitung…';
		try {
			const hasil = await api<{ jumlah_personel: number; rata_rata_nilai: number }>(
				`/penilaian/${periodeId}/hitung`,
				{ method: 'POST' }
			);
			infoHitung = `${hasil.jumlah_personel} personel dihitung · rata-rata ${hasil.rata_rata_nilai.toFixed(2)}`;
			await muatSemua(meta.page);
		} catch (err) {
			infoHitung = err instanceof ApiError ? err.message : 'Gagal menghitung penilaian';
		}
	}

	function bukaPanel(row: Penilaian) {
		dipilih = row;
		catatanPimpinan = row.catatan_pimpinan ?? '';
		keputusan = row.keputusan ?? 'disetujui';
		panelPimpinan = true;
	}

	async function simpanKeputusan() {
		if (!dipilih) return;
		menyimpan = true;
		try {
			await api(`/penilaian/${dipilih.id}/keputusan`, {
				method: 'POST',
				body: { catatan: catatanPimpinan || null, keputusan }
			});
			panelPimpinan = false;
			await muatSemua(meta.page);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan keputusan';
		} finally {
			menyimpan = false;
		}
	}

	function nada(kategori: Kategori) {
		if (kategori === 'sangat_baik') return 'green';
		if (kategori === 'baik') return 'blue';
		if (kategori === 'cukup') return 'amber';
		return 'red';
	}

	onMount(async () => {
		try {
			const [p, s] = await Promise.all([api<Periode[]>('/periode'), api<Satker[]>('/satker')]);
			periodeList = p;
			satkerList = s;
			periodeId = p[0]?.id ?? '';
			await muatSemua(1);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat data awal';
			memuat = false;
		}
	});
</script>

<svelte:head>
	<title>Dashboard Penilaian Kinerja — Polda Bali</title>
</svelte:head>

<div class="space-y-4">
	<!-- Filter -->
	<div class="card flex flex-wrap items-end gap-3 p-4 no-print">
		<div class="min-w-40 flex-1">
			<label class="label" for="periode">Periode Penilaian</label>
			<select id="periode" class="input" bind:value={periodeId} onchange={() => muatSemua(1)}>
				{#each periodeList as p}
					<option value={p.id}>{p.nama}</option>
				{/each}
			</select>
		</div>

		<div class="min-w-40 flex-1">
			<label class="label" for="satker">Satuan Kerja</label>
			<select id="satker" class="input" bind:value={satkerId} onchange={() => muatSemua(1)}>
				<option value="">Semua Satker</option>
				{#each satkerList as s}
					<option value={s.id}>{s.nama}</option>
				{/each}
			</select>
		</div>

		<div class="min-w-32 flex-1">
			<label class="label" for="unit">Subdit / Bagian</label>
			<input id="unit" class="input" placeholder="Semua unit" bind:value={unit} onchange={() => muatSemua(1)} />
		</div>

		<div class="flex gap-2">
			<button class="btn btn-ghost border border-slate-300" onclick={() => muatSemua(meta.page)}>
				<Icon name="refresh" size={16} /> Muat ulang
			</button>
			{#if bisaNilai}
				<button class="btn btn-primary" onclick={hitungUlang}>
					<Icon name="trend" size={16} /> Hitung Penilaian
				</button>
			{/if}
			<button class="btn btn-gold" onclick={() => window.print()}>
				<Icon name="laporan" size={16} /> Cetak
			</button>
		</div>
	</div>

	{#if infoHitung}
		<div class="rounded-lg bg-sky-50 px-4 py-2 text-sm text-sky-800 ring-1 ring-sky-200 no-print">
			{infoHitung}
		</div>
	{/if}

	{#if error}
		<div class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">
			{error}
		</div>
	{/if}

	<!-- KPI -->
	<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 2xl:grid-cols-6">
		<StatCard
			label="Total Personel"
			nilai={ringkasan?.total_personel ?? 0}
			sub="personel aktif"
			icon="users"
			warna="bg-navy-800"
		/>
		<StatCard
			label="Hadir Hari Ini"
			nilai={ringkasan?.hadir_hari_ini ?? 0}
			sub={`${(ringkasan?.persen_hadir ?? 0).toFixed(1)}% kehadiran`}
			icon="check"
			warna="bg-emerald-600"
		/>
		<StatCard
			label="Rencana Kerja"
			nilai={ringkasan?.total_rencana ?? 0}
			sub="periode berjalan"
			icon="rencana"
			warna="bg-amber-500"
		/>
		<StatCard
			label="Hasil Kerja"
			nilai={ringkasan?.total_hasil ?? 0}
			sub="terverifikasi"
			icon="hasil"
			warna="bg-sky-700"
		/>
		<StatCard
			label="Output"
			nilai={ringkasan?.total_output ?? 0}
			sub="disetujui"
			icon="output"
			warna="bg-violet-600"
		/>
		<StatCard
			label="Rata-rata Kinerja"
			nilai={`${(ringkasan?.rata_rata_nilai ?? 0).toFixed(1)}%`}
			sub={ringkasan?.periode_nama ?? ''}
			icon="trend"
			warna="bg-rose-600"
		/>
	</div>

	<!-- Chart -->
	<div class="grid gap-4 lg:grid-cols-3">
		<div class="card p-4">
			<h2 class="mb-3 text-sm font-semibold text-slate-700">Capaian Kinerja Per Indikator</h2>
			<div class="grid grid-cols-2 gap-3">
				<Donut nilai={capaian?.absensi ?? 0} label="Absensi" warna="#059669" />
				<Donut nilai={capaian?.rencana_kerja ?? 0} label="Rencana Kerja" warna="#0284c7" />
				<Donut nilai={capaian?.hasil_kerja ?? 0} label="Hasil Kerja" warna="#0b2545" />
				<Donut nilai={capaian?.output ?? 0} label="Output" warna="#d4a017" />
			</div>
		</div>

		<div class="card p-4">
			<h2 class="mb-3 text-sm font-semibold text-slate-700">Tren Capaian Kinerja</h2>
			<TrendChart points={tren} />
			<p class="mt-2 text-[11px] text-slate-500">
				Rata-rata nilai akhir per periode penilaian.
			</p>
		</div>

		<div class="card p-4">
			<h2 class="mb-3 text-sm font-semibold text-slate-700">Sebaran Kategori Kinerja</h2>
			<CategoryPie items={sebaran} />
			<p class="mt-3 text-[11px] leading-relaxed text-slate-500">
				Sangat Baik ≥ 95 · Baik 90–94,99 · Cukup 80–89,99 · Perlu Pembinaan &lt; 80
			</p>
		</div>
	</div>

	<!-- Rekap -->
	<div class="card overflow-hidden">
		<div class="flex items-center justify-between gap-3 border-b border-slate-200 px-4 py-3">
			<h2 class="text-sm font-semibold text-slate-700">Rekap Penilaian Kinerja Personel</h2>
			<span class="text-xs text-slate-500">
				Menampilkan {rekap.length} dari {meta.total} personel
			</span>
		</div>

		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">No</th>
						<th class="th">Nama Personel</th>
						<th class="th">Pangkat</th>
						<th class="th">Jabatan / Unit</th>
						<th class="th text-center">Absensi</th>
						<th class="th text-center">Rencana</th>
						<th class="th text-center">Hasil</th>
						<th class="th text-center">Output</th>
						<th class="th text-center">Nilai Akhir</th>
						<th class="th">Kategori</th>
						{#if bisaNilai}<th class="th no-print">Aksi</th>{/if}
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each rekap as row, i}
						<tr class="hover:bg-slate-50/70">
							<td class="td text-slate-500">{(meta.page - 1) * meta.per_page + i + 1}</td>
							<td class="td">
								<p class="font-medium text-slate-800">{row.personel_nama}</p>
								<p class="text-[11px] text-slate-500">NRP {row.personel_nrp}</p>
							</td>
							<td class="td text-slate-600">{row.pangkat}</td>
							<td class="td text-slate-600">{row.jabatan}</td>
							<td class="td text-center">{row.skor_absensi.toFixed(1)}</td>
							<td class="td text-center">{row.skor_rencana_kerja.toFixed(1)}</td>
							<td class="td text-center">{row.skor_hasil_kerja.toFixed(1)}</td>
							<td class="td text-center">{row.skor_output.toFixed(1)}</td>
							<td class="td text-center font-semibold text-slate-800">
								{row.nilai_akhir.toFixed(2)}
							</td>
							<td class="td"><Badge tone={nada(row.kategori)}>{LABEL_KATEGORI[row.kategori]}</Badge></td>
							{#if bisaNilai}
								<td class="td no-print">
									<button class="btn btn-ghost px-2 py-1 text-xs" onclick={() => bukaPanel(row)}>
										<Icon name="penilaian" size={14} /> Nilai
									</button>
								</td>
							{/if}
						</tr>
					{:else}
						<tr>
							<td class="td text-center text-slate-500" colspan={bisaNilai ? 11 : 10}>
								{memuat ? 'Memuat data…' : 'Belum ada penilaian pada periode ini. Klik "Hitung Penilaian".'}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="flex items-center justify-between gap-3 border-t border-slate-200 px-4 py-3 no-print">
			<p class="text-xs text-slate-500">Halaman {meta.page} dari {totalHalaman}</p>
			<div class="flex gap-2">
				<button
					class="btn btn-ghost border border-slate-300 px-2 py-1"
					disabled={meta.page <= 1}
					onclick={() => muatSemua(meta.page - 1)}
				>
					<Icon name="left" size={16} />
				</button>
				<button
					class="btn btn-ghost border border-slate-300 px-2 py-1"
					disabled={meta.page >= totalHalaman}
					onclick={() => muatSemua(meta.page + 1)}
				>
					<Icon name="right" size={16} />
				</button>
			</div>
		</div>
	</div>

	<!-- Keterangan status kehadiran (untuk cetak) -->
	<p class="hidden text-[10px] text-slate-500 print:block">
		Status kehadiran: {Object.values(LABEL_STATUS_KEHADIRAN).join(' · ')}. Dokumen dicetak dari
		Sistem Penilaian Kinerja Personel Polda Bali pada {new Date().toLocaleString('id-ID')}.
	</p>
</div>

<Modal open={panelPimpinan} title="Penilaian Pimpinan" onclose={() => (panelPimpinan = false)}>
	{#if dipilih}
		<div class="space-y-4">
			<div class="rounded-lg bg-slate-50 p-3 text-sm">
				<p class="font-semibold text-slate-800">{dipilih.personel_nama}</p>
				<p class="text-xs text-slate-500">
					{dipilih.pangkat} · {dipilih.jabatan} / {dipilih.unit ?? '-'}
				</p>
				<p class="mt-2 text-xs text-slate-600">
					Nilai akhir <strong>{dipilih.nilai_akhir.toFixed(2)}</strong> ·
					{LABEL_KATEGORI[dipilih.kategori]}
				</p>
			</div>

			<div>
				<label class="label" for="catatan">Catatan Pimpinan</label>
				<textarea
					id="catatan"
					class="input h-28 resize-none"
					placeholder="Contoh: Kinerja baik, tingkatkan kualitas output dan ketepatan waktu."
					bind:value={catatanPimpinan}
				></textarea>
			</div>

			<div>
				<label class="label" for="keputusan">Keputusan Penilaian</label>
				<select id="keputusan" class="input" bind:value={keputusan}>
					<option value="disetujui">Disetujui</option>
					<option value="perlu_perbaikan">Perlu Perbaikan</option>
				</select>
			</div>

			<div class="flex justify-end gap-2">
				<button class="btn btn-ghost" onclick={() => (panelPimpinan = false)}>Batal</button>
				<button class="btn btn-primary" disabled={menyimpan} onclick={simpanKeputusan}>
					<Icon name="check" size={16} /> {menyimpan ? 'Menyimpan…' : 'Simpan Penilaian'}
				</button>
			</div>
		</div>
	{/if}
</Modal>
