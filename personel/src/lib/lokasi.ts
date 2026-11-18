export interface Posisi {
	latitude: number;
	longitude: number;
	akurasi: number | null;
}

/** Ambil lokasi sekali (tanpa pelacakan latar belakang), setara `Lokasi.ambil` Android. */
export function ambilLokasi(): Promise<Posisi> {
	return new Promise((resolve, reject) => {
		if (typeof navigator === 'undefined' || !navigator.geolocation) {
			reject(new Error('Perangkat ini tidak mendukung layanan lokasi.'));
			return;
		}
		navigator.geolocation.getCurrentPosition(
			(posisi) =>
				resolve({
					latitude: posisi.coords.latitude,
					longitude: posisi.coords.longitude,
					akurasi: Number.isFinite(posisi.coords.accuracy) ? posisi.coords.accuracy : null
				}),
			(galat) => {
				switch (galat.code) {
					case galat.PERMISSION_DENIED:
						reject(new Error('Izin lokasi ditolak. Aktifkan izin lokasi di browser.'));
						break;
					case galat.POSITION_UNAVAILABLE:
						reject(new Error('Lokasi tidak tersedia. Pastikan GPS aktif.'));
						break;
					default:
						reject(new Error('Gagal mengambil lokasi. Coba lagi.'));
				}
			},
			{ enableHighAccuracy: true, timeout: 25_000, maximumAge: 0 }
		);
	});
}

/** Reverse geocoding memakai layanan Nominatim mandiri (sama dengan Android). */
export async function alamatLokasi(latitude: number, longitude: number): Promise<string | null> {
	const url =
		`https://geocode.vinrul.my.id/reverse?format=jsonv2&lat=${latitude}&lon=${longitude}` +
		'&zoom=18&addressdetails=1&accept-language=id';
	try {
		const res = await fetch(url, {
			headers: { Accept: 'application/json' },
			signal: AbortSignal.timeout(10_000)
		});
		if (!res.ok) return null;
		const data = (await res.json()) as { display_name?: string };
		return data.display_name?.trim() || null;
	} catch {
		return null;
	}
}

/** Kecilkan foto lalu ubah menjadi base64 JPEG (tanpa prefix data URL). */
export async function fotoKeBase64(
	file: File,
	sisiMaks = 1280,
	kualitas = 0.82
): Promise<string> {
	const bitmap = await createImageBitmap(file);
	const skala = Math.max(1, Math.max(bitmap.width, bitmap.height) / sisiMaks);
	const lebar = Math.round(bitmap.width / skala);
	const tinggi = Math.round(bitmap.height / skala);

	const kanvas = document.createElement('canvas');
	kanvas.width = lebar;
	kanvas.height = tinggi;
	const konteks = kanvas.getContext('2d');
	if (!konteks) throw new Error('Browser tidak dapat mengolah foto.');
	konteks.drawImage(bitmap, 0, 0, lebar, tinggi);
	bitmap.close();

	const dataUrl = kanvas.toDataURL('image/jpeg', kualitas);
	return dataUrl.split(',')[1] ?? '';
}

export function adaInternet(): boolean {
	return typeof navigator === 'undefined' ? true : navigator.onLine;
}
