<script lang="ts">
	import { onMount } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import type { Circle, DivIcon, Map as LeafletMap, Marker } from 'leaflet';
	import type { TitikDinas } from '#lib/types';

	type ModulLeaflet = typeof import('leaflet');

	let {
		latitude = null,
		longitude = null,
		akurasi = null,
		titik = null,
		warna = 'hijau',
		tinggi = 'h-56'
	}: {
		latitude?: number | null;
		longitude?: number | null;
		akurasi?: number | null;
		titik?: TitikDinas | null;
		warna?: 'hijau' | 'merah';
		tinggi?: string;
	} = $props();

	const PUSAT_BALI: [number, number] = [-8.65629, 115.22144];

	let kontainer: HTMLDivElement | undefined;
	let peta: LeafletMap | null = null;
	let L: ModulLeaflet | null = null;
	let siap = $state(false);

	let penandaLokasi: Marker | null = null;
	let lingkaranAkurasi: Circle | null = null;
	let penandaTitik: Marker | null = null;
	let lingkaranTitik: Circle | null = null;

	function valid(lat: number, lng: number) {
		return Number.isFinite(lat) && Number.isFinite(lng) && lat >= -90 && lat <= 90 && lng >= -180 && lng <= 180;
	}

	function ikon(warnaTitik: string): DivIcon {
		return L!.divIcon({
			className: 'peta-penanda',
			html: `<span style="display:block;box-sizing:border-box;width:20px;height:20px;border-radius:9999px;background:${warnaTitik};border:3px solid #fff;box-shadow:0 0 0 3px ${warnaTitik}55"></span>`,
			iconSize: [20, 20],
			iconAnchor: [10, 10]
		});
	}

	function perbarui() {
		if (!L || !peta) return;
		const lat = latitude;
		const lng = longitude;

		if (lat !== null && lng !== null && valid(lat, lng)) {
			const target: [number, number] = [lat, lng];
			if (!penandaLokasi) {
				penandaLokasi = L.marker(target, { icon: ikon(warna === 'hijau' ? '#16a34a' : '#dc2626') }).addTo(peta);
			} else {
				penandaLokasi.setLatLng(target);
			}
			const radiusAkurasi = Math.max(5, akurasi ?? 10);
			if (!lingkaranAkurasi) {
				lingkaranAkurasi = L.circle(target, {
					radius: radiusAkurasi,
					color: warna === 'hijau' ? '#16a34a' : '#dc2626',
					weight: 1,
					fillColor: warna === 'hijau' ? '#22c55e' : '#ef4444',
					fillOpacity: 0.12
				}).addTo(peta);
			} else {
				lingkaranAkurasi.setLatLng(target);
				lingkaranAkurasi.setRadius(radiusAkurasi);
			}
			if (!peta.getBounds().contains(target)) peta.panTo(target);
		}

		if (titik) {
			const target: [number, number] = [titik.latitude, titik.longitude];
			if (!penandaTitik) {
				penandaTitik = L.marker(target, { icon: ikon('#0b8a74') }).addTo(peta);
				penandaTitik.bindTooltip(titik.nama, { direction: 'top', offset: [0, -8] });
			} else {
				penandaTitik.setLatLng(target);
			}
			if (!lingkaranTitik) {
				lingkaranTitik = L.circle(target, {
					radius: Math.max(1, titik.radius_m),
					color: '#0b8a74',
					weight: 1,
					dashArray: '4 4',
					fillColor: '#14a88d',
					fillOpacity: 0.1
				}).addTo(peta);
			} else {
				lingkaranTitik.setLatLng(target);
				lingkaranTitik.setRadius(Math.max(1, titik.radius_m));
			}
		}
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
			if (latitude !== null && longitude !== null && valid(latitude, longitude)) {
				pusat = [latitude, longitude];
				zoom = 17;
			} else if (titik) {
				pusat = [titik.latitude, titik.longitude];
				zoom = 16;
			}

			peta = L.map(kontainer, { zoomControl: true, attributionControl: true }).setView(pusat, zoom);
			L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
				maxZoom: 19,
				attribution: '&copy; OpenStreetMap'
			}).addTo(peta);

			pengamat = new ResizeObserver(() => peta?.invalidateSize());
			pengamat.observe(kontainer);
			siap = true;
			perbarui();
			setTimeout(() => peta?.invalidateSize(), 80);
		})();

		return () => {
			dibatalkan = true;
			pengamat?.disconnect();
			peta?.remove();
			peta = null;
			penandaLokasi = null;
			lingkaranAkurasi = null;
			penandaTitik = null;
			lingkaranTitik = null;
		};
	});

	$effect(() => {
		latitude;
		longitude;
		akurasi;
		titik;
		warna;
		if (siap) perbarui();
	});
</script>

<div class="relative">
	<div
		bind:this={kontainer}
		class="relative z-0 isolate {tinggi} w-full overflow-hidden rounded-2xl bg-ink-100"
	></div>
	{#if !siap}
		<div class="pointer-events-none absolute inset-0 flex items-center justify-center text-xs text-ink-500">
			Memuat peta…
		</div>
	{/if}
	{#if latitude !== null && longitude !== null}
		<span class="absolute right-2 bottom-2 z-[500] rounded-full bg-white/90 px-2 py-0.5 text-[10px] font-semibold text-ink-600 shadow">
			Lokasi Anda
		</span>
	{/if}
</div>
