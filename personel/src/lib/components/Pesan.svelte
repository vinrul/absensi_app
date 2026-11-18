<script lang="ts">
	import Icon from '#lib/components/Icon.svelte';

	let {
		jenis = 'info',
		pesan,
		class: kelas = ''
	}: { jenis?: 'info' | 'sukses' | 'galat'; pesan: string; class?: string } = $props();

	const gaya = $derived(
		jenis === 'sukses'
			? 'bg-emerald-50 text-emerald-800 ring-emerald-200'
			: jenis === 'galat'
				? 'bg-rose-50 text-rose-800 ring-rose-200'
				: 'bg-brand-50 text-brand-900 ring-brand-100'
	);
</script>

<div class="flex items-start gap-2 rounded-xl px-3 py-2 text-sm ring-1 {gaya} {kelas}" role="alert">
	<span class="mt-0.5 shrink-0">
		<Icon name={jenis === 'sukses' ? 'checkCircle' : jenis === 'galat' ? 'alert' : 'info'} size={16} />
	</span>
	<p class="leading-snug">{pesan}</p>
</div>
