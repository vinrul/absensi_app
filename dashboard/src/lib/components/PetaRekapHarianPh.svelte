<script lang="ts">
	import { onMount } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import type { LayerGroup, Map as LeafletMap, TileLayer } from 'leaflet';
	import type { RekapPhHarian } from '#lib/types';

	let { data }: { data: RekapPhHarian[] } = $props();

	type ModulLeaflet = typeof import('leaflet');
	type ModePeta = 'peta' | 'satelit';

	const PUSAT_BALI: [number, number] = [-8.65629, 115.22144];
	const URL_PETA = 'https://tile.openstreetmap.org/{z}/{x}/{y}.png';
	const URL_SATELIT =
		'https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}';
	const ATRIBUSI_PETA =
		'&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors';
	const ATRIBUSI_SATELIT =
		'Tiles &copy; Esri &mdash; Source: Esri, i-cubed, USDA, USGS, AEX, GeoEye, Getmapping, Aerogrid, IGN, IGP, UPR-EGP, and the GIS User Community';

	let kontainer = $state<HTMLDivElement | undefined>();
	let peta: LeafletMap | null = null;
	let lapisan: TileLayer | null = null;
	let grupTitik: LayerGroup | null = null;
	let grupCheckIn: LayerGroup | null = null;
	let L: ModulLeaflet | null = null;
	let siap = $state(false);
	let mode = $state<ModePeta>('peta');

	const titikUnik = $derived.by(() => {
		const petaTitik = new Map<
			string,
			{
				id: string;
				nama: string;
				latitude: number;
				longitude: number;
				radius_m: number;
			}
		>();
		for (const row of data) {
			if (!petaTitik.has(row.strong_point_id)) {
				petaTitik.set(row.strong_point_id, {
					id: row.strong_point_id,
					nama: row.strong_point_nama,
					latitude: row.strong_point_latitude,
					longitude: row.strong_point_longitude,
					radius_m: row.strong_point_radius_m
				});
			}
		}
		return [...petaTitik.values()];
	});

	const checkIn = $derived(
		data.filter(
			(row) => row.masuk_latitude !== null && row.masuk_longitude !== null
		)
	);
	const jumlahPersonel = $derived(new Set(data.map((row) => row.personel_id)).size);
	const jumlahTc = $derived(
		data.filter((row) => row.masuk_jam !== null && row.pulang_jam === null).length
	);

	function esc(nilai: string) {
		return nilai.replace(
			/[&<>"']/g,
			(karakter) =>
				({
					'&': '&amp;',
					'<': '&lt;',
					'>': '&gt;',
					'"': '&quot;',
					"'": '&#39;'
				})[karakter] ?? karakter
		);
	}

	function labelSesi(sesi: string | null) {
		if (sesi === 'pagi') return 'Pagi';
		if (sesi === 'sore') return 'Sore';
		return 'Sesi';
	}

	function statusCheckIn(row: RekapPhHarian) {
		return row.pulang_jam ? 'Lengkap' : 'TC';
	}

	function popupCheckIn(row: RekapPhHarian) {
		const bagian = [
			`<strong>${esc(row.nama)}</strong>`,
			`${esc(row.pangkat)} · ${esc(row.nrp)}`,
			esc(row.unit ?? '-'),
			`Sesi: <b>${labelSesi(row.sesi)} ${esc(row.jam_mulai)}–${esc(row.jam_selesai)}</b>`,
			`Strong point: ${esc(row.strong_point_nama)}`,
			`Check-in: <b>${esc(row.masuk_jam ?? '-')}</b> · Check-out: <b>${esc(row.pulang_jam ?? '-')}</b>`
		];
		if (row.pulang_jam === null) {
			bagian.push('<span style="color:#b45309;font-weight:600">TC · belum check-out</span>');
		}
		if (row.menit_telat > 0) bagian.push(`Telat: <b>${row.menit_telat} menit</b>`);
		if (row.menit_pulang_cepat > 0) {
			bagian.push(`Pulang cepat: <b>${row.menit_pulang_cepat} menit</b>`);
		}
		return `<div style="min-width:210px;font-size:12px;line-height:1.55">${bagian.join('<br>')}</div>`;
	}

	function pasangLapisan(baru: ModePeta) {
		if (!L || !peta) return;
		mode = baru;
		if (lapisan) peta.removeLayer(lapisan);
		lapisan = L.tileLayer(baru === 'satelit' ? URL_SATELIT : URL_PETA, {
			maxZoom: 19,
			attribution: baru === 'satelit' ? ATRIBUSI_SATELIT : ATRIBUSI_PETA
		}).addTo(peta);
		if (lapisan) lapisan.bringToBack();
	}

	function gambarUlang() {
		if (!L || !peta || !grupTitik || !grupCheckIn) return;
		grupTitik.clearLayers();
		grupCheckIn.clearLayers();

		const batas: [number, number][] = [];

		for (const titik of titikUnik) {
			const posisi: [number, number] = [titik.latitude, titik.longitude];
			batas.push(posisi);
			L.circle(posisi, {
				radius: Math.max(10, titik.radius_m),
				color: '#b45309',
				weight: 1.5,
				dashArray: '5 4',
				fillColor: '#f59e0b',
				fillOpacity: 0.08
			}).addTo(grupTitik);
			L.circleMarker(posisi, {
				radius: 7,
				color: '#b45309',
				weight: 2,
				fillColor: '#fde68a',
				fillOpacity: 1
			})
				.bindTooltip(`Strong point · ${esc(titik.nama)}`)
				.bindPopup(
					`<div style="min-width:180px;font-size:12px;line-height:1.5">
						<strong>${esc(titik.nama)}</strong><br>
						Strong point<br>
						Radius: <b>${titik.radius_m} m</b>
					</div>`
				)
				.addTo(grupTitik);
		}

		for (const row of checkIn) {
			if (row.masuk_latitude === null || row.masuk_longitude === null) continue;
			const posisi: [number, number] = [row.masuk_latitude, row.masuk_longitude];
			batas.push(posisi);
			const lengkap = row.pulang_jam !== null;
			L.circleMarker(posisi, {
				radius: 7,
				color: '#ffffff',
				weight: 1.5,
				fillColor: lengkap ? '#059669' : '#d97706',
				fillOpacity: 0.95
			})
				.bindTooltip(`${esc(row.nama)} · ${statusCheckIn(row)}`)
				.bindPopup(popupCheckIn(row))
				.addTo(grupCheckIn);
		}

		if (batas.length > 0) {
			peta.fitBounds(batas, { padding: [30, 30], maxZoom: 16 });
		} else {
			peta.setView(PUSAT_BALI, 10);
		}
	}

	$effect(() => {
		if (!siap) return;
		void data;
		gambarUlang();
	});

	onMount(() => {
		let dibatalkan = false;
		let pengamat: ResizeObserver | null = null;

		void (async () => {
			const modul = await import('leaflet');
			if (dibatalkan || !kontainer) return;
			L = (modul as unknown as { default?: ModulLeaflet }).default ?? modul;

			peta = L.map(kontainer, { zoomControl: true }).setView(PUSAT_BALI, 10);
			pasangLapisan('peta');
			grupTitik = L.layerGroup().addTo(peta);
			grupCheckIn = L.layerGroup().addTo(peta);

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
			lapisan = null;
			grupTitik = null;
			grupCheckIn = null;
		};
	});
</script>

<div class="space-y-3">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div>
			<h3 class="text-sm font-semibold text-slate-700">Peta Rekap Harian Strong Point</h3>
			<p class="text-[11px] text-slate-500">
				{jumlahPersonel} personel dijadwalkan · {titikUnik.length} strong point · {checkIn.length} check-in
				berkoordinat · {jumlahTc} tanpa check-out
			</p>
		</div>
		<div class="flex overflow-hidden rounded-lg border border-slate-200 bg-white shadow-sm">
			<button
				type="button"
				class="px-2.5 py-1 text-xs font-medium transition"
				class:bg-navy-800={mode === 'peta'}
				class:text-white={mode === 'peta'}
				class:text-slate-600={mode !== 'peta'}
				aria-pressed={mode === 'peta'}
				onclick={() => pasangLapisan('peta')}
			>
				Jalan
			</button>
			<button
				type="button"
				class="border-l border-slate-200 px-2.5 py-1 text-xs font-medium transition"
				class:bg-navy-800={mode === 'satelit'}
				class:text-white={mode === 'satelit'}
				class:text-slate-600={mode !== 'satelit'}
				aria-pressed={mode === 'satelit'}
				onclick={() => pasangLapisan('satelit')}
			>
				Satelit
			</button>
		</div>
	</div>

	<div class="relative isolate z-0">
		<div
			bind:this={kontainer}
			class="relative z-0 h-[420px] w-full overflow-hidden rounded-xl border border-slate-200 bg-slate-100 sm:h-[560px]"
		></div>
		{#if !siap}
			<div
				class="pointer-events-none absolute inset-0 flex items-center justify-center text-xs text-slate-500"
			>
				Memuat peta…
			</div>
		{/if}
		{#if siap && data.length === 0}
			<div
				class="pointer-events-none absolute inset-0 flex items-center justify-center bg-white/60 text-xs text-slate-500"
			>
				Tidak ada jadwal strong point pada tanggal ini.
			</div>
		{/if}
	</div>

	<div class="flex flex-wrap gap-x-4 gap-y-1.5 text-[11px] text-slate-600">
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-amber-500"></span> Strong point
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-emerald-600"></span> Check-in lengkap
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-amber-600"></span> Check-in tanpa check-out
			(TC)
		</span>
		<span class="flex items-center gap-1 text-slate-400">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-slate-300"></span> Belum check-in tidak
			ditampilkan di peta
		</span>
	</div>
</div>
