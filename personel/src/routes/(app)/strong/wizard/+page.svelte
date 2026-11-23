<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import Peta from '#lib/components/Peta.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import WizardHeader from '#lib/components/WizardHeader.svelte';
	import { geserTanggal, tanggalHariIni } from '#lib/format';
	import { adaInternet, alamatLokasi, ambilLokasi, fotoKeBase64, type Posisi } from '#lib/lokasi';
	import { bacaAntreanPh, tambahAntreanPh } from '#lib/offline';
	import { session } from '#lib/stores/session.svelte';
	import { keteranganJadwal, statusStrongPoint } from '#lib/strong';
	import type {
		AbsensiPh,
		AbsensiPhAntrean,
		AbsensiPhRequest,
		JadwalPhSaya
	} from '#lib/types';

	let memuat = $state(true);
	let memproses = $state(false);
	let galat = $state<string | null>(null);

	let jadwal = $state<JadwalPhSaya[]>([]);
	let absensi = $state<AbsensiPh[]>([]);
	let antrean = $state<AbsensiPhAntrean[]>([]);

	let langkah = $state(1);
	let jenis = $state<'masuk' | 'pulang'>('masuk');
	let clientUuid = $state('');
	let capturedAt = $state('');
	let offline = $state(false);
	let gagalKirim = $state(false);
	let permintaanTersimpan = $state<AbsensiPhRequest | null>(null);

	let posisi = $state<Posisi | null>(null);
	let mengambilLokasi = $state(false);
	let alamat = $state<string | null>(null);
	let memuatAlamat = $state(false);
	let daring = $state(true);

	let fotoBase64 = $state<string | null>(null);
	let pratinjauFoto = $state<string | null>(null);
	let namaFoto = $state<string | null>(null);
	let lewatiFoto = $state(false);

	const hariIni = $derived(tanggalHariIni());
	const daftarHariIni = $derived(jadwal.filter((item) => item.tanggal === hariIni));
	const jadwalHariIni = $derived(
		daftarHariIni.find((item) => item.id === page.url.searchParams.get('jadwal')) ??
			daftarHariIni[0] ??
			null
	);
	const status = $derived(statusStrongPoint(jadwalHariIni?.id, absensi, antrean));
	const sudahCheckIn = $derived(status.sesi.some((item) => item.jenis === 'masuk'));
	const labelSesi = $derived(
		jadwalHariIni
			? `${jadwalHariIni.sesi === 'sore' ? 'Sore' : 'Pagi'} ${jadwalHariIni.jam_mulai}–${jadwalHariIni.jam_selesai}`
			: '-'
	);
	const totalLangkah = $derived(jenis === 'pulang' ? 2 : 3);
	const nomorLangkah = $derived(jenis === 'pulang' && langkah === 3 ? 2 : langkah);
	const judul = $derived(
		nomorLangkah === 1
			? 'Lokasi Strong Point'
			: nomorLangkah === 2
				? jenis === 'masuk'
					? 'Dokumentasi Strong Point'
					: 'Kirim Strong Point'
				: 'Kirim Strong Point'
	);
	const deskripsi = $derived(
		nomorLangkah === 1
			? 'Periksa lokasi saat ini. Titik strong point tidak ditampilkan pada peta.'
			: nomorLangkah === 2
				? jenis === 'masuk'
					? 'Ambil foto bukti pelaksanaan. Dokumentasi dapat dilewati.'
					: 'Periksa ringkasan lalu kirim check-out tanpa dokumentasi.'
				: 'Periksa ringkasan data sebelum dikirim ke server.'
	);

	async function muat() {
		const user = session.user;
		if (!user) return;
		memuat = true;
		antrean = bacaAntreanPh(user.id);
		const dari = geserTanggal(hariIni, -1);
		const sampai = geserTanggal(hariIni, 1);
		try {
			[jadwal, absensi] = await Promise.all([
				api<JadwalPhSaya[]>(`/ph/jadwal/saya?dari=${dari}&sampai=${sampai}`),
				api<AbsensiPh[]>(`/ph/absensi/saya?dari=${dari}&sampai=${sampai}`)
			]);
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Jadwal strong point tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
		daring = adaInternet();
		if (jadwalHariIni) await perbaruiLokasi();
	}

	async function perbaruiLokasi() {
		mengambilLokasi = true;
		galat = null;
		try {
			posisi = await ambilLokasi();
			memuatAlamat = true;
			alamat = await alamatLokasi(posisi.latitude, posisi.longitude);
		} catch (err) {
			galat = err instanceof Error ? err.message : 'Gagal mengambil lokasi.';
		} finally {
			memuatAlamat = false;
			mengambilLokasi = false;
			daring = adaInternet();
		}
	}

	function mulai(jenisDipilih: 'masuk' | 'pulang', modeOffline: boolean) {
		if (!jadwalHariIni) return;
		galat = null;
		gagalKirim = false;
		jenis = jenisDipilih;
		offline = modeOffline;
		clientUuid = crypto.randomUUID();
		capturedAt = new Date().toISOString();
		fotoBase64 = null;
		hapusFoto();
		lewatiFoto = false;
		langkah = jenisDipilih === 'masuk' ? 2 : 3;
	}

	async function pilihFoto(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		galat = null;
		try {
			if (pratinjauFoto) URL.revokeObjectURL(pratinjauFoto);
			pratinjauFoto = URL.createObjectURL(file);
			namaFoto = file.name;
			fotoBase64 = await fotoKeBase64(file);
		} catch {
			galat = 'Foto tidak dapat diolah. Coba foto lain.';
		}
	}

	function hapusFoto() {
		if (pratinjauFoto) URL.revokeObjectURL(pratinjauFoto);
		pratinjauFoto = null;
		namaFoto = null;
		fotoBase64 = null;
	}

	function bangunPermintaan(): AbsensiPhRequest | null {
		if (!jadwalHariIni) return null;
		return {
			client_uuid: clientUuid,
			jadwal_ph_id: jadwalHariIni.id,
			jenis,
			captured_at: capturedAt,
			latitude: posisi?.latitude ?? null,
			longitude: posisi?.longitude ?? null,
			akurasi_m: posisi?.akurasi ?? null,
			is_mock_suspect: false,
			is_offline: offline,
			foto_base64: jenis === 'masuk' ? fotoBase64 : null
		};
	}

	async function kirim(modeOffline = offline) {
		const user = session.user;
		const permintaan = bangunPermintaan();
		if (!user || !permintaan || memproses) return;
		if (!posisi) {
			galat = 'Lokasi belum tersedia. Perbarui lokasi terlebih dahulu.';
			return;
		}
		memproses = true;
		galat = null;
		gagalKirim = false;
		permintaanTersimpan = { ...permintaan, is_offline: modeOffline };

		if (modeOffline) {
			tambahAntreanPh(user.id, { ...permintaan, is_offline: true, dibuat_pada: Date.now() });
			await goto('/strong?pesan=Strong%20point%20disimpan%20di%20perangkat', {
				replaceState: true
			});
			return;
		}

		try {
			const hasil = await api<AbsensiPh>('/ph/absensi', {
				method: 'POST',
				body: { ...permintaan, is_offline: false }
			});
			const jarak = hasil.jarak_m !== null ? ` (${Math.round(hasil.jarak_m)} m dari strong point)` : '';
			await goto(
				`/strong?pesan=${encodeURIComponent(`Strong point ${jenis} tercatat pukul ${hasil.jam}${jarak}.`)}`,
				{ replaceState: true }
			);
		} catch (err) {
			if (err instanceof ApiError && err.status === 0) {
				gagalKirim = true;
				galat = 'Server tidak terjangkau. Anda dapat menyimpan strong point secara offline.';
			} else {
				galat = err instanceof ApiError ? err.message : 'Gagal mengirim strong point.';
			}
			memproses = false;
		}
	}

	function kembali() {
		galat = null;
		if (langkah === 1) goto('/strong');
		else if (langkah === 2) langkah = 1;
		else langkah = jenis === 'masuk' ? 2 : 1;
	}

	onMount(muat);

	onDestroy(() => {
		if (pratinjauFoto) URL.revokeObjectURL(pratinjauFoto);
	});
</script>

<svelte:head><title>Wizard Strong Point — Absensi Kinerja</title></svelte:head>

<div class="space-y-4">
	<WizardHeader
		langkah={nomorLangkah}
		total={totalLangkah}
		judul={judul}
		deskripsi={deskripsi}
		onkembali={kembali}
		labelKembali={langkah === 1 ? 'Batal' : 'Kembali'}
	/>

	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat jadwal strong point…</div>
	{:else if !jadwalHariIni}
		<Pesan
			jenis="info"
			pesan="Tidak ada jadwal strong point untuk Anda hari ini. Hubungi admin satker bila jadwal belum tersedia."
		/>
		<button class="btn btn-utama w-full" onclick={() => goto('/strong')}>Tutup</button>
	{:else if langkah === 1}
		<div class="kartu space-y-3 p-4">
			<Peta
				latitude={posisi?.latitude ?? null}
				longitude={posisi?.longitude ?? null}
				akurasi={posisi?.akurasi ?? null}
				warna="hijau"
				tinggi="h-60"
			/>

			{#if mengambilLokasi}
				<div class="flex items-center gap-2 text-xs text-ink-500">
					<span class="size-4 animate-spin rounded-full border-2 border-brand-200 border-t-brand-600"></span>
					Mengambil lokasi terkini…
				</div>
			{/if}

			<dl>
				<div class="baris-info">
					<dt>Koordinat saat ini</dt>
					<dd>
						{posisi
							? `${posisi.latitude.toFixed(6)}, ${posisi.longitude.toFixed(6)}`
							: '-'}
					</dd>
				</div>
				<div class="baris-info">
					<dt>Akurasi GPS</dt>
					<dd>{posisi?.akurasi ? `± ${Math.round(posisi.akurasi)} m` : '-'}</dd>
				</div>
				<div class="baris-info">
					<dt>Alamat</dt>
					<dd>
						{memuatAlamat ? 'Mencari alamat…' : (alamat ?? 'Alamat tidak tersedia')}
					</dd>
				</div>
			</dl>

			<button class="btn btn-kedua w-full" onclick={perbaruiLokasi} disabled={mengambilLokasi}>
				<Icon name="refresh" size={16} /> Perbarui Lokasi
			</button>
		</div>

		<div class="kartu p-4">
			<p class="judul-bagian">Jadwal Hari Ini</p>
			<p class="teks-kecil">
				{labelSesi} · {keteranganJadwal(jadwalHariIni.regu, jadwalHariIni.strong_point_nama)}
			</p>
			<dl class="mt-2">
				<div class="baris-info">
					<dt>Status</dt>
					<dd>
						{#if status.sesi.length === 0}
							Belum check-in
						{:else if status.terbuka}
							Check-in {status.masukTerakhir?.jam} · belum check-out
						{:else}
							Sudah check-out {status.pulangTerakhir?.jam}
						{/if}
					</dd>
				</div>
			</dl>
		</div>

		{#if !daring}
			<Pesan
				jenis="info"
				pesan="Tidak ada koneksi internet. Pilih check-in atau check-out; data disimpan di perangkat."
			/>
			<div class="grid grid-cols-2 gap-2">
				{#if !sudahCheckIn}
					<button class="btn btn-utama bg-emerald-600 py-4 hover:bg-emerald-700" onclick={() => mulai('masuk', true)}>
						CHECK-IN <span class="text-[10px] font-medium">(Offline)</span>
					</button>
				{/if}
				{#if status.terbuka}
					<button class="btn btn-bahaya py-4" onclick={() => mulai('pulang', true)}>
						CHECK-OUT <span class="text-[10px] font-medium">(Offline)</span>
					</button>
				{/if}
			</div>
		{:else if status.terbuka}
			<button class="btn btn-bahaya w-full py-4" onclick={() => mulai('pulang', false)}>
				CHECK-OUT <span class="text-[10px] font-medium">(Check Out)</span>
			</button>
		{:else if !sudahCheckIn}
			<button class="btn btn-utama w-full bg-emerald-600 py-4 hover:bg-emerald-700" onclick={() => mulai('masuk', false)}>
				CHECK-IN <span class="text-[10px] font-medium">(Check In)</span>
			</button>
		{:else}
			<Pesan jenis="sukses" pesan="Sesi strong point ini sudah selesai." />
		{/if}
	{:else if langkah === 2}
		<div class="kartu p-4">
			<p class="judul-bagian">Dokumentasi Check-in</p>
			<p class="teks-kecil">
				{fotoBase64 ? 'Foto siap dikirim bersama check-in.' : 'Ambil foto sebagai bukti pelaksanaan.'}
			</p>

			{#if pratinjauFoto}
				<img
					src={pratinjauFoto}
					alt="Dokumentasi strong point"
					class="mt-3 h-52 w-full rounded-xl object-cover ring-1 ring-ink-100"
				/>
				<p class="mt-1 truncate text-[10px] text-ink-400">{namaFoto}</p>
			{:else}
				<div class="mt-3 flex h-40 items-center justify-center rounded-xl bg-ink-100 text-xs text-ink-400">
					Belum ada foto
				</div>
			{/if}

			<div class="mt-3 flex gap-2">
				<label class="btn btn-kedua flex-1 cursor-pointer">
					<Icon name="camera" size={16} />
					{fotoBase64 ? 'Ganti Foto' : 'Ambil Foto'}
					<input
						type="file"
						accept="image/*"
						capture="environment"
						class="sr-only"
						onchange={pilihFoto}
					/>
				</label>
				{#if fotoBase64}
					<button class="btn btn-hantu text-rose-600" onclick={hapusFoto}>
						<Icon name="trash" size={16} />
					</button>
				{/if}
			</div>

			{#if !fotoBase64}
				<Pesan jenis="info" pesan="Foto dokumentasi disarankan sebagai bukti pelaksanaan strong point." class="mt-2" />
				<button class="btn btn-hantu mt-1 w-full text-xs" onclick={() => (lewatiFoto = true)}>
					Lewati dokumentasi
				</button>
			{/if}
		</div>

		<button
			class="btn btn-utama w-full py-3"
			disabled={!fotoBase64 && !lewatiFoto}
			onclick={() => (langkah = 3)}
		>
			Lanjut ke Kirim <Icon name="right" size={16} />
		</button>
	{:else}
		<div class="kartu p-4">
			<p class="judul-bagian">
				{jenis === 'masuk' ? 'Check-in dengan dokumentasi' : 'Check-out tanpa dokumentasi'}
			</p>
			<dl class="mt-2">
				<div class="baris-info">
					<dt>Jadwal</dt>
					<dd>{keteranganJadwal(jadwalHariIni.regu, jadwalHariIni.strong_point_nama)}</dd>
				</div>
				<div class="baris-info">
					<dt>Waktu</dt>
					<dd>
						{new Intl.DateTimeFormat('id-ID', {
							hour: '2-digit',
							minute: '2-digit',
							hour12: false,
							timeZone: 'Asia/Makassar'
						}).format(new Date(capturedAt))}
					</dd>
				</div>
				<div class="baris-info">
					<dt>Koordinat</dt>
					<dd>
						{posisi
							? `${posisi.latitude.toFixed(6)}, ${posisi.longitude.toFixed(6)}`
							: '-'}
					</dd>
				</div>
				<div class="baris-info">
					<dt>Akurasi GPS</dt>
					<dd>{posisi?.akurasi ? `± ${Math.round(posisi.akurasi)} m` : '-'}</dd>
				</div>
				<div class="baris-info">
					<dt>Alamat</dt>
					<dd>{alamat ?? 'Alamat tidak tersedia'}</dd>
				</div>
			</dl>

			{#if jenis === 'masuk' && pratinjauFoto}
				<img
					src={pratinjauFoto}
					alt="Dokumentasi strong point"
					class="mt-3 h-44 w-full rounded-xl object-cover ring-1 ring-ink-100"
				/>
			{/if}

			{#if offline}
				<Pesan
					jenis="info"
					pesan="Mode offline: data strong point disimpan di perangkat lalu dikirim saat sinyal tersedia."
					class="mt-3"
				/>
			{/if}
		</div>

		<button
			class="btn btn-utama w-full py-3"
			disabled={memproses || !posisi}
			onclick={() => kirim(offline)}
		>
			{#if memproses}
				<span class="size-4 animate-spin rounded-full border-2 border-white/40 border-t-white"></span>
				Mengirim…
			{:else}
				<Icon name="send" size={17} />
				{offline ? 'Kirim Strong Point (Offline)' : gagalKirim ? 'Coba Kirim Lagi' : 'Kirim Strong Point'}
			{/if}
		</button>

		{#if gagalKirim && !offline && permintaanTersimpan}
			<button class="btn btn-kedua w-full" onclick={() => kirim(true)}>
				<Icon name="wifiOff" size={16} /> Kirim Secara Offline
			</button>
		{/if}
	{/if}
</div>
