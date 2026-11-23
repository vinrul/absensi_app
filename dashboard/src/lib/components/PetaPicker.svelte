<script lang="ts">
	import { onMount } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import type {
		Circle,
		DivIcon,
		LeafletMouseEvent,
		Map as LeafletMap,
		Marker,
		TileLayer
	} from 'leaflet';
	import Icon from './Icon.svelte';

	type ModulLeaflet = typeof import('leaflet');
	type ModePeta = 'peta' | 'satelit';

	type HasilCari = {
		place_id: number;
		lat: string;
		lon: string;
		display_name: string;
		boundingbox?: [string, string, string, string];
	};

	let {
		latitude = null,
		longitude = null,
		radius = 100,
		onchange
	}: {
		latitude?: number | null;
		longitude?: number | null;
		radius?: number;
		onchange?: (latitude: number, longitude: number) => void;
	} = $props();

	const PUSAT_BALI: [number, number] = [-8.65629, 115.22144];
	const URL_PETA = 'https://tile.openstreetmap.org/{z}/{x}/{y}.png';
	const URL_SATELIT =
		'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}';
	const ATRIBUSI_PETA =
		'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors';
	const ATRIBUSI_SATELIT =
		'Tiles &copy; Esri &mdash; Source: Esri, i-cubed, USDA, USGS, AEX, GeoEye, Getmapping, Aerogrid, IGN, IGP, UPR-EGP, and the GIS User Community';
	const URL_NOMINATIM = 'https://nominatim.openstreetmap.org/search';

	let kontainer: HTMLDivElement | undefined;
	let areaCari = $state<HTMLDivElement | null>(null);
	let peta: LeafletMap | null = null;
	let penanda: Marker | null = null;
	let lingkaran: Circle | null = null;
	let lapisanPeta: TileLayer | null = null;
	let L: ModulLeaflet | null = null;
	let siap = $state(false);
	let modePeta = $state<ModePeta>('peta');
	let kataKunci = $state('');
	let hasilCari = $state<HasilCari[]>([]);
	let pesanCari = $state<string | null>(null);
	let mencari = $state(false);
	let waktuCariTerakhir = 0;
	let nomorCari = 0;
	let timerCari: ReturnType<typeof setTimeout> | null = null;
	let pengendaliCari: AbortController | null = null;
	const cacheCari = new Map<string, HasilCari[]>();

	const JEDA_CARI = 700;

	function koordinatValid(lat: number, lng: number): boolean {
		return (
			Number.isFinite(lat) &&
			Number.isFinite(lng) &&
			lat >= -90 &&
			lat <= 90 &&
			lng >= -180 &&
			lng <= 180
		);
	}

	function ikonPenanda(): DivIcon {
		return L!.divIcon({
			className: 'peta-penanda',
			html: '<span style="display:block;box-sizing:border-box;width:18px;height:18px;border-radius:9999px;background:#1d4ed8;border:3px solid #fff;box-shadow:0 0 0 2px #1d4ed8"></span>',
			iconSize: [18, 18],
			iconAnchor: [9, 9]
		});
	}

	function tampilkan(lat: number, lng: number, geser: boolean) {
		if (!L || !peta) return;
		const target: [number, number] = [lat, lng];
		const radiusAman = Math.max(1, Number.isFinite(radius) ? radius : 100);

		if (!penanda) {
			penanda = L.marker(target, { draggable: true, icon: ikonPenanda() }).addTo(peta);
			penanda.on('dragend', () => {
				const posisi = penanda?.getLatLng();
				if (posisi) pilih(posisi.lat, posisi.lng, false);
			});
			if (geser) peta.panTo(target);
		} else {
			const posisi = penanda.getLatLng();
			const berubah =
				Math.abs(posisi.lat - lat) > 1e-9 || Math.abs(posisi.lng - lng) > 1e-9;
			if (berubah) {
				penanda.setLatLng(target);
				if (geser && !peta.getBounds().contains(target)) peta.panTo(target);
			}
		}

		if (!lingkaran) {
			lingkaran = L.circle(target, {
				radius: radiusAman,
				color: '#1d4ed8',
				weight: 1,
				fillColor: '#3b82f6',
				fillOpacity: 0.15
			}).addTo(peta);
		} else {
			lingkaran.setLatLng(target);
			lingkaran.setRadius(radiusAman);
		}
	}

	function pilih(lat: number, lng: number, geser = false) {
		tampilkan(lat, lng, geser);
		onchange?.(lat, lng);
	}

	function hapusPenanda() {
		penanda?.remove();
		lingkaran?.remove();
		penanda = null;
		lingkaran = null;
	}

	function pasangLapisan(mode: ModePeta) {
		if (!L || !peta) return;
		modePeta = mode;
		if (lapisanPeta) peta.removeLayer(lapisanPeta);
		lapisanPeta = L.tileLayer(mode === 'satelit' ? URL_SATELIT : URL_PETA, {
			maxZoom: 19,
			attribution: mode === 'satelit' ? ATRIBUSI_SATELIT : ATRIBUSI_PETA
		}).addTo(peta);
	}

	async function cariAlamat(teks: string) {
		const q = teks.trim().replace(/\s+/g, ' ');
		if (q.length < 3) {
			pesanCari = 'Ketik minimal 3 karakter untuk mencari alamat.';
			hasilCari = [];
			return;
		}

		const kunci = q.toLowerCase();
		const tersimpan = cacheCari.get(kunci);
		if (tersimpan) {
			batalkanCari();
			hasilCari = tersimpan;
			pesanCari = tersimpan.length ? null : 'Alamat tidak ditemukan. Coba kata kunci lain.';
			return;
		}

		const nomor = ++nomorCari;
		pengendaliCari?.abort();
		const pengendali = new AbortController();
		pengendaliCari = pengendali;

		mencari = true;
		pesanCari = null;
		hasilCari = [];
		try {
			const jeda = 1000 - (Date.now() - waktuCariTerakhir);
			if (jeda > 0) await new Promise((selesai) => setTimeout(selesai, jeda));
			if (nomor !== nomorCari) return;

			waktuCariTerakhir = Date.now();
			const params = new URLSearchParams({
				format: 'jsonv2',
				q,
				limit: '5',
				'accept-language': 'id',
				countrycodes: 'id',
				viewbox: '114.40,-9.20,115.80,-8.00',
				bounded: '0'
			});
			const respon = await fetch(`${URL_NOMINATIM}?${params}`, {
				headers: { Accept: 'application/json' },
				signal: pengendali.signal
			});
			if (!respon.ok) throw new Error(`Nominatim ${respon.status}`);
			const hasil = (await respon.json()) as HasilCari[];
			if (nomor !== nomorCari) return;

			cacheCari.set(kunci, hasil);
			hasilCari = hasil;
			if (!hasil.length) pesanCari = 'Alamat tidak ditemukan. Coba kata kunci lain.';
		} catch (err) {
			if (nomor !== nomorCari || (err instanceof DOMException && err.name === 'AbortError')) {
				return;
			}
			pesanCari = 'Gagal mencari alamat. Periksa koneksi internet.';
		} finally {
			if (nomor === nomorCari) mencari = false;
		}
	}

	function batalkanCari() {
		nomorCari += 1;
		pengendaliCari?.abort();
		mencari = false;
	}

	function jadwalkanCari(teks: string) {
		if (timerCari) {
			clearTimeout(timerCari);
			timerCari = null;
		}
		if (mencari) batalkanCari();
		hasilCari = [];
		pesanCari = null;

		const q = teks.trim().replace(/\s+/g, ' ');
		if (q.length < 3) return;

		timerCari = setTimeout(() => {
			timerCari = null;
			void cariAlamat(q);
		}, JEDA_CARI);
	}

	function pilihHasil(hasil: HasilCari) {
		const lat = Number(hasil.lat);
		const lng = Number(hasil.lon);
		if (!koordinatValid(lat, lng)) return;

		if (timerCari) {
			clearTimeout(timerCari);
			timerCari = null;
		}
		batalkanCari();
		kataKunci = hasil.display_name;
		hasilCari = [];
		pesanCari = null;

		if (peta) {
			const batas = hasil.boundingbox?.map(Number);
			if (batas && batas.length === 4 && batas.every((nilai) => Number.isFinite(nilai))) {
				const kotak: [[number, number], [number, number]] = [
					[batas[0], batas[2]],
					[batas[1], batas[3]]
				];
				peta.fitBounds(kotak, { maxZoom: 17, padding: [24, 24] });
			} else {
				peta.setView([lat, lng], 17);
			}
		}
		pilih(lat, lng);
	}

	onMount(() => {
		let dibatalkan = false;
		let pengamat: ResizeObserver | null = null;

		void (async () => {
			const modul = await import('leaflet');
			if (dibatalkan || !kontainer) return;

			L = (modul as unknown as { default?: ModulLeaflet }).default ?? modul;

			let pusat: [number, number] = PUSAT_BALI;
			let zoom = 10;
			if (
				latitude !== null &&
				longitude !== null &&
				koordinatValid(latitude, longitude)
			) {
				pusat = [latitude, longitude];
				zoom = 16;
			}

			peta = L.map(kontainer, { zoomControl: true }).setView(pusat, zoom);
			pasangLapisan('peta');

			peta.on('click', (e: LeafletMouseEvent) => {
				hasilCari = [];
				pilih(e.latlng.lat, e.latlng.lng);
			});

			pengamat = new ResizeObserver(() => peta?.invalidateSize());
			pengamat.observe(kontainer);
			siap = true;
			setTimeout(() => peta?.invalidateSize(), 80);
		})();

		return () => {
			dibatalkan = true;
			nomorCari += 1;
			if (timerCari) clearTimeout(timerCari);
			pengendaliCari?.abort();
			pengamat?.disconnect();
			peta?.remove();
			peta = null;
			penanda = null;
			lingkaran = null;
			lapisanPeta = null;
		};
	});

	$effect(() => {
		const lat = latitude;
		const lng = longitude;
		const r = radius;
		if (!siap) return;
		if (lat === null && lng === null) {
			hapusPenanda();
			return;
		}
		if (lat === null || lng === null || !koordinatValid(lat, lng)) return;
		tampilkan(lat, lng, true);
		if (lingkaran) lingkaran.setRadius(Math.max(1, r));
	});

	$effect(() => {
		if (!hasilCari.length) return;
		const padaPointer = (e: PointerEvent) => {
			if (!areaCari || (e.target instanceof Node && areaCari.contains(e.target))) return;
			hasilCari = [];
		};
		document.addEventListener('pointerdown', padaPointer, true);
		return () => document.removeEventListener('pointerdown', padaPointer, true);
	});
