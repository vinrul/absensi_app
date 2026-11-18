<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { pulihkanSesi } from '#lib/stores/session.svelte';

	onMount(async () => {
		const masuk = await pulihkanSesi();
		await goto(masuk ? '/beranda' : '/login', { replaceState: true });
	});
</script>

<div class="flex min-h-dvh items-center justify-center">
	<div class="flex flex-col items-center gap-3 text-ink-500">
		<span class="size-8 animate-spin rounded-full border-2 border-brand-200 border-t-brand-600"></span>
		<p class="text-xs font-medium">Memuat sesi…</p>
	</div>
</div>
