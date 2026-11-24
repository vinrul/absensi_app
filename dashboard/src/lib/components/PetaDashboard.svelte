<script lang="ts">
	import { onMount } from 'svelte';
	import 'leaflet/dist/leaflet.css';
	import type { LayerGroup, Map as LeafletMap, TileLayer } from 'leaflet';
	import type {
		AbsensiHarian,
		RekapPhHarian,
		StatusKehadiran,
		StrongPoint,
		TitikDinas
	} from '#lib/types';
	import { LABEL_STATUS_KEHADIRAN } from '#lib/types';

	let {
		absensi,
		titikDinas,
		strongPoint,
		phHarian
	}: {
		absensi: AbsensiHarian[];
		titikDinas: TitikDinas[];
		strongPoint: StrongPoint[];
		phHarian: RekapPhHarian[];
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

	let kontainer = $state<HTMLDivElement | undefined>();
	let peta: LeafletMap | null = null;
	let lapisan: TileLayer | null = null;
	let grupAbsensi: LayerGroup | null = null;
	let grupPh: LayerGroup | null = null;
	let grupDinas: LayerGroup | null = null;
	let grupStrong: LayerGroup | null = null;
	let L: ModulLeaflet | null = null;
	let siap = $state(false);
	let mode = $state<ModePeta>('peta');
	let tampil = $state({ absensi: true, ph: true, dinas: true, strong: true });

	const absensiBerkoordinat = $derived(
		absensi.filter(
			(row) =>
				!row.ph_hadir && row.masuk_latitude !== null && row.masuk_longitude !== null
		)
	);
	const phBerkoordinat = $derived(
		phHarian.filter(
			(row) => row.masuk_latitude !== null && row.masuk_longitude !== null
		)
	);
	const dinasAktif = $derived(titikDinas.filter((row) => row.is_active));
	const strongAktif = $derived(strongPoint.filter((row) => row.is_active));

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

	function statusAbsensi(row: AbsensiHarian): StatusKehadiran | null {
		return row.status_masuk ?? row.status_pulang;
	}

	function warnaStatus(status: StatusKehadiran | null) {
		return status ? WARNA[status] : '#64748b';
	}

	function labelSesi(sesi: string | null) {
		if (sesi === 'pagi') return 'Pagi';
		if (sesi === 'sore') return 'Sore';
		return 'Sesi';
	}

	function popupAbsensi(row: AbsensiHarian) {
		const status = statusAbsensi(row);
		const bagian = [
			`<strong>${esc(row.personel_nama)}</strong>`,
			`${esc(row.personel_pangkat)} · ${esc(row.personel_nrp)}`,
			esc(row.unit ?? '-'),
			`Status: <b>${esc(status ? LABEL_STATUS_KEHADIRAN[status] : '-')}</b>`,
			`Masuk: <b>${esc(row.masuk_jam ?? '-')}</b> · Pulang: <b>${esc(row.pulang_jam ?? '-')}</b>`
		];
		if (row.menit_telat > 0) bagian.push(`Telat: <b>${row.menit_telat} menit</b>`);
		if (row.menit_pulang_cepat > 0) {
			bagian.push(`Pulang cepat: <b>${row.menit_pulang_cepat} menit</b>`);
		}
		if (row.ph_hadir) bagian.push('Check-in strong point');
		if (row.perlu_verifikasi) {
			bagian.push('<span style="color:#b45309;font-weight:600">Menunggu verifikasi</span>');
		}
		return `<div style="min-width:210px;font-size:12px;line-height:1.55">${bagian.join('<br>')}</div>`;
	}

	function popupPh(row: RekapPhHarian) {
		const bagian = [
			`<strong>${esc(row.nama)}</strong>`,
			`${esc(row.pangkat)} · ${esc(row.nrp)}`,
			esc(row.unit ?? '-'),
			`Sesi: <b>${labelSesi(row.sesi)} ${esc(row.jam_mulai)}–${esc(row.jam_selesai)}</b>`,
			`Titik: ${esc(row.strong_point_nama)}`,
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

	function terapkanVisibilitas() {
		if (!peta) return;
		const atur = (grup: LayerGroup | null, aktif: boolean) => {
			if (!grup) return;
			if (aktif) {
				if (!peta!.hasLayer(grup)) grup.addTo(peta!);
			} else if (peta!.hasLayer(grup)) {
				peta!.removeLayer(grup);
			}
		};
		atur(grupAbsensi, tampil.absensi);
		atur(grupPh, tampil.ph);
		atur(grupDinas, tampil.dinas);
		atur(grupStrong, tampil.strong);
	}

	function gambarUlang() {
		if (!L || !peta || !grupAbsensi || !grupPh || !grupDinas || !grupStrong) return;
		grupAbsensi.clearLayers();
		grupPh.clearLayers();
		grupDinas.clearLayers();
		grupStrong.clearLayers();

		const batas: [number, number][] = [];

		for (const titik of dinasAktif) {
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

		for (const titik of strongAktif) {
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
				radius: 7,
				color: '#b45309',
				weight: 2,
				fillColor: '#fde68a',
				fillOpacity: 1
			})
				.bindTooltip(`Titik strong point · ${esc(titik.nama)}`)
				.bindPopup(
					`<div style="min-width:180px;font-size:12px;line-height:1.5">
						<strong>${esc(titik.nama)}</strong><br>
						Titik strong point<br>
						${titik.alamat ? `${esc(titik.alamat)}<br>` : ''}
						Radius: <b>${titik.radius_m} m</b>
					</div>`
				)
				.addTo(grupStrong);
		}

		for (const row of absensiBerkoordinat) {
			if (row.masuk_latitude === null || row.masuk_longitude === null) continue;
			const posisi: [number, number] = [row.masuk_latitude, row.masuk_longitude];
			batas.push(posisi);
			const status = statusAbsensi(row);
			L.circleMarker(posisi, {
				radius: 7,
				color: '#ffffff',
				weight: 1.5,
				fillColor: warnaStatus(status),
				fillOpacity: 0.95
			})
				.bindTooltip(
					`${esc(row.personel_nama)} · ${status ? LABEL_STATUS_KEHADIRAN[status] : '-'}`
				)
				.bindPopup(popupAbsensi(row))
				.addTo(grupAbsensi);
		}

		for (const row of phBerkoordinat) {
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
				.bindTooltip(`${esc(row.nama)} · ${lengkap ? 'Lengkap' : 'TC'}`)
				.bindPopup(popupPh(row))
				.addTo(grupPh);
		}

		if (batas.length > 0) {
			peta.fitBounds(batas, { padding: [30, 30], maxZoom: 16 });
		} else {
			peta.setView(PUSAT_BALI, 10);
		}

		terapkanVisibilitas();
	}

	$effect(() => {
		if (!siap) return;
		void absensi;
		void phHarian;
		void titikDinas;
		void strongPoint;
		gambarUlang();
	});

	$effect(() => {
		if (!siap) return;
		void tampil.absensi;
		void tampil.ph;
		void tampil.dinas;
		void tampil.strong;
		terapkanVisibilitas();
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
			grupAbsensi = L.layerGroup().addTo(peta);
			grupPh = L.layerGroup().addTo(peta);

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
			grupAbsensi = null;
			grupPh = null;
			grupDinas = null;
			grupStrong = null;
		};
	});
</script>

<div class="card space-y-3 p-4">
	<div class="flex flex-wrap items-center justify-between gap-2">
		<div>
			<h2 class="text-sm font-semibold text-slate-700">Peta Absensi &amp; Strong Point</h2>
			<p class="text-[11px] text-slate-500">
				{absensiBerkoordinat.length} absensi berkoordinat · {phBerkoordinat.length} check-in strong
				point · {dinasAktif.length} titik dinas · {strongAktif.length} titik strong point
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

	<div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-[11px] text-slate-600">
		<label class="flex cursor-pointer items-center gap-1.5">
			<input type="checkbox" class="size-3.5 accent-navy-800" bind:checked={tampil.absensi} />
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-emerald-600"></span> Absensi
		</label>
		<label class="flex cursor-pointer items-center gap-1.5">
			<input type="checkbox" class="size-3.5 accent-navy-800" bind:checked={tampil.ph} />
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-amber-500"></span> Check-in strong point
		</label>
		<label class="flex cursor-pointer items-center gap-1.5">
			<input type="checkbox" class="size-3.5 accent-navy-800" bind:checked={tampil.dinas} />
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-blue-600"></span> Titik dinas
		</label>
		<label class="flex cursor-pointer items-center gap-1.5">
			<input type="checkbox" class="size-3.5 accent-navy-800" bind:checked={tampil.strong} />
			<span class="inline-block h-2.5 w-2.5 rounded-full bg-amber-600"></span> Titik strong point
		</label>
		<span class="text-slate-400">
			Klik marker untuk detail; warna absensi mengikuti status kehadiran.
		</span>
	</div>
</div>
