<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { bolehMasukDashboard, logout, pulihkanSesi } from '#lib/stores/session.svelte';

	onMount(async () => {
		const masuk = await pulihkanSesi();
		if (!masuk) {
			goto('/login', { replaceState: true });
			return;
		}
		if (!bolehMasukDashboard()) {
			await logout().catch(() => {});
			goto('/login', { replaceState: true });
			return;
		}
		goto('/dashboard', { replaceState: true });
	});
</script>

<div class="flex min-h-screen items-center justify-center text-sm text-slate-500">Memuat…</div>
