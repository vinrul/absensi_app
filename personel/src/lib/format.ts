import type { StatusKehadiran } from '#lib/types';

export {
	LABEL_JENIS_OUTPUT,
	LABEL_JENIS_RENCANA,
	LABEL_KATEGORI,
	LABEL_STATUS_HASIL,
	LABEL_STATUS_KEHADIRAN,
	LABEL_STATUS_OUTPUT,
	LABEL_STATUS_RENCANA
} from '#lib/types';

const BULAN = [
	'Januari',
	'Februari',
	'Maret',
	'April',
	'Mei',
	'Juni',
	'Juli',
	'Agustus',
	'September',
	'Oktober',
	'November',
	'Desember'
];

const HARI = ['Minggu', 'Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu'];

/** `2026-11-18` → `18 November 2026`. */
export function tanggalIndo(tanggal: string | null | undefined): string {
	if (!tanggal) return '-';
	const [tahun, bulan, hari] = tanggal.slice(0, 10).split('-').map(Number);
	if (!tahun || !bulan || !hari) return tanggal;
	return `${hari} ${BULAN[bulan - 1]} ${tahun}`;
}

export function tanggalPendek(tanggal: string | null | undefined): string {
	if (!tanggal) return '-';
	const [, bulan, hari] = tanggal.slice(0, 10).split('-').map(Number);
	if (!bulan || !hari) return tanggal;
	return `${hari} ${BULAN[bulan - 1].slice(0, 3)}`;
}

export function hariIndo(tanggal: string): string {
	const hari = new Date(`${tanggal.slice(0, 10)}T00:00:00`).getDay();
	return HARI[hari] ?? '';
}

export function tanggalHariIni(): string {
	const sekarang = new Date();
	const offset = sekarang.getTimezoneOffset();
	return new Date(sekarang.getTime() - offset * 60_000).toISOString().slice(0, 10);
}

export function awalBulan(): string {
	return `${tanggalHariIni().slice(0, 7)}-01`;
}

export function geserTanggal(tanggal: string, hari: number): string {
	const nilai = new Date(`${tanggal.slice(0, 10)}T00:00:00Z`);
	nilai.setUTCDate(nilai.getUTCDate() + hari);
	return nilai.toISOString().slice(0, 10);
}

export function jamTeks(iso: string | null | undefined): string {
	if (!iso) return '-';
	const cocok = iso.match(/T(\d{2}):(\d{2})/);
	return cocok ? `${cocok[1]}:${cocok[2]}` : iso.slice(11, 16);
}

export function tanggalJamTeks(iso: string | null | undefined): string {
	if (!iso) return '-';
	return `${tanggalIndo(iso.slice(0, 10))} · ${jamTeks(iso)}`;
}

export function angka(value: number | null | undefined, digit = 0): string {
	if (value === null || value === undefined || Number.isNaN(value)) return '-';
	return value.toFixed(digit).replace('.', ',');
}

export function meter(value: number | null | undefined): string {
	if (value === null || value === undefined) return '-';
	return `${Math.round(value)} m`;
}

export function ukuranBerkas(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
	return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function warnaStatus(status: StatusKehadiran): string {
	switch (status) {
		case 'dinas_kantor':
			return 'bg-emerald-100 text-emerald-700';
		case 'dinas_luar':
			return 'bg-sky-100 text-sky-700';
		case 'ph':
			return 'bg-indigo-100 text-indigo-700';
		case 'piket':
			return 'bg-violet-100 text-violet-700';
		case 'cuti':
		case 'izin':
			return 'bg-amber-100 text-amber-800';
		case 'sakit':
			return 'bg-rose-100 text-rose-700';
		case 'tanpa_keterangan':
			return 'bg-rose-100 text-rose-700';
		default:
			return 'bg-ink-100 text-ink-600';
	}
}

export function statusRencanaWarna(status: string): string {
	switch (status) {
		case 'disetujui':
			return 'bg-emerald-100 text-emerald-700';
		case 'perlu_revisi':
			return 'bg-amber-100 text-amber-800';
		case 'ditolak':
			return 'bg-rose-100 text-rose-700';
		case 'diajukan':
			return 'bg-sky-100 text-sky-700';
		default:
			return 'bg-ink-100 text-ink-600';
	}
}

export function statusHasilWarna(status: string): string {
	switch (status) {
		case 'terverifikasi':
			return 'bg-emerald-100 text-emerald-700';
		case 'perlu_revisi':
			return 'bg-amber-100 text-amber-800';
		case 'diajukan':
			return 'bg-sky-100 text-sky-700';
		default:
			return 'bg-ink-100 text-ink-600';
	}
}

export function statusOutputWarna(status: string): string {
	switch (status) {
		case 'disetujui':
			return 'bg-emerald-100 text-emerald-700';
		case 'perlu_revisi':
			return 'bg-amber-100 text-amber-800';
		case 'menunggu_verifikasi':
		case 'dalam_verifikasi':
			return 'bg-sky-100 text-sky-700';
		default:
			return 'bg-ink-100 text-ink-600';
	}
}

export function warnaKategori(kategori: string): string {
	switch (kategori) {
		case 'sangat_baik':
			return 'bg-emerald-100 text-emerald-700';
		case 'baik':
			return 'bg-sky-100 text-sky-700';
		case 'cukup':
			return 'bg-amber-100 text-amber-800';
		default:
			return 'bg-rose-100 text-rose-700';
	}
}
