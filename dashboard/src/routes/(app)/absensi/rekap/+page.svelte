<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, apiBlob, apiBlobUrl, ApiError } from '#lib/api/client';
	import type {
		AbsensiHarian,
		AbsensiMatriksHari,
		AbsensiMatriksPersonel,
		AbsensiMatriksResponse,
		AbsensiMatriksSesiStrong,
		FotoAbsensi,
		ListResponse,
		Personel,
		RekapAbsensi,
		Satker,
		StatusKehadiran,
		StrongPoint,
		TitikDinas
	} from '#lib/types';
	import { adalahPimpinanSaja } from '#lib/stores/session.svelte';
	import { LABEL_STATUS_KEHADIRAN } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Combobox from '#lib/components/Combobox.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';
	import Pagination from '#lib/components/Pagination.svelte';
	import PetaLokasi from '#lib/components/PetaLokasi.svelte';
	import PetaRekapHarian from '#lib/components/PetaRekapHarian.svelte';
	import { alamatDariKoordinat } from '#lib/geocode';

	type TabId = 'rekap' | 'harian' | 'verifikasi' | 'detail';

	const TAB: { id: TabId; label: string; keterangan: string }[] = [
		{
			id: 'rekap',
			label: 'Rekap Kehadiran',
			keterangan:
				'Matriks status, jam, telat, dan pulang cepat per tanggal (maks. 31 hari) atau ringkasan per personel.'
		},
		{
			id: 'harian',
			label: 'Rekap Harian',
			keterangan:
				'Ringkasan satu tanggal: tidak absen, jadwal PH, sudah PH, dan jumlah tiap status kehadiran.'
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
		harian: { tanggal: tanggalHariIni(), satker_id: '' },
		verifikasi: { ...rentangBawaan(), satker_id: '', personel_id: '' },
		detail: { ...rentangBawaan(), satker_id: '', personel_id: '' }
	});

	    let rekap = $state<RekapAbsensi[]>([]);
	    let rekapRentang = $state('');
	    let halamanRekap = $state(1);

	    let modeRekap = $state<'matriks' | 'ringkasan'>('matriks');
	    let matriks = $state<AbsensiMatriksResponse | null>(null);

	    type PilihanMatriks = {
	        personel: AbsensiMatriksPersonel;
	        tanggal: string;
	        hari: AbsensiMatriksHari;
	    };
	    let pilihMatriks = $state<PilihanMatriks | null>(null);
	    let alamatMatriks = $state<string | null>(null);
	    let memuatAlamat = $state(false);
	    let fotoMasukMatriks = $state<string | null>(null);
	    let fotoPulangMatriks = $state<string | null>(null);
	    let tokenMatriks = 0;

	    let matriksHarian = $state<AbsensiMatriksResponse | null>(null);
	    let modeHarian = $state<'ringkasan' | 'peta'>('ringkasan');
	    let titikDinasHarian = $state<TitikDinas[]>([]);
	    let strongPointHarian = $state<StrongPoint[]>([]);

	    type KategoriHarian = {
	        id: string;
	        label: string;
	        keterangan: string;
	        tone: 'green' | 'amber' | 'red' | 'blue' | 'slate' | 'gold';
	        orang: AbsensiMatriksPersonel[];
	        /** Khusus kartu Jadwal PH. */
	        phSudah?: number;
	        phBelum?: number;
	    };
	    let kategoriHarian = $state<KategoriHarian | null>(null);

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

	const opsiSatker = $derived(
		satkerList.map((s) => ({ value: s.id, label: s.nama }))
	);

	function opsiPersonel(satkerId: string) {
		return personelUntuk(satkerId).map((orang) => ({
			value: orang.id,
			label: orang.nama,
			keterangan: orang.nrp
		}));
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

	    function selisihHari(dari: string, sampai: string) {
	        const a = new Date(`${dari}T00:00:00Z`).getTime();
	        const b = new Date(`${sampai}T00:00:00Z`).getTime();
	        if (!Number.isFinite(a) || !Number.isFinite(b)) return 0;
	        return Math.floor((b - a) / 86_400_000) + 1;
	    }

	    function validasiRentangRekap() {
	        const hari = selisihHari(filter.rekap.dari, filter.rekap.sampai);
	        if (hari <= 0) return 'Tanggal selesai harus setelah tanggal mulai.';
	        if (hari > 31) return 'Rentang rekap maksimal 31 hari.';
	        return null;
	    }

	    function namaHariSingkat(tanggal: string) {
	        return new Date(`${tanggal}T00:00:00`).toLocaleDateString('id-ID', { weekday: 'short' });
	    }

	    function akhirPekan(tanggal: string) {
	        const hari = new Date(`${tanggal}T00:00:00`).getDay();
	        return hari === 0 || hari === 6;
	    }

	    function statusEfektif(hari: AbsensiMatriksHari): StatusKehadiran | null {
	        const status = hari.status ?? hari.status_masuk ?? hari.status_pulang;
	        if (hari.ph_absen && (status === null || status === 'dinas_kantor')) return 'ph';
	        return status;
	    }

	    const LABEL_MATRIKS: Record<StatusKehadiran, string> = {
	        dinas_kantor: 'DK',
	        dinas_luar: 'DL',
	        ph: 'PH',
	        piket: 'Piket',
	        operasi: 'Ops',
	        libur_nasional: 'Libur',
	        cuti: 'Cuti',
	        izin: 'Izin',
	        sakit: 'Sakit',
	        tanpa_keterangan: 'TK'
	    };

	    function labelMatriks(status: StatusKehadiran | null) {
	        return status ? LABEL_MATRIKS[status] : '-';
	    }

	    function kelasMatriks(status: StatusKehadiran | null) {
	        if (!status) return 'bg-slate-100 text-slate-500 ring-slate-200';
	        if (status === 'dinas_luar') return 'bg-sky-50 text-sky-700 ring-sky-200';
	        if (status === 'cuti' || status === 'izin' || status === 'sakit') {
	            return 'bg-amber-50 text-amber-700 ring-amber-200';
	        }
	        if (status === 'tanpa_keterangan') return 'bg-rose-50 text-rose-700 ring-rose-200';
	        if (status === 'libur_nasional') return 'bg-slate-100 text-slate-500 ring-slate-200';
	        return 'bg-emerald-50 text-emerald-700 ring-emerald-200';
	    }

	    function labelSesi(sesi: AbsensiMatriksSesiStrong) {
	        if (!sesi.sesi) return 'Sesi';
	        return sesi.sesi.charAt(0).toUpperCase() + sesi.sesi.slice(1);
	    }

	    function waktuSesi(sesi: AbsensiMatriksSesiStrong) {
	        if (sesi.masuk_jam || sesi.pulang_jam) {
	            return `${sesi.masuk_jam ?? '–'}–${sesi.pulang_jam ?? ''}`;
	        }
	        return `${sesi.jam_mulai}–${sesi.jam_selesai}`;
	    }

	    function menitSesi(sesi: AbsensiMatriksSesiStrong) {
	        const bagian: string[] = [];
	        if (sesi.menit_telat > 0) bagian.push(`T ${sesi.menit_telat}m`);
	        if (sesi.menit_pulang_cepat > 0) bagian.push(`PC ${sesi.menit_pulang_cepat}m`);
	        return bagian.join(' · ');
	    }

	    function waktuMatriks(hari: AbsensiMatriksHari) {
	        if (hari.masuk_jam && hari.pulang_jam) return `${hari.masuk_jam}–${hari.pulang_jam}`;
	        return hari.masuk_jam ?? hari.pulang_jam ?? '-';
	    }

	    function menitMatriks(hari: AbsensiMatriksHari) {
	        const bagian: string[] = [];
	        if (hari.menit_telat > 0) bagian.push(`T ${hari.menit_telat}m`);
	        if (hari.menit_pulang_cepat > 0) bagian.push(`PC ${hari.menit_pulang_cepat}m`);
	        return bagian.join(' · ');
	    }

	    function judulMatriks(
	        orang: AbsensiMatriksPersonel,
	        tanggal: string,
	        hari: AbsensiMatriksHari
	    ) {
	        const status = statusEfektif(hari);
	        const bagian = [orang.nama, tanggalSingkat(tanggal)];
	        if (hari.tidak_absen) bagian.push('Tidak absen');
	        if (status) bagian.push(LABEL_STATUS_KEHADIRAN[status]);
	        if (hari.jadwal_strong && status !== 'ph') bagian.push('ada jadwal strong point');
	        if (hari.tc) bagian.push('check-in tanpa check-out');
	        if (hari.masuk_jam === null && hari.pulang_jam) bagian.push('check-out tanpa check-in');
	        if (hari.sesi_strong.length > 0) {
	            for (const sesi of hari.sesi_strong) {
	                let teks = `${labelSesi(sesi)} ${sesi.jam_mulai}–${sesi.jam_selesai}`;
	                if (sesi.masuk_jam || sesi.pulang_jam) {
	                    teks += ` (${sesi.masuk_jam ?? '–'}–${sesi.pulang_jam ?? ''})`;
	                }
	                if (sesi.tc) teks += ' TC';
	                if (sesi.tidak_absen) teks += ' TA';
	                bagian.push(teks);
	            }
	        } else if (!hari.tidak_absen) {
	            bagian.push(waktuMatriks(hari));
	        }
	        if (hari.menit_telat > 0) bagian.push(`telat ${hari.menit_telat} menit`);
	        if (hari.menit_pulang_cepat > 0) bagian.push(`pulang cepat ${hari.menit_pulang_cepat} menit`);
	        return bagian.join(' · ');
	    }

	    const STATUS_HARIAN: {
	        status: StatusKehadiran;
	        label: string;
	        keterangan: string;
	        tone: KategoriHarian['tone'];
	    }[] = [
	        {
	            status: 'dinas_kantor',
	            label: 'Dinas Kantor',
	            keterangan: 'Absensi kantor',
	            tone: 'green'
	        },
	        {
	            status: 'dinas_luar',
	            label: 'Dinas Luar',
	            keterangan: 'Tugas di luar kantor',
	            tone: 'blue'
	        },
	        {
	            status: 'piket',
	            label: 'Piket/Shift',
	            keterangan: 'Piket atau shift',
	            tone: 'green'
	        },
	        {
	            status: 'operasi',
	            label: 'Operasi/Pam',
	            keterangan: 'Operasi atau pengamanan',
	            tone: 'green'
	        },
	        { status: 'cuti', label: 'Cuti', keterangan: 'Cuti disetujui', tone: 'amber' },
	        { status: 'izin', label: 'Izin', keterangan: 'Izin tidak masuk', tone: 'amber' },
	        { status: 'sakit', label: 'Sakit', keterangan: 'Sakit', tone: 'amber' }
	    ];

	    const kategoriHarianList = $derived.by(() => {
	        const data = matriksHarian?.data ?? [];
	        const tanggal = filter.harian.tanggal;
	        const tidakAbsen: AbsensiMatriksPersonel[] = [];
	        const jadwalPh: AbsensiMatriksPersonel[] = [];
	        const sudahPh: AbsensiMatriksPersonel[] = [];
	        const perStatus = new Map<StatusKehadiran, AbsensiMatriksPersonel[]>();

	        for (const orang of data) {
	            const hari = orang.hari[tanggal];
	            if (!hari) continue;
	            if (hari.tidak_absen) tidakAbsen.push(orang);
	            if (hari.jadwal_strong) jadwalPh.push(orang);
	            if (hari.ph_absen) sudahPh.push(orang);
	            const status = statusEfektif(hari);
	            if (status) {
	                const daftar = perStatus.get(status) ?? [];
	                daftar.push(orang);
	                perStatus.set(status, daftar);
	            }
	        }

	        const urut = (daftar: AbsensiMatriksPersonel[]) =>
	            [...daftar].sort((a, b) => a.nama.localeCompare(b.nama, 'id'));

	        const sudahPhSet = new Set(sudahPh.map((orang) => orang.personel_id));
	        const jadwalBelumPh = jadwalPh.filter((orang) => !sudahPhSet.has(orang.personel_id));

	        const kartu: KategoriHarian[] = [
	            {
	                id: 'tidak_absen',
	                label: 'Tidak Absen',
	                keterangan: 'Hari kerja tanpa catatan absensi',
	                tone: 'red',
	                orang: urut(tidakAbsen)
	            },
	            {
	                id: 'jadwal_ph',
	                label: 'Jadwal PH',
	                keterangan: 'Punya jadwal strong point',
	                tone: 'blue',
	                orang: urut(jadwalPh),
	                phSudah: jadwalPh.length - jadwalBelumPh.length,
	                phBelum: jadwalBelumPh.length
	            },
	            ...STATUS_HARIAN.map((item) => ({
	                id: item.status,
	                label: item.label,
	                keterangan: item.keterangan,
	                tone: item.tone,
	                orang: urut(perStatus.get(item.status) ?? [])
	            }))
	        ];
	        return kartu;
	    });

	    function kelasBarKategori(tone: KategoriHarian['tone']) {
	        if (tone === 'red') return 'bg-rose-500';
	        if (tone === 'amber') return 'bg-amber-500';
	        if (tone === 'blue') return 'bg-sky-500';
	        if (tone === 'green') return 'bg-emerald-500';
	        if (tone === 'gold') return 'bg-gold-500';
	        return 'bg-slate-400';
	    }

	    function bukaKategoriHarian(kategori: KategoriHarian) {
	        kategoriHarian = kategori;
	    }

	    function tutupKategoriHarian() {
	        kategoriHarian = null;
	    }

	    function bukaDetailHarian(orang: AbsensiMatriksPersonel) {
	        const hari = orang.hari[filter.harian.tanggal];
	        if (!hari) return;
	        tutupKategoriHarian();
	        void bukaDetailMatriks(orang, filter.harian.tanggal, hari);
	    }

	    function ubahModeRekap(mode: 'matriks' | 'ringkasan') {
	        if (mode === modeRekap) return;
	        modeRekap = mode;
	        void muatRekap();
	    }

	    async function bukaDetailMatriks(
	        orang: AbsensiMatriksPersonel,
	        tanggal: string,
	        hari: AbsensiMatriksHari
	    ) {
	        tutupDetailMatriks();
	        const token = ++tokenMatriks;
	        pilihMatriks = { personel: orang, tanggal, hari };

	        const lat = hari.masuk_latitude ?? hari.pulang_latitude;
	        const lng = hari.masuk_longitude ?? hari.pulang_longitude;
	        if (lat !== null && lng !== null) {
	            memuatAlamat = true;
	            const hasil = await alamatDariKoordinat(lat, lng);
	            if (token === tokenMatriks) {
	                alamatMatriks = hasil;
	                memuatAlamat = false;
	            }
	        }

	        if (hari.foto_masuk_url) {
	            try {
	                const url = await apiBlobUrl(hari.foto_masuk_url);
	                if (token === tokenMatriks) fotoMasukMatriks = url;
	                else URL.revokeObjectURL(url);
	            } catch {
	                // abaikan foto yang gagal dimuat
	            }
	        }
	        if (hari.foto_pulang_url) {
	            try {
	                const url = await apiBlobUrl(hari.foto_pulang_url);
	                if (token === tokenMatriks) fotoPulangMatriks = url;
	                else URL.revokeObjectURL(url);
	            } catch {
	                // abaikan foto yang gagal dimuat
	            }
	        }
	    }

	    function tutupDetailMatriks() {
	        tokenMatriks += 1;
	        if (fotoMasukMatriks) URL.revokeObjectURL(fotoMasukMatriks);
	        if (fotoPulangMatriks) URL.revokeObjectURL(fotoPulangMatriks);
	        fotoMasukMatriks = null;
	        fotoPulangMatriks = null;
	        alamatMatriks = null;
	        memuatAlamat = false;
	        pilihMatriks = null;
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
	            status === 'operasi' ||
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
	        const masalah = validasiRentangRekap();
	        if (masalah) {
	            error = masalah;
	            return;
	        }

	        memuat = true;
	        error = null;
	        try {
	            const params = new URLSearchParams({
	                dari: filter.rekap.dari,
	                sampai: filter.rekap.sampai
	            });
	if (filter.rekap.satker_id) params.set('satker_id', filter.rekap.satker_id);
	if (modeRekap === 'matriks' && filter.rekap.personel_id) {
		params.set('personel_id', filter.rekap.personel_id);
	}

	            if (modeRekap === 'matriks') {
	                const hasil = await api<AbsensiMatriksResponse>(`/absensi/matriks?${params}`);
	                matriks = hasil;
	                rekapRentang = rentangTeks(hasil.rentang.dari, hasil.rentang.sampai);
	            } else {
	                const hasil = await api<{
	                    rentang: { dari: string; sampai: string };
	                    data: RekapAbsensi[];
	                }>(`/absensi/rekap?${params}`);
	                rekap = hasil.data;
	                rekapRentang = rentangTeks(hasil.rentang.dari, hasil.rentang.sampai);
	                halamanRekap = 1;
	            }
	        } catch (err) {
	            error =
	                err instanceof ApiError
	                    ? err.message
	                    : `Gagal memuat ${modeRekap === 'matriks' ? 'matriks kehadiran' : 'rekap kehadiran'}`;
	        } finally {
	            memuat = false;
	        }
	    }

	    async function muatRekapHarian() {
	        tutupKategoriHarian();
	        if (!filter.harian.tanggal) {
	            error = 'Pilih tanggal rekap harian.';
	            return;
	        }
	        memuat = true;
	        error = null;
	        try {
	            const params = new URLSearchParams({
	                dari: filter.harian.tanggal,
	                sampai: filter.harian.tanggal
	            });
	            if (filter.harian.satker_id) params.set('satker_id', filter.harian.satker_id);
	            const paramsTitik = new URLSearchParams();
	            if (filter.harian.satker_id) paramsTitik.set('satker_id', filter.harian.satker_id);
	            const queryTitik = paramsTitik.toString();
	            const akhiranTitik = queryTitik ? `?${queryTitik}` : '';
	            const [hasilMatriks, titik, strong] = await Promise.all([
	                api<AbsensiMatriksResponse>(`/absensi/matriks?${params}`),
	                api<TitikDinas[]>(`/titik-dinas${akhiranTitik}`),
	                api<StrongPoint[]>(`/ph/strong-point${akhiranTitik}`)
	            ]);
	            matriksHarian = hasilMatriks;
	            titikDinasHarian = titik.filter((item) => item.is_active);
	            strongPointHarian = strong.filter((item) => item.is_active);
	        } catch (err) {
	            error =
	                err instanceof ApiError ? err.message : 'Gagal memuat rekap harian';
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
		if (id === 'harian') return muatRekapHarian();
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
		if (nilai === 'rekap' || nilai === 'harian' || nilai === 'verifikasi' || nilai === 'detail') {
			if (nilai !== tab) {
				tab = nilai;
				if (siapMuat) muatTab(nilai);
			}
		}
	});

	function ubahSatker(id: TabId) {
		if (id === 'harian') return muatRekapHarian();
		const aktif = filter[id];
		const daftar = personelUntuk(aktif.satker_id);
		if (aktif.personel_id && !daftar.some((p) => p.id === aktif.personel_id)) {
			aktif.personel_id = '';
		}
		muatTab(id);
	}

	async function eksporPdf(tampilan: 'ringkasan' | 'matriks' = modeRekap) {
		if (tab === 'harian') return;
		const aktif = filter[tab];
		if (tab === 'rekap' && tampilan === 'matriks') {
			const masalah = validasiRentangRekap();
			if (masalah) {
				error = masalah;
				return;
			}
		}
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
			if (tab === 'rekap' && tampilan === 'matriks' && aktif.personel_id) {
				params.set('personel_id', aktif.personel_id);
			}

			const jalur =
				tab === 'rekap'
					? tampilan === 'matriks'
						? '/absensi/matriks/ekspor'
						: '/absensi/rekap/ekspor'
					: tab === 'verifikasi'
						? '/absensi/verifikasi/ekspor'
						: '/absensi/detail/ekspor';

			const blob = await apiBlob(`${jalur}?${params}`);
			const url = URL.createObjectURL(blob);
			const tautan = document.createElement('a');
			tautan.href = url;
			const label = tab === 'rekap' && tampilan === 'matriks' ? 'matriks-harian' : `${tab}-absensi`;
			tautan.download = `${label}-${new Date().toISOString().slice(0, 10)}.pdf`;
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
		if (tabAwal === 'harian') tab = 'harian';
		else if (tabAwal === 'detail') tab = 'detail';
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
			{#if tab === 'rekap'}
				<div class="flex flex-wrap gap-2">
					<button
						class="btn btn-ghost border border-slate-300"
						disabled={mengekspor}
						onclick={() => eksporPdf('ringkasan')}
					>
						<Icon name="download" size={16} />
						{mengekspor ? 'Menyiapkan…' : 'PDF Ringkasan'}
					</button>
					<button
						class="btn btn-ghost border border-slate-300"
						disabled={mengekspor}
						onclick={() => eksporPdf('matriks')}
					>
						<Icon name="download" size={16} />
						{mengekspor ? 'Menyiapkan…' : 'PDF Matriks Harian'}
					</button>
				</div>
			{:else if tab !== 'harian'}
				<button
					class="btn btn-ghost border border-slate-300"
					disabled={mengekspor}
					onclick={() => eksporPdf()}
				>
					<Icon name="download" size={16} />
					{mengekspor ? 'Menyiapkan PDF…' : 'Export PDF'}
				</button>
			{/if}
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
			<div class="space-y-3">
				<div class="flex flex-wrap items-center gap-2">
					<span class="text-xs font-medium text-slate-600">Tampilan</span>
					<div class="flex overflow-hidden rounded-lg border border-slate-200">
						<button
							type="button"
							class="px-3 py-1.5 text-xs font-medium transition"
							class:bg-navy-800={modeRekap === 'matriks'}
							class:text-white={modeRekap === 'matriks'}
							class:text-slate-600={modeRekap !== 'matriks'}
							aria-pressed={modeRekap === 'matriks'}
							onclick={() => ubahModeRekap('matriks')}
						>
							Matriks Harian
						</button>
						<button
							type="button"
							class="border-l border-slate-200 px-3 py-1.5 text-xs font-medium transition"
							class:bg-navy-800={modeRekap === 'ringkasan'}
							class:text-white={modeRekap === 'ringkasan'}
							class:text-slate-600={modeRekap !== 'ringkasan'}
							aria-pressed={modeRekap === 'ringkasan'}
							onclick={() => ubahModeRekap('ringkasan')}
						>
							Ringkasan
						</button>
					</div>
				</div>
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
						<Combobox
							id="rekap-satker"
							bind:value={filter.rekap.satker_id}
							options={opsiSatker}
							placeholder="Semua Satker"
							labelKosong="Semua Satker"
							onchange={() => ubahSatker('rekap')}
						/>
					</div>
					{#if modeRekap === 'matriks'}
						<div class="min-w-56">
							<label class="label" for="rekap-personel">Personel</label>
							<Combobox
								id="rekap-personel"
								bind:value={filter.rekap.personel_id}
								options={opsiPersonel(filter.rekap.satker_id)}
								placeholder="Semua Personel"
								labelKosong="Semua Personel"
								onchange={() => muatRekap()}
							/>
						</div>
					{/if}
					<p class="pb-2 text-[11px] text-slate-500">Maksimal rentang 31 hari.</p>
				</div>
			</div>
		{:else if tab === 'harian'}
			<div class="flex flex-wrap items-end gap-3">
				<div>
					<label class="label" for="harian-tanggal">Tanggal</label>
					<input
						id="harian-tanggal"
						type="date"
						class="input"
						bind:value={filter.harian.tanggal}
						onchange={() => muatRekapHarian()}
					/>
				</div>
				<div class="min-w-48">
					<label class="label" for="harian-satker">Satker</label>
					<Combobox
						id="harian-satker"
						bind:value={filter.harian.satker_id}
						options={opsiSatker}
						placeholder="Semua Satker"
						labelKosong="Semua Satker"
						onchange={() => muatRekapHarian()}
					/>
				</div>
				<button
					type="button"
					class="btn btn-ghost border border-slate-300"
					onclick={() => {
						filter.harian.tanggal = tanggalHariIni();
						muatRekapHarian();
					}}
				>
					Hari Ini
				</button>
				<p class="pb-2 text-[11px] text-slate-500">Default tanggal hari ini.</p>
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
					<Combobox
						id="verif-satker"
						bind:value={filter.verifikasi.satker_id}
						options={opsiSatker}
						placeholder="Semua Satker"
						labelKosong="Semua Satker"
						onchange={() => ubahSatker('verifikasi')}
					/>
				</div>
				<div class="min-w-56">
					<label class="label" for="verif-personel">Personel</label>
					<Combobox
						id="verif-personel"
						bind:value={filter.verifikasi.personel_id}
						options={opsiPersonel(filter.verifikasi.satker_id)}
						placeholder="Semua Personel"
						labelKosong="Semua Personel"
						onchange={() => muatVerifikasi()}
					/>
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
					<Combobox
						id="detail-satker"
						bind:value={filter.detail.satker_id}
						options={opsiSatker}
						placeholder="Semua Satker"
						labelKosong="Semua Satker"
						onchange={() => ubahSatker('detail')}
					/>
				</div>
				<div class="min-w-56">
					<label class="label" for="detail-personel">Personel</label>
					<Combobox
						id="detail-personel"
						bind:value={filter.detail.personel_id}
						options={opsiPersonel(filter.detail.satker_id)}
						placeholder="Pilih personel…"
						labelKosong="Pilih personel…"
						onchange={() => muatDetail()}
					/>
				</div>
			</div>
		{/if}
	</div>

	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	{#if tab === 'harian'}
		<div class="space-y-4">
			<div class="flex flex-wrap items-end justify-between gap-3">
				<div>
					<h2 class="text-sm font-semibold text-slate-700">
						Rekap Harian {tanggalSingkat(filter.harian.tanggal)}
					</h2>
					<p class="text-[11px] text-slate-500">
						{matriksHarian?.data.length ?? 0} personel ·
						{filter.harian.satker_id
							? (satkerMap.get(filter.harian.satker_id) ?? '-')
							: 'Semua Satker'} ·
						{modeHarian === 'ringkasan'
							? 'klik kartu untuk melihat daftar orang'
							: 'marker check-in, titik dinas, dan strong point'}
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
							class:bg-navy-800={modeHarian === 'ringkasan'}
							class:text-white={modeHarian === 'ringkasan'}
							class:text-slate-600={modeHarian !== 'ringkasan'}
							aria-pressed={modeHarian === 'ringkasan'}
							onclick={() => (modeHarian = 'ringkasan')}
						>
							Ringkasan
						</button>
						<button
							type="button"
							class="border-l border-slate-200 px-3 py-1.5 text-xs font-medium transition"
							class:bg-navy-800={modeHarian === 'peta'}
							class:text-white={modeHarian === 'peta'}
							class:text-slate-600={modeHarian !== 'peta'}
							aria-pressed={modeHarian === 'peta'}
							onclick={() => (modeHarian = 'peta')}
						>
							Peta
						</button>
					</div>
				</div>
			</div>
			{#if memuat && !matriksHarian}
				<p class="rounded-lg bg-slate-50 px-4 py-6 text-center text-sm text-slate-500">
					Memuat rekap harian…
				</p>
			{:else if modeHarian === 'ringkasan'}
				<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
					{#each kategoriHarianList as kategori (kategori.id)}
						<button
							type="button"
							class="card flex flex-col items-start gap-1 p-4 text-left transition hover:-translate-y-0.5 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-navy-400"
							onclick={() => bukaKategoriHarian(kategori)}
						>
							<span class="h-1 w-10 rounded-full {kelasBarKategori(kategori.tone)}"></span>
							<span class="mt-2 text-2xl font-semibold text-slate-800">
								{kategori.orang.length}
							</span>
							<span class="text-sm font-medium text-slate-700">{kategori.label}</span>
							<span class="text-[11px] text-slate-500">{kategori.keterangan}</span>
							{#if kategori.id === 'jadwal_ph'}
								<span class="mt-1 flex flex-wrap gap-x-2 gap-y-0.5 text-[10px] font-medium">
									<span class="text-emerald-700">Sudah PH {kategori.phSudah ?? 0}</span>
									<span class="text-rose-600">Belum PH {kategori.phBelum ?? 0}</span>
								</span>
							{/if}
						</button>
					{/each}
				</div>
				{#if (matriksHarian?.data.length ?? 0) === 0}
					<p
						class="rounded-lg border border-dashed border-slate-300 p-4 text-center text-sm text-slate-500"
					>
						Tidak ada personel pada filter ini.
					</p>
				{:else if kategoriHarianList.every((k) => k.orang.length === 0)}
					<p
						class="rounded-lg border border-dashed border-slate-300 p-4 text-center text-sm text-slate-500"
					>
						Tidak ada absensi atau jadwal strong point pada tanggal ini.
					</p>
				{/if}
			{:else}
				<PetaRekapHarian
					data={matriksHarian?.data ?? []}
					tanggal={filter.harian.tanggal}
					titikDinas={titikDinasHarian}
					strongPoint={strongPointHarian}
				/>
			{/if}
		</div>
	{:else if tab === 'rekap' && modeRekap === 'matriks'}
		<div class="card overflow-hidden">
			<div class="flex flex-wrap items-center justify-between gap-2 border-b border-slate-200 px-4 py-3">
				<div>
					<h2 class="text-sm font-semibold text-slate-700">
						Rekap Harian {rekapRentang}
					</h2>
					<p class="text-[11px] text-slate-500">
						{matriks?.data.length ?? 0} personel · {matriks?.tanggal.length ?? 0} hari · klik sel untuk detail lokasi
					</p>
				</div>
				<div class="flex flex-wrap gap-x-3 gap-y-1 text-[10px] text-slate-500">
					<span><b class="text-slate-700">DK</b> Dinas Kantor</span>
					<span><b class="text-slate-700">DL</b> Dinas Luar</span>
					<span><b class="text-slate-700">PH</b> Strong Point</span>
					<span><b class="text-slate-700">Ops</b> Operasi/Pam</span>
					<span><b class="text-slate-500">TA</b> tidak absen</span>
					<span><b class="text-rose-700">TC</b> tanpa check-out</span>
					<span><b class="text-rose-700">TM</b> tanpa check-in</span>
					<span><b class="text-amber-700">T</b> telat</span>
					<span><b class="text-amber-700">PC</b> pulang cepat</span>
				</div>
			</div>
			<div class="max-h-[75vh] overflow-auto">
				<table class="min-w-full border-collapse text-xs">
					<thead class="sticky top-0 z-20 bg-slate-50">
						<tr>
							<th
								class="sticky left-0 top-0 z-30 min-w-56 border-b border-r border-slate-200 bg-slate-50 px-3 py-2 text-left"
							>
								Personel
							</th>
							{#each matriks?.tanggal ?? [] as tgl}
								<th
									class="min-w-[106px] border-b border-r border-slate-200 px-1 py-2 text-center {akhirPekan(tgl)
										? 'bg-slate-100 text-slate-500'
										: ''} {tgl === tanggalHariIni()
										? 'bg-amber-50 text-amber-800'
										: ''}"
								>
									<span class="block text-xs font-semibold">
										{new Date(`${tgl}T00:00:00`).getDate()}
									</span>
									<span class="block text-[10px] font-normal">{namaHariSingkat(tgl)}</span>
								</th>
							{/each}
						</tr>
					</thead>
					<tbody class="divide-y divide-slate-100">
						{#each matriks?.data ?? [] as orang (orang.personel_id)}
							<tr class="hover:bg-slate-50/70">
								<td
									class="sticky left-0 z-10 min-w-56 border-r border-slate-200 bg-white px-3 py-2 align-top"
								>
									<p class="font-medium text-slate-800">{orang.nama}</p>
									<p class="text-[10px] text-slate-500">
										{orang.pangkat} · {orang.nrp}
									</p>
									<p class="text-[10px] text-slate-400">
										{satkerMap.get(orang.satker_id) ?? orang.unit ?? '-'}
									</p>
								</td>
								{#each matriks?.tanggal ?? [] as tgl}
									{@const hari = orang.hari[tgl]}
									{@const status = hari ? statusEfektif(hari) : null}
									<td
										class="border-r border-slate-100 p-1 align-top {akhirPekan(tgl)
											? 'bg-slate-50/40'
											: ''}"
									>
										{#if hari}
											<button
												type="button"
												class="block w-full rounded-lg px-1.5 py-1 text-left ring-1 ring-transparent transition hover:ring-navy-200 focus:outline-none focus:ring-2 focus:ring-navy-400"
												title={judulMatriks(orang, tgl, hari)}
												onclick={() => bukaDetailMatriks(orang, tgl, hari)}
											>
												<span class="flex flex-wrap gap-0.5">
													{#if status}
														<span
															class="inline-flex rounded-full px-1.5 py-0.5 text-[10px] font-semibold ring-1 ring-inset {kelasMatriks(
																status
															)}"
														>
															{labelMatriks(status)}
														</span>
													{/if}
													{#if hari.jadwal_strong && status !== 'ph'}
														<span
															class="inline-flex rounded-full bg-sky-50 px-1.5 py-0.5 text-[10px] font-semibold text-sky-700 ring-1 ring-inset ring-sky-200"
															title="Ada jadwal strong point"
														>
															PH
														</span>
													{/if}
													{#if hari.tc}
														<span
															class="inline-flex rounded-full bg-rose-50 px-1.5 py-0.5 text-[10px] font-semibold text-rose-700 ring-1 ring-inset ring-rose-200"
															title="Check-in tanpa check-out"
														>
															TC
														</span>
													{/if}
													{#if hari.masuk_jam === null && hari.pulang_jam}
														<span
															class="inline-flex rounded-full bg-rose-50 px-1.5 py-0.5 text-[10px] font-semibold text-rose-700 ring-1 ring-inset ring-rose-200"
															title="Check-out tanpa check-in"
														>
															TM
														</span>
													{/if}
													{#if hari.tidak_absen}
														<span
															class="inline-flex rounded-full bg-slate-100 px-1.5 py-0.5 text-[10px] font-semibold text-slate-400 ring-1 ring-inset ring-slate-200"
														>
															TA
														</span>
													{/if}
												</span>
												{#if hari.tidak_absen}
													<span
														class="mt-0.5 block text-[9px] leading-tight text-slate-400"
														>Tidak absen</span
													>
												{/if}
												{#if hari.jadwal_strong && status && status !== 'ph'}
													<span
														class="mt-0.5 block text-[9px] font-medium leading-tight text-sky-700"
														>Jadwal PH</span
													>
												{/if}
												{#if hari.sesi_strong.length > 0}
													{#each hari.sesi_strong as item}
														<span
															class="mt-0.5 block text-[9px] leading-tight {item.masuk_jam ||
															item.pulang_jam
																? 'text-slate-700'
																: 'text-slate-400'}"
														>
															{labelSesi(item)} {waktuSesi(item)}{#if item.tc}<span
																	class="font-semibold text-rose-600">&nbsp;TC</span
																>{/if}{#if item.tidak_absen}<span
																	class="font-semibold text-slate-400">&nbsp;TA</span
																>{/if}{#if menitSesi(item)}<span class="text-amber-700"
																	>&nbsp;· {menitSesi(item)}</span
																>{/if}
														</span>
													{/each}
												{:else if !hari.tidak_absen}
													<span
														class="mt-0.5 block font-mono text-[10px] leading-tight text-slate-700"
														>{waktuMatriks(hari)}</span
													>
												{/if}
												{#if hari.tc}
													<span
														class="block text-[9px] font-medium leading-tight text-rose-600"
														>tanpa check-out</span
													>
												{/if}
												{#if hari.sesi_strong.length === 0 && menitMatriks(hari)}
													<span
														class="block text-[9px] font-medium leading-tight text-amber-700"
														>{menitMatriks(hari)}</span
													>
												{/if}
											</button>
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
									colspan={(matriks?.tanggal.length ?? 0) + 1}
								>
									{memuat ? 'Memuat matriks kehadiran…' : 'Belum ada personel pada filter ini.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{:else if tab === 'rekap'}
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

<Modal
	open={!!pilihMatriks}
	title="Detail Absensi"
	lebar="max-w-4xl"
	onclose={tutupDetailMatriks}
>
	{#if pilihMatriks}
		{@const orang = pilihMatriks.personel}
		{@const hari = pilihMatriks.hari}
		{@const status = statusEfektif(hari)}
		{@const lat = hari.masuk_latitude ?? hari.pulang_latitude}
		{@const lng = hari.masuk_longitude ?? hari.pulang_longitude}
		<div class="space-y-4 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<div class="flex flex-wrap items-start justify-between gap-2">
					<div>
						<p class="font-semibold text-slate-800">{orang.nama}</p>
						<p class="text-xs text-slate-500">{orang.pangkat} · {orang.nrp}</p>
						<p class="text-xs text-slate-500">
							{satkerMap.get(orang.satker_id) ?? orang.unit ?? '-'}
						</p>
					</div>
					<p class="text-xs font-medium text-slate-600">
						{tanggalSingkat(pilihMatriks.tanggal)}
					</p>
				</div>
				<div class="mt-2 flex flex-wrap gap-1">
					{#if status}
						<Badge tone={nadaStatus(status)}>{LABEL_STATUS_KEHADIRAN[status]}</Badge>
					{/if}
					{#if hari.tidak_absen}<Badge tone="slate">Tidak absen</Badge>{/if}
					{#if hari.tc}<Badge tone="red">TC · tanpa check-out</Badge>{/if}
					{#if hari.masuk_jam === null && hari.pulang_jam}
						<Badge tone="red">TM · tanpa check-in</Badge>
					{/if}
					{#if hari.jadwal_strong && status !== 'ph'}
						<Badge tone="blue">Jadwal Strong Point</Badge>
					{/if}
					{#if hari.luar_radius}<Badge tone="red">Di luar radius</Badge>{/if}
					{#if hari.is_mock_suspect}<Badge tone="red">Mock GPS</Badge>{/if}
					{#if hari.is_offline}<Badge tone="amber">Offline</Badge>{/if}
					{#if hari.perlu_verifikasi}<Badge tone="amber">Perlu verifikasi</Badge>{/if}
				</div>
			</div>

			<div class="grid gap-3 sm:grid-cols-2">
				<div class="rounded-lg border border-emerald-200 bg-emerald-50/50 p-3">
					<p class="text-[11px] font-semibold uppercase tracking-wide text-emerald-700">
						Check-in
					</p>
					<p class="mt-1 font-mono text-xl font-semibold text-slate-800">
						{hari.masuk_jam ?? '-'}
					</p>
					{#if hari.menit_telat > 0}
						<p class="text-xs font-medium text-amber-700">
							Telat {hari.menit_telat} menit
						</p>
					{:else}
						<p class="text-xs text-slate-400">Tidak telat</p>
					{/if}
				</div>
				<div class="rounded-lg border border-sky-200 bg-sky-50/50 p-3">
					<p class="text-[11px] font-semibold uppercase tracking-wide text-sky-700">
						Check-out
					</p>
					<p class="mt-1 font-mono text-xl font-semibold text-slate-800">
						{hari.pulang_jam ?? '-'}
					</p>
					{#if hari.menit_pulang_cepat > 0}
						<p class="text-xs font-medium text-amber-700">
							Pulang cepat {hari.menit_pulang_cepat} menit
						</p>
					{:else}
						<p class="text-xs text-slate-400">
							{hari.pulang_jam ? 'Tidak pulang cepat' : 'Belum check-out'}
						</p>
					{/if}
				</div>
			</div>

			{#if hari.sesi_strong.length > 0}
				<div class="space-y-2">
					<p class="text-[11px] font-semibold uppercase tracking-wide text-slate-500">
						Sesi Strong Point
					</p>
					<div class="divide-y divide-slate-100 rounded-lg border border-slate-200">
						{#each hari.sesi_strong as sesi}
							<div class="flex flex-wrap items-start justify-between gap-2 p-3 text-xs">
								<div>
									<p class="font-semibold text-slate-700">
										{labelSesi(sesi)} · {sesi.jam_mulai}–{sesi.jam_selesai}
									</p>
									<p class="text-slate-500">{sesi.strong_point_nama}</p>
								</div>
								<div class="text-right">
									{#if sesi.masuk_jam || sesi.pulang_jam}
										<p class="font-mono text-slate-700">
											{sesi.masuk_jam ?? '–'}–{sesi.pulang_jam ?? ''}
										</p>
									{:else}
										<p class="text-slate-400">Belum check-in</p>
									{/if}
									<div class="mt-1 flex flex-wrap justify-end gap-1">
										{#if sesi.tc}<Badge tone="red">TC</Badge>{/if}
										{#if sesi.tidak_absen}<Badge tone="slate">TA</Badge>{/if}
										{#if menitSesi(sesi)}
											<span class="text-[10px] font-medium text-amber-700"
												>{menitSesi(sesi)}</span
											>
										{/if}
									</div>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			{#if lat !== null && lng !== null}
				<div class="space-y-2">
					<p class="text-[11px] font-semibold uppercase tracking-wide text-slate-500">
						Lokasi absensi
					</p>
					<p class="text-xs text-slate-600">
						{memuatAlamat ? 'Mencari alamat…' : alamatMatriks ?? 'Alamat tidak ditemukan.'}
					</p>
					{#key `${lat},${lng}`}
						<PetaLokasi latitude={lat} longitude={lng} label={`Lokasi ${orang.nama}`} />
					{/key}
				</div>
			{:else}
				<div
					class="rounded-lg border border-dashed border-slate-300 p-3 text-xs text-slate-500"
				>
					Lokasi tidak dicatat untuk status ini.
				</div>
			{/if}

			{#if fotoMasukMatriks || fotoPulangMatriks}
				<div class="grid gap-3 sm:grid-cols-2">
					{#if fotoMasukMatriks}
						<figure class="space-y-1">
							<img
								src={fotoMasukMatriks}
								alt="Foto check-in {orang.nama}"
								class="max-h-56 w-full rounded-lg object-contain ring-1 ring-slate-200"
							/>
							<figcaption class="text-[11px] text-slate-500">Foto check-in</figcaption>
						</figure>
					{/if}
					{#if fotoPulangMatriks}
						<figure class="space-y-1">
							<img
								src={fotoPulangMatriks}
								alt="Foto check-out {orang.nama}"
								class="max-h-56 w-full rounded-lg object-contain ring-1 ring-slate-200"
							/>
							<figcaption class="text-[11px] text-slate-500">Foto check-out</figcaption>
						</figure>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</Modal>

<Modal
	open={!!kategoriHarian}
	title={kategoriHarian?.label ?? 'Rekap Harian'}
	lebar="max-w-3xl"
	onclose={tutupKategoriHarian}
>
	{#if kategoriHarian}
		{@const tanggal = filter.harian.tanggal}
		<div class="space-y-3 text-sm">
			<div class="rounded-lg bg-slate-50 p-3">
				<p class="text-sm font-semibold text-slate-800">
					{kategoriHarian.label} · {tanggalSingkat(tanggal)}
				</p>
				<p class="text-xs text-slate-500">
					{kategoriHarian.keterangan} · {kategoriHarian.orang.length} personel
				</p>
				{#if kategoriHarian.id === 'jadwal_ph'}
					<p class="text-[11px] font-medium">
						<span class="text-emerald-700">Sudah PH {kategoriHarian.phSudah ?? 0}</span>
						<span class="text-slate-400"> · </span>
						<span class="text-rose-600">Belum PH {kategoriHarian.phBelum ?? 0}</span>
					</p>
				{/if}
			</div>
			{#if kategoriHarian.orang.length === 0}
				<p
					class="rounded-lg border border-dashed border-slate-300 p-4 text-center text-sm text-slate-500"
				>
					Tidak ada personel pada kategori ini.
				</p>
			{:else}
				<div
					class="max-h-[65vh] divide-y divide-slate-100 overflow-auto rounded-lg border border-slate-200"
				>
					{#each kategoriHarian.orang as orang (orang.personel_id)}
						{@const hari = orang.hari[tanggal]}
						{@const status = hari ? statusEfektif(hari) : null}
						<div class="flex flex-wrap items-start justify-between gap-3 p-3">
							<div class="min-w-0">
								<p class="font-medium text-slate-800">{orang.nama}</p>
								<p class="text-[11px] text-slate-500">
									{orang.pangkat} · {orang.nrp}
								</p>
								<p class="text-[11px] text-slate-400">
									{orang.unit || orang.satker_nama}
								</p>
								<div class="mt-1 flex flex-wrap gap-1">
									{#if status}
										<Badge tone={nadaStatus(status)}>{LABEL_STATUS_KEHADIRAN[status]}</Badge>
									{/if}
									{#if hari?.tidak_absen}<Badge tone="slate">TA</Badge>{/if}
									{#if hari?.tc}<Badge tone="red">TC</Badge>{/if}
									{#if hari?.jadwal_strong && status !== 'ph'}
										<Badge tone="blue">Jadwal PH</Badge>
									{/if}
								</div>
							</div>
							<div class="text-right text-[11px] text-slate-600">
								{#if hari}
									{#if hari.sesi_strong.length > 0}
										{#each hari.sesi_strong as sesi}
											<p class="font-mono">
												{labelSesi(sesi)} {waktuSesi(sesi)}
												{#if sesi.tc}
													<span class="font-semibold text-rose-600">TC</span>
												{/if}
												{#if sesi.tidak_absen}
													<span class="font-semibold text-slate-400">TA</span>
												{/if}
											</p>
										{/each}
									{:else if !hari.tidak_absen}
										<p class="font-mono">{waktuMatriks(hari)}</p>
									{/if}
									{#if hari.tidak_absen}
										<p class="text-slate-400">Tidak absen</p>
									{/if}
								{/if}
								<button
									type="button"
									class="mt-1 text-[11px] font-medium text-navy-700 hover:underline"
									onclick={() => bukaDetailHarian(orang)}
								>
									Detail
								</button>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</Modal>
