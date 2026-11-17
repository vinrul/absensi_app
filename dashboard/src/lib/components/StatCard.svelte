<script lang="ts">
	let {
		label,
		nilai,
		sub = '',
		warna = 'bg-navy-800',
		icon = 'users'
	}: { label: string; nilai: string | number; sub?: string; warna?: string; icon?: string } =
		$props();

	import Icon from './Icon.svelte';
</script>

<div class="card p-4">
	<div class="flex items-start justify-between gap-3">
		<div>
			<p class="text-xs font-semibold tracking-wide text-slate-500 uppercase">{label}</p>
			<p class="mt-2 text-2xl font-bold text-slate-800">{nilai}</p>
			{#if sub}
				<p class="mt-1 text-xs text-slate-500">{sub}</p>
			{/if}
		</div>
		<div class="flex size-10 shrink-0 items-center justify-center rounded-lg text-white {warna}">
			<Icon name={icon} size={20} />
		</div>
	</div>
</div>
