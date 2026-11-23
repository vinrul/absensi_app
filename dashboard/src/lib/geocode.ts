// Reverse geocode memakai Nominatim. Permintaan diserialkan maksimal 1/detik
// sesuai kebijakan penggunaan Nominatim dan hasilnya di-cache.
const cache = new Map<string, string | null>();
let antrean: Promise<void> = Promise.resolve();
let waktuTerakhir = 0;

export async function alamatDariKoordinat(
	latitude: number,
	longitude: number
): Promise<string | null> {
	const kunci = `${latitude.toFixed(6)},${longitude.toFixed(6)}`;
	if (cache.has(kunci)) return cache.get(kunci) ?? null;

	let hasil: string | null = null;
	antrean = antrean.then(async () => {
		const jeda = 1000 - (Date.now() - waktuTerakhir);
		if (jeda > 0) await new Promise((selesai) => setTimeout(selesai, jeda));
		waktuTerakhir = Date.now();

		try {
			const params = new URLSearchParams({
				format: 'jsonv2',
				lat: String(latitude),
				lon: String(longitude),
				zoom: '18',
				'accept-language': 'id'
			});
			const respon = await fetch(`https://nominatim.openstreetmap.org/reverse?${params}`, {
				headers: { Accept: 'application/json' },
				signal: AbortSignal.timeout(15_000)
			});
			if (respon.ok) {
				const data = (await respon.json()) as { display_name?: string };
				hasil = data.display_name?.trim() || null;
			}
		} catch {
			hasil = null;
		}

		cache.set(kunci, hasil);
	});

	await antrean;
	return hasil;
}
