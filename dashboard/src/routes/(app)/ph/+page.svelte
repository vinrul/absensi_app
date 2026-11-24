<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, apiBlob, apiBlobUrl, ApiError } from '#lib/api/client';
	import { adalahAdmin, session } from '#lib/stores/session.svelte';
	import type {
		AbsensiPh,
		FotoPh,
		JadwalPh,
		ListResponse,
		PenempatanPh,
		Personel,
		RekapPh,
		RekapPhHarian,
		Satker,
		StatusJadwalPh,
		StrongPoint
	} from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Combobox from '#lib/components/Combobox.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';
	import PetaPicker from '#lib/components/PetaPicker.svelte';
	import PetaRekapHarianPh from '#lib/components/PetaRekapHarianPh.svelte';

	type Tab = 'jadwal' | 'rekap' | 'harian' | 'penempatan' | 'titik';

	type FormJadwal = {
		id: string | null;
		satker_id: string;
		sesi: 'pagi' | 'sore';
		tanggal: string;
		tanggal_selesai: string;
		jam_mulai: string;
		jam_selesai: string;
		regu: string;
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

	const HARI = ['Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu', 'Minggu'];
	const TAB: { id: Tab; label: string; roles: 'dashboard' | 'admin' | 'titik' }[] = [
		{ id: 'jadwal', label: 'Jadwal', roles: 'dashboard' },
		{ id: 'rekap', label: 'Rekap', roles: 'dashboard' },
		{ id: 'harian', label: 'Rekap Harian', roles: 'dashboard' },
		{ id: 'penempatan', label: 'Penempatan Anggota', roles: 'admin' },
		{ id: 'titik', label: 'Titik Strong Point', roles: 'titik' }
	];

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

	function geserTanggal(tanggal: string, hari: number) {
		const nilai = new Date(`${tanggal}T00:00:00Z`);
		nilai.setUTCDate(nilai.getUTCDate() + hari);
		return nilai.toISOString().slice(0, 10);
	}

	function akhirBulan(label: string) {
		const [tahun, bulan] = label.split('-').map(Number);
		if (!Number.isFinite(tahun) || !Number.isFinite(bulan)) {
			return new Date().toISOString().slice(0, 10);
		}
		const berikut = new Date(Date.UTC(tahun, bulan, 1));
		berikut.setUTCDate(berikut.getUTCDate() - 1);
		return berikut.toISOString().slice(0, 10);
	}

	function labelBulan(tanggal: string) {
		return new Date(`${tanggal}T00:00:00`).toLocaleDateString('id-ID', {
			month: 'long',
			year: 'numeric'
		});
	}

	const hariIni = tanggalHariIni();
	const bulanIni = hariIni.slice(0, 7);
	let bulanJadwal = $state(bulanIni);
	let dariRekap = $state(`${hariIni.slice(0, 7)}-01`);
	let sampaiRekap = $state(hariIni);
	let tab = $state<Tab>('jadwal');
	let satkerId = $state('');
	let satkerList = $state<Satker[]>([]);
	let jadwal = $state<JadwalPh[]>([]);
	let statusJadwal = $state<StatusJadwalPh | null>(null);
	let rekap = $state<RekapPh[]>([]);
	let rekapHarian = $state<RekapPhHarian[]>([]);
	let modeRekapPh = $state<'ringkasan' | 'harian'>('ringkasan');
	let tanggalHarianPh = $state(hariIni);
	let harianPh = $state<RekapPhHarian[]>([]);
	let modeHarianPh = $state<'ringkasan' | 'peta'>('ringkasan');
	let kategoriHarianPhAktif = $state<KategoriHarianPh | null>(null);
	let detailId = $state('');
	let detailPersonel = $state<Personel | null>(null);
	let detailBaris = $state<RekapPhHarian[]>([]);
	let memuatDetail = $state(false);
	let rekapRentang = $state('');
	let personelSatker = $state<Personel[]>([]);
	let cariPersonelJadwal = $state('');
	let penempatan = $state<PenempatanPh[]>([]);
	let memuat = $state(true);
	let siap = $state(false);
	let mengekspor = $state(false);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);

	let formJadwalTerbuka = $state(false);
	let formJadwal = $state<FormJadwal>(kosongJadwal());
	let halamanJadwal = $state(1);
	let halamanRekap = $state(1);
	const perPage = 10;

	let detail = $state<JadwalPh | null>(null);
	let absensiDetail = $state<AbsensiPh[]>([]);

	let titikOwnerId = $state('');
	let titikList = $state<StrongPoint[]>([]);
	let formStrongTerbuka = $state(false);
	let formStrong = $state<FormStrong>(kosongStrong());

	let formTempatTerbuka = $state(false);
	let satkerTempat = $state('');
	let titikTempat = $state('');
	let titikTempatList = $state<StrongPoint[]>([]);
	let personelTempat = $state<Personel[]>([]);
	let anggotaTempat = $state<string[]>([]);
	let modeTempatPersonel = $state<string | null>(null);
	let memuatTempat = $state(false);
	let cariPersonelTempat = $state('');

	type ItemGaleri = { url: string; caption: string };
	let galeri = $state<{ nama: string; memuat: boolean; item: ItemGaleri[] } | null>(null);

	const admin = $derived(adalahAdmin());
	const peran = $derived(session.user?.roles ?? []);
	const bisaKelolaTitik = $derived(
		peran.some((role) => ['admin_sistem', 'admin_polda', 'admin_polres'].includes(role))
	);
	const ownerMap = $derived(new Map(satkerList.map((row) => [row.id, row])));
	const ownerOptions = $derived.by(() => {
		const daftar = satkerList.filter((row) => row.level === 'polda' || row.level === 'polres');
		if (peran.includes('admin_sistem')) return daftar;
		const scope = new Set(
			(session.user?.role_scopes ?? [])
				.filter((item) =>
					['admin_polda', 'admin_polres'].includes(item.role)
				)
				.map((item) => item.scope_satker_id)
				.filter((id): id is string => Boolean(id))
		);
		return daftar.filter((row) => scope.has(row.id));
	});
	const tabTampil = $derived(
		TAB.filter((item) => {
			if (item.roles === 'admin') return admin;
			if (item.roles === 'titik') return bisaKelolaTitik;
			return true;
		})
	);
	const totalHalamanJadwal = $derived(Math.max(1, Math.ceil(jadwal.length / perPage)));
	const halamanJadwalAman = $derived(Math.min(halamanJadwal, totalHalamanJadwal));
	const jadwalHalaman = $derived(
		jadwal.slice((halamanJadwalAman - 1) * perPage, halamanJadwalAman * perPage)
	);
	const totalHalamanRekap = $derived(Math.max(1, Math.ceil(rekap.length / perPage)));
	const halamanRekapAman = $derived(Math.min(halamanRekap, totalHalamanRekap));
	const rekapHalaman = $derived(
		rekap.slice((halamanRekapAman - 1) * perPage, halamanRekapAman * perPage)
	);

	type BarisMatriksPh = {
		personel_id: string;
		nama: string;
		pangkat: string;
		nrp: string;
		unit: string | null;
		sel: Map<string, RekapPhHarian[]>;
	};
	type KategoriHarianPh = {
		id: string;
		label: string;
		keterangan: string;
		warna: string;
		nilai: number | string;
		baris: RekapPhHarian[];
	};
	const tanggalRekapPh = $derived.by(() => {
		const hasil: string[] = [];
		let cursor = dariRekap;
		let jumlah = 0;
		while (cursor <= sampaiRekap && jumlah < 31) {
			hasil.push(cursor);
			cursor = geserTanggal(cursor, 1);
			jumlah += 1;
		}
		return hasil;
	});
	const matriksRekapPh = $derived.by(() => {
		const peta = new Map<string, BarisMatriksPh>();
		const tanggalSet = new Set(tanggalRekapPh);
		for (const row of rekapHarian) {
			if (!tanggalSet.has(row.tanggal)) continue;
			let baris = peta.get(row.personel_id);
			if (!baris) {
				baris = {
					personel_id: row.personel_id,
					nama: row.nama,
					pangkat: row.pangkat,
					nrp: row.nrp,
					unit: row.unit,
					sel: new Map()
				};
				peta.set(row.personel_id, baris);
			}
			const daftar = baris.sel.get(row.tanggal) ?? [];
			daftar.push(row);
			baris.sel.set(row.tanggal, daftar);
		}
		return [...peta.values()].sort((a, b) => a.nama.localeCompare(b.nama, 'id'));
	});

	const ringkasanDetailPh = $derived.by(() => {
		const baris = detailBaris;
		return {
			jadwal: baris.length,
			hadir: baris.filter((row) => row.masuk_jam !== null).length,
			hanyaCheckIn: baris.filter((row) => row.masuk_jam !== null && row.pulang_jam === null)
				.length,
			tidakSp: baris.filter((row) => row.masuk_jam === null).length,
			telat: baris.reduce((jumlah, row) => jumlah + row.menit_telat, 0),
			pulangCepat: baris.reduce((jumlah, row) => jumlah + row.menit_pulang_cepat, 0)
		};
	});
	const kartuDetailPh = $derived([
		{
			label: 'Jadwal',
			nilai: ringkasanDetailPh.jadwal,
			keterangan: 'Sesi terjadwal',
			warna: 'bg-sky-500'
		},
		{
			label: 'Hadir',
			nilai: ringkasanDetailPh.hadir,
			keterangan: 'Sesi dengan check-in',
			warna: 'bg-emerald-500'
		},
		{
			label: 'Hanya Check-in',
			nilai: ringkasanDetailPh.hanyaCheckIn,
			keterangan: 'Tanpa check-out',
			warna: 'bg-amber-500'
		},
		{
			label: 'Tidak SP',
			nilai: ringkasanDetailPh.tidakSp,
			keterangan: 'Tanpa check-in',
			warna: 'bg-rose-500'
		},
		{
			label: 'Telat',
			nilai: `${ringkasanDetailPh.telat} mnt`,
			keterangan: 'Total menit telat',
			warna: 'bg-amber-500'
		},
		{
			label: 'Pulang Cepat',
			nilai: `${ringkasanDetailPh.pulangCepat} mnt`,
			keterangan: 'Total menit pulang cepat',
			warna: 'bg-rose-500'
		}
	]);

	const ringkasanHarianPh = $derived.by(() => {
		const baris = harianPh;
		return {
			sesi: new Set(baris.map((row) => row.jadwal_ph_id)).size,
			personel: new Set(baris.map((row) => row.personel_id)).size,
			titik: new Set(baris.map((row) => row.strong_point_id)).size,
			hadir: baris.filter((row) => row.masuk_jam !== null).length,
			belum: baris.filter((row) => row.masuk_jam === null).length,
			tc: baris.filter((row) => row.masuk_jam !== null && row.pulang_jam === null).length,
			telat: baris.reduce((jumlah, row) => jumlah + row.menit_telat, 0),
			pulangCepat: baris.reduce((jumlah, row) => jumlah + row.menit_pulang_cepat, 0)
		};
	});
	const kartuHarianPh = $derived<KategoriHarianPh[]>([
		{
			id: 'personel',
			label: 'Personel',
			keterangan: 'Dijadwalkan',
			warna: 'bg-sky-500',
			nilai: ringkasanHarianPh.personel,
			baris: harianPh
		},
		{
			id: 'sesi',
			label: 'Sesi Jadwal',
			keterangan: 'Jadwal pagi/sore',
			warna: 'bg-slate-400',
			nilai: ringkasanHarianPh.sesi,
			baris: harianPh
		},
		{
			id: 'titik',
			label: 'Strong Point',
			keterangan: 'Titik dengan jadwal',
			warna: 'bg-amber-500',
			nilai: ringkasanHarianPh.titik,
			baris: harianPh
		},
		{
			id: 'hadir',
			label: 'Sesi Hadir',
			keterangan: 'Sesi dengan check-in',
			warna: 'bg-emerald-500',
			nilai: ringkasanHarianPh.hadir,
			baris: harianPh.filter((row) => row.masuk_jam !== null)
		},
		{
			id: 'belum',
			label: 'Sesi Belum',
			keterangan: 'Tidak strong point (TA)',
			warna: 'bg-rose-500',
			nilai: ringkasanHarianPh.belum,
			baris: harianPh.filter((row) => row.masuk_jam === null)
		},
		{
			id: 'tc',
			label: 'Sesi TC',
			keterangan: 'Check-in tanpa check-out',
			warna: 'bg-amber-500',
			nilai: ringkasanHarianPh.tc,
			baris: harianPh.filter((row) => row.masuk_jam !== null && row.pulang_jam === null)
		},
		{
			id: 'telat',
			label: 'Telat',
			keterangan: 'Total menit telat',
			warna: 'bg-amber-500',
			nilai: `${ringkasanHarianPh.telat} mnt`,
			baris: harianPh.filter((row) => row.menit_telat > 0)
		},
		{
			id: 'pulang_cepat',
			label: 'Pulang Cepat',
			keterangan: 'Total menit pulang cepat',
			warna: 'bg-rose-500',
			nilai: `${ringkasanHarianPh.pulangCepat} mnt`,
			baris: harianPh.filter((row) => row.menit_pulang_cepat > 0)
		}
	]);
	const dariAktif = $derived(
		tab === 'jadwal'
			? `${bulanJadwal || bulanIni}-01`
			: tab === 'harian'
				? tanggalHarianPh
				: dariRekap
	);
	const sampaiAktif = $derived(
		tab === 'jadwal'
			? akhirBulan(bulanJadwal || bulanIni)
			: tab === 'harian'
				? tanggalHarianPh
				: sampaiRekap
	);
	const rentangHari = $derived.by(() => {
		const hasil: string[] = [];
		let cursor = dariAktif;
		while (cursor <= sampaiAktif && hasil.length < 62) {
			hasil.push(cursor);
			cursor = geserTanggal(cursor, 1);
		}
		return hasil;
	});
	const tampilkanRoster = $derived(rentangHari.length > 0 && rentangHari.length <= 45);
	const satkerMap = $derived(new Map(satkerList.map((row) => [row.id, row.nama])));
	const satkerRowMap = $derived(new Map(satkerList.map((row) => [row.id, row])));
	const LABEL_LEVEL: Record<Satker['level'], string> = {
		polda: 'Polda',
		polres: 'Polres',
		satker: 'Satker'
	};
	const opsiSatker = $derived(
		satkerList.map((s) => ({ value: s.id, label: s.nama, grup: LABEL_LEVEL[s.level] }))
	);
	const opsiOwner = $derived(
		ownerOptions.map((s) => ({ value: s.id, label: s.nama, grup: LABEL_LEVEL[s.level] }))
	);
	const satkerTempatOptions = $derived(daftarSatkerTempat());
	const grupSatkerTempat = $derived.by(() => {
		return (['polda', 'polres', 'satker'] as const)
			.map((level) => ({
				level,
				label: LABEL_LEVEL[level],
				data: satkerTempatOptions.filter((row) => row.level === level)
			}))
			.filter((grup) => grup.data.length > 0);
	});
	const opsiSatkerTempat = $derived(
		grupSatkerTempat.flatMap((grup) =>
			grup.data.map((s) => ({ value: s.id, label: s.nama, grup: grup.label }))
		)
	);
	const personelJadwalTampil = $derived(
		personelSatker.filter((orang) => cocokCari(orang, cariPersonelJadwal))
	);
	const personelTempatTampil = $derived(
		personelTempat.filter((orang) => cocokCari(orang, cariPersonelTempat))
	);

	const roster = $derived.by(() => {
		type Anggota = { id: string; nama: string; pangkat: string; nrp: string };
		type SesiSel = { pagi?: JadwalPh; sore?: JadwalPh };
		type Grup = {
			satker_id: string;
			anggota: Map<string, Anggota>;
			hari: Map<string, Map<string, SesiSel>>;
		};
		const grup = new Map<string, Grup>();
		for (const row of jadwal) {
			let data = grup.get(row.satker_id);
			if (!data) {
				data = { satker_id: row.satker_id, anggota: new Map(), hari: new Map() };
				grup.set(row.satker_id, data);
			}
			for (const orang of row.personel) {
				if (!data.anggota.has(orang.personel_id)) {
					data.anggota.set(orang.personel_id, {
						id: orang.personel_id,
						nama: orang.nama,
						pangkat: orang.pangkat,
						nrp: orang.nrp
					});
				}
				let perHari = data.hari.get(orang.personel_id);
				if (!perHari) {
					perHari = new Map();
					data.hari.set(orang.personel_id, perHari);
				}
				const sel = perHari.get(row.tanggal) ?? {};
				if (row.sesi === 'sore') sel.sore = row;
				else sel.pagi = row;
				perHari.set(row.tanggal, sel);
			}
		}
		return [...grup.values()].map((data) => ({
			...data,
			anggota: [...data.anggota.values()].sort((a, b) => a.nama.localeCompare(b.nama))
		}));
	});

	const statusJadwalTampil = $derived(tab === 'jadwal' ? statusJadwal : null);

	function tanggalPendek(tanggal: string) {
		return new Date(`${tanggal}T00:00:00`).toLocaleDateString('id-ID', {
			day: '2-digit',
			month: 'short',
			year: 'numeric'
		});
	}

	function tanggalPanjang(tanggal: string) {
		return new Date(`${tanggal}T00:00:00`).toLocaleDateString('id-ID', {
			weekday: 'long',
			day: '2-digit',
			month: 'long',
			year: 'numeric'
		});
	}

	function namaHariSingkat(tanggal: string) {
		return new Date(`${tanggal}T00:00:00`).toLocaleDateString('id-ID', { weekday: 'short' });
	}

	function akhirPekan(tanggal: string) {
		const hari = new Date(`${tanggal}T00:00:00`).getDay();
		return hari === 0 || hari === 6;
	}

	function labelSesiPh(sesi: string | null) {
		if (sesi === 'pagi') return 'Pagi';
		if (sesi === 'sore') return 'Sore';
		return 'Sesi';
	}

	function validasiRentangRekap() {
		const awal = new Date(`${dariRekap}T00:00:00Z`).getTime();
		const akhir = new Date(`${sampaiRekap}T00:00:00Z`).getTime();
		if (!Number.isFinite(awal) || !Number.isFinite(akhir)) {
			return 'Tanggal rekap tidak valid.';
		}
		const hari = Math.floor((akhir - awal) / 86_400_000) + 1;
		if (hari <= 0) return 'Tanggal selesai harus setelah tanggal mulai.';
		if (hari > 31) return 'Rentang rekap maksimal 31 hari.';
		return null;
	}

	function kosongJadwal(): FormJadwal {
		return {
			id: null,
			satker_id: '',
			sesi: 'pagi',
			tanggal: hariIni,
			tanggal_selesai: hariIni,
			jam_mulai: '07:00',
			jam_selesai: '09:00',
			regu: '',
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

	function bolehTab(nilai: Tab) {
		const item = TAB.find((row) => row.id === nilai);
		if (!item) return false;
		if (item.roles === 'admin') return admin;
		if (item.roles === 'titik') return bisaKelolaTitik;
		return true;
	}

	function pilihTab(nilai: Tab) {
		if (!bolehTab(nilai)) nilai = 'jadwal';
		tab = nilai;
		if (page.url.searchParams.get('tab') !== nilai) {
			goto(`/ph?tab=${nilai}`, { replace: true, reset: false });
		}
		void muatTab();
	}

	async function muatAwal() {
		try {
			satkerList = await api<Satker[]>('/satker');
			if (!titikOwnerId) titikOwnerId = ownerOptions[0]?.id ?? '';
			const urlTab = page.url.searchParams.get('tab');
			if (
				urlTab === 'jadwal' ||
				urlTab === 'rekap' ||
				urlTab === 'harian' ||
				urlTab === 'penempatan' ||
				urlTab === 'titik'
			) {
				tab = bolehTab(urlTab) ? urlTab : 'jadwal';
			}
			await muatTab();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat data awal';
			memuat = false;
		} finally {
			siap = true;
		}
	}

	async function muatTab() {
		memuat = true;
		error = null;
		try {
			if (tab === 'jadwal') {
				const params = new URLSearchParams({ dari: dariAktif, sampai: sampaiAktif });
				if (satkerId) params.set('satker_id', satkerId);
				const paramsStatus = new URLSearchParams({ bulan: bulanJadwal || bulanIni });
				if (satkerId) paramsStatus.set('satker_id', satkerId);
				jadwal = await api<JadwalPh[]>(`/ph/jadwal?${params}`);
				statusJadwal = await api<StatusJadwalPh>(`/ph/jadwal/status?${paramsStatus}`);
			} else if (tab === 'rekap') {
				const masalah = validasiRentangRekap();
				if (masalah) {
					error = masalah;
					return;
				}
				const params = new URLSearchParams({ dari: dariRekap, sampai: sampaiRekap });
				if (satkerId) params.set('satker_id', satkerId);
				const [hasilRekap, hasilHarian] = await Promise.all([
					api<{
						rentang: { dari: string; sampai: string };
						data: RekapPh[];
					}>(`/ph/rekap?${params}`),
					api<{
						rentang: { dari: string; sampai: string };
						data: RekapPhHarian[];
					}>(`/ph/rekap/harian?${params}`)
				]);
				rekap = hasilRekap.data;
				rekapHarian = hasilHarian.data;
				rekapRentang = `${tanggalPendek(hasilRekap.rentang.dari)} – ${tanggalPendek(hasilRekap.rentang.sampai)}`;
			} else if (tab === 'harian') {
				if (!tanggalHarianPh) {
					error = 'Pilih tanggal rekap harian.';
					return;
				}
				const params = new URLSearchParams({
					dari: tanggalHarianPh,
					sampai: tanggalHarianPh
				});
				if (satkerId) params.set('satker_id', satkerId);
				const hasil = await api<{
					rentang: { dari: string; sampai: string };
					data: RekapPhHarian[];
				}>(`/ph/rekap/harian?${params}`);
				harianPh = hasil.data;
				tutupKategoriHarianPh();
			} else if (tab === 'penempatan') {
				let pilihan = satkerId;
				if (!pilihan || !satkerRowMap.has(pilihan)) {
					pilihan = satkerList.find((row) => row.level === 'satker')?.id ?? satkerList[0]?.id ?? '';
					satkerId = pilihan;
				}
				if (!pilihan) return;
				penempatan = await api<PenempatanPh[]>(`/ph/penempatan?satker_id=${pilihan}`);
			} else if (tab === 'titik') {
				if (!titikOwnerId) {
					titikOwnerId = ownerOptions[0]?.id ?? '';
				}
				if (!titikOwnerId) return;
				titikList = await api<StrongPoint[]>(`/ph/strong-point?satker_id=${titikOwnerId}`);
			}
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat data strong point';
		} finally {
			memuat = false;
		}
		if (tab === 'rekap' && detailId) await muatDetail(detailId);
	}

	function pilihSatker(nilai: string) {
		satkerId = nilai;
		void muatTab();
	}

	async function muatPilihanJadwal(satker: string) {
		cariPersonelJadwal = '';
		if (!satker) {
			personelSatker = [];
			return;
		}
		const orang = await api<ListResponse<Personel>>(`/personel?satker_id=${satker}&per_page=200`);
		personelSatker = orang.data.filter((row) => row.is_active);
	}

	function pilihSatkerAktif() {
		if (satkerId) return satkerId;
		satkerId = satkerList.find((row) => row.level === 'satker')?.id ?? '';
		return satkerId;
	}

	function bukaTambahJadwal() {
		const target = pilihSatkerAktif();
		if (!target) {
			error = 'Pilih satker terlebih dahulu.';
			return;
		}
		formJadwal = kosongJadwal();
		formJadwal.satker_id = target;
		const tanggalAwal =
			bulanJadwal === bulanIni ? hariIni : `${bulanJadwal || bulanIni}-01`;
		formJadwal.tanggal = tanggalAwal;
		formJadwal.tanggal_selesai = tanggalAwal;
		formJadwalTerbuka = true;
		void muatPilihanJadwal(target);
	}

	function bukaEditJadwal(row: JadwalPh) {
		formJadwal = {
			id: row.id,
			satker_id: row.satker_id,
			sesi: row.sesi === 'sore' ? 'sore' : 'pagi',
			tanggal: row.tanggal,
			tanggal_selesai: row.tanggal,
			jam_mulai: row.jam_mulai,
			jam_selesai: row.jam_selesai,
			regu: row.regu ?? '',
			catatan: row.catatan ?? '',
			personel_ids: row.personel.map((item) => item.personel_id)
		};
		formJadwalTerbuka = true;
		void muatPilihanJadwal(row.satker_id);
	}

	function togglePersonel(id: string) {
		formJadwal.personel_ids = formJadwal.personel_ids.includes(id)
			? formJadwal.personel_ids.filter((item) => item !== id)
			: [...formJadwal.personel_ids, id];
	}

	async function simpanJadwal() {
		error = null;
		info = null;
		if (!formJadwal.personel_ids.length) {
			error = 'Pilih minimal satu anggota.';
			return;
		}
		try {
			const body = {
				satker_id: formJadwal.satker_id,
				sesi: formJadwal.sesi,
				tanggal: formJadwal.tanggal,
				tanggal_selesai: formJadwal.id ? null : formJadwal.tanggal_selesai || null,
				jam_mulai: formJadwal.jam_mulai,
				jam_selesai: formJadwal.jam_selesai,
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
			await muatTab();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan jadwal strong point';
		}
	}

	async function hapusJadwal(row: JadwalPh) {
		if (!confirm(`Hapus jadwal ${row.tanggal} sesi ${row.sesi ?? 'pagi'}?`)) return;
		try {
			await api(`/ph/jadwal/${row.id}`, { method: 'DELETE' });
			await muatTab();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menghapus jadwal';
		}
	}

	async function bukaDetail(row: JadwalPh) {
		detail = row;
		absensiDetail = [];
		try {
			absensiDetail = await api<AbsensiPh[]>(`/ph/absensi/jadwal/${row.id}`);
		} catch {
			absensiDetail = [];
		}
	}

	function anggotaJadwal(row: JadwalPh) {
		return row.personel.map((item) => item.nama).join(', ');
	}

	function cocokCari(orang: Personel, cari: string) {
		const kunci = cari.trim().toLowerCase();
		if (!kunci) return true;
		return `${orang.nama} ${orang.nrp} ${orang.pangkat}`.toLowerCase().includes(kunci);
	}

	function daftarSatkerTempat() {
		const aktif = satkerList.filter((row) => row.is_active);
		// Satker Polda/Polres juga dapat memiliki personel sendiri, jadi ikut ditampilkan.
		return aktif.length ? aktif : satkerList;
	}

	async function salinJadwalBulan() {
		const target = pilihSatkerAktif();
		if (!target) {
			error = 'Pilih satker terlebih dahulu.';
			return;
		}
		const tujuan = labelBulan(`${bulanJadwal || bulanIni}-01`);
		if (
			!confirm(
				`Salin jadwal bulan sebelumnya ke ${tujuan}? Jadwal ${tujuan} yang belum memiliki absensi akan diganti.`
			)
		)
			return;
		error = null;
		info = null;
		try {
			const hasil = await api<{ dibuat: number; dihapus: number; sumber: string }>(
				'/ph/jadwal/salin',
				{
					method: 'POST',
					body: { satker_id: target, bulan: bulanJadwal || bulanIni, timpa: true }
				}
			);
			info = `${hasil.dibuat} jadwal disalin dari ${labelBulan(hasil.sumber)}${hasil.dihapus ? `, ${hasil.dihapus} jadwal lama diganti` : ''}.`;
			await muatTab();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyalin jadwal bulan sebelumnya';
		}
	}

	async function muatPilihanTempat() {
		cariPersonelTempat = '';
		if (!satkerTempat) {
			titikTempatList = [];
			personelTempat = [];
			return;
		}
		memuatTempat = true;
		try {
			const [titik, orang] = await Promise.all([
				api<StrongPoint[]>(`/ph/strong-point?satker_id=${satkerTempat}`),
				api<ListResponse<Personel>>(`/personel?satker_id=${satkerTempat}&per_page=200`)
			]);
			titikTempatList = titik.filter((row) => row.is_active);
			personelTempat = orang.data.filter((row) => row.is_active);
			if (!titikTempatList.some((row) => row.id === titikTempat)) {
				titikTempat = titikTempatList[0]?.id ?? '';
			}
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat pilihan penempatan';
		} finally {
			memuatTempat = false;
		}
	}

	async function aturSatkerTempat(nilai: string) {
		if (modeTempatPersonel) return;
		satkerTempat = nilai;
		titikTempat = '';
		anggotaTempat = [];
		await muatPilihanTempat();
	}

	function toggleAnggotaTempat(id: string) {
		if (modeTempatPersonel) return;
		anggotaTempat = anggotaTempat.includes(id)
			? anggotaTempat.filter((item) => item !== id)
			: [...anggotaTempat, id];
	}

	async function bukaTambahPenempatan() {
		error = null;
		info = null;
		modeTempatPersonel = null;
		anggotaTempat = [];
		const opsi = satkerTempatOptions;
		satkerTempat = opsi.some((row) => row.id === satkerId) ? satkerId : (opsi[0]?.id ?? '');
		titikTempat = '';
		formTempatTerbuka = true;
		await muatPilihanTempat();
	}

	async function bukaEditPenempatan(row: PenempatanPh) {
		error = null;
		info = null;
		satkerTempat = row.satker_id;
		modeTempatPersonel = row.personel_id;
		anggotaTempat = [row.personel_id];
		titikTempat = row.strong_point_id ?? '';
		formTempatTerbuka = true;
		await muatPilihanTempat();
		titikTempat = row.strong_point_id ?? titikTempatList[0]?.id ?? '';
	}

	async function simpanPenempatan() {
		error = null;
		info = null;
		if (!satkerTempat) {
			error = 'Pilih satker terlebih dahulu.';
			return;
		}
		if (!titikTempat) {
			error = 'Pilih titik strong point.';
			return;
		}
		if (!anggotaTempat.length) {
			error = 'Pilih minimal satu anggota.';
			return;
		}
		try {
			await api('/ph/penempatan', {
				method: 'POST',
				body: {
					satker_id: satkerTempat,
					items: anggotaTempat.map((personel_id) => ({
						personel_id,
						strong_point_id: titikTempat
					}))
				}
			});
			const titik = titikTempatList.find((row) => row.id === titikTempat);
			info = `${anggotaTempat.length} anggota ditempatkan ke ${titik?.nama ?? 'titik strong point'}.`;
			formTempatTerbuka = false;
			modeTempatPersonel = null;
			await muatTab();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan penempatan';
		}
	}

	async function eksporPdf() {
		mengekspor = true;
		error = null;
		try {
			const params = new URLSearchParams({ dari: dariAktif, sampai: sampaiAktif });
			if (satkerId) params.set('satker_id', satkerId);
			const jenis =
				tab === 'rekap' ? (modeRekapPh === 'harian' ? 'rekap/harian' : 'rekap') : 'jadwal';
			const blob = await apiBlob(`/ph/${jenis}/ekspor?${params}`);
			const url = URL.createObjectURL(blob);
			const tautan = document.createElement('a');
			tautan.href = url;
			tautan.download = `strong-point-${jenis.replace('/', '-')}-${dariAktif}-${sampaiAktif}.pdf`;
			tautan.click();
			URL.revokeObjectURL(url);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal membuat PDF';
		} finally {
			mengekspor = false;
		}
	}

	function bukaDetailPersonel(id: string) {
		if (tab === 'rekap' && detailId === id) return;
		void goto(`/ph?tab=rekap&personel_id=${id}`, { reset: false });
	}

	function tutupDetailPersonel() {
		void goto('/ph?tab=rekap', { reset: false });
	}

	function bukaKategoriHarianPh(kategori: KategoriHarianPh) {
		kategoriHarianPhAktif = kategori;
	}

	function tutupKategoriHarianPh() {
		kategoriHarianPhAktif = null;
	}

	async function muatDetail(id: string) {
		const masalah = validasiRentangRekap();
		if (masalah) {
			error = masalah;
			return;
		}
		memuatDetail = true;
		error = null;
		try {
			const params = new URLSearchParams({
				dari: dariRekap,
				sampai: sampaiRekap,
				personel_id: id
			});
			if (satkerId) params.set('satker_id', satkerId);
			const [orang, hasil] = await Promise.all([
				api<Personel>(`/personel/${id}`),
				api<{ rentang: { dari: string; sampai: string }; data: RekapPhHarian[] }>(
					`/ph/rekap/harian?${params}`
				)
			]);
			detailPersonel = orang;
			detailBaris = hasil.data;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat detail rekap strong point';
			detailPersonel = null;
			detailBaris = [];
		} finally {
			memuatDetail = false;
		}
	}

	async function eksporDetailPdf() {
		if (!detailId) return;
		mengekspor = true;
		error = null;
		try {
			const params = new URLSearchParams({ dari: dariRekap, sampai: sampaiRekap });
			if (satkerId) params.set('satker_id', satkerId);
			const blob = await apiBlob(`/ph/rekap/personel/${detailId}/ekspor?${params}`);
			const url = URL.createObjectURL(blob);
			const tautan = document.createElement('a');
			tautan.href = url;
			tautan.download = `strong-point-detail-${detailPersonel?.nrp ?? detailId}-${dariRekap}-${sampaiRekap}.pdf`;
			tautan.click();
			URL.revokeObjectURL(url);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal membuat PDF detail';
		} finally {
			mengekspor = false;
		}
	}

	function tutupGaleri() {
		galeri?.item.forEach((item) => URL.revokeObjectURL(item.url));
		galeri = null;
	}

	async function bukaGaleri(row: RekapPh) {
		tutupGaleri();
		galeri = { nama: row.nama, memuat: true, item: [] };
		try {
			const params = new URLSearchParams({
				personel_id: row.personel_id,
				dari: dariRekap,
				sampai: sampaiRekap
			});
			if (satkerId) params.set('satker_id', satkerId);
			const hasil = await api<FotoPh[]>(`/ph/foto?${params}`);
			const item = await Promise.all(
				hasil.map(async (foto) => ({
					url: await apiBlobUrl(foto.foto_url),
					caption: `${tanggalPendek(foto.tanggal)} · ${foto.jenis === 'masuk' ? 'Masuk' : 'Pulang'} ${foto.jam} · ${foto.strong_point_nama}`
				}))
			);
			if (galeri) galeri = { ...galeri, memuat: false, item };
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat dokumentasi strong point';
			if (galeri) galeri = { ...galeri, memuat: false };
		}
	}

	async function muatTitik() {
		if (!titikOwnerId) return;
		try {
			titikList = await api<StrongPoint[]>(`/ph/strong-point?satker_id=${titikOwnerId}`);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat titik strong point';
		}
	}

	function gantiOwnerTitik(nilai: string) {
		titikOwnerId = nilai;
		void muatTitik();
	}

	function bukaTambahStrong() {
		formStrong = kosongStrong();
		formStrong.satker_id = titikOwnerId;
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

	function pilihDariPeta(lat: number, lng: number) {
		formStrong.latitude = lat.toFixed(6);
		formStrong.longitude = lng.toFixed(6);
	}

	function angkaPeta(nilai: string): number | null {
		const teks = nilai.trim();
		if (!teks) return null;
		const angka = Number(teks);
		return Number.isFinite(angka) ? angka : null;
	}

	const latPeta = $derived(angkaPeta(formStrong.latitude));
	const lngPeta = $derived(angkaPeta(formStrong.longitude));

	async function simpanStrong() {
		error = null;
		info = null;
		const lat = Number(formStrong.latitude);
		const lng = Number(formStrong.longitude);
		if (!Number.isFinite(lat) || !Number.isFinite(lng)) {
			error = 'Pilih koordinat pada peta atau isi latitude/longitude.';
			return;
		}
		const body = {
			nama: formStrong.nama,
			alamat: formStrong.alamat || null,
			latitude: lat,
			longitude: lng,
			radius_m: Number(formStrong.radius_m) || 100,
			is_active: formStrong.is_active
		};
		try {
			if (formStrong.id) {
				await api(`/ph/strong-point/${formStrong.id}`, { method: 'PATCH', body });
			} else {
				await api('/ph/strong-point', {
					method: 'POST',
					body: { ...body, satker_id: formStrong.satker_id }
				});
			}
			formStrongTerbuka = false;
			info = formStrong.id ? 'Strong point diperbarui.' : 'Strong point dibuat.';
			await muatTitik();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan strong point';
		}
	}

	async function hapusStrong(row: StrongPoint) {
		if (!confirm(`Hapus titik "${row.nama}"?`)) return;
		try {
			await api(`/ph/strong-point/${row.id}`, { method: 'DELETE' });
			await muatTitik();
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menghapus strong point';
		}
	}

	function bukaJam(jam: string | undefined) {
		return jam ?? '-';
	}

	onMount(() => {
		void muatAwal();
	});

	// Sidebar memakai ?tab= pada route yang sama, sehingga komponen tidak di-mount ulang.
	$effect(() => {
		const url = page.url;
		const nilai = url.searchParams.get('tab');
		if (!siap) return;
		if (nilai !== 'jadwal' && nilai !== 'rekap' && nilai !== 'harian' && nilai !== 'penempatan' && nilai !== 'titik') {
			return;
		}
		const tujuan = bolehTab(nilai) ? nilai : 'jadwal';
		const idUrl = tujuan === 'rekap' ? (url.searchParams.get('personel_id') ?? '') : '';
		const tabBerubah = tab !== tujuan;
		const detailBerubah = detailId !== idUrl;
		if (!tabBerubah && !detailBerubah) return;

		if (tabBerubah) {
			tab = tujuan;
			detailId = idUrl;
			tutupKategoriHarianPh();
			if (!idUrl) {
				detailPersonel = null;
				detailBaris = [];
			}
			void muatTab();
			return;
		}

		detailId = idUrl;
		if (idUrl) {
			void muatDetail(idUrl);
		} else {
			detailPersonel = null;
			detailBaris = [];
			error = null;
		}
	});
</script>

<svelte:head><title>Strong Point — Absensi & Strong Point</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-end justify-between gap-3 p-4">
		<div>
			<h2 class="text-sm font-semibold text-slate-800">Strong Point</h2>
			<p class="mt-1 text-xs text-slate-500">
				Jadwal disusun per bulan. Satu anggota ditempatkan pada satu titik tetap.
			</p>
		</div>
		{#if admin}
			<div class="flex flex-wrap gap-2">
				{#if tab === 'jadwal'}
					<button class="btn btn-ghost border border-slate-300" onclick={salinJadwalBulan}>
						<Icon name="refresh" size={16} /> Salin Bulan Lalu
					</button>
					<button class="btn btn-primary" onclick={bukaTambahJadwal}>
						<Icon name="plus" size={16} /> Tambah Jadwal
					</button>
				{:else if tab === 'rekap' && !detailId}
					<button
						class="btn btn-ghost border border-slate-300"
						disabled={mengekspor}
						onclick={eksporPdf}
					>
						<Icon name="download" size={16} />
						{mengekspor
							? 'Menyiapkan PDF…'
							: modeRekapPh === 'harian'
								? 'Export PDF Matriks'
								: 'Export PDF'}
					</button>
				{:else if tab === 'penempatan'}
					<button class="btn btn-primary" onclick={bukaTambahPenempatan}>
						<Icon name="plus" size={16} /> Tambah Penempatan
					</button>
				{/if}
			</div>
		{/if}
	</div>

	<div class="card space-y-3 p-4">
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

		{#if tab === 'titik'}
			<div class="flex flex-wrap items-end gap-3">
				<div class="min-w-64 flex-1">
					<label class="label" for="owner-titik">Pemilik Titik</label>
					<Combobox
						id="owner-titik"
						bind:value={titikOwnerId}
						options={opsiOwner}
						placeholder="Pilih pemilik titik"
						labelKosong={null}
						onchange={() => gantiOwnerTitik(titikOwnerId)}
					/>
				</div>
				<button class="btn btn-primary" onclick={bukaTambahStrong}>
					<Icon name="plus" size={16} /> Tambah Titik
				</button>
			</div>
		{:else}
			<div class="flex flex-wrap items-end gap-3">
				{#if tab === 'jadwal'}
					<div>
						<label class="label" for="bulan">Bulan</label>
						<input
							id="bulan"
							type="month"
							class="input"
							bind:value={bulanJadwal}
							onchange={() => {
								if (!bulanJadwal) bulanJadwal = bulanIni;
								void muatTab();
							}}
						/>
					</div>
				{:else if tab === 'rekap'}
					<div>
						<label class="label" for="dari">Dari Tanggal</label>
						<input
							id="dari"
							type="date"
							class="input"
							max={hariIni}
							bind:value={dariRekap}
							onchange={() => muatTab()}
						/>
					</div>
					<div>
						<label class="label" for="sampai">Sampai Tanggal</label>
						<input
							id="sampai"
							type="date"
							class="input"
							max={hariIni}
							bind:value={sampaiRekap}
							onchange={() => muatTab()}
						/>
					</div>
					<button
						type="button"
						class="btn btn-ghost border border-slate-300"
						onclick={() => {
							const sekarang = tanggalHariIni();
							dariRekap = `${sekarang.slice(0, 7)}-01`;
							sampaiRekap = sekarang;
							void muatTab();
						}}
					>
						Bulan Ini
					</button>
					<p class="pb-2 text-[11px] text-slate-500">Maksimal 31 hari.</p>
				{:else if tab === 'harian'}
					<div>
						<label class="label" for="harian-tanggal">Tanggal</label>
						<input
							id="harian-tanggal"
							type="date"
							class="input"
							max={hariIni}
							bind:value={tanggalHarianPh}
							onchange={() => muatTab()}
						/>
					</div>
					<button
						type="button"
						class="btn btn-ghost border border-slate-300"
						onclick={() => {
							tanggalHarianPh = tanggalHariIni();
							void muatTab();
						}}
					>
						Hari Ini
					</button>
					<p class="pb-2 text-[11px] text-slate-500">Default hari ini.</p>
				{/if}
				<div class="min-w-56 flex-1">
					<label class="label" for="satker">Satker</label>
					<Combobox
						id="satker"
						bind:value={satkerId}
						options={opsiSatker}
						placeholder="Semua Satker"
						labelKosong="Semua Satker"
						onchange={() => pilihSatker(satkerId)}
					/>
				</div>
				{#if tab === 'penempatan'}
					<span class="pb-2 text-xs text-slate-500">
						Satu anggota hanya boleh memiliki satu titik strong point.
					</span>
				{/if}
			</div>
		{/if}
	</div>

	{#if statusJadwalTampil}
		{#if statusJadwalTampil.disalin_dari_bulan}
			<p class="rounded-lg bg-amber-50 px-4 py-2 text-sm text-amber-800 ring-1 ring-amber-200">
				Jadwal {labelBulan(statusJadwalTampil.bulan)} belum disusun, sehingga sistem memakai salinan
				jadwal {labelBulan(statusJadwalTampil.disalin_dari_bulan)}. Segera susun jadwal bulan ini.
			</p>
		{:else if statusJadwalTampil.jumlah_jadwal === 0}
			<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">
				Jadwal {labelBulan(statusJadwalTampil.bulan)} belum disusun.
			</p>
		{/if}
		{#if statusJadwalTampil.perlu_susun_bulan_depan && bulanJadwal === bulanIni}
			<p class="rounded-lg bg-sky-50 px-4 py-2 text-sm text-sky-800 ring-1 ring-sky-200">
				Jadwal {labelBulan(statusJadwalTampil.bulan_depan)} belum disusun. Susun sebelum tanggal 1;
				jika terlewat, sistem akan memakai jadwal bulan sebelumnya.
			</p>
		{/if}
	{/if}
	{#if info}
		<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">
			{info}
		</p>
	{/if}
	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	{#if tab === 'jadwal'}
		{#if tampilkanRoster}
			<div class="space-y-4">
				{#each roster as grup (grup.satker_id)}
					<div class="card overflow-hidden">
						<div class="flex items-center justify-between border-b border-slate-200 px-4 py-3">
							<h3 class="text-sm font-semibold text-slate-800">
								{satkerMap.get(grup.satker_id) ?? '-'}
							</h3>
							<span class="text-xs text-slate-500">{grup.anggota.length} anggota</span>
						</div>
						<div class="overflow-x-auto">
							<table class="w-full border-collapse text-xs">
								<thead class="bg-slate-50">
									<tr>
										<th class="th min-w-64">Anggota</th>
										{#each rentangHari as hari}
											<th class="th w-10 px-1 text-center">
												<span class="block">{hari.slice(8, 10)}</span>
												<span class="block text-[10px] font-normal text-slate-400">
													{HARI[new Date(`${hari}T00:00:00`).getDay() === 0 ? 6 : new Date(`${hari}T00:00:00`).getDay() - 1].slice(0, 3)}
												</span>
											</th>
										{/each}
									</tr>
								</thead>
								<tbody class="divide-y divide-slate-100">
									{#each grup.anggota as orang (orang.id)}
										<tr>
											<td class="td whitespace-nowrap">
												<p class="font-medium text-slate-800">{orang.nama}</p>
												<p class="text-[11px] text-slate-500">{orang.pangkat} · {orang.nrp}</p>
											</td>
											{#each rentangHari as hari}
												{@const sel = grup.hari.get(orang.id)?.get(hari)}
												<td class="td px-1 text-center">
													{#if sel?.pagi || sel?.sore}
														<div class="flex flex-col items-center gap-0.5">
															{#if sel?.pagi}
																<span
																	class="rounded bg-sky-100 px-1 py-0.5 text-[10px] font-semibold text-sky-800"
																	title="Pagi {sel.pagi.jam_mulai}–{sel.pagi.jam_selesai} · {sel.pagi.strong_point_nama}"
																>
																	P
																</span>
															{/if}
															{#if sel?.sore}
																<span
																	class="rounded bg-amber-100 px-1 py-0.5 text-[10px] font-semibold text-amber-800"
																	title="Sore {sel.sore.jam_mulai}–{sel.sore.jam_selesai} · {sel.sore.strong_point_nama}"
																>
																	S
																</span>
															{/if}
														</div>
													{:else}
														<span class="text-slate-300">·</span>
													{/if}
												</td>
											{/each}
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
					</div>
				{/each}
				{#if !roster.length}
					<div class="card px-4 py-10 text-center text-sm text-slate-500">
						{memuat ? 'Memuat…' : 'Belum ada jadwal strong point pada rentang ini.'}
					</div>
				{/if}
			</div>
		{:else if jadwal.length > 0}
			<p class="text-xs text-slate-500">
				Rentang lebih dari 45 hari; roster disembunyikan. Gunakan daftar jadwal di bawah.
			</p>
		{/if}

		<details class="card overflow-hidden" open>
			<summary class="cursor-pointer px-4 py-3 text-sm font-semibold text-slate-700">
				Daftar Jadwal ({jadwal.length})
			</summary>
			<div class="overflow-x-auto border-t border-slate-200">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Tanggal</th>
							<th class="th">Sesi / Jam</th>
							<th class="th">Titik</th>
							<th class="th">Regu</th>
							<th class="th">Anggota</th>
							<th class="th no-print">Aksi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each jadwalHalaman as row (row.id)}
							<tr class="hover:bg-slate-50/70">
								<td class="td text-slate-700">{tanggalPendek(row.tanggal)}</td>
								<td class="td">
									<p class="font-medium capitalize text-slate-800">{row.sesi ?? 'pagi'}</p>
									<p class="text-[11px] text-slate-500">{row.jam_mulai}–{row.jam_selesai}</p>
								</td>
								<td class="td text-slate-600">{row.strong_point_nama}</td>
								<td class="td text-slate-600">{row.regu ?? '-'}</td>
								<td class="td max-w-72 truncate text-xs text-slate-600">
									{anggotaJadwal(row)}
								</td>
								<td class="td no-print">
									<div class="flex flex-wrap gap-1">
										<button
											class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
											onclick={() => bukaDetail(row)}
										>
											Detail
										</button>
										{#if admin}
											<button
												class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
												onclick={() => bukaEditJadwal(row)}
											>
												Edit
											</button>
											<button
												class="btn btn-ghost px-2 py-1 text-xs text-rose-600"
												onclick={() => hapusJadwal(row)}
											>
												Hapus
											</button>
										{/if}
									</div>
								</td>
							</tr>
						{:else}
							<tr>
								<td class="td text-center text-slate-500" colspan="6">
									{memuat ? 'Memuat…' : 'Belum ada jadwal pada rentang ini.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<Pagination
				page={halamanJadwalAman}
				perPage={perPage}
				total={jadwal.length}
				label="jadwal"
				onchange={(p) => (halamanJadwal = p)}
			/>
		</details>
	{:else if tab === 'harian'}
		<div class="space-y-4">
			<div class="flex flex-wrap items-center justify-between gap-2">
				<div>
					<h2 class="text-sm font-semibold text-slate-700">
						Rekap Harian Strong Point {tanggalPendek(tanggalHarianPh)}
					</h2>
					<p class="text-[11px] text-slate-500">
						{ringkasanHarianPh.personel} personel · {ringkasanHarianPh.sesi} sesi · {satkerId
							? (satkerMap.get(satkerId) ?? '-')
							: 'Semua Satker'} ·
						{modeHarianPh === 'ringkasan'
							? 'klik kartu untuk melihat daftar personel'
							: 'marker strong point dan check-in'}
					</p>
				</div>
				<div class="flex items-center gap-2">
					{#if memuat}
						<span class="text-[11px] text-slate-400">Memuat…</span>
					{/if}
					<div class="flex overflow-hidden rounded-lg border border-slate-200">
						<button
							type="button"
							class="px-3 py-1.5 text-xs font-medium transition"
							class:bg-navy-800={modeHarianPh === 'ringkasan'}
							class:text-white={modeHarianPh === 'ringkasan'}
							class:text-slate-600={modeHarianPh !== 'ringkasan'}
							aria-pressed={modeHarianPh === 'ringkasan'}
							onclick={() => (modeHarianPh = 'ringkasan')}
						>
							Ringkasan
						</button>
						<button
							type="button"
							class="border-l border-slate-200 px-3 py-1.5 text-xs font-medium transition"
							class:bg-navy-800={modeHarianPh === 'peta'}
							class:text-white={modeHarianPh === 'peta'}
							class:text-slate-600={modeHarianPh !== 'peta'}
							aria-pressed={modeHarianPh === 'peta'}
							onclick={() => (modeHarianPh = 'peta')}
						>
							Peta
						</button>
					</div>
				</div>
			</div>
			{#if memuat && harianPh.length === 0}
				<p class="rounded-lg bg-slate-50 px-4 py-6 text-center text-sm text-slate-500">
					Memuat rekap harian…
				</p>
			{:else if modeHarianPh === 'ringkasan'}
				<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
					{#each kartuHarianPh as kartu (kartu.id)}
						<button
							type="button"
							class="card flex flex-col items-start gap-1 p-4 text-left transition hover:-translate-y-0.5 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-navy-400"
							onclick={() => bukaKategoriHarianPh(kartu)}
						>
							<span class="h-1 w-10 rounded-full {kartu.warna}"></span>
							<span class="mt-2 text-2xl font-semibold text-slate-800">{kartu.nilai}</span>
							<span class="text-sm font-medium text-slate-700">{kartu.label}</span>
							<span class="text-[11px] text-slate-500">{kartu.keterangan}</span>
						</button>
					{/each}
				</div>
				{#if harianPh.length === 0}
					<p
						class="rounded-lg border border-dashed border-slate-300 p-4 text-center text-sm text-slate-500"
					>
						Tidak ada jadwal strong point pada tanggal ini.
					</p>
				{/if}
			{:else}
				<PetaRekapHarianPh data={harianPh} />
			{/if}
		</div>
	{:else if tab === 'rekap'}
		{#if detailId}
			<div class="space-y-4">
				<div class="card p-4">
					<div class="flex flex-wrap items-start justify-between gap-3">
						<div class="flex items-start gap-3">
							<button
								type="button"
								class="btn btn-ghost border border-slate-300"
								onclick={tutupDetailPersonel}
							>
								<Icon name="left" size={16} /> Kembali
							</button>
							<div>
								<h2 class="text-sm font-semibold text-slate-800">
									Detail Rekap Strong Point
								</h2>
								<p class="text-[11px] text-slate-500">
									{rekapRentang} · {satkerId
										? (satkerMap.get(satkerId) ?? '-')
										: 'Semua Satker'}
								</p>
							</div>
						</div>
						<div class="flex flex-wrap items-center gap-2">
							{#if memuatDetail}
								<span class="text-[11px] text-slate-400">Memuat…</span>
							{/if}
							<button
								type="button"
								class="btn btn-ghost border border-slate-300"
								disabled={mengekspor}
								onclick={eksporDetailPdf}
							>
								<Icon name="download" size={16} />
								{mengekspor ? 'Menyiapkan PDF…' : 'Export PDF Detail'}
							</button>
						</div>
					</div>
					{#if detailPersonel}
						<div class="mt-3 flex flex-wrap items-center gap-3">
							<div
								class="flex h-10 w-10 items-center justify-center rounded-full bg-navy-800 text-sm font-semibold text-white"
							>
								{detailPersonel.nama.charAt(0)}
							</div>
							<div>
								<p class="text-sm font-semibold text-slate-800">
									{detailPersonel.nama}
								</p>
								<p class="text-xs text-slate-500">
									{detailPersonel.pangkat} · {detailPersonel.nrp} · {detailPersonel.unit ??
										detailPersonel.satker_nama}
								</p>
							</div>
						</div>
					{/if}
				</div>

				<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
					{#each kartuDetailPh as kartu (kartu.label)}
						<div class="card flex flex-col items-start gap-1 p-4">
							<span class="h-1 w-10 rounded-full {kartu.warna}"></span>
							<span class="mt-2 text-2xl font-semibold text-slate-800">{kartu.nilai}</span>
							<span class="text-sm font-medium text-slate-700">{kartu.label}</span>
							<span class="text-[11px] text-slate-500">{kartu.keterangan}</span>
						</div>
					{/each}
				</div>

				<div class="card overflow-hidden">
					<div
						class="flex flex-wrap items-center justify-between gap-2 border-b border-slate-200 px-4 py-3"
					>
						<h2 class="text-sm font-semibold text-slate-700">Rincian per Jadwal</h2>
						<span class="text-xs text-slate-500">{detailBaris.length} jadwal</span>
					</div>
					<div class="overflow-x-auto">
						<table class="w-full">
							<thead class="bg-slate-50">
								<tr>
									<th class="th">Tanggal</th>
									<th class="th">Sesi & Jam</th>
									<th class="th">Titik / Regu</th>
									<th class="th text-center">Check-in</th>
									<th class="th text-center">Check-out</th>
									<th class="th text-center">Telat (mnt)</th>
									<th class="th text-center">Pulang Cepat (mnt)</th>
									<th class="th text-center">Status</th>
								</tr>
							</thead>
							<tbody class="divide-y divide-slate-100">
								{#each detailBaris as row (row.jadwal_ph_id)}
									<tr class="hover:bg-slate-50/70">
										<td class="td whitespace-nowrap text-slate-600">
											{tanggalPendek(row.tanggal)}
											<p class="text-[10px] text-slate-400">
												{namaHariSingkat(row.tanggal)}
											</p>
										</td>
										<td class="td text-slate-600">
											{labelSesiPh(row.sesi)} · {row.jam_mulai}–{row.jam_selesai}
										</td>
										<td class="td text-slate-600">
											{row.strong_point_nama}
											{#if row.regu}
												<p class="text-[10px] text-slate-400">{row.regu}</p>
											{/if}
										</td>
										<td class="td text-center font-medium text-slate-800">
											{row.masuk_jam ?? '—'}
										</td>
										<td class="td text-center font-medium text-slate-800">
											{row.pulang_jam ?? '—'}
										</td>
										<td
											class="td text-center {row.menit_telat > 0
												? 'font-semibold text-amber-700'
												: 'text-slate-400'}"
										>
											{row.menit_telat > 0 ? row.menit_telat : '-'}
										</td>
										<td
											class="td text-center {row.menit_pulang_cepat > 0
												? 'font-semibold text-rose-700'
												: 'text-slate-400'}"
										>
											{row.menit_pulang_cepat > 0 ? row.menit_pulang_cepat : '-'}
										</td>
										<td class="td text-center">
											{#if !row.masuk_jam}
												<Badge tone="red">TA</Badge>
											{:else if !row.pulang_jam}
												<Badge tone="amber">TC</Badge>
											{:else}
												<Badge tone="green">Lengkap</Badge>
											{/if}
										</td>
									</tr>
								{:else}
									<tr>
										<td class="td text-center text-slate-500" colspan="8">
											{memuatDetail
												? 'Memuat detail…'
												: 'Tidak ada jadwal strong point untuk personel ini pada rentang filter.'}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				</div>
			</div>
		{:else}
		<div class="card overflow-hidden">
			<div
				class="flex flex-wrap items-center justify-between gap-2 border-b border-slate-200 px-4 py-3"
			>
				<div>
					<h2 class="text-sm font-semibold text-slate-700">
						Rekap Strong Point {rekapRentang}
					</h2>
					<p class="text-[11px] text-slate-500">
						{modeRekapPh === 'ringkasan'
							? `${rekap.length} personel · klik nama untuk detail`
							: `${matriksRekapPh.length} personel berjadwal · ${tanggalRekapPh.length} hari`}
					</p>
				</div>
				<div class="flex overflow-hidden rounded-lg border border-slate-200">
					<button
						type="button"
						class="px-3 py-1.5 text-xs font-medium transition"
						class:bg-navy-800={modeRekapPh === 'ringkasan'}
						class:text-white={modeRekapPh === 'ringkasan'}
						class:text-slate-600={modeRekapPh !== 'ringkasan'}
						aria-pressed={modeRekapPh === 'ringkasan'}
						onclick={() => (modeRekapPh = 'ringkasan')}
					>
						Ringkasan
					</button>
					<button
						type="button"
						class="border-l border-slate-200 px-3 py-1.5 text-xs font-medium transition"
						class:bg-navy-800={modeRekapPh === 'harian'}
						class:text-white={modeRekapPh === 'harian'}
						class:text-slate-600={modeRekapPh !== 'harian'}
						aria-pressed={modeRekapPh === 'harian'}
						onclick={() => (modeRekapPh = 'harian')}
					>
						Matriks Harian
					</button>
				</div>
			</div>
			{#if modeRekapPh === 'ringkasan'}
				<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Personel</th>
							<th class="th text-center">Jadwal</th>
							<th class="th text-center">Check-in</th>
							<th class="th text-center">Hanya CI</th>
							<th class="th text-center">Tidak SP</th>
							<th class="th text-center">Telat (mnt)</th>
							<th class="th text-center">Pulang Cepat (mnt)</th>
							<th class="th text-center">Menunggu</th>
							<th class="th no-print">Dokumentasi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each rekapHalaman as row (row.personel_id)}
							<tr class="hover:bg-slate-50/70">
								<td class="td">
									<button
										type="button"
										class="text-left font-medium text-navy-700 hover:underline"
										title="Lihat detail rekap strong point"
										onclick={() => bukaDetailPersonel(row.personel_id)}
									>
										{row.nama}
									</button>
									<p class="text-[11px] text-slate-500">{row.pangkat} · {row.nrp}</p>
								</td>
								<td class="td text-center text-slate-600">{row.jumlah_jadwal}</td>
								<td class="td text-center font-semibold text-emerald-700">{row.hadir}</td>
								<td class="td text-center">
									{#if row.hanya_check_in > 0}
										<span class="font-semibold text-amber-700">{row.hanya_check_in}</span>
										<p class="text-[10px] text-amber-700">
											{row.tanggal_hanya_check_in.map((t) => t.slice(8, 10)).join(', ')}
										</p>
									{:else}
										<span class="text-slate-300">-</span>
									{/if}
								</td>
								<td class="td text-center">
									{#if row.tidak_hadir > 0}
										<span class="font-semibold text-rose-700">{row.tidak_hadir}</span>
										<p class="text-[10px] text-rose-700">
											{row.tanggal_tidak_hadir.map((t) => t.slice(8, 10)).join(', ')}
										</p>
									{:else}
										<span class="text-slate-300">-</span>
									{/if}
								</td>
								<td class="td text-center {row.total_menit_telat > 0 ? 'font-semibold text-amber-700' : 'text-slate-400'}">
									{row.total_menit_telat}
								</td>
								<td class="td text-center {row.total_menit_pulang_cepat > 0 ? 'font-semibold text-rose-700' : 'text-slate-400'}">
									{row.total_menit_pulang_cepat}
								</td>
								<td class="td text-center">
									{#if row.menunggu_verifikasi > 0}
										<Badge tone="amber">{row.menunggu_verifikasi}</Badge>
									{:else}
										<span class="text-slate-300">-</span>
									{/if}
								</td>
								<td class="td no-print">
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
								<td class="td text-center text-slate-500" colspan="9">
									{memuat ? 'Memuat…' : 'Belum ada data rekap pada rentang ini.'}
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
					label="rekap PH"
					onchange={(p) => (halamanRekap = p)}
				/>
			{:else}
				<div
					class="flex flex-wrap gap-x-3 gap-y-1 border-b border-slate-200 px-4 py-2 text-[10px] text-slate-500"
				>
					<span><b class="text-slate-700">Pagi/Sore</b> jadwal</span>
					<span><b class="text-emerald-700">Ya</b> check-in</span>
					<span><b class="text-rose-700">TA</b> tidak strong point</span>
					<span><b class="text-amber-700">TC</b> tanpa check-out</span>
					<span><b class="text-amber-700">T</b> telat</span>
					<span><b class="text-rose-700">PC</b> pulang cepat</span>
					<span class="text-slate-400">Hanya personel dengan jadwal pada rentang filter · klik nama untuk detail</span>
				</div>
				<div class="max-h-[72vh] overflow-auto">
					<table class="min-w-full border-collapse text-xs">
						<thead class="sticky top-0 z-20 bg-slate-50">
							<tr>
								<th
									class="sticky top-0 left-0 z-30 min-w-56 border-r border-b border-slate-200 bg-slate-50 px-3 py-2 text-left"
								>
									Personel
								</th>
								{#each tanggalRekapPh as tgl (tgl)}
									<th
										class="min-w-[124px] border-r border-b border-slate-200 px-1 py-2 text-center {akhirPekan(
											tgl
										)
											? 'bg-slate-100 text-slate-500'
											: ''} {tgl === hariIni
											? 'bg-amber-50 text-amber-800'
											: ''}"
									>
										<span class="block text-xs font-semibold">
											{new Date(`${tgl}T00:00:00`).getDate()}
										</span>
										<span class="block text-[10px] font-normal">
											{namaHariSingkat(tgl)}
										</span>
									</th>
								{/each}
							</tr>
						</thead>
						<tbody class="divide-y divide-slate-100">
							{#each matriksRekapPh as baris (baris.personel_id)}
								<tr class="hover:bg-slate-50/70">
									<td
										class="sticky left-0 z-10 min-w-56 border-r border-slate-200 bg-white px-3 py-2 align-top"
									>
										<button
											type="button"
											class="text-left font-medium text-navy-700 hover:underline"
											title="Lihat detail rekap strong point"
											onclick={() => bukaDetailPersonel(baris.personel_id)}
										>
											{baris.nama}
										</button>
										<p class="text-[10px] text-slate-500">
											{baris.pangkat} · {baris.nrp}
										</p>
										{#if baris.unit}
											<p class="text-[10px] text-slate-400">{baris.unit}</p>
										{/if}
									</td>
									{#each tanggalRekapPh as tgl (tgl)}
										{@const jadwalHari = baris.sel.get(tgl) ?? []}
										<td
											class="border-r border-slate-100 p-1 align-top {akhirPekan(tgl)
												? 'bg-slate-50/40'
												: ''}"
										>
											{#if jadwalHari.length > 0}
												<div class="space-y-1">
													{#each jadwalHari as jadwal (jadwal.jadwal_ph_id)}
														<div class="text-[9px] leading-tight">
															<p class="font-semibold text-slate-700">
																{jadwal.sesi === 'pagi'
																	? 'Pagi'
																	: jadwal.sesi === 'sore'
																		? 'Sore'
																		: 'Sesi'}
																<span class="ml-1 font-mono font-normal text-slate-500">
																	{jadwal.jam_mulai}–{jadwal.jam_selesai}
																</span>
															</p>
															<div class="mt-0.5 flex flex-wrap gap-0.5">
																{#if jadwal.masuk_jam}
																	<span
																		class="inline-flex rounded-full bg-emerald-50 px-1 py-0.5 font-semibold text-emerald-700 ring-1 ring-emerald-200 ring-inset"
																		title="Check-in {jadwal.masuk_jam}"
																	>
																		Ya {jadwal.masuk_jam}
																	</span>
																{:else}
																	<span
																		class="inline-flex rounded-full bg-rose-50 px-1 py-0.5 font-semibold text-rose-700 ring-1 ring-rose-200 ring-inset"
																		title="Tidak melakukan strong point"
																	>
																		TA
																	</span>
																{/if}
																{#if jadwal.masuk_jam && !jadwal.pulang_jam}
																	<span
																		class="inline-flex rounded-full bg-amber-50 px-1 py-0.5 font-semibold text-amber-700 ring-1 ring-amber-200 ring-inset"
																		title="Check-in tanpa check-out"
																	>
																		TC
																	</span>
																{/if}
																{#if jadwal.menit_telat > 0}
																	<span
																		class="inline-flex rounded-full bg-amber-50 px-1 py-0.5 font-semibold text-amber-700 ring-1 ring-amber-200 ring-inset"
																		title="Telat {jadwal.menit_telat} menit"
																	>
																		T {jadwal.menit_telat}m
																	</span>
																{/if}
																{#if jadwal.menit_pulang_cepat > 0}
																	<span
																		class="inline-flex rounded-full bg-rose-50 px-1 py-0.5 font-semibold text-rose-700 ring-1 ring-rose-200 ring-inset"
																		title="Pulang cepat {jadwal.menit_pulang_cepat} menit"
																	>
																		PC {jadwal.menit_pulang_cepat}m
																	</span>
																{/if}
															</div>
															<p
																class="mt-0.5 truncate text-slate-400"
																title={jadwal.strong_point_nama}
															>
																{jadwal.strong_point_nama}
															</p>
														</div>
													{/each}
												</div>
											{:else}
												<span class="block py-2 text-center text-slate-200">·</span>
											{/if}
										</td>
									{/each}
								</tr>
							{:else}
								<tr>
									<td
										class="px-4 py-8 text-center text-xs text-slate-500"
										colspan={tanggalRekapPh.length + 1}
									>
										{memuat
											? 'Memuat matriks strong point…'
											: 'Belum ada jadwal strong point pada rentang ini.'}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	{/if}
{:else if tab === 'penempatan'}
		<div class="card overflow-hidden">
			<div class="border-b border-slate-200 px-4 py-3">
				<h2 class="text-sm font-semibold text-slate-700">Penempatan Anggota</h2>
				<p class="text-xs text-slate-500">
					Satu anggota hanya memiliki satu titik strong point. Gunakan tombol Ubah untuk memindahkan
					anggota ke titik lain.
				</p>
			</div>
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Anggota</th>
							<th class="th">Satker</th>
							<th class="th">Titik Strong Point</th>
							<th class="th no-print">Aksi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each penempatan as row (row.personel_id)}
							<tr class="hover:bg-slate-50/70">
								<td class="td">
									<p class="font-medium text-slate-800">{row.nama}</p>
									<p class="text-[11px] text-slate-500">{row.pangkat} · {row.nrp}</p>
								</td>
								<td class="td text-xs text-slate-600">{satkerMap.get(row.satker_id) ?? '-'}</td>
								<td class="td text-slate-600">
									{#if row.strong_point_nama}
										{row.strong_point_nama}
									{:else}
										<Badge tone="amber">Belum ditempatkan</Badge>
									{/if}
								</td>
								<td class="td no-print">
									<button
										class="btn btn-ghost border border-slate-300"
										onclick={() => bukaEditPenempatan(row)}
									>
										{row.strong_point_id ? 'Ubah' : 'Tempatkan'}
									</button>
								</td>
							</tr>
						{:else}
							<tr>
								<td class="td text-center text-slate-500" colspan="4">
									{memuat ? 'Memuat…' : 'Belum ada anggota pada satker ini.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{:else if tab === 'titik'}
		<div class="card overflow-hidden">
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Nama Titik</th>
							<th class="th">Pemilik</th>
							<th class="th">Alamat</th>
							<th class="th">Koordinat</th>
							<th class="th text-center">Radius</th>
							<th class="th">Status</th>
							<th class="th no-print">Aksi</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each titikList as row (row.id)}
							<tr>
								<td class="td font-medium text-slate-800">{row.nama}</td>
								<td class="td text-slate-600">
									{ownerMap.get(row.satker_id)?.nama ?? row.satker_id}
								</td>
								<td class="td text-slate-600">{row.alamat ?? '-'}</td>
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
									<div class="flex gap-1">
										<button
											class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs"
											onclick={() => bukaEditStrong(row)}
										>
											Edit
										</button>
										<button
											class="btn btn-ghost px-2 py-1 text-xs text-rose-600"
											onclick={() => hapusStrong(row)}
										>
											Hapus
										</button>
									</div>
								</td>
							</tr>
						{:else}
							<tr>
								<td class="td text-center text-slate-500" colspan="7">
									{memuat ? 'Memuat…' : 'Belum ada titik strong point untuk unit ini.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
</div>

<Modal
	open={formJadwalTerbuka}
	title={formJadwal.id ? 'Edit Jadwal Strong Point' : 'Tambah Jadwal Strong Point'}
	lebar="max-w-3xl"
	onclose={() => (formJadwalTerbuka = false)}
>
	<div class="grid gap-3 sm:grid-cols-2">
		<div class="sm:col-span-2">
			<label class="label" for="jsatker">Satker</label>
			<Combobox
				id="jsatker"
				bind:value={formJadwal.satker_id}
				options={opsiSatker}
				placeholder="Pilih satker"
				labelKosong={null}
				disabled={Boolean(formJadwal.id)}
				onchange={() => muatPilihanJadwal(formJadwal.satker_id)}
			/>
		</div>
		<div>
			<label class="label" for="jsesi">Sesi</label>
			<select id="jsesi" class="input" bind:value={formJadwal.sesi}>
				<option value="pagi">Pagi</option>
				<option value="sore">Sore</option>
			</select>
		</div>
		<div>
			<label class="label" for="jregu">Regu (opsional)</label>
			<input id="jregu" class="input" placeholder="Regu 1" bind:value={formJadwal.regu} />
		</div>
		<div>
			<label class="label" for="jtgl">Tanggal</label>
			<input id="jtgl" type="date" class="input" bind:value={formJadwal.tanggal} />
		</div>
		{#if !formJadwal.id}
			<div>
				<label class="label" for="jselesai">Tanggal Selesai</label>
				<input id="jselesai" type="date" class="input" bind:value={formJadwal.tanggal_selesai} />
				<p class="mt-1 text-[11px] text-slate-500">
					Satu jadwal dibuat untuk setiap tanggal dalam rentang.
				</p>
			</div>
		{/if}
		<div>
			<label class="label" for="jmulai">Jam Mulai</label>
			<input id="jmulai" type="time" class="input" bind:value={formJadwal.jam_mulai} />
		</div>
		<div>
			<label class="label" for="jakhir">Jam Selesai</label>
			<input id="jakhir" type="time" class="input" bind:value={formJadwal.jam_selesai} />
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="jcat">Catatan</label>
			<input id="jcat" class="input" bind:value={formJadwal.catatan} />
		</div>
		<div class="sm:col-span-2">
			<span class="label">Anggota</span>
			<p class="mb-1 text-[11px] text-slate-500">
				Titik strong point diambil otomatis dari penempatan anggota. Anggota yang belum
				ditempatkan akan ditolak; atur di tab <strong>Penempatan Anggota</strong>.
			</p>
			<input
				class="input mb-2 py-1.5"
				placeholder="Cari nama atau NRP…"
				bind:value={cariPersonelJadwal}
			/>
			<div class="max-h-48 overflow-auto rounded-lg border border-slate-200 p-2">
				{#each personelJadwalTampil as p (p.id)}
					<label class="flex items-center gap-2 py-1 text-xs text-slate-700">
						<input
							type="checkbox"
							checked={formJadwal.personel_ids.includes(p.id)}
							onchange={() => togglePersonel(p.id)}
						/>
						{p.nama} · {p.pangkat} · {p.nrp}
					</label>
				{:else}
					<p class="text-xs text-slate-500">
						{cariPersonelJadwal.trim()
							? 'Tidak ada personel yang cocok.'
							: 'Belum ada personel aktif pada satker ini.'}
					</p>
				{/each}
			</div>
		</div>
	</div>
	<div class="mt-4 flex justify-end gap-2">
		<button class="btn btn-ghost" onclick={() => (formJadwalTerbuka = false)}>Batal</button>
		<button class="btn btn-primary" onclick={simpanJadwal}>Simpan</button>
	</div>
</Modal>

<Modal
	open={formTempatTerbuka}
	title={modeTempatPersonel ? 'Ubah Titik Anggota' : 'Tambah Penempatan Anggota'}
	lebar="max-w-2xl"
	onclose={() => {
		formTempatTerbuka = false;
		modeTempatPersonel = null;
	}}
>
	<div class="space-y-3">
		{#if modeTempatPersonel}
			<div class="rounded-lg bg-slate-50 px-3 py-2 text-sm text-slate-700">
				Anggota:
				<strong>
					{personelTempat.find((p) => p.id === modeTempatPersonel)?.nama ?? '-'}
				</strong>
			</div>
		{:else}
			<div>
				<label class="label" for="tsatker">Satker</label>
				<Combobox
					id="tsatker"
					bind:value={satkerTempat}
					options={opsiSatkerTempat}
					placeholder="Pilih satker"
					labelKosong={null}
					onchange={() => aturSatkerTempat(satkerTempat)}
				/>
			</div>
		{/if}

		<div>
			<label class="label" for="ttitik">Titik Strong Point</label>
			<select id="ttitik" class="input" bind:value={titikTempat} disabled={memuatTempat}>
				{#each titikTempatList as titik (titik.id)}
					<option value={titik.id}>{titik.nama}</option>
				{/each}
			</select>
			{#if !titikTempatList.length}
				<p class="mt-1 text-[11px] text-slate-500">
					Belum ada titik strong point pada induk satker ini.
				</p>
			{/if}
		</div>

		<div>
			<span class="label">Anggota</span>
			{#if modeTempatPersonel}
				<p class="rounded-lg border border-slate-200 px-3 py-2 text-xs text-slate-600">
					Titik anggota ini akan diganti tanpa mengubah anggota lain.
				</p>
			{:else}
				<p class="mb-1 text-[11px] text-slate-500">
					Satu anggota hanya boleh ditempatkan pada satu titik. Memilih anggota yang sudah punya
					titik akan memindahkannya.
				</p>
				<input
					class="input mb-2 py-1.5"
					placeholder="Cari nama atau NRP…"
					bind:value={cariPersonelTempat}
				/>
				<div class="max-h-56 overflow-auto rounded-lg border border-slate-200 p-2">
					{#each personelTempatTampil as p (p.id)}
						<label class="flex items-center gap-2 py-1 text-xs text-slate-700">
							<input
								type="checkbox"
								checked={anggotaTempat.includes(p.id)}
								onchange={() => toggleAnggotaTempat(p.id)}
							/>
							{p.nama} · {p.pangkat} · {p.nrp}
						</label>
					{:else}
						<p class="text-xs text-slate-500">
							{cariPersonelTempat.trim()
								? 'Tidak ada personel yang cocok.'
								: memuatTempat
									? 'Memuat…'
									: 'Belum ada personel aktif pada satker ini.'}
						</p>
					{/each}
				</div>
			{/if}
		</div>
	</div>
	<div class="mt-4 flex justify-end gap-2">
		<button
			class="btn btn-ghost"
			onclick={() => {
				formTempatTerbuka = false;
				modeTempatPersonel = null;
			}}
		>
			Batal
		</button>
		<button class="btn btn-primary" onclick={simpanPenempatan} disabled={memuatTempat}>Simpan</button>
	</div>
</Modal>

<Modal
	open={formStrongTerbuka}
	title={formStrong.id ? 'Edit Titik Strong Point' : 'Tambah Titik Strong Point'}
	lebar="max-w-3xl"
	onclose={() => (formStrongTerbuka = false)}
>
	<div class="grid gap-3 sm:grid-cols-2">
		<div class="sm:col-span-2">
			<label class="label" for="ssatker">Pemilik Titik</label>
			<Combobox
				id="ssatker"
				bind:value={formStrong.satker_id}
				options={opsiOwner}
				placeholder="Pilih pemilik titik"
				labelKosong={null}
				disabled={Boolean(formStrong.id)}
			/>
			<p class="mt-1 text-[11px] text-slate-500">
				Titik dipakai otomatis oleh satker di bawah unit ini.
			</p>
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="snama">Nama Titik</label>
			<input id="snama" class="input" bind:value={formStrong.nama} />
		</div>
		<div class="sm:col-span-2">
			<label class="label" for="salamat">Alamat</label>
			<input id="salamat" class="input" bind:value={formStrong.alamat} />
		</div>
		<div>
			<label class="label" for="slat">Latitude</label>
			<input id="slat" class="input" bind:value={formStrong.latitude} />
		</div>
		<div>
			<label class="label" for="slng">Longitude</label>
			<input id="slng" class="input" bind:value={formStrong.longitude} />
		</div>
		<div>
			<label class="label" for="srad">Radius (meter)</label>
			<input id="srad" class="input" bind:value={formStrong.radius_m} />
		</div>
		<label class="flex items-center gap-2 pt-5 text-xs text-slate-600">
			<input type="checkbox" bind:checked={formStrong.is_active} /> Aktif
		</label>
		<div class="sm:col-span-2">
			<PetaPicker latitude={latPeta} longitude={lngPeta} radius={Number(formStrong.radius_m) || 100} onchange={pilihDariPeta} />
		</div>
	</div>
	<div class="mt-4 flex justify-end gap-2">
		<button class="btn btn-ghost" onclick={() => (formStrongTerbuka = false)}>Batal</button>
		<button class="btn btn-primary" onclick={simpanStrong}>Simpan</button>
	</div>
</Modal>

<Modal open={!!detail} title="Detail Jadwal Strong Point" lebar="max-w-2xl" onclose={() => (detail = null)}>
	{#if detail}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="font-semibold text-slate-800">
					{tanggalPanjang(detail.tanggal)} · {detail.sesi ?? 'pagi'} · {detail.jam_mulai}–{detail.jam_selesai}
				</p>
				<p class="text-xs text-slate-600">
					{detail.strong_point_nama} · {detail.regu ?? 'Tanpa regu'} · {detail.jumlah_personel} anggota
				</p>
			</div>
			<div class="overflow-x-auto">
				<table class="w-full text-xs">
					<thead>
						<tr>
							<th class="th">Anggota</th>
							<th class="th">Check-in</th>
							<th class="th">Check-out</th>
							<th class="th">Telat</th>
							<th class="th">Pulang Cepat</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each detail.personel as p (p.personel_id)}
							{@const masuk = absensiDetail.find((a) => a.personel_id === p.personel_id && a.jenis === 'masuk')}
							{@const pulang = absensiDetail.find((a) => a.personel_id === p.personel_id && a.jenis === 'pulang')}
							<tr>
								<td class="td">
									<p class="font-medium text-slate-800">{p.nama}</p>
									<p class="text-[11px] text-slate-500">{p.pangkat} · {p.nrp}</p>
								</td>
								<td class="td">{bukaJam(masuk?.jam)}</td>
								<td class="td">{bukaJam(pulang?.jam)}</td>
								<td class="td text-amber-700">{masuk?.menit_telat ?? 0} mnt</td>
								<td class="td text-rose-700">{pulang?.menit_pulang_cepat ?? 0} mnt</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
</Modal>

<Modal open={!!galeri} title="Dokumentasi Strong Point" lebar="max-w-3xl" onclose={tutupGaleri}>
	{#if galeri}
		{#if galeri.memuat}
			<p class="py-8 text-center text-sm text-slate-500">Memuat dokumentasi…</p>
		{:else if galeri.item.length === 0}
			<p class="py-8 text-center text-sm text-slate-500">
				Belum ada foto dokumentasi untuk {galeri.nama} pada rentang ini.
			</p>
		{:else}
			<div class="grid grid-cols-2 gap-3 sm:grid-cols-3">
				{#each galeri.item as foto, indeks (indeks)}
					<figure class="overflow-hidden rounded-lg ring-1 ring-slate-200">
						<img src={foto.url} alt={foto.caption} class="aspect-square w-full object-cover" />
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
	open={!!kategoriHarianPhAktif}
	title={kategoriHarianPhAktif?.label ?? 'Rekap Harian Strong Point'}
	lebar="max-w-3xl"
	onclose={tutupKategoriHarianPh}
>
	{#if kategoriHarianPhAktif}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="text-sm font-semibold text-slate-800">
					{kategoriHarianPhAktif.label} · {tanggalPendek(tanggalHarianPh)}
				</p>
				<p class="text-xs text-slate-500">
					{kategoriHarianPhAktif.keterangan} · {kategoriHarianPhAktif.baris.length} baris jadwal
				</p>
			</div>
			{#if kategoriHarianPhAktif.baris.length === 0}
				<p
					class="rounded-lg border border-dashed border-slate-300 p-4 text-center text-sm text-slate-500"
				>
					Tidak ada personel pada kategori ini.
				</p>
			{:else}
				<div
					class="max-h-[65vh] divide-y divide-slate-100 overflow-auto rounded-lg border border-slate-200"
				>
					{#each kategoriHarianPhAktif.baris as row (row.jadwal_ph_id + row.personel_id)}
						<div class="flex flex-wrap items-start justify-between gap-3 p-3">
							<div class="min-w-0">
								<p class="font-medium text-slate-800">{row.nama}</p>
								<p class="text-[11px] text-slate-500">{row.pangkat} · {row.nrp}</p>
								<p class="text-[11px] text-slate-400">{row.unit ?? '-'}</p>
								<p class="mt-1 text-[11px] text-slate-600">
									{labelSesiPh(row.sesi)} {row.jam_mulai}–{row.jam_selesai} ·
									{row.strong_point_nama}
								</p>
							</div>
							<div class="text-right text-[11px] text-slate-600">
								<p>
									Masuk
									<span class="font-mono font-medium text-slate-800">
										{row.masuk_jam ?? '—'}
									</span>
								</p>
								<p>
									Pulang
									<span class="font-mono font-medium text-slate-800">
										{row.pulang_jam ?? '—'}
									</span>
								</p>
								{#if row.menit_telat > 0}
									<p class="text-amber-700">Telat {row.menit_telat} mnt</p>
								{/if}
								{#if row.menit_pulang_cepat > 0}
									<p class="text-rose-700">PC {row.menit_pulang_cepat} mnt</p>
								{/if}
								<div class="mt-1 flex justify-end">
									{#if !row.masuk_jam}
										<Badge tone="red">TA</Badge>
									{:else if !row.pulang_jam}
										<Badge tone="amber">TC</Badge>
									{:else}
										<Badge tone="green">Lengkap</Badge>
									{/if}
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</Modal>
