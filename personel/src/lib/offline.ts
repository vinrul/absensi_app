import { api } from '#lib/api/client';
import type { AbsensiAntrean, AbsensiPhAntrean } from '#lib/types';

function kunciAbsensi(userId: string) {
	return `personel.antrean.absensi.${userId}`;
}

function kunciPh(userId: string) {
	return `personel.antrean.ph.${userId}`;
}

function baca<T>(kunci: string): T[] {
	try {
		const teks = localStorage.getItem(kunci);
		if (!teks) return [];
		const data = JSON.parse(teks) as T[];
		return Array.isArray(data) ? data : [];
	} catch {
		return [];
	}
}

function tulis<T>(kunci: string, data: T[]) {
	try {
		localStorage.setItem(kunci, JSON.stringify(data));
	} catch {
		// Penyimpanan penuh atau mode privat: antrean diabaikan.
	}
}

export function bacaAntreanAbsensi(userId: string): AbsensiAntrean[] {
	return baca<AbsensiAntrean>(kunciAbsensi(userId));
}

export function bacaAntreanPh(userId: string): AbsensiPhAntrean[] {
	return baca<AbsensiPhAntrean>(kunciPh(userId));
}

export function tambahAntreanAbsensi(userId: string, item: AbsensiAntrean) {
	tulis(kunciAbsensi(userId), [...bacaAntreanAbsensi(userId), item]);
}

export function tambahAntreanPh(userId: string, item: AbsensiPhAntrean) {
	tulis(kunciPh(userId), [...bacaAntreanPh(userId), item]);
}

export function hapusAntreanAbsensi(userId: string, clientUuid: string) {
	tulis(
		kunciAbsensi(userId),
		bacaAntreanAbsensi(userId).filter((item) => item.client_uuid !== clientUuid)
	);
}

export function hapusAntreanPh(userId: string, clientUuid: string) {
	tulis(
		kunciPh(userId),
		bacaAntreanPh(userId).filter((item) => item.client_uuid !== clientUuid)
	);
}

export function kosongkanAntrean(userId: string) {
	try {
		localStorage.removeItem(kunciAbsensi(userId));
		localStorage.removeItem(kunciPh(userId));
	} catch {
		// abaikan
	}
}

export interface HasilSinkron {
	absensi: number;
	ph: number;
}

/** Kirim seluruh antrean offline; kegagalan jaringan menyisakan item untuk dicoba lagi. */
export async function sinkronkanAntrean(userId: string): Promise<HasilSinkron> {
	let absensi = 0;
	let ph = 0;

	for (const item of bacaAntreanAbsensi(userId)) {
		try {
			await api('/absensi', {
				method: 'POST',
				body: { ...item, is_offline: true }
			});
			hapusAntreanAbsensi(userId, item.client_uuid);
			absensi += 1;
		} catch {
			// Antrean absensi dipertahankan agar dapat diperiksa ulang (mis. di luar radius).
		}
	}

	for (const item of bacaAntreanPh(userId)) {
		try {
			await api('/ph/absensi', {
				method: 'POST',
				body: { ...item, is_offline: true }
			});
			hapusAntreanPh(userId, item.client_uuid);
			ph += 1;
		} catch (err) {
			if (err instanceof Error && 'status' in err) {
				const status = (err as { status: number }).status;
				if (status >= 400 && status < 500 && status !== 401) {
					hapusAntreanPh(userId, item.client_uuid);
				}
			}
		}
	}

	return { absensi, ph };
}