</script>

<div class="space-y-2">
	<div class="relative z-10" bind:this={areaCari}>
		<div class="relative">
			<span
				class="pointer-events-none absolute inset-y-0 left-2.5 flex items-center text-slate-400"
			>
				<Icon name="search" size={15} />
			</span>
			<input
				class="input pl-8"
				type="search"
				placeholder="Cari alamat atau nama tempat…"
				aria-label="Cari alamat pada peta"
				autocomplete="off"
				bind:value={kataKunci}
				oninput={(e) => jadwalkanCari(e.currentTarget.value)}
				onkeydown={(e) => {
					if (e.key === 'Enter') {
						e.preventDefault();
						if (timerCari) {
							clearTimeout(timerCari);
							timerCari = null;
						}
						void cariAlamat(kataKunci);
					} else if (e.key === 'Escape') {
						hasilCari = [];
					}
				}}
			/>
		</div>
		{#if mencari}
			<p class="mt-1 text-[11px] text-slate-500">Mencari alamat…</p>
		{:else if pesanCari}
			<p class="mt-1 text-[11px] text-slate-500">{pesanCari}</p>
		{/if}
		{#if hasilCari.length}
			<ul
				class="absolute inset-x-0 top-full z-20 mt-1 max-h-56 overflow-y-auto rounded-lg border border-slate-200 bg-white py-1 shadow-xl"
			>
				{#each hasilCari as hasil (hasil.place_id)}
					<li>
						<button
							type="button"
							class="block w-full px-3 py-2 text-left text-xs text-slate-700 hover:bg-slate-50"
							onclick={() => pilihHasil(hasil)}
						>
							<span class="line-clamp-2">{hasil.display_name}</span>
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</div>

	<div class="relative isolate z-0">
		<div
			bind:this={kontainer}
			class="relative z-0 h-72 w-full overflow-hidden rounded-lg border border-slate-200 bg-slate-100 sm:h-96"
		></div>
		<div
			class="absolute right-2 top-2 z-[1000] flex overflow-hidden rounded-lg border border-slate-200 bg-white shadow"
		>
			<button
				type="button"
				class="px-2.5 py-1 text-xs font-medium transition"
				class:bg-navy-800={modePeta === 'peta'}
				class:text-white={modePeta === 'peta'}
				class:text-slate-600={modePeta !== 'peta'}
				aria-pressed={modePeta === 'peta'}
				onclick={() => pasangLapisan('peta')}
			>
				Peta
			</button>
			<button
				type="button"
				class="px-2.5 py-1 text-xs font-medium transition"
				class:bg-navy-800={modePeta === 'satelit'}
				class:text-white={modePeta === 'satelit'}
				class:text-slate-600={modePeta !== 'satelit'}
				aria-pressed={modePeta === 'satelit'}
				onclick={() => pasangLapisan('satelit')}
			>
				Satelit
			</button>
		</div>
		{#if !siap}
			<div
				class="pointer-events-none absolute inset-0 flex items-center justify-center text-xs text-slate-500"
			>
				Memuat peta…
			</div>
		{/if}
	</div>
	<p class="mt-1 text-[11px] text-slate-500">
		Klik peta atau geser penanda untuk memilih koordinat. Lingkaran biru menunjukkan radius titik
		dinas. Pencarian alamat berjalan otomatis setelah minimal 3 karakter, oleh <a
			class="underline"
			href="https://www.openstreetmap.org/copyright"
			target="_blank"
			rel="noreferrer">OpenStreetMap (Nominatim)</a
		>.
	</p>
</div>
