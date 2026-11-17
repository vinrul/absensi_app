<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import type { KesiapanResponse, Periode } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';

	let periodeList = $state<Periode[]>([]);
	let periodeId = $state('');
	let bobot = $state({ absensi: '20', rencana_kerja: '20', hasil_kerja: '30', output: '30' });
	let formPeriode = $state(false);
	let menyimpan = $state(false);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);
	let formP = $state({ nama: '', tanggal_mulai: '', tanggal_selesai: '' });

	async function muatPeriode() {
		periodeList = await api<Periode[]>('/periode');
		periodeId = periodeList[0]?.id ?? '';
	}

	async function simpanPeriode() {
		menyimpan = true;
		error = null;
		try {
			await api('/periode', {
				method: 'POST',
				body: {
					nama: formP.nama || null,
					tanggal_mulai: formP.tanggal_mulai,
					tanggal_selesai: formP.tanggal_selesai
				}
			});
			formPeriode = false;
			await muatPeriode();
			info = 'Periode penilaian dibuat dengan bobot default 20/20/30/30.';
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal membuat periode';
		} finally {
			menyimpan = false;
		}
	}

	async function simpanBobot() {
		menyimpan = true;
		error = null;
		try {
			await api(`/periode/${periodeId}/bobot`, {
				method: 'PUT',
				body: {
					absensi: Number(bobot.absensi),
					rencana_kerja: Number(bobot.rencana_kerja),
					hasil_kerja: Number(bobot.hasil_kerja),
					output: Number(bobot.output)
				}
			});
			info = 'Bobot indikator diperbarui. Hitung ulang penilaian agar berlaku.';
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menyimpan bobot';
		} finally {
			menyimpan = false;
		}
	}

	async function tutupPeriode() {
		let pesan = 'Tutup periode ini? Penilaian tidak dapat diubah setelah ditutup.';
		try {
			const kesiapan = await api<KesiapanResponse>(`/periode/${periodeId}/kesiapan`);
			if (kesiapan.ringkasan.belum_siap > 0) {
				pesan = `Masih ada ${kesiapan.ringkasan.belum_siap} satker yang belum menandai siap. Tetap tutup periode?`;
			}
		} catch {
			// Tetap lanjut dengan konfirmasi standar bila ringkasan tidak dapat dimuat.
		}
		if (!confirm(pesan)) return;
		try {
			await api(`/periode/${periodeId}/tutup`, { method: 'POST' });
			await muatPeriode();
			info = 'Periode ditutup.';
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal menutup periode';
		}
	}

	async function bukaKembaliPeriode() {
		const alasan = prompt('Alasan membuka kembali periode (opsional):');
		if (alasan === null) return;
		if (!confirm('Buka kembali periode ini? Data dapat diubah lagi.')) return;
		try {
			await api(`/periode/${periodeId}/buka`, {
				method: 'POST',
				body: { alasan: alasan.trim() || null }
			});
			await muatPeriode();
			info = 'Periode dibuka kembali. Data dapat diubah lagi.';
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal membuka kembali periode';
		}
	}

	const periodeTerpilih = $derived(periodeList.find((p) => p.id === periodeId) ?? null);

	onMount(muatPeriode);
</script>

