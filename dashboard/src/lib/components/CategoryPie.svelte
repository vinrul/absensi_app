<script lang="ts">
	import type { Kategori } from '#lib/types';
	import { LABEL_KATEGORI } from '#lib/types';

	interface Item {
		kategori: Kategori;
		jumlah: number;
		persen: number;
	}

	let { items = [] }: { items?: Item[] } = $props();

	const warna: Record<Kategori, string> = {
		sangat_baik: '#16a34a',
		baik: '#2563eb',
		cukup: '#f59e0b',
		perlu_pembinaan: '#e11d48'
	};

	const total = $derived(items.reduce((jumlah, item) => jumlah + item.jumlah, 0));
	const r = 46;
	const keliling = 2 * Math.PI * r;

	let akumulasi = 0;
	const segmen = $derived.by(() => {
		akumulasi = 0;
		return items.map((item) => {
			const porsi = total > 0 ? item.jumlah / total : 0;
			const mulai = akumulasi;
			akumulasi += porsi;
			return {
				...item,
				panjang: porsi * keliling,
				offset: -mulai * keliling
			};
		});
	});
</script>

<div class="flex flex-col items-center gap-4 sm:flex-row">
	<div class="relative">
		<svg viewBox="0 0 120 120" width="132" height="132" role="img" aria-label="Sebaran kategori kinerja">
			<circle cx="60" cy="60" r={r} fill="none" stroke="#e2e8f0" stroke-width="14" />
			{#each segmen as s}
				{#if s.panjang > 0}
					<circle
						cx="60"
						cy="60"
						r={r}
						fill="none"
						stroke={warna[s.kategori]}
						stroke-width="14"
						stroke-dasharray="{s.panjang} {keliling - s.panjang}"
						stroke-dashoffset={s.offset}
						transform="rotate(-90 60 60)"
					/>
				{/if}
			{/each}
			<text x="60" y="58" text-anchor="middle" font-size="20" font-weight="700" fill="#1e293b">
				{total}
			</text>
			<text x="60" y="74" text-anchor="middle" font-size="9" fill="#64748b">Personel</text>
		</svg>
	</div>

	<ul class="w-full max-w-[15rem] flex-1 space-y-1.5">
		{#each items as item}
			<li class="flex items-center justify-between gap-2 text-xs">
				<span class="flex items-center gap-2 text-slate-600">
					<span class="size-2.5 rounded-full" style="background:{warna[item.kategori]}"></span>
					{LABEL_KATEGORI[item.kategori]}
				</span>
				<span class="font-semibold text-slate-800">
					{item.jumlah} <span class="font-normal text-slate-400">({item.persen.toFixed(1)}%)</span>
				</span>
			</li>
		{/each}
	</ul>
</div>
