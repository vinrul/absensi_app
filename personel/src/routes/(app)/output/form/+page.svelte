<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, ApiError, unggahLampiran } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import LampiranList from '#lib/components/LampiranList.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import WizardHeader from '#lib/components/WizardHeader.svelte';
	import {
		LABEL_JENIS_OUTPUT,
		tanggalHariIni,
		tanggalIndo,
		ukuranBerkas
	} from '#lib/format';
	import { session } from '#lib/stores/session.svelte';
	import type {
		Hasil,
		Lampiran,
		ListResponse,
		OutputDetail,
		OutputRequest,
		UbahOutputRequest
	} from '#lib/types';

	const JUDUL = ['Pilih Hasil Kerja', 'Data Output', 'Berkas Bukti'];
	const KETERANGAN = [
		'Output hanya dapat dibuat dari hasil kerja yang sudah diverifikasi atasan.',
		'Isi jenis, judul, uraian singkat, dan tanggal hasil.',
		'Unggah berkas bukti. Jenis Link/Sistem cukup menyertakan URL.'
	];

	const JENIS = Object.entries(LABEL_JENIS_OUTPUT);

	const idOutput = page.url.searchParams.get('id');
	const modeUbah = Boolean(idOutput);

	let siap = $state(!modeUbah);
	let memuat = $state(modeUbah);
	let memproses = $state(false);
	let langkah = $state(1);
	let galat = $state<string | null>(null);

	let daftarHasil = $state<Hasil[]>([]);
	let hasilId = $state('');
	let hasilLabel = $state('');
	let jenis = $state('laporan');
	let judul = $state('');
	let uraian = $state('');
	let tanggal = $state(tanggalHariIni());
	let url = $state('');
	let lampiranLama = $state<Lampiran[]>([]);
	let lampiranDihapus = $state<string[]>([]);
	let berkasBaru = $state<File[]>([]);

	const urlValid = $derived(url.trim().startsWith('http://') || url.trim().startsWith('https://'));
	const dataLengkap = $derived(
		judul.trim().length > 0 && /^\d{4}-\d{2}-\d{2}$/.test(tanggal) && (jenis !== 'link_sistem' || urlValid)
	);
	const bisaKirim = $derived(
		dataLengkap && (jenis === 'link_sistem' || lampiranLama.length - lampiranDihapus.length + berkasBaru.length > 0)
	);
	const bisaLanjut = $derived(langkah === 1 ? hasilId !== '' : dataLengkap);

	async function muat() {
		const user = session.user;
		if (!user) return;
		try {
			if (idOutput) {
				const detail = await api<OutputDetail>(`/output/${idOutput}`);
				hasilId = detail.hasil_kerja_id;
				hasilLabel = detail.rencana_kegiatan ?? '';
				jenis = detail.jenis_output;
				judul = detail.judul;
				uraian = detail.uraian ?? '';
				tanggal = detail.tanggal_hasil;
				url = detail.url_eksternal ?? '';
				lampiranLama = detail.lampiran;
			}
			const hasil = await api<ListResponse<Hasil>>(
				`/hasil-kerja?per_page=50&personel_id=${user.id}`
			);
			daftarHasil = hasil.data.filter((item) => item.status === 'terverifikasi');
			if (!hasilId && daftarHasil.length > 0) {
				hasilId = daftarHasil[0].id;
				hasilLabel = daftarHasil[0].rencana_kegiatan ?? '';
			}
			siap = true;
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Data output tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	function pilihHasil(item: Hasil) {
		hasilId = item.id;
		hasilLabel = item.rencana_kegiatan ?? '';
	}

	function pilihBerkas(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		berkasBaru = [...berkasBaru, ...Array.from(input.files ?? [])];
		input.value = '';
	}

	async function unggahBerkas(id: string) {
		for (const file of berkasBaru) await unggahLampiran('output', id, 'bukti', file);
		berkasBaru = [];
	}

	async function hapusLampiran() {
		for (const id of lampiranDihapus) await api(`/lampiran/${id}`, { method: 'DELETE' });
		lampiranDihapus = [];
	}

	async function simpan(ajukan: boolean) {
		if (memproses || !hasilId || !bisaKirim) return;
		memproses = true;
		galat = null;
		try {
			let id = idOutput;
			if (modeUbah && id) {
				const ubah: UbahOutputRequest = {
					jenis_output: jenis,
					judul: judul.trim(),
					uraian: uraian.trim() || null,
					tanggal_hasil: tanggal,
					url_eksternal: jenis === 'link_sistem' ? url.trim() : null
				};
				await api(`/output/${id}`, { method: 'PATCH', body: ubah });
				await hapusLampiran();
				await unggahBerkas(id);
			} else {
				const data: OutputRequest = {
					hasil_kerja_id: hasilId,
					jenis_output: jenis,
					judul: judul.trim(),
					uraian: uraian.trim() || null,
					tanggal_hasil: tanggal,
					url_eksternal: jenis === 'link_sistem' ? url.trim() : null,
					ajukan: false
				};
				const dibuat = await api<{ id: string }>('/output', { method: 'POST', body: data });
				id = dibuat.id;
				await unggahBerkas(id);
			}

			if (ajukan && id) await api(`/output/${id}/ajukan`, { method: 'POST' });
			await goto('/output', { replaceState: true });
		} catch (err) {
			galat =
				err instanceof ApiError
					? `${err.message}${ajukan ? ' Data tersimpan sebagai draft.' : ''}`
					: 'Gagal menyimpan output.';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>{modeUbah ? 'Ubah Output' : 'Output Baru'} — Absensi Kinerja</title></svelte:head>

<div class="space-y-4">
	<WizardHeader
		langkah={langkah}
		total={3}
		judul={JUDUL[langkah - 1]}
		deskripsi={KETERANGAN[langkah - 1]}
		onkembali={() => (langkah > 1 ? (langkah -= 1) : goto('/output'))}
		labelKembali={langkah === 1 ? 'Batal' : 'Kembali'}
	/>

	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat output…</div>
	{:else if siap}
		{#if langkah === 1}
			{#if daftarHasil.length === 0}
				<div class="kartu p-6 text-center">
					<p class="text-sm font-semibold text-ink-600">Belum ada hasil kerja terverifikasi.</p>
					<p class="mt-1 text-xs text-ink-400">
						Ajukan hasil kerja dan tunggu verifikasi atasan sebelum membuat output.
					</p>
					<a href="/hasil" class="btn btn-kedua mt-3 w-full">Buka Hasil Kerja</a>
				</div>
			{:else}
				<div class="space-y-2">
					{#each daftarHasil as item (item.id)}
						<button
							type="button"
							class="w-full rounded-2xl border p-4 text-left {hasilId === item.id
								? 'border-brand-500 bg-brand-50 ring-2 ring-brand-500/20'
								: 'border-ink-200 bg-white'}"
							onclick={() => pilihHasil(item)}
						>
							<p class="text-sm font-bold text-ink-800">{item.rencana_kegiatan ?? 'Hasil kerja'}</p>
							<p class="text-[11px] text-ink-500">{tanggalIndo(item.tanggal_realisasi)} · progres {item.progres_persen}%</p>
						</button>
					{/each}
				</div>
			{/if}
		{:else if langkah === 2}
			<div class="kartu space-y-3 p-4">
				<div class="rounded-xl bg-brand-50 px-3 py-2">
					<p class="text-[10px] font-bold uppercase tracking-wider text-brand-600">Hasil Kerja</p>
					<p class="text-xs font-semibold text-brand-900">{hasilLabel}</p>
				</div>
				<div>
					<label class="label" for="jenis">Jenis Output</label>
					<select id="jenis" class="input" bind:value={jenis}>
						{#each JENIS as [nilai, label] (nilai)}
							<option value={nilai}>{label}</option>
						{/each}
					</select>
				</div>
				<div>
					<label class="label" for="judul">Judul</label>
					<input id="judul" class="input" bind:value={judul} placeholder="Judul output" />
				</div>
				<div>
					<label class="label" for="uraian">Uraian</label>
					<textarea id="uraian" class="input min-h-24" bind:value={uraian} placeholder="Opsional"></textarea>
				</div>
				<div class="grid gap-3 sm:grid-cols-2">
					<div>
						<label class="label" for="tanggal-output">Tanggal Hasil</label>
						<input id="tanggal-output" type="date" class="input" bind:value={tanggal} />
					</div>
					{#if jenis === 'link_sistem'}
						<div>
							<label class="label" for="url">URL</label>
							<input id="url" class="input" bind:value={url} placeholder="https://…" />
						</div>
					{/if}
				</div>
				{#if jenis === 'link_sistem' && url && !urlValid}
					<Pesan jenis="galat" pesan="URL harus diawali http:// atau https://." />
				{/if}
			</div>
		{:else}
			<div class="space-y-3">
				<div class="kartu p-4">
					<p class="judul-bagian">Berkas Bukti</p>
					{#if jenis === 'link_sistem'}
						<Pesan jenis="info" pesan="Jenis Link/Sistem cukup menyertakan URL, berkas tidak wajib." />
					{/if}
					{#if lampiranLama.length > 0}
						<div class="mt-2">
							<LampiranList
								lampiran={lampiranLama.filter((item) => !lampiranDihapus.includes(item.id))}
								onhapus={(item) => (lampiranDihapus = [...lampiranDihapus, item.id])}
							/>
						</div>
					{/if}
					<label class="btn btn-kedua mt-3 w-full cursor-pointer">
						<Icon name="upload" size={16} /> Tambah Berkas
						<input type="file" multiple class="sr-only" onchange={pilihBerkas} />
					</label>
					{#if berkasBaru.length > 0}
						<ul class="mt-3 space-y-2">
							{#each berkasBaru as file, i (file.name + i)}
								<li class="flex items-center gap-2 rounded-xl border border-ink-100 px-3 py-2">
									<Icon name="file" size={15} class="shrink-0 text-brand-600" />
									<div class="min-w-0 flex-1">
										<p class="truncate text-xs font-semibold text-ink-700">{file.name}</p>
										<p class="text-[10px] text-ink-400">{ukuranBerkas(file.size)}</p>
									</div>
									<button class="p-1.5 text-rose-600" onclick={() => (berkasBaru = berkasBaru.filter((_, x) => x !== i))}>
										<Icon name="x" size={15} />
									</button>
								</li>
							{/each}
						</ul>
					{/if}
					{#if jenis !== 'link_sistem' && lampiranLama.length - lampiranDihapus.length + berkasBaru.length === 0}
						<Pesan jenis="info" pesan="Tambahkan minimal satu berkas bukti sebelum mengajukan." />
					{/if}
				</div>

				<button class="btn btn-utama w-full py-3" disabled={memproses || !bisaKirim} onclick={() => simpan(true)}>
					<Icon name="send" size={17} />
					{memproses ? 'Menyimpan…' : 'Ajukan ke Atasan'}
				</button>
				<button class="btn btn-kedua w-full" disabled={memproses || !dataLengkap} onclick={() => simpan(false)}>
					<Icon name="file" size={16} /> Simpan Draft
				</button>
			</div>
		{/if}

		{#if langkah < 3}
			<button class="btn btn-utama w-full py-3" disabled={!bisaLanjut} onclick={() => (langkah += 1)}>
				Lanjut <Icon name="right" size={16} />
			</button>
		{/if}
	{/if}
</div>
