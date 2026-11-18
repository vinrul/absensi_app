<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import LampiranList from '#lib/components/LampiranList.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import Sheet from '#lib/components/Sheet.svelte';
	import { LABEL_JENIS_RENCANA, tanggalIndo } from '#lib/format';
	import type { ListResponse, Rencana, RencanaDetail } from '#lib/types';

	const KEPUTUSAN = [
		{ nilai: 'disetujui', label: 'Setujui', ket: 'Rencana menjadi aktif dan dapat dilaksanakan.' },
		{ nilai: 'perlu_revisi', label: 'Minta Revisi', ket: 'Personel harus memperbaiki rencana sesuai catatan.' },
		{ nilai: 'ditolak', label: 'Tolak', ket: 'Rencana tidak dilanjutkan.' }
	];

	let daftar = $state<Rencana[]>([]);
	let memuat = $state(true);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);

	let terpilih = $state<Rencana | null>(null);
	let detail = $state<RencanaDetail | null>(null);
	let memuatDetail = $state(false);
	let aksi = $state('disetujui');
	let catatan = $state('');
	let memproses = $state(false);

	async function muat() {
		memuat = true;
		galat = null;
		try {
			const hasil = await api<ListResponse<Rencana>>('/rencana-kerja/persetujuan?per_page=50');
			daftar = hasil.data;
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Daftar persetujuan tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	async function buka(item: Rencana) {
		terpilih = item;
		detail = null;
		memuatDetail = true;
		aksi = 'disetujui';
		catatan = '';
		galat = null;
		try {
			detail = await api<RencanaDetail>(`/rencana-kerja/${item.id}`);
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Detail rencana tidak dapat dimuat.';
		} finally {
			memuatDetail = false;
		}
	}

	function tutup() {
		terpilih = null;
		detail = null;
		galat = null;
	}

	async function putuskan() {
		const item = terpilih;
		if (!item || memproses) return;
		if (aksi !== 'disetujui' && !catatan.trim()) {
			galat = 'Catatan wajib diisi saat meminta revisi atau menolak.';
			return;
		}
		memproses = true;
		galat = null;
		try {
			await api(`/rencana-kerja/${item.id}/keputusan`, {
				method: 'POST',
				body: { status: aksi, catatan: catatan.trim() || null }
			});
			pesan =
				aksi === 'disetujui'
					? 'Rencana kerja disetujui.'
					: aksi === 'perlu_revisi'
						? 'Rencana dikembalikan untuk revisi.'
						: 'Rencana ditolak.';
			tutup();
			await muat();
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Keputusan gagal disimpan.';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Persetujuan Rencana — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	<a href="/rencana" class="btn btn-hantu -ml-2 px-2 py-1">
		<Icon name="left" size={16} /> Kembali
	</a>

	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	<div>
		<h1 class="text-lg font-bold text-ink-900">Persetujuan Rencana</h1>
		<p class="text-[11px] text-ink-500">{daftar.length} dokumen menunggu keputusan</p>
	</div>

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat persetujuan…</div>
	{:else if daftar.length === 0}
		<div class="kartu p-6 text-center">
			<p class="text-sm font-semibold text-ink-600">Tidak ada rencana yang menunggu keputusan.</p>
		</div>
	{:else}
		{#each daftar as item (item.id)}
			<button type="button" class="kartu w-full p-4 text-left" onclick={() => buka(item)}>
				<div class="flex items-start justify-between gap-3">
					<div class="min-w-0">
						<p class="text-sm font-bold text-ink-900">{item.kegiatan}</p>
						<p class="text-[11px] text-ink-500">{item.program}</p>
						<p class="mt-1 text-[11px] text-ink-400">
							{item.personel_nama ?? 'Personel'} · {item.personel_nrp ?? '-'}
						</p>
					</div>
					<Icon name="right" size={18} class="shrink-0 text-ink-300" />
				</div>
				<div class="mt-2 flex flex-wrap gap-1.5">
					<span class="chip bg-ink-100 text-ink-600">
						{LABEL_JENIS_RENCANA[item.jenis] ?? item.jenis}
					</span>
					<span class="chip bg-ink-100 text-ink-600">
						{tanggalIndo(item.tanggal_mulai)} – {tanggalIndo(item.tanggal_selesai)}
					</span>
					{#if item.jumlah_revisi > 0}
						<span class="chip bg-amber-100 text-amber-800">Revisi {item.jumlah_revisi}×</span>
					{/if}
				</div>
			</button>
		{/each}
	{/if}
</div>

{#if terpilih}
	<Sheet judul="Detail & Keputusan" onTutup={tutup}>
		{#if memuatDetail}
			<p class="text-sm text-ink-400">Memuat detail rencana…</p>
		{:else if detail}
			<div class="space-y-3">
				{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

				<div>
					<h3 class="text-base font-bold text-ink-900">{detail.kegiatan}</h3>
					<p class="text-xs text-ink-500">{detail.program}</p>
					<p class="mt-1 text-[11px] text-ink-400">
						{detail.personel_nama ?? '-'} · {detail.personel_nrp ?? '-'}
					</p>
				</div>

				<dl>
					<div class="baris-info">
						<dt>Jenis</dt>
						<dd>{LABEL_JENIS_RENCANA[detail.jenis] ?? detail.jenis}</dd>
					</div>
					<div class="baris-info">
						<dt>Periode</dt>
						<dd>{tanggalIndo(detail.tanggal_mulai)} – {tanggalIndo(detail.tanggal_selesai)}</dd>
					</div>
					<div class="baris-info">
						<dt>Target</dt>
						<dd>{detail.target_kuantitas ?? '-'} {detail.satuan ?? ''}</dd>
					</div>
					{#if detail.dasar_nomor}
						<div class="baris-info">
							<dt>Dasar</dt>
							<dd>{detail.dasar_nomor}</dd>
						</div>
					{/if}
				</dl>

				<div>
					<p class="text-[11px] font-bold uppercase tracking-wider text-ink-400">Uraian</p>
					<p class="mt-1 whitespace-pre-line text-sm leading-relaxed text-ink-600">{detail.uraian}</p>
				</div>

				{#if detail.tahapan.length > 0}
					<div>
						<p class="text-[11px] font-bold uppercase tracking-wider text-ink-400">Rincian</p>
						<ul class="mt-1 space-y-1.5">
							{#each detail.tahapan as tahap, i (tahap.id)}
								<li class="rounded-xl bg-ink-50 px-3 py-2">
									<p class="text-xs font-semibold text-ink-700">{i + 1}. {tahap.uraian}</p>
									<p class="text-[10px] text-ink-500">
										{tanggalIndo(tahap.tanggal_mulai)} – {tanggalIndo(tahap.tanggal_selesai)}
									</p>
								</li>
							{/each}
						</ul>
					</div>
				{/if}

				<LampiranList lampiran={detail.lampiran} judul="Lampiran" />

				<div class="space-y-2">
					<p class="text-[11px] font-bold uppercase tracking-wider text-ink-400">Keputusan</p>
					{#each KEPUTUSAN as opsi (opsi.nilai)}
						<button
							type="button"
							class="flex w-full items-start gap-2 rounded-xl border p-3 text-left {aksi ===
							opsi.nilai
								? 'border-brand-500 bg-brand-50'
								: 'border-ink-200'}"
							onclick={() => (aksi = opsi.nilai)}
						>
							<span
								class="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-full border-2 {aksi ===
								opsi.nilai
									? 'border-brand-600 bg-brand-600'
									: 'border-ink-300'}"
							>
								{#if aksi === opsi.nilai}
									<span class="size-1 rounded-full bg-white"></span>
								{/if}
							</span>
							<span>
								<span class="block text-xs font-bold text-ink-800">{opsi.label}</span>
								<span class="block text-[10px] text-ink-500">{opsi.ket}</span>
							</span>
						</button>
					{/each}
				</div>

				<div>
					<label class="label" for="catatan">Catatan</label>
					<textarea
						id="catatan"
						class="input min-h-20"
						placeholder={aksi === 'disetujui' ? 'Opsional' : 'Wajib diisi'}
						bind:value={catatan}
					></textarea>
				</div>

				<button class="btn btn-utama w-full" disabled={memproses} onclick={putuskan}>
					{memproses ? 'Menyimpan…' : 'Simpan Keputusan'}
				</button>
			</div>
		{:else}
			<Pesan jenis="galat" pesan={galat ?? 'Detail tidak tersedia.'} />
		{/if}
	</Sheet>
{/if}
