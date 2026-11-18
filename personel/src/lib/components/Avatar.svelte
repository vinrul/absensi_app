<script lang="ts">
	import { onDestroy } from 'svelte';
	import { apiBlobUrl } from '#lib/api/client';
	import type { Pengguna } from '#lib/types';

	let {
		pengguna,
		size = 44,
		versi = 0,
		class: kelas = ''
	}: { pengguna: Pengguna | null; size?: number; versi?: number; class?: string } = $props();

	let url = $state<string | null>(null);
	let urlLama: string | null = null;

	function pasang(baru: string | null) {
		if (urlLama && urlLama !== baru) URL.revokeObjectURL(urlLama);
		urlLama = baru;
		url = baru;
	}

	$effect(() => {
		const foto = pengguna?.foto_url ?? null;
		const penanda = versi;
		let hasil: string | null = null;
		let batal = false;

		if (!foto) {
			pasang(null);
			return;
		}

		void (async () => {
			try {
				hasil = await apiBlobUrl(penanda ? `${foto}?v=${penanda}` : foto);
				if (batal) {
					URL.revokeObjectURL(hasil);
					return;
				}
				pasang(hasil);
			} catch {
				if (!batal) pasang(null);
			}
		})();

		return () => {
			batal = true;
		};
	});

	onDestroy(() => {
		if (urlLama) URL.revokeObjectURL(urlLama);
		urlLama = null;
	});
</script>

<div
	class="flex shrink-0 items-center justify-center overflow-hidden rounded-full bg-gradient-to-br from-brand-600 to-brand-800 font-bold text-white {kelas}"
	style="width: {size}px; height: {size}px; font-size: {Math.round(size * 0.4)}px"
>
	{#if url}
		<img src={url} alt="Foto {pengguna?.nama ?? ''}" class="h-full w-full object-cover" />
	{:else}
		{pengguna?.nama?.charAt(0)?.toUpperCase() ?? '?'}
	{/if}
</div>
