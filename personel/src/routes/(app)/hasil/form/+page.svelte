<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, ApiError, unggahLampiran } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import LampiranList from '#lib/components/LampiranList.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import WizardHeader from '#lib/components/WizardHeader.svelte';
	import { tanggalHariIni, tanggalIndo, ukuranBerkas } from '#lib/format';
	import { session } from '#lib/stores/session.svelte';
	import type { HasilDetail, HasilRequest, Lampiran, ListResponse, Rencana, UbahHasilRequest } from '#lib/types';

	const JUDUL = ['Pilih Rencana Kerja', 'Input Hasil Kerja', 'Bukti Pendukung'];
	const KETERANGAN = [
		'Hasil kerja hanya dapat dilaporkan dari rencana yang sudah disetujui.',
		'Isi uraian pelaksanaan, progres, dan realisasi capaian.',
		'Lampirkan dokumen, foto, atau laporan pendukung.'
	];

	const idHasil = page.url.searchParams.get('id');
	const modeUbah = Boolean(idHasil);

	let siap = $state(!modeUbah);
	let memuat = $state(modeUbah);
	let memproses = $state(false);
	let langkah = $state(1);
	let galat = $state<string | null>(null);

	let daftarRencana = $state<Rencana[]>([]);
	let rencanaId = $state('');
	let rencanaLabel = $state('');
	let uraian = $state('');
	let progres = $state(100);
	let realisasi = $state('');
	let kendala = $state('');
	let tanggal = $state(tanggalHariIni());
	let lampiranLama = $state<Lampiran[]>([]);
	let lampiranDihapus = $state<string[]>([]);
	let buktiBaru = $state<File[]>([]);

	const bisaLanjut = $derived(
		langkah === 1
			? rencanaId !== ''
			: uraian.trim().length > 0 && progres >= 0 && progres <= 100 && /^\d{4}-\d{2}-\d{2}$/.test(tanggal)
	);

	async function muat() {
		const user = session.user;
		if (!user) return;
		try {
			if (idHasil) {
				const detail = await api<HasilDetail>(`/hasil-kerja/${idHasil}`);
				rencanaId = detail.rencana_id;
				rencanaLabel = detail.rencana_kegiatan ?? '';
				uraian = detail.uraian_pelaksanaan;
				progres = detail.progres_persen;
				realisasi = detail.jumlah_realisasi !== null ? String(detail.jumlah_realisasi) : '';
				kendala = detail.kendala ?? '';
				tanggal = detail.tanggal_realisasi;
				lampiranLama = detail.lampiran;
			}
			const hasil = await api<ListResponse<Rencana>>(
				`/rencana-kerja?per_page=50&personel_id=${user.id}`
			);
			daftarRencana = hasil.data.filter((item) => item.status === 'disetujui');
			if (!rencanaId && daftarRencana.length > 0) {
				rencanaId = daftarRencana[0].id;
				rencanaLabel = daftarRencana[0].kegiatan;
			}
			siap = true;
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Data hasil kerja tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	function pilihRencana(item: Rencana) {
		rencanaId = item.id;
		rencanaLabel = item.kegiatan;
	}

	function pilihBerkas(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		buktiBaru = [...buktiBaru, ...Array.from(input.files ?? [])];
		input.value = '';
	}

	async function unggahBukti(id: string) {
		for (const file of buktiBaru) await unggahLampiran('hasil', id, 'bukti', file);
		buktiBaru = [];
	}

	async function hapusLampiran() {
		for (const id of lampiranDihapus) await api(`/lampiran/${id}`, { method: 'DELETE' });
		lampiranDihapus = [];
	}

	async function simpan(ajukan: boolean) {
		if (memproses || !rencanaId) return;
		memproses = true;
		galat = null;
		try {
			let id = idHasil;
			if (modeUbah && id) {
				const ubah: UbahHasilRequest = {
					uraian_pelaksanaan: uraian.trim(),
					progres_persen: progres,
					jumlah_realisasi: realisasi ? Number(realisasi) : null,
					kendala: kendala.trim() || null,
					tanggal_realisasi: tanggal
				};
				await api(`/hasil-kerja/${id}`, { method: 'PATCH', body: ubah });
				await hapusLampiran();
				await unggahBukti(id);
			} else {
				const data: HasilRequest = {
					rencana_id: rencanaId,
					uraian_pelaksanaan: uraian.trim(),
					progres_persen: progres,
					jumlah_realisasi: realisasi ? Number(realisasi) : null,
					kendala: kendala.trim() || null,
					tanggal_realisasi: tanggal,
					ajukan: false
				};
				const dibuat = await api<{ id: string }>('/hasil-kerja', { method: 'POST', body: data });
				id = dibuat.id;
				await unggahBukti(id);
			}

			if (ajukan && id) await api(`/hasil-kerja/${id}/ajukan`, { method: 'POST' });
			await goto('/hasil', { replaceState: true });
		} catch (err) {
			galat =
				err instanceof ApiError
					? `${err.message}${ajukan ? ' Data tersimpan sebagai draft.' : ''}`
					: 'Gagal menyimpan hasil kerja.';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>{modeUbah ? 'Ubah Hasil' : 'Hasil Baru'} — Absensi Kinerja</title></svelte:head>

<div class="space-y-4">
	<WizardHeader
		langkah={langkah}
		total={3}
		judul={JUDUL[langkah - 1]}
		deskripsi={KETERANGAN[langkah - 1]}
		onkembali={() => (langkah > 1 ? (langkah -= 1) : goto('/hasil'))}
		labelKembali={langkah === 1 ? 'Batal' : 'Kembali'}
	/>

	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat hasil kerja…</div>
	{:else if siap}
		{#if langkah === 1}
			{#if daftarRencana.length === 0}
				<div class="kartu p-6 text-center">
					<p class="text-sm font-semibold text-ink-600">Belum ada rencana yang disetujui.</p>
					<p class="mt-1 text-xs text-ink-400">
						Ajukan rencana kerja dan tunggu persetujuan atasan sebelum melaporkan hasil.
					</p>
					<a href="/rencana" class="btn btn-kedua mt-3 w-full">Buka Rencana Kerja</a>
				</div>
			{:else}
				<div class="space-y-2">
					{#each daftarRencana as item (item.id)}
						<button
							type="button"
							class="w-full rounded-2xl border p-4 text-left {rencanaId === item.id
								? 'border-brand-500 bg-brand-50 ring-2 ring-brand-500/20'
								: 'border-ink-200 bg-white'}"
							onclick={() => pilihRencana(item)}
						>
							<p class="text-sm font-bold text-ink-800">{item.kegiatan}</p>
							<p class="text-[11px] text-ink-500">{item.program}</p>
							<p class="mt-1 text-[10px] text-ink-400">
								{tanggalIndo(item.tanggal_mulai)} – {tanggalIndo(item.tanggal_selesai)}
							</p>
						</button>
					{/each}
				</div>
			{/if}
		{:else if langkah === 2}
			<div class="kartu space-y-3 p-4">
				<div class="rounded-xl bg-brand-50 px-3 py-2">
					<p class="text-[10px] font-bold uppercase tracking-wider text-brand-600">Rencana</p>
					<p class="text-xs font-semibold text-brand-900">{rencanaLabel}</p>
				</div>
				<div>
					<label class="label" for="uraian">Uraian Pelaksanaan</label>
					<textarea
						id="uraian"
						class="input min-h-28"
						bind:value={uraian}
						placeholder="Jelaskan pelaksanaan kegiatan…"
					></textarea>
				</div>
				<div>
					<div class="flex justify-between text-[11px] text-ink-500">
						<span class="label !mb-0">Progres</span>
						<span class="font-bold text-brand-700">{progres}%</span>
					</div>
					<input type="range" min="0" max="100" step="1" class="mt-2 w-full accent-brand-600" bind:value={progres} />
				</div>
				<div class="grid grid-cols-2 gap-3">
					<div>
						<label class="label" for="realisasi">Jumlah Realisasi</label>
						<input id="realisasi" class="input" inputmode="decimal" bind:value={realisasi} placeholder="Opsional" />
					</div>
					<div>
						<label class="label" for="tanggal">Tanggal Realisasi</label>
						<input id="tanggal" type="date" class="input" bind:value={tanggal} />
					</div>
				</div>
				<div>
					<label class="label" for="kendala">Kendala</label>
					<textarea id="kendala" class="input min-h-20" bind:value={kendala} placeholder="Opsional"></textarea>
				</div>
			</div>
		{:else}
			<div class="space-y-3">
				<div class="kartu p-4">
					<p class="judul-bagian">Bukti Pendukung</p>
					{#if lampiranLama.length > 0}
						<div class="mt-2">
							<LampiranList
								lampiran={lampiranLama.filter((item) => !lampiranDihapus.includes(item.id))}
								onhapus={(item) => (lampiranDihapus = [...lampiranDihapus, item.id])}
							/>
						</div>
					{:else}
						<p class="mt-1 text-xs text-ink-400">Belum ada bukti tersimpan.</p>
					{/if}
					<label class="btn btn-kedua mt-3 w-full cursor-pointer">
						<Icon name="upload" size={16} /> Tambah Berkas
						<input type="file" multiple class="sr-only" onchange={pilihBerkas} />
					</label>
					{#if buktiBaru.length > 0}
						<ul class="mt-3 space-y-2">
							{#each buktiBaru as file, i (file.name + i)}
								<li class="flex items-center gap-2 rounded-xl border border-ink-100 px-3 py-2">
									<Icon name="file" size={15} class="shrink-0 text-brand-600" />
									<div class="min-w-0 flex-1">
										<p class="truncate text-xs font-semibold text-ink-700">{file.name}</p>
										<p class="text-[10px] text-ink-400">{ukuranBerkas(file.size)}</p>
									</div>
									<button class="p-1.5 text-rose-600" onclick={() => (buktiBaru = buktiBaru.filter((_, x) => x !== i))}>
										<Icon name="x" size={15} />
									</button>
								</li>
							{/each}
						</ul>
					{/if}
				</div>

				<button class="btn btn-utama w-full py-3" disabled={memproses || !rencanaId} onclick={() => simpan(true)}>
					<Icon name="send" size={17} />
					{memproses ? 'Menyimpan…' : 'Ajukan ke Atasan'}
				</button>
				<button class="btn btn-kedua w-full" disabled={memproses} onclick={() => simpan(false)}>
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
