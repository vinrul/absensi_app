<script lang="ts">
	import { onMount } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import type { DivIcon, Map as LeafletMap, Marker, TileLayer } from 'leaflet';

	type ModulLeaflet = typeof import('leaflet');
	type ModePeta = 'peta' | 'satelit';

	let {
		latitude,
		longitude,
		label = 'Lokasi absensi'
	}: {
		latitude: number;
		longitude: number;
		label?: string;
	} = $props();

	const URL_PETA = 'https://tile.openstreetmap.org/{z}/{x}/{y}.png';
	const URL_SATELIT =
		'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}';
	const ATRIBUSI_PETA =
		'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors';
	const ATRIBUSI_SATELIT =
		'Tiles &copy; Esri &mdash; Source: Esri, i-cubed, USDA, USGS, AEX, GeoEye, Getmapping, Aerogrid, IGN, IGP, UPR-EGP, and the GIS User Community';

	let kontainer = $state<HTMLDivElement | undefined>();
	let peta: LeafletMap | null = null;
	let penanda: Marker | null = null;
	let lapisan: TileLayer | null = null;
	let L: ModulLeaflet | null = null;
	let siap = $state(false);
	let mode = $state<ModePeta>('peta');

	const valid = $derived(Number.isFinite(latitude) && Number.isFinite(longitude));

	function ikon(): DivIcon {
		return L!.divIcon({
			className: 'peta-penanda',
			html: '<span style="display:block;box-sizing:border-box;width:20px;height:20px;border-radius:9999px;background:#dc2626;border:3px solid #fff;box-shadow:0 0 0 2px #dc2626"></span>',
			iconSize: [20, 20],
			iconAnchor: [10, 10]
		});
	}

	function pasangLapisan(baru: ModePeta) {
		if (!L || !peta) return;
		mode = baru;
		if (lapisan) peta.removeLayer(lapisan);
		lapisan = L.tileLayer(baru === 'satelit' ? URL_SATELIT : URL_PETA, {
			maxZoom: 19,
			attribution: baru === 'satelit' ? ATRIBUSI_SATELIT : ATRIBUSI_PETA
		}).addTo(peta);
	}

	onMount(() => {
		if (!valid) return;
		let dibatalkan = false;
		let pengamat: ResizeObserver | null = null;

		void (async () => {
			const modul = await import('leaflet');
			if (dibatalkan || !kontainer) return;
			L = (modul as unknown as { default?: ModulLeaflet }).default ?? modul;

			peta = L.map(kontainer, { zoomControl: true }).setView([latitude, longitude], 17);
			pasangLapisan('peta');
			penanda = L.marker([latitude, longitude], { icon: ikon(), title: label })
				.addTo(peta)
				.bindPopup(label);

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
			lapisan = null;
		};
	});
</script>

{#if !valid}
	<div
		class="flex h-64 items-center justify-center rounded-lg border border-slate-200 bg-slate-50 text-xs text-slate-500"
	>
		Koordinat lokasi tidak valid.
	</div>
{:else}
	<div class="space-y-2">
		<div class="relative isolate z-0">
			<div
				bind:this={kontainer}
				class="relative z-0 h-64 w-full overflow-hidden rounded-lg border border-slate-200 bg-slate-100"
			></div>
			<div
				class="absolute right-2 top-2 z-[1000] flex overflow-hidden rounded-lg border border-slate-200 bg-white shadow"
			>
				<button
					type="button"
					class="px-2.5 py-1 text-xs font-medium transition"
					class:bg-navy-800={mode === 'peta'}
					class:text-white={mode === 'peta'}
					class:text-slate-600={mode !== 'peta'}
					aria-pressed={mode === 'peta'}
					onclick={() => pasangLapisan('peta')}
				>
					Peta
				</button>
				<button
					type="button"
					class="px-2.5 py-1 text-xs font-medium transition"
					class:bg-navy-800={mode === 'satelit'}
					class:text-white={mode === 'satelit'}
					class:text-slate-600={mode !== 'satelit'}
					aria-pressed={mode === 'satelit'}
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
		<div class="flex flex-wrap items-center justify-between gap-2 text-[11px] text-slate-500">
			<span>{latitude.toFixed(6)}, {longitude.toFixed(6)}</span>
			<a
				class="underline hover:text-slate-700"
				href="https://www.openstreetmap.org/?mlat={latitude}&mlon={longitude}#map=17/{latitude}/{longitude}"
				target="_blank"
				rel="noreferrer"
			>
				Buka di OpenStreetMap
			</a>
		</div>
	</div>
{/if}
