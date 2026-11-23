<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, apiBlob, apiBlobUrl, ApiError } from '#lib/api/client';
	import type {
		AbsensiHarian,
		FotoAbsensi,
		ListResponse,
		Personel,
		RekapAbsensi,
		Satker,
		StatusKehadiran
	} from '#lib/types';
	import { adalahPimpinanSaja } from '#lib/stores/session.svelte';
	import { LABEL_STATUS_KEHADIRAN } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	type TabId = 'rekap' | 'verifikasi' | 'detail';

	const TAB: { id: TabId; label: string; keterangan: string }[] = [
		{
			id: 'rekap',
			label: 'Rekap Kehadiran',
			keterangan: 'Jumlah kehadiran, keterlambatan, dan ketidakhadiran per personel.'
		},
		{
			id: 'verifikasi',
			label: 'Verifikasi',
			keterangan: 'Absensi offline, mock GPS, atau di luar radius yang menunggu keputusan.'
		},
		{
			id: 'detail',
			label: 'Detail per Orang',
			keterangan: 'Riwayat absensi harian satu personel pada periode terpilih.'
		}
	];

	let tab = $state<TabId>('rekap');
	const tabTampil = $derived(
		TAB.filter((item) => !(item.id === 'verifikasi' && adalahPimpinanSaja()))
	);
	let satkerList = $state<Satker[]>([]);
	let personelList = $state<Personel[]>([]);
	let siapMuat = $state(false);

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

	function awalBulan() {
		return `${tanggalHariIni().slice(0, 7)}-01`;
	}

	function rentangBawaan() {
		return { dari: awalBulan(), sampai: tanggalHariIni() };
	}

	// Filter dipisah per tab agar tidak saling mengubah.
	let filter = $state({
		rekap: { ...rentangBawaan(), satker_id: '', personel_id: '' },
		verifikasi: { ...rentangBawaan(), satker_id: '', personel_id: '' },
		detail: { ...rentangBawaan(), satker_id: '', personel_id: '' }
	});

	let rekap = $state<RekapAbsensi[]>([]);
	let rekapRentang = $state('');
	let halamanRekap = $state(1);

	let daftarVerifikasi = $state<ListResponse<AbsensiHarian> | null>(null);
	let halamanVerifikasi = $state(1);

	let detail = $state<ListResponse<AbsensiHarian> | null>(null);
	let halamanDetail = $state(1);

	let memuat = $state(false);
	let mengekspor = $state(false);
	let error = $state<string | null>(null);

	let pilihVerifikasi = $state<AbsensiHarian | null>(null);
	let catatan = $state('');
	let fotoVerifikasi = $state<string | null>(null);

	type ItemGaleri = { url: string; caption: string };
	let galeri = $state<{ nama: string; memuat: boolean; item: ItemGaleri[] } | null>(null);
	let fotoDetail = $state<ItemGaleri | null>(null);

	const perPage = 10;
	const totalHalamanRekap = $derived(Math.max(1, Math.ceil(rekap.length / perPage)));
	const halamanRekapAman = $derived(Math.min(halamanRekap, totalHalamanRekap));
	const rekapHalaman = $derived(
		rekap.slice((halamanRekapAman - 1) * perPage, halamanRekapAman * perPage)
	);

	const tabAktif = $derived(tabTampil.find((item) => item.id === tab) ?? TAB[0]);
	const satkerMap = $derived(new Map(satkerList.map((s) => [s.id, s.nama])));
	const detailPersonel = $derived(
		personelList.find((p) => p.id === filter.detail.personel_id) ?? null
	);

	function personelUntuk(satkerId: string) {
		return satkerId ? personelList.filter((p) => p.satker_id === satkerId) : personelList;
	}

	function tanggalSingkat(tanggal: string) {
		return new Date(`${tanggal}T00:00:00`).toLocaleDateString('id-ID', {
			day: '2-digit',
			month: 'short',
			year: 'numeric'
		});
	}

	function namaSatker(row: AbsensiHarian) {
		return satkerMap.get(row.satker_id) ?? row.unit ?? '-';
	}

	function statusHarian(row: AbsensiHarian): StatusKehadiran | null {
		return row.status_masuk ?? row.status_pulang;
	}

	function telatHarian(row: AbsensiHarian) {
		return row.menit_telat > 0 ? `${row.menit_telat} mnt` : '-';
	}

	function waktuLengkap(nilai: string) {
		return new Date(nilai).toLocaleString('id-ID', {
			day: '2-digit',
			month: 'short',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function nadaStatus(status: StatusKehadiran) {
		if (
			status === 'dinas_kantor' ||
			status === 'dinas_luar' ||
			status === 'piket' ||
			status === 'ph'
		) {
			return 'green';
		}
		if (status === 'tanpa_keterangan') return 'red';
		if (status === 'cuti' || status === 'izin' || status === 'sakit') return 'amber';
		return 'blue';
	}

	function rentangTeks(dari: string, sampai: string) {
		return `${tanggalSingkat(dari)} – ${tanggalSingkat(sampai)}`;
	}

	async function muatRekap() {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams({
				dari: filter.rekap.dari,
				sampai: filter.rekap.sampai
			});
			if (filter.rekap.satker_id) params.set('satker_id', filter.rekap.satker_id);

			const hasil = await api<{
				rentang: { dari: string; sampai: string };
				data: RekapAbsensi[];
			}>(`/absensi/rekap?${params}`);
			rekap = hasil.data;
			rekapRentang = rentangTeks(hasil.rentang.dari, hasil.rentang.sampai);
			halamanRekap = 1;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat rekap kehadiran';
		} finally {
			memuat = false;
		}
	}

	async function muatVerifikasi(page = 1) {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams();
			params.set('perlu_verifikasi', 'true');
			params.set('page', String(page));
			params.set('per_page', String(perPage));

			params.set('dari', filter.verifikasi.dari);
			params.set('sampai', filter.verifikasi.sampai);
			if (filter.verifikasi.satker_id) params.set('satker_id', filter.verifikasi.satker_id);
			if (filter.verifikasi.personel_id) params.set('personel_id', filter.verifikasi.personel_id);

			const hasil = await api<ListResponse<AbsensiHarian>>(`/absensi/harian?${params}`);
			daftarVerifikasi = hasil;
			halamanVerifikasi = hasil.meta.page;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat daftar verifikasi';
		} finally {
			memuat = false;
		}
	}

	async function muatDetail(page = 1) {
		if (!filter.detail.personel_id) {
			detail = null;
			return;
		}

		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams();
			params.set('personel_id', filter.detail.personel_id);
			params.set('page', String(page));
			params.set('per_page', String(perPage));

			params.set('dari', filter.detail.dari);
			params.set('sampai', filter.detail.sampai);
			if (filter.detail.satker_id) params.set('satker_id', filter.detail.satker_id);

			const hasil = await api<ListResponse<AbsensiHarian>>(`/absensi/harian?${params}`);
			detail = hasil;
			halamanDetail = hasil.meta.page;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat detail absensi';
		} finally {
			memuat = false;
		}
	}

	function muatTab(id: TabId, page = 1) {
		if (id === 'verifikasi' && adalahPimpinanSaja()) return muatRekap();
		if (id === 'rekap') return muatRekap();
		if (id === 'verifikasi') return muatVerifikasi(page);
		return muatDetail(page);
	}

	function pilihTab(nilai: TabId) {
		const id: TabId = nilai === 'verifikasi' && adalahPimpinanSaja() ? 'rekap' : nilai;
		tab = id;
		error = null;
		if (page.url.searchParams.get('tab') !== id) {
			goto(`/absensi/rekap?tab=${id}`, { replace: true, reset: false });
		}
		muatTab(id);
	}

	let tabUrlTerakhir = '';

	// Sidebar dapat berpindah submenu pada route yang sama; ikuti nilai `?tab=`.
	$effect(() => {
		const nilai = page.url.searchParams.get('tab') ?? 'rekap';
		if (nilai === tabUrlTerakhir) return;
		tabUrlTerakhir = nilai;
		if (nilai === 'verifikasi' && adalahPimpinanSaja()) {
			goto('/absensi/rekap?tab=rekap', { replace: true, reset: false });
			return;
		}
		if (nilai === 'rekap' || nilai === 'verifikasi' || nilai === 'detail') {
			if (nilai !== tab) {
				tab = nilai;
				if (siapMuat) muatTab(nilai);
			}
		}
	});

	function ubahSatker(id: TabId) {
		const aktif = filter[id];
		const daftar = personelUntuk(aktif.satker_id);
		if (aktif.personel_id && !daftar.some((p) => p.id === aktif.personel_id)) {
			aktif.personel_id = '';
		}
		muatTab(id);
	}

	async function eksporPdf() {
		const aktif = filter[tab];
		if (tab === 'detail' && !aktif.personel_id) {
			error = 'Pilih personel terlebih dahulu sebelum mengekspor detail absensi.';
			return;
		}

		mengekspor = true;
		error = null;
		try {
			const params = new URLSearchParams({
				dari: aktif.dari,
				sampai: aktif.sampai
			});
			if (aktif.satker_id) params.set('satker_id', aktif.satker_id);
			if (tab !== 'rekap' && aktif.personel_id) params.set('personel_id', aktif.personel_id);

			const jalur =
				tab === 'rekap'
					? '/absensi/rekap/ekspor'
					: tab === 'verifikasi'
						? '/absensi/verifikasi/ekspor'
						: '/absensi/detail/ekspor';

			const blob = await apiBlob(`${jalur}?${params}`);
			const url = URL.createObjectURL(blob);
			const tautan = document.createElement('a');
			tautan.href = url;
			tautan.download = `${tab}-absensi-${new Date().toISOString().slice(0, 10)}.pdf`;
			tautan.click();
			URL.revokeObjectURL(url);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal membuat PDF';
		} finally {
			mengekspor = false;
		}
	}

	async function bukaVerifikasi(row: AbsensiHarian) {
		pilihVerifikasi = row;
		catatan = '';
		if (fotoVerifikasi) URL.revokeObjectURL(fotoVerifikasi);
		fotoVerifikasi = null;

		if (row.foto_url) {
			try {
				fotoVerifikasi = await apiBlobUrl(row.foto_url);
			} catch {
				fotoVerifikasi = null;
			}
		}
	}

	function tutupVerifikasi() {
		pilihVerifikasi = null;
		catatan = '';
		if (fotoVerifikasi) URL.revokeObjectURL(fotoVerifikasi);
		fotoVerifikasi = null;
	}

	function tutupGaleri() {
		galeri?.item.forEach((item) => URL.revokeObjectURL(item.url));
		galeri = null;
	}

	async function bukaGaleri(row: RekapAbsensi) {
		tutupGaleri();
		galeri = { nama: row.nama, memuat: true, item: [] };
		try {
			const params = new URLSearchParams({
				personel_id: row.personel_id,
				dari: filter.rekap.dari,
				sampai: filter.rekap.sampai
			});
			if (filter.rekap.satker_id) params.set('satker_id', filter.rekap.satker_id);

			const hasil = await api<{ data: FotoAbsensi[] }>(`/absensi/foto?${params}`);
			const item = await Promise.all(
				hasil.data.map(async (foto) => ({
					url: await apiBlobUrl(foto.foto_url),
					caption: `${tanggalSingkat(foto.tanggal)} · ${foto.jenis === 'masuk' ? 'Masuk' : 'Pulang'} ${foto.jam} · ${LABEL_STATUS_KEHADIRAN[foto.status_kehadiran]}`
				}))
			);
			if (galeri) galeri = { ...galeri, memuat: false, item };
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat dokumentasi absensi';
			if (galeri) galeri = { ...galeri, memuat: false };
		}
	}

	async function bukaFotoDetail(url: string, caption: string) {
		if (fotoDetail) URL.revokeObjectURL(fotoDetail.url);
		fotoDetail = null;
		try {
			fotoDetail = { url: await apiBlobUrl(url), caption };
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat foto';
		}
	}

	function tutupFotoDetail() {
		if (fotoDetail) URL.revokeObjectURL(fotoDetail.url);
		fotoDetail = null;
	}

	async function verifikasi() {
		if (!pilihVerifikasi) return;
		const ids = pilihVerifikasi.perlu_verifikasi_ids;
		let pesan: string | null = null;

		try {
			for (const id of ids) {
				await api(`/absensi/${id}/verifikasi`, {
					method: 'POST',
					body: { catatan: catatan || null }
				});
			}
		} catch (err) {
			pesan = err instanceof ApiError ? err.message : 'Gagal memverifikasi absensi';
		} finally {
			tutupVerifikasi();
			const sisa = Math.max(0, (daftarVerifikasi?.meta.total ?? 1) - 1);
			const halamanBaru = Math.min(halamanVerifikasi, Math.max(1, Math.ceil(sisa / perPage)));
			await muatVerifikasi(halamanBaru);
			if (pesan) error = pesan;
		}
	}

	onMount(async () => {
		const [s, orang] = await Promise.all([
			api<Satker[]>('/satker'),
			api<ListResponse<Personel>>('/personel?per_page=200')
		]);
		satkerList = s;
		personelList = orang.data;

		const tabAwal = page.url.searchParams.get('tab');
		if (tabAwal === 'detail') tab = 'detail';
		else if (tabAwal === 'verifikasi') tab = adalahPimpinanSaja() ? 'rekap' : 'verifikasi';

		siapMuat = true;
		await muatTab(tab);
	});
