<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		children,
		tone = 'slate'
	}: { children?: Snippet; tone?: 'green' | 'amber' | 'red' | 'blue' | 'slate' | 'gold' } = $props();

	const gaya: Record<string, string> = {
		green: 'bg-emerald-50 text-emerald-700 ring-emerald-200',
		amber: 'bg-amber-50 text-amber-700 ring-amber-200',
		red: 'bg-rose-50 text-rose-700 ring-rose-200',
		blue: 'bg-sky-50 text-sky-700 ring-sky-200',
		slate: 'bg-slate-100 text-slate-600 ring-slate-200',
		gold: 'bg-amber-100 text-amber-800 ring-amber-300'
	};
</script>

<span
	class="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ring-1 ring-inset {gaya[
		tone
	]}"
>
	{@render children?.()}
</span>
