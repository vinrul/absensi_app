<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from './Icon.svelte';

	let {
		open = false,
		title = '',
		onclose,
		children,
		footer,
		lebar = 'max-w-lg'
	}: {
		open?: boolean;
		title?: string;
		onclose?: () => void;
		children?: Snippet;
		footer?: Snippet;
		lebar?: string;
	} = $props();
</script>

{#if open}
	<div class="fixed inset-0 z-50 flex items-center justify-center p-4">
		<button
			type="button"
			class="absolute inset-0 cursor-default bg-slate-900/50"
			aria-label="Tutup"
			onclick={() => onclose?.()}
		></button>

		<div
			class="card relative z-10 flex max-h-[calc(100vh-2rem)] w-full {lebar} flex-col overflow-hidden p-0"
		>
			<div class="flex shrink-0 items-start justify-between gap-3 border-b border-slate-200 px-5 py-4">
				<h2 class="text-base font-semibold text-slate-800">{title}</h2>
				<button
					type="button"
					class="btn btn-ghost -mr-1 shrink-0 p-1"
					onclick={() => onclose?.()}
					aria-label="Tutup"
				>
					<Icon name="x" />
				</button>
			</div>

			<div class="min-h-0 flex-1 overflow-y-auto px-5 py-4">
				{@render children?.()}
			</div>

			{#if footer}
				<div class="shrink-0 border-t border-slate-200 bg-white px-5 py-3">
					{@render footer()}
				</div>
			{/if}
		</div>
	</div>
{/if}
