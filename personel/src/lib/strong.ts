import type { AbsensiPh, AbsensiPhAntrean } from '#lib/types';

export interface SesiStrongPoint {
	jenis: 'masuk' | 'pulang';
	waktu: number;
	jam: string;
	menungguKirim: boolean;
}

export interface StatusStrongPoint {
	sesi: SesiStrongPoint[];
	masukTerakhir: SesiStrongPoint | null;
	pulangTerakhir: SesiStrongPoint | null;
	terbuka: boolean;
	perluCheckInBaru: boolean;
}

function jamWita(waktu: Date): string {
	return new Intl.DateTimeFormat('id-ID', {
		hour: '2-digit',
		minute: '2-digit',
		hour12: false,
		timeZone: 'Asia/Makassar'
	}).format(waktu);
}

function dariIso(iso: string): number {
	const waktu = Date.parse(iso);
	return Number.isFinite(waktu) ? waktu : Date.now();
}

/**
 * Menggabungkan absensi server dan antrean offline menjadi urutan sesi.
 * Check-in terbuka lebih dari 2 jam dianggap sesi pagi yang tidak check-out.
 */
export function statusStrongPoint(
	jadwalId: string | null | undefined,
	absensi: AbsensiPh[],
	antrean: AbsensiPhAntrean[],
	sekarang = Date.now()
): StatusStrongPoint {
	if (!jadwalId) {
		return {
			sesi: [],
			masukTerakhir: null,
			pulangTerakhir: null,
			terbuka: false,
			perluCheckInBaru: false
		};
	}

	const dariServer: SesiStrongPoint[] = absensi
		.filter((item) => item.jadwal_ph_id === jadwalId)
		.map((item) => ({
			jenis: item.jenis,
			waktu: dariIso(item.captured_at),
			jam: item.jam,
			menungguKirim: false
		}));

	const dariAntrean: SesiStrongPoint[] = antrean
		.filter((item) => item.jadwal_ph_id === jadwalId)
		.map((item) => ({
			jenis: item.jenis,
			waktu: dariIso(item.captured_at),
			jam: jamWita(new Date(item.captured_at)),
			menungguKirim: true
		}));

	const sesi = [...dariServer, ...dariAntrean].sort((a, b) => a.waktu - b.waktu);
	const masukTerakhir = [...sesi].reverse().find((item) => item.jenis === 'masuk') ?? null;
	const pulangTerakhir = [...sesi].reverse().find((item) => item.jenis === 'pulang') ?? null;
	const terbuka =
		masukTerakhir !== null &&
		(pulangTerakhir === null || masukTerakhir.waktu > pulangTerakhir.waktu);

	return { sesi, masukTerakhir, pulangTerakhir, terbuka, perluCheckInBaru: false };
}

export function keteranganJadwal(regu: string | null, strongPoint: string): string {
	return [strongPoint, regu?.trim()].filter(Boolean).join(' · ');
}
