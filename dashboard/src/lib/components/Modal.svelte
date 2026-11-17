<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from './Icon.svelte';

	let {
		open = false,
		title = '',
		onclose,
		children,
		lebar = 'max-w-lg'
	}: {
		open?: boolean;
		title?: string;
		onclose?: () => void;
		children?: Snippet;
		lebar?: string;
	} = $props();
</script>

{#if open}
	<div class="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto p-4 sm:items-center">
		<button
			type="button"
			class="absolute inset-0 cursor-default bg-slate-900/50"
			aria-label="Tutup"
			onclick={() => onclose?.()}
		></button>

		<div class="card relative z-10 w-full {lebar} p-5">
			<div class="mb-4 flex items-start justify-between gap-3">
				<h2 class="text-base font-semibold text-slate-800">{title}</h2>
				<button type="button" class="btn btn-ghost p-1" onclick={() => onclose?.()} aria-label="Tutup">
					<Icon name="x" />
				</button>
			</div>
			{@render children?.()}
		</div>
	</div>
{/if}
