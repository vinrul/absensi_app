<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from '#lib/components/Icon.svelte';

	let {
		judul,
		onTutup,
		children,
		footer
	}: {
		judul: string;
		onTutup: () => void;
		children?: Snippet;
		footer?: Snippet;
	} = $props();
</script>

<div class="fixed inset-0 z-50 flex items-end justify-center sm:items-center">
	<button
		type="button"
		class="sheet-backdrop absolute inset-0"
		aria-label="Tutup"
		onclick={onTutup}
	></button>

	<div
		class="relative z-10 flex max-h-[90dvh] w-full max-w-lg flex-col overflow-hidden rounded-t-3xl bg-white shadow-2xl sm:rounded-3xl"
	>
		<header class="flex shrink-0 items-center justify-between gap-3 border-b border-ink-100 px-4 py-3">
			<h2 class="text-base font-bold text-ink-900">{judul}</h2>
			<button type="button" class="btn btn-hantu p-1.5" aria-label="Tutup" onclick={onTutup}>
				<Icon name="x" size={18} />
			</button>
		</header>

		<div class="min-h-0 flex-1 overflow-y-auto px-4 py-4">
			{@render children?.()}
		</div>

		{#if footer}
			<footer class="shrink-0 border-t border-ink-100 bg-white px-4 py-3">
				{@render footer()}
			</footer>
		{/if}
	</div>
</div>
