<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import LampiranList from '#lib/components/LampiranList.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import Sheet from '#lib/components/Sheet.svelte';
	import { LABEL_JENIS_OUTPUT, tanggalIndo } from '#lib/format';
	import type { ListResponse, Output, OutputDetail } from '#lib/types';

	let daftar = $state<Output[]>([]);
	let memuat = $state(true);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);

	let terpilih = $state<Output | null>(null);
	let detail = $state<OutputDetail | null>(null);
	let memuatDetail = $state(false);
	let setuju = $state(true);
	let skorKesesuaian = $state(90);
	let skorKeabsahan = $state(92);
	let catatan = $state('');
	let memproses = $state(false);

	async function muat() {
		memuat = true;
		galat = null;
		try {
			const hasil = await api<ListResponse<Output>>('/output/verifikasi?per_page=50');
			daftar = hasil.data;
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Daftar verifikasi tidak dapat dimuat.';
		} finally {
			memuat = false;
		}
	}

	async function buka(item: Output) {
		terpilih = item;
		detail = null;
		memuatDetail = true;
		setuju = true;
		skorKesesuaian = 90;
		skorKeabsahan = 92;
		catatan = '';
		galat = null;
		try {
			detail = await api<OutputDetail>(`/output/${item.id}`);
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Detail output tidak dapat dimuat.';
		} finally {
			memuatDetail = false;
		}
	}

	function tutup() {
		terpilih = null;
		detail = null;
		galat = null;
	}

	async function kirim() {
		const item = terpilih;
		if (!item || memproses) return;
		if (!setuju && !catatan.trim()) {
			galat = 'Catatan wajib diisi saat mengembalikan output untuk revisi.';
			return;
		}
		memproses = true;
		galat = null;
		try {
			await api(`/output/${item.id}/verifikasi`, {
				method: 'POST',
				body: {
					setuju,
					skor_kesesuaian: setuju ? skorKesesuaian : null,
					skor_keabsahan: setuju ? skorKeabsahan : null,
					catatan: catatan.trim() || null
				}
			});
			pesan = setuju ? 'Output disetujui.' : 'Output dikembalikan untuk revisi.';
			tutup();
			await muat();
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Verifikasi gagal disimpan.';
		} finally {
			memproses = false;
		}
	}

	onMount(muat);
</script>

<svelte:head><title>Verifikasi Output — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	<a href="/output" class="btn btn-hantu -ml-2 px-2 py-1"><Icon name="left" size={16} /> Kembali</a>

	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

	<div>
		<h1 class="text-lg font-bold text-ink-900">Verifikasi Output</h1>
		<p class="text-[11px] text-ink-500">{daftar.length} output menunggu verifikasi</p>
	</div>

	{#if memuat}
		<div class="kartu p-6 text-center text-sm text-ink-400">Memuat verifikasi…</div>
	{:else if daftar.length === 0}
		<div class="kartu p-6 text-center text-sm text-ink-500">
			Tidak ada output yang menunggu verifikasi.
		</div>
	{:else}
		{#each daftar as item (item.id)}
			<button type="button" class="kartu w-full p-4 text-left" onclick={() => buka(item)}>
				<div class="flex items-start justify-between gap-3">
					<div class="min-w-0">
						<p class="text-sm font-bold text-ink-900">{item.judul}</p>
						<p class="text-[11px] text-ink-500">
							{LABEL_JENIS_OUTPUT[item.jenis_output] ?? item.jenis_output} · {tanggalIndo(item.tanggal_hasil)}
						</p>
						<p class="text-[10px] text-ink-400">{item.personel_nama ?? '-'} · {item.personel_nrp ?? '-'}</p>
					</div>
					<Icon name="right" size={18} class="shrink-0 text-ink-300" />
				</div>
			</button>
		{/each}
	{/if}
</div>

{#if terpilih}
	<Sheet judul="Review Output" onTutup={tutup}>
		{#if memuatDetail}
			<p class="text-sm text-ink-400">Memuat detail…</p>
		{:else if detail}
			<div class="space-y-3">
				{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}

				<div>
					<h3 class="text-base font-bold text-ink-900">{detail.judul}</h3>
					<p class="text-[11px] text-ink-500">
						{LABEL_JENIS_OUTPUT[detail.jenis_output] ?? detail.jenis_output} ·
						{tanggalIndo(detail.tanggal_hasil)} · {detail.personel_nama ?? '-'}
					</p>
				</div>

				{#if detail.uraian}
					<p class="whitespace-pre-line text-sm leading-relaxed text-ink-600">{detail.uraian}</p>
				{/if}

				{#if detail.url_eksternal}
					<a
						href={detail.url_eksternal}
						target="_blank"
						rel="noopener noreferrer"
						class="block break-all text-xs font-semibold text-brand-700 underline"
					>
						{detail.url_eksternal}
					</a>
				{/if}

				<LampiranList lampiran={detail.lampiran} judul="Berkas Bukti" />

				<div class="space-y-2">
					<p class="text-[11px] font-bold uppercase tracking-wider text-ink-400">Keputusan</p>
					<div class="grid grid-cols-2 gap-2">
						<button
							type="button"
							class="rounded-xl border p-3 text-xs font-bold {setuju
								? 'border-emerald-500 bg-emerald-50 text-emerald-800'
								: 'border-ink-200 text-ink-500'}"
							onclick={() => (setuju = true)}
						>
							Disetujui (Valid)
						</button>
						<button
							type="button"
							class="rounded-xl border p-3 text-xs font-bold {!setuju
								? 'border-rose-500 bg-rose-50 text-rose-800'
								: 'border-ink-200 text-ink-500'}"
							onclick={() => (setuju = false)}
						>
							Kembalikan &amp; Revisi
						</button>
					</div>
				</div>

				{#if setuju}
					<div class="space-y-3">
						<div>
							<div class="flex justify-between text-[11px] text-ink-500">
								<span class="label !mb-0">Skor Kesesuaian</span>
								<span class="font-bold text-brand-700">{skorKesesuaian}</span>
							</div>
							<input
								type="range"
								min="0"
								max="100"
								step="1"
								class="mt-2 w-full accent-brand-600"
								bind:value={skorKesesuaian}
							/>
						</div>
						<div>
							<div class="flex justify-between text-[11px] text-ink-500">
								<span class="label !mb-0">Skor Keabsahan</span>
								<span class="font-bold text-brand-700">{skorKeabsahan}</span>
							</div>
							<input
								type="range"
								min="0"
								max="100"
								step="1"
								class="mt-2 w-full accent-brand-600"
								bind:value={skorKeabsahan}
							/>
						</div>
					</div>
				{/if}

				<div>
					<label class="label" for="catatan-output">Catatan</label>
					<textarea
						id="catatan-output"
						class="input min-h-20"
						placeholder={setuju ? 'Opsional' : 'Wajib diisi'}
						bind:value={catatan}
					></textarea>
				</div>

				<button class="btn btn-utama w-full" disabled={memproses} onclick={kirim}>
					{memproses ? 'Menyimpan…' : 'Simpan Keputusan'}
				</button>
			</div>
		{:else}
			<Pesan jenis="galat" pesan={galat ?? 'Detail tidak tersedia.'} />
		{/if}
	</Sheet>
{/if}
