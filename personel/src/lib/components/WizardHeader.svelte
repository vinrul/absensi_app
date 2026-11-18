<script lang="ts">
	import Icon from '#lib/components/Icon.svelte';

	let {
		langkah,
		total,
		judul,
		deskripsi,
		onkembali,
		labelKembali = 'Kembali'
	}: {
		langkah: number;
		total: number;
		judul: string;
		deskripsi?: string;
		onkembali: () => void;
		labelKembali?: string;
	} = $props();

	const persen = $derived(Math.min(100, Math.max(0, (langkah / total) * 100)));
</script>

<div class="space-y-3">
	<button type="button" class="btn btn-hantu -ml-2 px-2 py-1" onclick={onkembali}>
		<Icon name="left" size={16} />
		{labelKembali}
	</button>

	<div class="flex items-end justify-between gap-3">
		<div>
			<h1 class="text-xl font-bold tracking-tight text-ink-900">{judul}</h1>
			{#if deskripsi}
				<p class="mt-0.5 text-xs leading-relaxed text-ink-500">{deskripsi}</p>
			{/if}
		</div>
		<span class="shrink-0 text-[11px] font-bold text-brand-700">
			Langkah {langkah} dari {total}
		</span>
	</div>

	<div class="relative h-1.5 w-full overflow-hidden rounded-full bg-ink-100">
		<div
			class="absolute inset-y-0 left-0 rounded-full bg-gradient-to-r from-brand-800 to-brand-500 transition-all duration-300"
			style="width: {persen}%"
		></div>
		<span
			class="absolute top-1/2 size-1.5 -translate-y-1/2 rounded-full bg-brand-800 transition-all duration-300"
			style="left: calc({persen}% - 3px)"
		></span>
	</div>
</div>