</script>

<svelte:head><title>Rekap Absensi — Absensi & Strong Point</title></svelte:head>

<div class="space-y-4">
	<div class="card space-y-3 p-4">
		<div class="flex flex-wrap items-start justify-between gap-3">
			<div>
				<h2 class="text-sm font-semibold text-slate-800">Rekap Absensi</h2>
				<p class="text-xs text-slate-500">{tabAktif.keterangan}</p>
			</div>
			<button
				class="btn btn-ghost border border-slate-300"
				disabled={mengekspor}
				onclick={eksporPdf}
			>
				<Icon name="download" size={16} />
				{mengekspor ? 'Menyiapkan PDF…' : 'Export PDF'}
			</button>
		</div>

		<div class="flex flex-wrap gap-1 border-b border-slate-200 pb-2">
			{#each tabTampil as item (item.id)}
				<button
					class="rounded-lg px-3 py-1.5 text-sm font-medium transition {tab === item.id
						? 'bg-navy-800 text-white'
						: 'text-slate-600 hover:bg-slate-100'}"
					onclick={() => pilihTab(item.id)}
				>
					{item.label}
				</button>
			{/each}
		</div>

		{#if tab === 'rekap'}
			<div class="flex flex-wrap items-end gap-3">
				<div>
					<label class="label" for="rekap-dari">Dari Tanggal</label>
					<input
						id="rekap-dari"
						type="date"
						class="input"
						bind:value={filter.rekap.dari}
						onchange={() => muatRekap()}
					/>
				</div>
				<div>
					<label class="label" for="rekap-sampai">Sampai Tanggal</label>
					<input
						id="rekap-sampai"
						type="date"
						class="input"
						bind:value={filter.rekap.sampai}
						onchange={() => muatRekap()}
					/>
				</div>
				<div class="min-w-48">
					<label class="label" for="rekap-satker">Satker</label>
					<select
						id="rekap-satker"
						class="input"
						bind:value={filter.rekap.satker_id}
						onchange={() => muatRekap()}
					>
						<option value="">Semua Satker</option>
						{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
					</select>
				</div>
			</div>
		{:else if tab === 'verifikasi'}
			<div class="flex flex-wrap items-end gap-3">
				<div>
					<label class="label" for="verif-dari">Dari Tanggal</label>
					<input
						id="verif-dari"
						type="date"
						class="input"
						bind:value={filter.verifikasi.dari}
						onchange={() => muatVerifikasi()}
					/>
				</div>
				<div>
					<label class="label" for="verif-sampai">Sampai Tanggal</label>
					<input
						id="verif-sampai"
						type="date"
						class="input"
						bind:value={filter.verifikasi.sampai}
						onchange={() => muatVerifikasi()}
					/>
				</div>
				<div class="min-w-48">
					<label class="label" for="verif-satker">Satker</label>
					<select
						id="verif-satker"
						class="input"
						bind:value={filter.verifikasi.satker_id}
						onchange={() => ubahSatker('verifikasi')}
					>
						<option value="">Semua Satker</option>
						{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
					</select>
				</div>
				<div class="min-w-56">
					<label class="label" for="verif-personel">Personel</label>
					<select
						id="verif-personel"
						class="input"
						bind:value={filter.verifikasi.personel_id}
						onchange={() => muatVerifikasi()}
					>
						<option value="">Semua Personel</option>
						{#each personelUntuk(filter.verifikasi.satker_id) as orang}
							<option value={orang.id}>{orang.nama} ({orang.nrp})</option>
						{/each}
					</select>
				</div>
			</div>
		{:else}
			<div class="flex flex-wrap items-end gap-3">
				<div>
					<label class="label" for="detail-dari">Dari Tanggal</label>
					<input
						id="detail-dari"
						type="date"
						class="input"
						bind:value={filter.detail.dari}
						onchange={() => muatDetail()}
					/>
				</div>
				<div>
					<label class="label" for="detail-sampai">Sampai Tanggal</label>
					<input
						id="detail-sampai"
						type="date"
						class="input"
						bind:value={filter.detail.sampai}
						onchange={() => muatDetail()}
					/>
				</div>
				<div class="min-w-48">
					<label class="label" for="detail-satker">Satker</label>
					<select
						id="detail-satker"
						class="input"
						bind:value={filter.detail.satker_id}
						onchange={() => ubahSatker('detail')}
					>
						<option value="">Semua Satker</option>
						{#each satkerList as s}<option value={s.id}>{s.nama}</option>{/each}
					</select>
				</div>
				<div class="min-w-56">
					<label class="label" for="detail-personel">Personel</label>
					<select
						id="detail-personel"
						class="input"
						bind:value={filter.detail.personel_id}
						onchange={() => muatDetail()}
					>
						<option value="">Pilih personel…</option>
						{#each personelUntuk(filter.detail.satker_id) as orang}
							<option value={orang.id}>{orang.nama} ({orang.nrp})</option>
						{/each}
					</select>
				</div>
			</div>
		{/if}
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	{#if tab === 'rekap'}
		<div class="card overflow-hidden">
			<div class="flex items-center justify-between border-b border-slate-200 px-4 py-3">
				<h2 class="text-sm font-semibold text-slate-700">
					Rekap Kehadiran {rekapRentang}
				</h2>
				<span class="text-xs text-slate-500">{rekap.length} personel</span>
			</div>

			<div class="max-h-[70vh] overflow-auto">
				<table class="w-full">
					<thead class="sticky top-0 bg-slate-50">
						<tr>
							<th class="th">Personel</th>
							<th class="th">Unit</th>
							<th class="th text-center">Hadir</th>
							<th class="th text-center">Telat</th>
							<th class="th text-center">Menit</th>
							<th class="th text-center">DL</th>
							<th class="th text-center">Piket</th>
							<th class="th text-center">Cuti</th>
							<th class="th text-center">Izin</th>
							<th class="th text-center">Sakit</th>
							<th class="th text-center">PH</th>
							<th class="th text-center">TK</th>
							<th class="th text-center">Menunggu</th>
							<th class="th text-center">Dokumentasi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each rekapHalaman as row}
							<tr class="hover:bg-slate-50/70">
								<td class="td">
									<p class="font-medium text-slate-800">{row.nama}</p>
									<p class="text-[11px] text-slate-500">{row.pangkat} · {row.nrp}</p>
								</td>
								<td class="td text-slate-600">{row.unit ?? '-'}</td>
								<td class="td text-center font-semibold text-emerald-700">{row.hadir}</td>
								<td
									class="td text-center {row.telat > 0 ? 'text-amber-700' : 'text-slate-400'}"
								>
									{row.telat}
								</td>
								<td
									class="td text-center {row.menit_telat > 0
										? 'text-amber-700'
										: 'text-slate-400'}"
								>
									{row.menit_telat > 0 ? row.menit_telat : '-'}
								</td>
								<td class="td text-center text-slate-600">{row.dinas_luar}</td>
								<td class="td text-center text-slate-600">{row.piket}</td>
								<td class="td text-center text-slate-600">{row.cuti}</td>
								<td class="td text-center text-slate-600">{row.izin}</td>
								<td class="td text-center text-slate-600">{row.sakit}</td>
								<td class="td text-center text-slate-600">{row.libur_nasional}</td>
								<td
									class="td text-center {row.tanpa_keterangan > 0
										? 'font-semibold text-rose-700'
										: 'text-slate-400'}"
								>
									{row.tanpa_keterangan}
								</td>
								<td class="td text-center">
									{#if row.menunggu_verifikasi > 0}
										<Badge tone="amber">{row.menunggu_verifikasi}</Badge>
									{:else}
										<span class="text-slate-300">-</span>
									{/if}
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
							<tr>
								<td class="td text-center text-slate-500" colspan="14">
									{memuat ? 'Memuat…' : 'Belum ada data.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<Pagination
				page={halamanRekapAman}
				perPage={perPage}
				total={rekap.length}
				label="personel"
				onchange={(p) => (halamanRekap = p)}
			/>
		</div>
	{:else if tab === 'verifikasi'}
		<div class="card overflow-hidden">
			<div class="flex items-center justify-between border-b border-slate-200 px-4 py-3">
				<div class="flex items-center gap-2">
					<Icon name="alert" size={16} class="text-amber-600" />
					<h2 class="text-sm font-semibold text-slate-700">Menunggu Verifikasi</h2>
				</div>
				<span class="text-xs text-slate-500">
					{daftarVerifikasi?.meta.total ?? 0} tanggal pada periode terpilih
				</span>
			</div>

			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Personel</th>
							<th class="th">Satker / Unit</th>
							<th class="th">Tanggal</th>
							<th class="th text-center">Masuk</th>
							<th class="th text-center">Pulang</th>
							<th class="th">Status</th>
							<th class="th text-center">Telat</th>
							<th class="th">Kondisi</th>
							<th class="th no-print">Aksi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each daftarVerifikasi?.data ?? [] as row (row.personel_id + row.tanggal)}
							{@const status = statusHarian(row)}
							<tr class="hover:bg-slate-50/70">
								<td class="td">
									<p class="font-medium text-slate-800">{row.personel_nama}</p>
									<p class="text-[11px] text-slate-500">
										{row.personel_pangkat} · {row.personel_nrp}
									</p>
								</td>
								<td class="td text-slate-600">
									{namaSatker(row)}
									{#if row.unit}
										<p class="text-[11px] text-slate-500">{row.unit}</p>
									{/if}
								</td>
								<td class="td text-slate-600">{tanggalSingkat(row.tanggal)}</td>
								<td class="td text-center font-medium text-slate-800">
									{row.masuk_jam ?? '—'}
								</td>
								<td class="td text-center font-medium text-slate-800">
									{row.pulang_jam ?? '—'}
								</td>
								<td class="td">
									{#if status}
										<Badge tone={nadaStatus(status)}>{LABEL_STATUS_KEHADIRAN[status]}</Badge>
									{:else}
										<span class="text-xs text-slate-400">—</span>
									{/if}
								</td>
								<td
									class="td text-center {row.menit_telat > 0
										? 'font-medium text-amber-700'
										: 'text-slate-400'}"
								>
									{telatHarian(row)}
								</td>
								<td class="td">
									<div class="flex flex-wrap gap-1">
										{#if row.is_offline}<Badge tone="amber">offline</Badge>{/if}
										{#if row.is_mock_suspect}<Badge tone="red">mock GPS</Badge>{/if}
										{#if row.luar_radius}<Badge tone="red">luar radius</Badge>{/if}
										{#if !row.is_offline && !row.is_mock_suspect && !row.luar_radius}
											<Badge tone="green">normal</Badge>
										{/if}
									</div>
								</td>
								<td class="td no-print">
									<button
										class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
										onclick={() => bukaVerifikasi(row)}
									>
										Verifikasi
									</button>
								</td>
							</tr>
						{:else}
							<tr>
								<td class="td text-center text-slate-500" colspan="9">
									{memuat
										? 'Memuat…'
										: 'Tidak ada absensi yang menunggu verifikasi pada filter ini.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			<Pagination
				page={halamanVerifikasi}
				perPage={perPage}
				total={daftarVerifikasi?.meta.total ?? 0}
				label="tanggal"
				onchange={(p) => muatVerifikasi(p)}
			/>
		</div>
	{:else}
		{#if !filter.detail.personel_id}
			<div class="card flex flex-col items-center gap-2 px-4 py-12 text-center">
				<Icon name="users" size={28} class="text-slate-300" />
				<p class="text-sm font-medium text-slate-700">Pilih personel terlebih dahulu</p>
				<p class="max-w-md text-xs text-slate-500">
					Detail absensi harian (tanggal, jam masuk/pulang, status, menit telat, dan
					catatan verifikator) ditampilkan per personel sesuai periode yang dipilih.
				</p>
			</div>
		{:else}
			{#if detailPersonel}
				<div class="card flex flex-wrap items-center justify-between gap-3 px-4 py-3">
					<div class="flex items-center gap-3">
						<div
							class="flex h-10 w-10 items-center justify-center rounded-full bg-navy-800 text-sm font-semibold text-white"
						>
							{detailPersonel.nama.charAt(0)}
						</div>
						<div>
							<p class="text-sm font-semibold text-slate-800">{detailPersonel.nama}</p>
							<p class="text-xs text-slate-500">
								{detailPersonel.pangkat} · {detailPersonel.nrp} · {detailPersonel.satker_nama}
							</p>
						</div>
					</div>
					<span class="text-xs text-slate-500">
						{detail?.meta.total ?? 0} hari absensi
					</span>
				</div>
			{/if}

			<div class="card overflow-hidden">
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead class="bg-slate-50">
							<tr>
								<th class="th">Tanggal</th>
								<th class="th text-center">Masuk</th>
								<th class="th text-center">Pulang</th>
								<th class="th">Status</th>
								<th class="th text-center">Telat</th>
								<th class="th text-center">Radius</th>
								<th class="th">Kondisi</th>
								<th class="th">Verifikasi</th>
								<th class="th text-center">Dokumentasi</th>
								<th class="th">Catatan</th>
							</tr>
						</thead>
						<tbody class="divide-y divide-slate-100">
							{#each detail?.data ?? [] as row (row.personel_id + row.tanggal)}
								{@const status = statusHarian(row)}
								<tr class="hover:bg-slate-50/70">
									<td class="td text-slate-600">
										{tanggalSingkat(row.tanggal)}
										{#if row.ph_hadir}
											<Badge tone="blue">Strong point</Badge>
										{/if}
									</td>
									<td class="td text-center font-medium text-slate-800">
										{row.masuk_jam ?? '—'}
									</td>
									<td class="td text-center font-medium text-slate-800">
										{row.pulang_jam ?? '—'}
									</td>
									<td class="td">
										{#if status}
											<Badge tone={nadaStatus(status)}>{LABEL_STATUS_KEHADIRAN[status]}</Badge>
										{:else}
											<span class="text-xs text-slate-400">—</span>
										{/if}
									</td>
									<td
										class="td text-center {row.menit_telat > 0
											? 'font-medium text-amber-700'
											: 'text-slate-400'}"
										title={row.ph_hadir ? 'Telat kantor diabaikan karena check-in strong point' : undefined}
									>
										{telatHarian(row)}
									</td>
									<td class="td text-center">
										{#if row.luar_radius}
											<Badge tone="red">Luar</Badge>
										{:else}
											<span class="text-xs text-slate-500">Dalam</span>
										{/if}
									</td>
									<td class="td">
										<div class="flex flex-wrap gap-1">
											{#if row.is_offline}<Badge tone="amber">offline</Badge>{/if}
											{#if row.is_mock_suspect}<Badge tone="red">mock GPS</Badge>{/if}
											{#if !row.is_offline && !row.is_mock_suspect}
												<span class="text-xs text-slate-400">-</span>
											{/if}
										</div>
									</td>
									<td class="td">
										{#if row.perlu_verifikasi}
											<Badge tone="amber">Menunggu</Badge>
										{:else if row.verified_at}
											<Badge tone="green">Terverifikasi</Badge>
											<p class="mt-1 text-[10px] text-slate-500">
												{waktuLengkap(row.verified_at)}
											</p>
										{:else}
											<span class="text-xs text-slate-400">-</span>
										{/if}
									</td>
									<td class="td">
										<div class="flex flex-wrap gap-1">
											{#if row.foto_masuk_url}
												<button
													class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
													onclick={() =>
														bukaFotoDetail(
															row.foto_masuk_url ?? '',
															`${tanggalSingkat(row.tanggal)} · Masuk ${row.masuk_jam ?? ''}`
														)}
												>
													<Icon name="image" size={14} /> Masuk
												</button>
											{/if}
											{#if row.foto_pulang_url}
												<button
													class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
													onclick={() =>
														bukaFotoDetail(
															row.foto_pulang_url ?? '',
															`${tanggalSingkat(row.tanggal)} · Pulang ${row.pulang_jam ?? ''}`
														)}
												>
													<Icon name="image" size={14} /> Pulang
												</button>
											{/if}
											{#if !row.foto_masuk_url && !row.foto_pulang_url}
												<span class="text-xs text-slate-400">-</span>
											{/if}
										</div>
									</td>
									<td class="td text-xs text-slate-600">
										{row.catatan_verifikator ?? '-'}
									</td>
								</tr>
							{:else}
								<tr>
									<td class="td text-center text-slate-500" colspan="10">
										{memuat
											? 'Memuat…'
											: 'Belum ada absensi untuk personel ini pada periode terpilih.'}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				<Pagination
					page={halamanDetail}
					perPage={perPage}
					total={detail?.meta.total ?? 0}
					label="hari absensi"
					onchange={(p) => muatDetail(p)}
				/>
			</div>
		{/if}
	{/if}
</div>

{#snippet aksiVerifikasi()}
	<div class="flex justify-end gap-2">
		<button class="btn btn-ghost" onclick={tutupVerifikasi}>Batal</button>
		<button class="btn btn-primary" onclick={verifikasi}>
			<Icon name="check" size={16} /> Setujui
		</button>
	</div>
{/snippet}

<Modal
	open={!!pilihVerifikasi}
	title="Verifikasi Absensi"
	footer={aksiVerifikasi}
	onclose={tutupVerifikasi}
>
	{#if pilihVerifikasi}
		{@const status = statusHarian(pilihVerifikasi)}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="font-medium text-slate-800">{pilihVerifikasi.personel_nama}</p>
				<p class="text-xs text-slate-500">
					{pilihVerifikasi.personel_pangkat} · {pilihVerifikasi.personel_nrp} ·
					{namaSatker(pilihVerifikasi)}
				</p>
				<p class="mt-1 text-xs text-slate-600">
					{tanggalSingkat(pilihVerifikasi.tanggal)} · Masuk:
					{pilihVerifikasi.masuk_jam ?? '—'} · Pulang: {pilihVerifikasi.pulang_jam ?? '—'}
				</p>
				<p class="mt-1 text-xs text-slate-600">
					Status: {status ? LABEL_STATUS_KEHADIRAN[status] : '—'}
					{#if pilihVerifikasi.menit_telat > 0}
						· Telat {pilihVerifikasi.menit_telat} menit
					{/if}
					· Radius: {pilihVerifikasi.luar_radius ? 'di luar' : 'di dalam'}
				</p>
				{#if pilihVerifikasi.perlu_verifikasi_ids.length > 1}
					<p class="mt-1 text-xs text-amber-700">
						{pilihVerifikasi.perlu_verifikasi_ids.length} catatan absensi pada tanggal ini akan
						diverifikasi sekaligus.
					</p>
				{/if}
				<div class="mt-2 flex flex-wrap gap-1">
					{#if pilihVerifikasi.is_offline}<Badge tone="amber">offline</Badge>{/if}
					{#if pilihVerifikasi.is_mock_suspect}<Badge tone="red">mock GPS</Badge>{/if}
					{#if !pilihVerifikasi.is_offline && !pilihVerifikasi.is_mock_suspect && !pilihVerifikasi.luar_radius}
						<Badge tone="green">normal</Badge>
					{/if}
				</div>
			</div>

			{#if fotoVerifikasi}
				<img
					src={fotoVerifikasi}
					alt="Foto absensi {pilihVerifikasi.personel_nama}"
					class="max-h-64 w-full rounded-lg object-contain ring-1 ring-slate-200"
				/>
			{/if}

			<div>
				<label class="label" for="catatan">Catatan Verifikator</label>
				<textarea
					id="catatan"
					class="input h-24 resize-none"
					placeholder="Opsional, mis. alasan tetap disetujui"
					bind:value={catatan}
				></textarea>
			</div>
		</div>
	{/if}
</Modal>

<Modal
	open={!!galeri}
	title={galeri ? `Dokumentasi Absensi — ${galeri.nama}` : 'Dokumentasi Absensi'}
	lebar="max-w-4xl"
	onclose={tutupGaleri}
>
	{#if galeri}
		{#if galeri.memuat}
			<p class="py-8 text-center text-sm text-slate-500">Memuat dokumentasi…</p>
		{:else if galeri.item.length === 0}
			<p class="py-8 text-center text-sm text-slate-500">
				Belum ada foto dokumentasi absensi pada periode ini.
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

<Modal
	open={!!fotoDetail}
	title="Dokumentasi Absensi"
	lebar="max-w-3xl"
	onclose={tutupFotoDetail}
>
	{#if fotoDetail}
		<div class="space-y-2">
			<img
				src={fotoDetail.url}
				alt={fotoDetail.caption}
				class="max-h-[70vh] w-full rounded-lg object-contain ring-1 ring-slate-200"
			/>
			<p class="text-xs text-slate-500">{fotoDetail.caption}</p>
		</div>
	{/if}
</Modal>
