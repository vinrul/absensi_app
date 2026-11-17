<script lang="ts">
	import { onMount } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import type {
		Circle,
		DivIcon,
		LeafletMouseEvent,
		Map as LeafletMap,
		Marker
	} from 'leaflet';

	type ModulLeaflet = typeof import('leaflet');

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

	let kontainer: HTMLDivElement | undefined;
	let peta: LeafletMap | null = null;
	let penanda: Marker | null = null;
	let lingkaran: Circle | null = null;
	let L: ModulLeaflet | null = null;
	let siap = $state(false);

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
			L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
				maxZoom: 19,
				attribution:
					'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
			}).addTo(peta);

			peta.on('click', (e: LeafletMouseEvent) => pilih(e.latlng.lat, e.latlng.lng));

			pengamat = new ResizeObserver(() => peta?.invalidateSize());
			pengamat.observe(kontainer);
			siap = true;
			setTimeout(() => peta?.invalidateSize(), 80);
		})();

		return () => {
			dibatalkan = true;
			pengamat?.disconnect();
			peta?.remove();
			peta = null;
			penanda = null;
			lingkaran = null;
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
</script>

<div class="relative">
	<div
		bind:this={kontainer}
		class="relative z-0 isolate h-64 w-full overflow-hidden rounded-lg border border-slate-200 bg-slate-100"
	></div>
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
	dinas.
</p>
