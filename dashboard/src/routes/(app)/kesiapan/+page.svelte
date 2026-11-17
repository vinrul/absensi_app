<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import type { KesiapanResponse, KesiapanSatker, Periode } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Pagination from '#lib/components/Pagination.svelte';

	let periodeList = $state<Periode[]>([]);
	let periodeId = $state('');
	let data = $state<KesiapanResponse | null>(null);
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);
	let halaman = $state(1);
	const perPage = 10;

	const baris = $derived(data?.data ?? []);
	const totalHalaman = $derived(Math.max(1, Math.ceil(baris.length / perPage)));
	const halamanAman = $derived(Math.min(halaman, totalHalaman));
	const barisHalaman = $derived(baris.slice((halamanAman - 1) * perPage, halamanAman * perPage));

	const periodeTerpilih = $derived(periodeList.find((p) => p.id === periodeId) ?? null);
	const bisaTandai = $derived(periodeTerpilih?.status === 'open');

	function pesan(err: unknown): string {
		return err instanceof ApiError ? err.message : 'Terjadi kesalahan';
	}

	async function muatPeriode() {
		periodeList = await api<Periode[]>('/periode');
		periodeId = periodeList[0]?.id ?? '';
	}

	async function muat() {
		if (!periodeId) return;
		memuat = true;
		error = null;
		halaman = 1;
		try {
			data = await api<KesiapanResponse>(`/periode/${periodeId}/kesiapan`);
		} catch (err) {
			error = pesan(err);
			data = null;
		} finally {
			memuat = false;
		}
	}

	onMount(async () => {
		try {
			await muatPeriode();
			await muat();
		} catch (err) {
			error = pesan(err);
			memuat = false;
		}
	});

	async function gantiPeriode() {
		info = null;
		await muat();
	}

	async function tandai(row: KesiapanSatker, siap: boolean) {
		error = null;
		info = null;
		if (!bisaTandai) {
			error = 'Periode sudah ditutup. Buka kembali periode untuk mengubah kesiapan.';
			return;
		}
		if (siap && row.belum_dinilai > 0) {
			const lanjut = confirm(
				`${row.belum_dinilai} personel di ${row.satker_nama} belum memiliki penilaian. ` +
					'Tetap tandai siap ditutup?'
			);
			if (!lanjut) return;
		}
		if (!siap && !confirm(`Batalkan tanda siap untuk ${row.satker_nama}?`)) return;

		let catatan: string | null = null;
		if (siap) {
			const isian = prompt('Catatan kesiapan (opsional):', row.catatan ?? '');
			if (isian === null) return;
			catatan = isian.trim() || null;
		}

		try {
			await api(`/periode/${periodeId}/kesiapan/${row.satker_id}`, {
				method: 'POST',
				body: { siap, catatan }
			});
			info = siap
				? `${row.satker_nama} ditandai siap ditutup.`
				: `Tanda siap ${row.satker_nama} dibatalkan.`;
			await muat();
		} catch (err) {
			error = pesan(err);
		}
	}

	function statusBadge(row: KesiapanSatker) {
		return row.siap_ditutup ? 'green' : 'amber';
	}
</script>

