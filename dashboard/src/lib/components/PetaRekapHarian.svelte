<script lang="ts">
	import { onMount } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import type { LayerGroup, Map as LeafletMap, TileLayer } from 'leaflet';
	import type {
		AbsensiMatriksHari,
		AbsensiMatriksPersonel,
		StatusKehadiran,
		StrongPoint,
		TitikDinas
	} from '#lib/types';

	let {
		data,
		tanggal,
		titikDinas,
		strongPoint
	}: {
		data: AbsensiMatriksPersonel[];
		tanggal: string;
		titikDinas: TitikDinas[];
		strongPoint: StrongPoint[];
	} = $props();

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

	const WARNA: Record<StatusKehadiran, string> = {
		dinas_kantor: '#059669',
		dinas_luar: '#0284c7',
		ph: '#7c3aed',
		piket: '#d97706',
		operasi: '#dc2626',
		cuti: '#a16207',
		izin: '#b45309',
		sakit: '#be123c',
		tanpa_keterangan: '#475569',
		libur_nasional: '#94a3b8'
	};

	const LABEL_PENDEK: Record<StatusKehadiran, string> = {
		dinas_kantor: 'DK',
		dinas_luar: 'DL',
		ph: 'PH',
		piket: 'Piket',
		operasi: 'Ops',
		cuti: 'Cuti',
		izin: 'Izin',
		sakit: 'Sakit',
		tanpa_keterangan: 'TK',
		libur_nasional: 'Libur'
	};

	const LABEL_PANJANG: Record<StatusKehadiran, string> = {
		dinas_kantor: 'Dinas Kantor',
		dinas_luar: 'Dinas Luar',
		ph: 'Strong Point / PH',
		piket: 'Piket/Shift',
		operasi: 'Operasi/Pam',
		cuti: 'Cuti',
		izin: 'Izin',
		sakit: 'Sakit',
		tanpa_keterangan: 'Tanpa Keterangan',
		libur_nasional: 'Libur Nasional'
	};

	let kontainer = $state<HTMLDivElement | undefined>();
	let peta: LeafletMap | null = null;
	let lapisan: TileLayer | null = null;
	let grupPersonel: LayerGroup | null = null;
	let grupDinas: LayerGroup | null = null;
	let grupStrong: LayerGroup | null = null;
	let L: ModulLeaflet | null = null;
	let siap = $state(false);
	let mode = $state<ModePeta>('peta');

	const jumlahPersonel = $derived(
		data.filter((orang) => {
			const hari = orang.hari[tanggal];
			return hari?.masuk_latitude != null && hari.masuk_longitude != null;
		}).length
	);

	function statusEfektif(hari: AbsensiMatriksHari): StatusKehadiran | null {
		const status = hari.status ?? hari.status_masuk ?? hari.status_pulang;
		if (hari.ph_absen && (status === null || status === 'dinas_kantor')) return 'ph';
		return status;
	}

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

	function warnaStatus(status: StatusKehadiran | null) {
		return status ? WARNA[status] : '#64748b';
	}

	function labelSesi(nama: string | null) {
		if (nama === 'pagi') return 'Pagi';
		if (nama === 'sore') return 'Sore';
		return 'Sesi';
	}

	function popupPersonel(orang: AbsensiMatriksPersonel, hari: AbsensiMatriksHari) {
		const status = statusEfektif(hari);
		const bagian = [
			`<strong>${esc(orang.nama)}</strong>`,
			`${esc(orang.pangkat)} · ${esc(orang.nrp)}`,
			esc(orang.unit || orang.satker_nama),
			`Status: <b>${esc(status ? LABEL_PANJANG[status] : '-')}</b>`,
			`Check-in: <b>${esc(hari.masuk_jam ?? '-')}</b>`
		];
		if (hari.tc) {
			bagian.push('<span style="color:#be123c;font-weight:600">TC · belum check-out</span>');
		}
		for (const sesi of hari.sesi_strong) {
			bagian.push(
				`${labelSesi(sesi.sesi)} ${esc(sesi.jam_mulai)}–${esc(sesi.jam_selesai)}: ` +
					`${esc(sesi.masuk_jam ?? '-')}–${esc(sesi.pulang_jam ?? '-')}`
			);
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
		if (!L || !peta || !grupPersonel || !grupDinas || !grupStrong) return;
		grupPersonel.clearLayers();
		grupDinas.clearLayers();
		grupStrong.clearLayers();

		const batas: [number, number][] = [];

		for (const titik of titikDinas) {
			if (!titik.is_active) continue;
			const posisi: [number, number] = [titik.latitude, titik.longitude];
			batas.push(posisi);
			L.circle(posisi, {
				radius: Math.max(10, titik.radius_m),
				color: '#1d4ed8',
				weight: 1.5,
				dashArray: '5 4',
				fillColor: '#3b82f6',
				fillOpacity: 0.08
			}).addTo(grupDinas);
			L.circleMarker(posisi, {
				radius: 6,
				color: '#1d4ed8',
				weight: 2,
				fillColor: '#bfdbfe',
				fillOpacity: 1
			})
				.bindTooltip(`Titik dinas · ${esc(titik.nama)}`)
				.bindPopup(
					`<div style="min-width:180px;font-size:12px;line-height:1.5">
						<strong>${esc(titik.nama)}</strong><br>
						Titik dinas ${titik.jenis === 'luar' ? 'luar' : 'kantor'}<br>
						Radius: <b>${titik.radius_m} m</b>
					</div>`
				)
				.addTo(grupDinas);
		}

		for (const titik of strongPoint) {
			if (!titik.is_active) continue;
			const posisi: [number, number] = [titik.latitude, titik.longitude];
			batas.push(posisi);
			L.circle(posisi, {
				radius: Math.max(10, titik.radius_m),
				color: '#b45309',
				weight: 1.5,
				dashArray: '5 4',
				fillColor: '#f59e0b',
				fillOpacity: 0.08
			}).addTo(grupStrong);
			L.circleMarker(posisi, {
				radius: 6,
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
						${titik.alamat ? `${esc(titik.alamat)}<br>` : ''}
						Radius: <b>${titik.radius_m} m</b>
					</div>`
				)
				.addTo(grupStrong);
		}

		for (const orang of data) {
			const hari = orang.hari[tanggal];
			const lat = hari?.masuk_latitude;
			const lng = hari?.masuk_longitude;
			if (hari == null || lat == null || lng == null) continue;
			const status = statusEfektif(hari);
			const posisi: [number, number] = [lat, lng];
			batas.push(posisi);
			L.circleMarker(posisi, {
				radius: 7,
				color: '#ffffff',
				weight: 1.5,
				fillColor: warnaStatus(status),
				fillOpacity: 0.95
			})
				.bindTooltip(`${esc(orang.nama)} · ${status ? LABEL_PENDEK[status] : '-'}`)
				.bindPopup(popupPersonel(orang, hari))
				.addTo(grupPersonel);
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
		void tanggal;
		void titikDinas;
		void strongPoint;
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
			grupDinas = L.layerGroup().addTo(peta);
			grupStrong = L.layerGroup().addTo(peta);
			grupPersonel = L.layerGroup().addTo(peta);

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
			grupPersonel = null;
			grupDinas = null;
			grupStrong = null;
		};
	});
</script>

<div class="space-y-3">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div>
			<h3 class="text-sm font-semibold text-slate-700">Peta Rekap Harian</h3>
			<p class="text-[11px] text-slate-500">
				{jumlahPersonel} personel dengan koordinat check-in · {titikDinas.filter((t) => t.is_active)
					.length} titik dinas · {strongPoint.filter((s) => s.is_active).length} strong point
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
	</div>

	<div class="flex flex-wrap gap-x-4 gap-y-1.5 text-[11px] text-slate-600">
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-emerald-600"></span> DK
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-sky-600"></span> DL
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-violet-600"></span> PH
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-amber-600"></span> Piket
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-red-600"></span> Ops
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-yellow-700"></span> Cuti
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-orange-700"></span> Izin
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-rose-700"></span> Sakit
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-slate-600"></span> TK
		</span>
		<span class="flex items-center gap-1">
			<span class="inline-block h-2.5 w-2.5 rounded-full border-2 border-blue-700 bg-blue-200"></span>
			Titik dinas
		</span>
		<span class="flex items-center gap-1">
			<span
				class="inline-block h-2.5 w-2.5 rounded-full border-2 border-amber-700 bg-amber-200"
			></span>
			Strong point
		</span>
		<span class="text-slate-400">Lingkaran putus-putus = radius titik</span>
	</div>

	{#if jumlahPersonel === 0}
		<p class="rounded-lg border border-dashed border-slate-300 p-3 text-xs text-slate-500">
			Belum ada personel dengan koordinat check-in pada tanggal ini. Titik dinas dan strong point
			tetap ditampilkan.
		</p>
	{/if}
</div>
