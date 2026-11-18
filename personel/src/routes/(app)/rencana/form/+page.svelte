<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, ApiError, unggahLampiran } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import LampiranList from '#lib/components/LampiranList.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import Sheet from '#lib/components/Sheet.svelte';
	import WizardHeader from '#lib/components/WizardHeader.svelte';
	import { tanggalHariIni, tanggalIndo, ukuranBerkas } from '#lib/format';
	import { session } from '#lib/stores/session.svelte';
	import type {
		Lampiran,
		RencanaDetail,
		RencanaRequest,
		TahapanRequest,
		UbahRencanaRequest
	} from '#lib/types';

	interface KegiatanForm {
		program: string;
		kegiatan: string;
		uraian: string;
		tujuan: string;
		indikator: string;
		targetKuantitas: string;
		satuan: string;
		targetKualitas: string;
		mulai: string;
		selesai: string;
	}

	const JENIS = [
		{ nilai: 'harian', label: 'Harian', ket: 'Kegiatan rutin satu hari, misalnya apel atau patroli.' },
		{ nilai: 'mingguan', label: 'Mingguan', ket: 'Rencana untuk satu minggu kerja.' },
		{ nilai: 'bulanan', label: 'Bulanan', ket: 'Rencana kerja satu bulan berjalan.' },
		{ nilai: 'triwulan', label: 'Triwulanan / Semester', ket: 'Rencana untuk triwulan atau semester.' },
		{ nilai: 'penugasan_khusus', label: 'Penugasan Khusus', ket: 'Tugas khusus berdasarkan perintah atasan.' }
	];

	const DASAR = [
		{ nilai: 'sp', label: 'Surat Perintah' },
		{ nilai: 'tor', label: 'TOR' },
		{ nilai: 'perintah_lisan', label: 'Perintah Lisan' }
	];

	const JUDUL = [
		'Jenis Rencana Kerja',
		'Dasar / Referensi Tugas',
		'Rincian Rencana Kegiatan',
		'Lampiran Pendukung'
	];

	const KETERANGAN = [
		'Tentukan bentuk rencana yang akan disusun.',
		'Isi dasar penugasan yang menjadi acuan kegiatan.',
		'Tambahkan satu atau beberapa kegiatan beserta targetnya.',
		'Lampirkan dokumen pendukung, lalu simpan atau ajukan.'
	];

	const idRencana = page.url.searchParams.get('id');
	const modeUbah = Boolean(idRencana);

	let siap = $state(!modeUbah);
	let memuat = $state(modeUbah);
	let memproses = $state(false);
	let galat = $state<string | null>(null);
	let pesan = $state<string | null>(null);

	let langkah = $state(1);
	let jenis = $state('bulanan');
	let dasarJenis = $state('sp');
	let dasarNomor = $state('');
	let dasarTanggal = $state(tanggalHariIni());
	let dasarPerihal = $state('');
	let daftarKegiatan = $state<KegiatanForm[]>([]);
	let lampiranLama = $state<Lampiran[]>([]);
	let lampiranDihapus = $state<string[]>([]);
	let lampiranBaru = $state<File[]>([]);

	let bukaEditor = $state(false);
	let indeksSunting = $state<number | null>(null);
	let formKegiatan = $state<KegiatanForm>(kosong());
	let galatKegiatan = $state<string | null>(null);

	function kosong(): KegiatanForm {
		const hari = tanggalHariIni();
		return {
			program: '',
			kegiatan: '',
			uraian: '',
			tujuan: '',
			indikator: '',
			targetKuantitas: '',
			satuan: '',
			targetKualitas: '',
			mulai: hari,
			selesai: hari
		};
	}

	function dariTahapan(tahap: RencanaDetail['tahapan'][number], induk: RencanaDetail): KegiatanForm {
		return {
			program: tahap.program ?? induk.program,
			kegiatan: tahap.kegiatan ?? induk.kegiatan,
			uraian: tahap.uraian,
			tujuan: tahap.tujuan ?? induk.tujuan ?? '',
			indikator: tahap.indikator_kinerja ?? induk.indikator_kinerja ?? '',
			targetKuantitas:
				tahap.target_kuantitas !== null ? String(tahap.target_kuantitas) : '',
			satuan: tahap.satuan ?? '',
			targetKualitas: tahap.target_kualitas ?? '',
			mulai: tahap.tanggal_mulai ?? induk.tanggal_mulai,
			selesai: tahap.tanggal_selesai ?? induk.tanggal_selesai
		};
	}

	const rentangValid = $derived(
		daftarKegiatan.every((item) => !item.mulai || !item.selesai || item.mulai <= item.selesai)
	);
	const dasarValid = $derived(dasarJenis === 'perintah_lisan' || dasarNomor.trim().length > 0);
	const bisaLanjut = $derived(
		langkah === 1
			? true
			: langkah === 2
				? dasarValid
				: daftarKegiatan.length > 0 && rentangValid
	);

	async function muat() {
		if (!idRencana) return;
		try {
			const detail = await api<RencanaDetail>(`/rencana-kerja/${idRencana}`);
			jenis = detail.jenis;
			dasarJenis = detail.dasar_jenis ?? 'sp';
			dasarNomor = detail.dasar_nomor ?? '';
			dasarTanggal = detail.dasar_tanggal ?? tanggalHariIni();
			dasarPerihal = detail.dasar_perihal ?? '';
			lampiranLama = detail.lampiran;
			daftarKegiatan = detail.tahapan
				.slice()
				.sort((a, b) => a.urutan - b.urutan)
				.map((tahap) => dariTahapan(tahap, detail));
			if (daftarKegiatan.length === 0) daftarKegiatan = [kosong()];
			siap = true;
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Rencana tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	function bukaTambah() {
		indeksSunting = null;
		formKegiatan = kosong();
		galatKegiatan = null;
		bukaEditor = true;
	}

	function bukaUbah(index: number) {
		indeksSunting = index;
		formKegiatan = { ...daftarKegiatan[index] };
		galatKegiatan = null;
		bukaEditor = true;
	}

	function simpanKegiatan() {
		galat = null;
		if (!formKegiatan.program.trim() || !formKegiatan.kegiatan.trim() || !formKegiatan.uraian.trim()) {
			galatKegiatan = 'Program, kegiatan, dan uraian wajib diisi.';
			return;
		}
		if (!formKegiatan.mulai || !formKegiatan.selesai) {
			galatKegiatan = 'Tanggal mulai dan selesai wajib diisi.';
			return;
		}
		if (formKegiatan.mulai > formKegiatan.selesai) {
			galatKegiatan = 'Tanggal mulai tidak boleh melewati tanggal selesai.';
			return;
		}
		if (indeksSunting === null) daftarKegiatan = [...daftarKegiatan, { ...formKegiatan }];
		else {
			daftarKegiatan = daftarKegiatan.map((item, i) =>
				i === indeksSunting ? { ...formKegiatan } : item
			);
		}
		bukaEditor = false;
	}

	function hapusKegiatan(index: number) {
		daftarKegiatan = daftarKegiatan.filter((_, i) => i !== index);
	}

	function pilihBerkas(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const daftar = Array.from(input.files ?? []);
		galat = null;
		for (const file of daftar) {
			if (file.size > 25 * 1024 * 1024) {
				galat = `Berkas ${file.name} melebihi 25 MB.`;
				continue;
			}
			lampiranBaru = [...lampiranBaru, file];
		}
		input.value = '';
	}

	async function unggahSemua(id: string) {
		for (const file of lampiranBaru) {
			await unggahLampiran('rencana', id, 'lampiran', file);
		}
		lampiranBaru = [];
	}

	async function hapusLampiranLama() {
		for (const id of lampiranDihapus) {
			await api(`/lampiran/${id}`, { method: 'DELETE' });
		}
		lampiranDihapus = [];
	}

	function payload(ajukan: boolean): RencanaRequest {
		const pertama = daftarKegiatan[0];
		const mulai = daftarKegiatan.map((item) => item.mulai).sort()[0] ?? tanggalHariIni();
		const selesai =
			daftarKegiatan
				.map((item) => item.selesai)
				.sort()
				.at(-1) ?? tanggalHariIni();
		const jumlahLain = daftarKegiatan.length - 1;
		const kegiatanInduk =
			jumlahLain === 0 ? pertama.kegiatan : `${pertama.kegiatan} (+${jumlahLain} kegiatan)`;

		const tahapan: TahapanRequest[] = daftarKegiatan.map((item) => ({
			uraian: item.uraian.trim(),
			target_kuantitas: item.targetKuantitas ? Number(item.targetKuantitas) : null,
			satuan: item.satuan.trim() || null,
			bobot: null,
			program: item.program.trim() || null,
			kegiatan: item.kegiatan.trim() || null,
			tujuan: item.tujuan.trim() || null,
			indikator_kinerja: item.indikator.trim() || null,
			target_kualitas: item.targetKualitas.trim() || null,
			tanggal_mulai: item.mulai || null,
			tanggal_selesai: item.selesai || null
		}));

		return {
			periode_id: null,
			program: pertama.program.trim(),
			kegiatan: kegiatanInduk,
			uraian: pertama.uraian.trim(),
			tujuan: pertama.tujuan.trim() || null,
			indikator_kinerja: pertama.indikator.trim() || null,
			jenis,
			dasar_jenis: dasarJenis,
			dasar_nomor: dasarNomor.trim() || null,
			dasar_tanggal: dasarTanggal || null,
			dasar_perihal: dasarPerihal.trim() || null,
			target_kuantitas: pertama.targetKuantitas ? Number(pertama.targetKuantitas) : null,
			satuan: pertama.satuan.trim() || null,
			target_kualitas: pertama.targetKualitas.trim() || null,
			target_waktu: selesai,
			tanggal_mulai: mulai,
			tanggal_selesai: selesai,
			tahapan,
			ajukan
		};
	}

	async function simpan(ajukan: boolean) {
		if (memproses) return;
		if (daftarKegiatan.length === 0) {
			galat = 'Tambahkan minimal satu rincian kegiatan.';
			return;
		}
		if (ajukan && !session.user?.atasan_id) {
			galat = 'Atasan langsung belum ditetapkan untuk akun Anda.';
			return;
		}
		memproses = true;
		galat = null;
		pesan = null;

		const data = payload(ajukan);
		try {
			let id = idRencana;
			if (modeUbah && id) {
				const ubah: UbahRencanaRequest = data;
				await api(`/rencana-kerja/${id}`, { method: 'PATCH', body: ubah });
				await hapusLampiranLama();
				await unggahSemua(id);
			} else {
				const dibuat = await api<{ id: string }>('/rencana-kerja', {
					method: 'POST',
					body: { ...data, ajukan: false }
				});
				id = dibuat.id;
				await unggahSemua(id);
			}

			if (ajukan && id) await api(`/rencana-kerja/${id}/ajukan`, { method: 'POST' });
			await goto('/rencana', { replaceState: true });
		} catch (err) {
			galat =
				err instanceof ApiError
					? `${err.message}${ajukan ? ' Rencana tersimpan sebagai draft.' : ''}`
					: 'Gagal menyimpan rencana kerja.';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>{modeUbah ? 'Ubah Rencana' : 'Rencana Baru'} — Absensi Kinerja</title></svelte:head>

<div class="space-y-4">
	<WizardHeader
		langkah={langkah}
		total={4}
		judul={JUDUL[langkah - 1]}
		deskripsi={KETERANGAN[langkah - 1]}
		onkembali={() => (langkah > 1 ? (langkah -= 1) : goto('/rencana'))}
		labelKembali={langkah === 1 ? 'Batal' : 'Kembali'}
	/>

	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat rencana kerja…</div>
	{:else if siap}
		{#if langkah === 1}
			<div class="space-y-2">
				{#each JENIS as opsi (opsi.nilai)}
					<button
						type="button"
						class="flex w-full items-start gap-3 rounded-2xl border p-4 text-left transition-colors {jenis ===
						opsi.nilai
							? 'border-brand-500 bg-brand-50 ring-2 ring-brand-500/20'
							: 'border-ink-200 bg-white'}"
						onclick={() => (jenis = opsi.nilai)}
					>
						<span
							class="mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full border-2 {jenis ===
							opsi.nilai
								? 'border-brand-600 bg-brand-600'
								: 'border-ink-300'}"
						>
							{#if jenis === opsi.nilai}
								<span class="size-1.5 rounded-full bg-white"></span>
							{/if}
						</span>
						<span>
							<span class="block text-sm font-bold text-ink-800">{opsi.label}</span>
							<span class="block text-[11px] text-ink-500">{opsi.ket}</span>
						</span>
					</button>
				{/each}
			</div>
		{:else if langkah === 2}
			<div class="kartu space-y-4 p-4">
				<div>
					<span class="label">Jenis Dasar</span>
					<div class="flex flex-wrap gap-2">
						{#each DASAR as opsi (opsi.nilai)}
							<button
								type="button"
								class="chip border px-3 py-1.5 {dasarJenis === opsi.nilai
									? 'border-brand-600 bg-brand-600 text-white'
									: 'border-ink-200 bg-white text-ink-500'}"
								onclick={() => (dasarJenis = opsi.nilai)}
							>
								{opsi.label}
							</button>
						{/each}
					</div>
				</div>

				{#if dasarJenis !== 'perintah_lisan'}
					<div>
						<label class="label" for="dasar-nomor">Nomor Surat / Dokumen</label>
						<input
							id="dasar-nomor"
							class="input"
							placeholder="Contoh: Sprin/123/XI/2026"
							bind:value={dasarNomor}
						/>
					</div>
				{/if}

				<div class="grid gap-3 sm:grid-cols-2">
					<div>
						<label class="label" for="dasar-tanggal">Tanggal Dasar</label>
						<input id="dasar-tanggal" type="date" class="input" bind:value={dasarTanggal} />
					</div>
					<div>
						<label class="label" for="dasar-perihal">Perihal</label>
						<input id="dasar-perihal" class="input" placeholder="Opsional" bind:value={dasarPerihal} />
					</div>
				</div>

				{#if !dasarValid}
					<Pesan jenis="info" pesan="Nomor surat wajib diisi kecuali dasar berupa perintah lisan." />
				{/if}
			</div>
		{:else if langkah === 3}
			<div class="space-y-3">
				{#if daftarKegiatan.length === 0}
					<div class="kartu p-6 text-center">
						<p class="text-sm font-semibold text-ink-600">Belum ada rincian kegiatan.</p>
						<p class="mt-1 text-xs text-ink-400">Tambahkan minimal satu kegiatan beserta targetnya.</p>
					</div>
				{:else}
					{#each daftarKegiatan as item, i (i)}
						<article class="kartu p-4">
							<div class="flex items-start justify-between gap-2">
								<div class="min-w-0">
									<p class="text-sm font-bold text-ink-900">{item.kegiatan}</p>
									<p class="text-[11px] text-ink-500">{item.program}</p>
								</div>
								<span class="chip bg-brand-50 text-brand-700">#{i + 1}</span>
							</div>
							<p class="mt-2 text-xs leading-relaxed text-ink-600">{item.uraian}</p>
							<p class="mt-2 text-[11px] text-ink-400">
								{tanggalIndo(item.mulai)} – {tanggalIndo(item.selesai)}
								{#if item.targetKuantitas}
									· Target {item.targetKuantitas} {item.satuan}
								{/if}
							</p>
							<div class="mt-3 flex gap-2">
								<button class="btn btn-kedua flex-1 py-1.5 text-xs" onclick={() => bukaUbah(i)}>
									<Icon name="edit" size={14} /> Ubah
								</button>
								<button
									class="btn btn-hantu py-1.5 text-xs text-rose-600"
									onclick={() => hapusKegiatan(i)}
								>
									<Icon name="trash" size={14} /> Hapus
								</button>
							</div>
						</article>
					{/each}
				{/if}

				<button class="btn btn-kedua w-full py-3" onclick={bukaTambah}>
					<Icon name="plus" size={16} /> Tambah Kegiatan
				</button>

				{#if !rentangValid}
					<Pesan jenis="galat" pesan="Tanggal mulai tidak boleh melewati tanggal selesai." />
				{/if}
			</div>
		{:else}
			<div class="space-y-3">
				<div class="kartu p-4">
					<p class="judul-bagian">Lampiran Tersimpan</p>
					{#if lampiranLama.length === 0}
						<p class="mt-1 text-xs text-ink-400">Belum ada lampiran tersimpan.</p>
					{:else}
						<div class="mt-2">
							<LampiranList
								lampiran={lampiranLama.filter((item) => !lampiranDihapus.includes(item.id))}
								judul="Berkas"
								onhapus={(item) => (lampiranDihapus = [...lampiranDihapus, item.id])}
							/>
						</div>
						<p class="mt-2 text-[10px] text-ink-400">
							Berkas yang dihapus akan diproses saat Anda menyimpan.
						</p>
					{/if}
				</div>

				<div class="kartu p-4">
					<p class="judul-bagian">Lampiran Baru</p>
					<label class="btn btn-kedua mt-2 w-full cursor-pointer">
						<Icon name="upload" size={16} /> Pilih Berkas
						<input type="file" multiple class="sr-only" onchange={pilihBerkas} />
					</label>
					{#if lampiranBaru.length > 0}
						<ul class="mt-3 space-y-2">
							{#each lampiranBaru as file, i (file.name + i)}
								<li class="flex items-center gap-2 rounded-xl border border-ink-100 px-3 py-2">
									<Icon name="file" size={15} class="shrink-0 text-brand-600" />
									<div class="min-w-0 flex-1">
										<p class="truncate text-xs font-semibold text-ink-700">{file.name}</p>
										<p class="text-[10px] text-ink-400">{ukuranBerkas(file.size)}</p>
									</div>
									<button
										type="button"
										class="p-1.5 text-rose-600"
										aria-label="Buang berkas"
										onclick={() => (lampiranBaru = lampiranBaru.filter((_, x) => x !== i))}
									>
										<Icon name="x" size={15} />
									</button>
								</li>
							{/each}
						</ul>
					{/if}
				</div>

				<button class="btn btn-utama w-full py-3" disabled={memproses} onclick={() => simpan(true)}>
					<Icon name="send" size={17} />
					{memproses ? 'Menyimpan…' : 'Ajukan ke Atasan'}
				</button>
				<button class="btn btn-kedua w-full" disabled={memproses} onclick={() => simpan(false)}>
					<Icon name="file" size={16} /> Simpan Draft
				</button>
			</div>
		{/if}

		{#if langkah < 4}
			<button class="btn btn-utama w-full py-3" disabled={!bisaLanjut} onclick={() => (langkah += 1)}>
				Lanjut
				<Icon name="right" size={16} />
			</button>
		{/if}
	{/if}
</div>

{#if bukaEditor}
	<Sheet
		judul={indeksSunting === null ? 'Tambah Kegiatan' : 'Ubah Kegiatan'}
		onTutup={() => (bukaEditor = false)}
	>
		<div class="space-y-3">
			{#if galatKegiatan}<Pesan jenis="galat" pesan={galatKegiatan} />{/if}
			<div>
				<label class="label" for="k-program">Program</label>
				<input id="k-program" class="input" bind:value={formKegiatan.program} placeholder="Nama program" />
			</div>
			<div>
				<label class="label" for="k-kegiatan">Kegiatan</label>
				<input id="k-kegiatan" class="input" bind:value={formKegiatan.kegiatan} placeholder="Nama kegiatan" />
			</div>
			<div>
				<label class="label" for="k-uraian">Uraian</label>
				<textarea id="k-uraian" class="input min-h-20" bind:value={formKegiatan.uraian}></textarea>
			</div>
			<div>
				<label class="label" for="k-tujuan">Tujuan</label>
				<input id="k-tujuan" class="input" bind:value={formKegiatan.tujuan} placeholder="Opsional" />
			</div>
			<div>
				<label class="label" for="k-indikator">Indikator Kinerja</label>
				<input id="k-indikator" class="input" bind:value={formKegiatan.indikator} placeholder="Opsional" />
			</div>
			<div class="grid grid-cols-2 gap-3">
				<div>
					<label class="label" for="k-target">Target Kuantitas</label>
					<input
						id="k-target"
						class="input"
						inputmode="decimal"
						bind:value={formKegiatan.targetKuantitas}
						placeholder="0"
					/>
				</div>
				<div>
					<label class="label" for="k-satuan">Satuan</label>
					<input id="k-satuan" class="input" bind:value={formKegiatan.satuan} placeholder="kegiatan" />
				</div>
			</div>
			<div>
				<label class="label" for="k-kualitas">Target Kualitas</label>
				<input id="k-kualitas" class="input" bind:value={formKegiatan.targetKualitas} placeholder="Opsional" />
			</div>
			<div class="grid grid-cols-2 gap-3">
				<div>
					<label class="label" for="k-mulai">Tanggal Mulai</label>
					<input id="k-mulai" type="date" class="input" bind:value={formKegiatan.mulai} />
				</div>
				<div>
					<label class="label" for="k-selesai">Tanggal Selesai</label>
					<input id="k-selesai" type="date" class="input" bind:value={formKegiatan.selesai} />
				</div>
			</div>
		</div>

		{#snippet footer()}
			<div class="flex gap-2">
				<button class="btn btn-kedua flex-1" onclick={() => (bukaEditor = false)}>Batal</button>
				<button class="btn btn-utama flex-1" onclick={simpanKegiatan}>Simpan</button>
			</div>
		{/snippet}
	</Sheet>
{/if}