<svelte:head><title>Kesiapan Periode — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div>
		<h1 class="text-xl font-semibold text-slate-800">Kesiapan Periode</h1>
		<p class="mt-1 text-sm text-slate-500">
			Admin satker menandai satkernya siap. Admin global memakai daftar ini sebelum menutup
			periode.
		</p>
	</div>

	{#if info}
		<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>
	{/if}
	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	<div class="card flex flex-wrap items-end gap-3 p-4">
		<div class="min-w-64 flex-1">
			<label class="label" for="periode">Periode</label>
			<select id="periode" class="input" bind:value={periodeId} onchange={gantiPeriode}>
				{#each periodeList as p}
					<option value={p.id}>{p.nama} — {p.status === 'open' ? 'Aktif' : 'Ditutup'}</option>
				{/each}
			</select>
		</div>
		{#if periodeTerpilih?.status === 'closed'}
			<p class="text-xs text-amber-700">
				Periode ditutup. Buka kembali lewat Pengaturan untuk mengubah kesiapan.
			</p>
		{/if}
	</div>

	{#if data}
		<div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
			<div class="card p-4">
				<p class="text-xs text-slate-500">Total satker</p>
				<p class="text-2xl font-semibold text-slate-800">{data.ringkasan.total_satker}</p>
			</div>
			<div class="card p-4">
				<p class="text-xs text-slate-500">Siap ditutup</p>
				<p class="text-2xl font-semibold text-emerald-600">{data.ringkasan.siap}</p>
			</div>
			<div class="card p-4">
				<p class="text-xs text-slate-500">Belum siap</p>
				<p class="text-2xl font-semibold text-amber-600">{data.ringkasan.belum_siap}</p>
			</div>
			<div class="card p-4">
				<p class="text-xs text-slate-500">Personel dinilai</p>
				<p class="text-2xl font-semibold text-slate-800">
					{data.ringkasan.sudah_dinilai}
					<span class="text-sm font-normal text-slate-500">/ {data.ringkasan.total_personel}</span>
				</p>
			</div>
		</div>

		<div class="card overflow-hidden">
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-slate-50">
						<tr>
							<th class="th">Satker</th>
							<th class="th text-center">Personel</th>
							<th class="th text-center">Dinilai</th>
							<th class="th text-center">Belum</th>
							<th class="th">Status</th>
							<th class="th">Ditandai</th>
							<th class="th text-right">Aksi</th>
						</tr>
					</thead>
					<tbody>
						{#each barisHalaman as row (row.satker_id)}
							<tr>
								<td class="td font-medium text-slate-800">{row.satker_nama}</td>
								<td class="td text-center text-slate-600">{row.jumlah_personel}</td>
								<td class="td text-center text-slate-600">{row.sudah_dinilai}</td>
								<td class="td text-center">
									<span class:font-semibold={row.belum_dinilai > 0} class="text-slate-600">
										{row.belum_dinilai}
									</span>
								</td>
								<td class="td">
									<Badge tone={statusBadge(row)}>
										{row.siap_ditutup ? 'Siap ditutup' : 'Belum siap'}
									</Badge>
								</td>
								<td class="td text-xs text-slate-500">
									{#if row.ditandai_at}
										{row.ditandai_oleh_nama ?? 'Admin'}
										<span class="block">{row.ditandai_at.slice(0, 16).replace('T', ' ')}</span>
										{#if row.catatan}
											<span class="block italic">“{row.catatan}”</span>
										{/if}
									{:else}
										—
									{/if}
								</td>
								<td class="td text-right">
									{#if bisaTandai}
										{#if row.siap_ditutup}
											<button
												class="btn btn-ghost px-2 py-1 text-xs"
												onclick={() => tandai(row, false)}
											>
												Batalkan
											</button>
										{:else}
											<button
												class="btn btn-primary px-3 py-1 text-xs"
												onclick={() => tandai(row, true)}
											>
												<Icon name="check" size={13} /> Tandai Siap
											</button>
										{/if}
									{:else}
										<span class="text-xs text-slate-400">Terkunci</span>
									{/if}
								</td>
							</tr>
						{:else}
							<tr>
								<td class="td text-slate-500" colspan="7">
									{memuat ? 'Memuat…' : 'Tidak ada satker dalam cakupan Anda.'}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
			<Pagination page={halamanAman} perPage={perPage} total={baris.length} label="satker" onchange={(p) => (halaman = p)} />
		</div>
	{:else}
		<div class="card p-6 text-sm text-slate-500">
			{memuat ? 'Memuat kesiapan periode…' : 'Belum ada data kesiapan.'}
		</div>
	{/if}
</div>