<svelte:head><title>Pengaturan — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
		<a href="/admin" class="card flex items-center gap-3 p-4 transition hover:border-navy-300">
			<div class="flex size-10 items-center justify-center rounded-lg bg-rose-100 text-rose-700"><Icon name="users" size={20} /></div>
			<div>
				<p class="text-sm font-semibold text-slate-800">Akun Admin</p>
				<p class="text-xs text-slate-500">Admin Polda/Polres/Satker, username, reset password</p>
			</div>
		</a>
		<a href="/personel" class="card flex items-center gap-3 p-4 transition hover:border-navy-300">
			<div class="flex size-10 items-center justify-center rounded-lg bg-navy-100 text-navy-800"><Icon name="users" size={20} /></div>
			<div>
				<p class="text-sm font-semibold text-slate-800">Data Personel</p>
				<p class="text-xs text-slate-500">Tambah, edit, atasan, role, reset password</p>
			</div>
		</a>
		<a href="/satker" class="card flex items-center gap-3 p-4 transition hover:border-navy-300">
			<div class="flex size-10 items-center justify-center rounded-lg bg-gold-500/20 text-gold-600"><Icon name="pin" size={20} /></div>
			<div>
				<p class="text-sm font-semibold text-slate-800">Satker & Jam Kerja</p>
				<p class="text-xs text-slate-500">Polda, Polres, satker, hari kerja</p>
			</div>
		</a>
		<a href="/kesiapan" class="card flex items-center gap-3 p-4 transition hover:border-navy-300">
			<div class="flex size-10 items-center justify-center rounded-lg bg-emerald-100 text-emerald-700"><Icon name="check" size={20} /></div>
			<div>
				<p class="text-sm font-semibold text-slate-800">Kesiapan Periode</p>
				<p class="text-xs text-slate-500">Satker yang sudah/belum siap ditutup</p>
			</div>
		</a>
	</div>

	{#if info}<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>{/if}
	{#if error}<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>{/if}

	<div class="grid gap-4 lg:grid-cols-2">
		<div class="card p-4">
			<div class="flex items-center justify-between">
				<h2 class="text-sm font-semibold text-slate-700">Periode Penilaian</h2>
				<button class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs" onclick={() => (formPeriode = true)}>
					<Icon name="plus" size={14} /> Periode Baru
				</button>
			</div>
			<p class="mt-1 text-xs text-slate-500">
				Periode bulan berjalan dibuat otomatis. Penutupan tetap manual setelah data
				bulan tersebut lengkap dan diverifikasi.
			</p>
			<ul class="mt-3 divide-y divide-slate-100">
				{#each periodeList as p}
					<li class="flex items-center justify-between gap-2 py-2">
						<div>
							<button class="text-left text-sm font-medium text-navy-800" onclick={() => (periodeId = p.id)}>
								{p.nama}
							</button>
							<p class="text-[11px] text-slate-500">{p.tanggal_mulai} → {p.tanggal_selesai}</p>
							{#if p.dibuka_kembali_at}
								<p class="text-[11px] text-amber-600">
									Terakhir dibuka kembali {p.dibuka_kembali_at.slice(0, 10)}
									{#if p.alasan_buka_kembali}· {p.alasan_buka_kembali}{/if}
								</p>
							{/if}
						</div>
						<Badge tone={p.status === 'open' ? 'green' : 'slate'}>{p.status === 'open' ? 'Aktif' : 'Ditutup'}</Badge>
					</li>
				{:else}
					<li class="py-3 text-xs text-slate-500">Belum ada periode.</li>
				{/each}
			</ul>
		</div>

		<div class="card p-4">
			<h2 class="text-sm font-semibold text-slate-700">Bobot Indikator</h2>
			<p class="mt-1 text-xs text-slate-500">Total bobot harus 100.</p>
			<div class="mt-3 grid grid-cols-2 gap-3">
				<div>
					<label class="label" for="b1">Absensi</label>
					<input id="b1" class="input" bind:value={bobot.absensi} />
				</div>
				<div>
					<label class="label" for="b2">Rencana Kerja</label>
					<input id="b2" class="input" bind:value={bobot.rencana_kerja} />
				</div>
				<div>
					<label class="label" for="b3">Hasil Kerja</label>
					<input id="b3" class="input" bind:value={bobot.hasil_kerja} />
				</div>
				<div>
					<label class="label" for="b4">Output</label>
					<input id="b4" class="input" bind:value={bobot.output} />
				</div>
			</div>
			<div class="mt-4 flex justify-between gap-2">
				{#if periodeTerpilih?.status === 'closed'}
					<button class="btn btn-ghost border border-amber-300 text-amber-700" onclick={bukaKembaliPeriode}>
						Buka Kembali
					</button>
				{:else}
					<button
						class="btn btn-ghost border border-rose-200 text-rose-600"
						disabled={!periodeId}
						onclick={tutupPeriode}
					>
						Tutup Periode
					</button>
				{/if}
				<button
					class="btn btn-primary"
					disabled={menyimpan || !periodeId || periodeTerpilih?.status === 'closed'}
					onclick={simpanBobot}
				>
					Simpan Bobot
				</button>
			</div>
		</div>
	</div>
</div>

<Modal open={formPeriode} title="Periode Penilaian Baru" onclose={() => (formPeriode = false)}>
	<div class="space-y-3">
		<div><label class="label" for="pnama2">Nama Periode</label><input id="pnama2" class="input" placeholder="November 2026" bind:value={formP.nama} /></div>
		<div class="grid grid-cols-2 gap-3">
			<div><label class="label" for="pmulai">Tanggal Mulai</label><input id="pmulai" type="date" class="input" bind:value={formP.tanggal_mulai} /></div>
			<div><label class="label" for="pselesai">Tanggal Selesai</label><input id="pselesai" type="date" class="input" bind:value={formP.tanggal_selesai} /></div>
		</div>
		<div class="flex justify-end gap-2">
			<button class="btn btn-ghost" onclick={() => (formPeriode = false)}>Batal</button>
			<button class="btn btn-primary" disabled={menyimpan} onclick={simpanPeriode}>Simpan</button>
		</div>
	</div>
</Modal>
