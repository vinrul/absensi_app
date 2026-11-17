<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import type { Absensi } from '#lib/types';
	import { LABEL_STATUS_KEHADIRAN } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	let dari = $state('');
	let sampai = $state('');
	let data = $state<Absensi[]>([]);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);
	let memproses = $state(false);
	let halaman = $state(1);
	const perPage = 10;

	const totalHalaman = $derived(Math.max(1, Math.ceil(data.length / perPage)));
	const halamanAman = $derived(Math.min(halaman, totalHalaman));
	const dataHalaman = $derived(data.slice((halamanAman - 1) * perPage, halamanAman * perPage));

	const hariIni = new Date().toISOString().slice(0, 10);
	const awalBulan = hariIni.slice(0, 8) + '01';

	const sudahMasuk = $derived(data.some((row) => row.tanggal === hariIni && row.jenis === 'masuk'));
	const sudahPulang = $derived(
		data.some((row) => row.tanggal === hariIni && row.jenis === 'pulang')
	);

	function nada(status: Absensi['status_kehadiran']) {
		if (status === 'dinas_kantor' || status === 'piket' || status === 'dinas_luar') return 'green';
		if (status === 'tanpa_keterangan') return 'red';
		if (status === 'libur_nasional') return 'slate';
		return 'amber';
	}

	async function muat() {
		memuat = true;
		error = null;
		halaman = 1;
		try {
			const params = new URLSearchParams();
			if (dari) params.set('dari', dari);
			if (sampai) params.set('sampai', sampai);
			data = await api<Absensi[]>(`/absensi/saya?${params}`);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat riwayat absensi';
		} finally {
			memuat = false;
		}
	}

	function ambilLokasi(): Promise<GeolocationPosition> {
		return new Promise((resolve, reject) => {
			if (!navigator.geolocation) {
				reject(new Error('Perangkat tidak mendukung geolokasi'));
				return;
			}
			navigator.geolocation.getCurrentPosition(resolve, reject, {
				enableHighAccuracy: true,
				timeout: 20000,
				maximumAge: 0
			});
		});
	}

	async function absen(jenis: 'masuk' | 'pulang') {
		memproses = true;
		error = null;
		info = null;
		try {
			const posisi = await ambilLokasi();
			await api('/absensi', {
				method: 'POST',
				body: {
					client_uuid: crypto.randomUUID(),
					jenis,
					status_kehadiran: 'dinas_kantor',
					captured_at: new Date().toISOString(),
					latitude: posisi.coords.latitude,
					longitude: posisi.coords.longitude,
					akurasi_m: posisi.coords.accuracy,
					is_offline: false,
					is_mock_suspect: false,
					device_id: 'dashboard-web'
				}
			});
			info = `Absen ${jenis} berhasil dicatat.`;
			await muat();
		} catch (err) {
			error =
				err instanceof ApiError
					? err.message
					: err instanceof Error
						? err.message
						: 'Gagal mencatat absensi';
		} finally {
			memproses = false;
		}
	}

	onMount(() => {
		dari = awalBulan;
		sampai = hariIni;
		muat();
	});
</script>

<svelte:head><title>Absensi Saya — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card p-4">
		<div class="flex flex-wrap items-center justify-between gap-3">
			<div>
				<h2 class="text-sm font-semibold text-slate-800">Kehadiran Hari Ini</h2>
				<p class="text-xs text-slate-500">
					{new Date().toLocaleDateString('id-ID', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric' })}
				</p>
			</div>
			<div class="flex gap-2">
				<Badge tone={sudahMasuk ? 'green' : 'slate'}>{sudahMasuk ? 'Sudah absen masuk' : 'Belum absen masuk'}</Badge>
				<Badge tone={sudahPulang ? 'green' : 'slate'}>{sudahPulang ? 'Sudah absen pulang' : 'Belum absen pulang'}</Badge>
			</div>
		</div>

		<div class="mt-4 flex flex-wrap gap-2">
			<button class="btn btn-primary" disabled={memproses || sudahMasuk} onclick={() => absen('masuk')}>
				<Icon name="check" size={16} /> Absen Masuk
			</button>
			<button
				class="btn btn-gold"
				disabled={memproses || !sudahMasuk || sudahPulang}
				onclick={() => absen('pulang')}
			>
				<Icon name="clock" size={16} /> Absen Pulang
			</button>
			<p class="self-center text-[11px] text-slate-500">
				Lokasi diambil dari GPS perangkat. Absensi utama tetap melalui aplikasi Android.
			</p>
		</div>

		{#if info}
			<p class="mt-3 rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>
		{/if}
		{#if error}
			<p class="mt-3 rounded-lg bg-rose-50 px-3 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
		{/if}
	</div>

	<div class="card overflow-hidden">
		<div class="flex flex-wrap items-end justify-between gap-3 border-b border-slate-200 px-4 py-3">
			<h2 class="text-sm font-semibold text-slate-700">Riwayat Absensi</h2>
			<div class="flex flex-wrap items-end gap-2">
				<div>
					<label class="label" for="dari">Dari</label>
					<input id="dari" type="date" class="input" bind:value={dari} onchange={muat} />
				</div>
				<div>
					<label class="label" for="sampai">Sampai</label>
					<input id="sampai" type="date" class="input" bind:value={sampai} onchange={muat} />
				</div>
				<button class="btn btn-ghost border border-slate-300" onclick={muat}>
					<Icon name="refresh" size={16} />
				</button>
			</div>
		</div>

		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">Tanggal</th>
						<th class="th">Jenis</th>
						<th class="th">Status</th>
						<th class="th">Jam</th>
						<th class="th text-center">Jarak</th>
						<th class="th">Catatan</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each dataHalaman as row}
						<tr class="hover:bg-slate-50/70">
							<td class="td text-slate-700">
								{new Date(row.tanggal + 'T00:00:00').toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' })}
							</td>
							<td class="td capitalize text-slate-600">{row.jenis}</td>
							<td class="td"><Badge tone={nada(row.status_kehadiran)}>{LABEL_STATUS_KEHADIRAN[row.status_kehadiran]}</Badge></td>
							<td class="td font-medium text-slate-800">{row.jam}</td>
							<td class="td text-center text-slate-600">
								{row.jarak_m != null ? `${row.jarak_m.toFixed(0)} m` : '-'}
								{#if row.dalam_radius === false && row.status_kehadiran === 'dinas_kantor'}
									<span class="ml-1 text-[10px] text-rose-600">luar radius</span>
								{/if}
							</td>
							<td class="td text-xs text-slate-500">
								{#if row.perlu_verifikasi}<span class="text-amber-600">menunggu verifikasi</span>{/if}
								{#if row.catatan_verifikator} · {row.catatan_verifikator}{/if}
								{#if !row.perlu_verifikasi && !row.catatan_verifikator}-{/if}
							</td>
						</tr>
					{:else}
						<tr>
							<td class="td text-center text-slate-500" colspan="6">
								{memuat ? 'Memuat…' : 'Belum ada data absensi pada rentang ini.'}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<Pagination page={halamanAman} perPage={perPage} total={data.length} label="riwayat" onchange={(p) => (halaman = p)} />
	</div>
</div>
