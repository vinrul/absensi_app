<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { pulihkanSesi, session } from '#lib/stores/session.svelte';

	onMount(async () => {
		const masuk = await pulihkanSesi();
		if (!masuk) {
			goto('/login', { replaceState: true });
			return;
		}
		const bolehDashboard = session.user?.roles.some((role) =>
			['atasan', 'pimpinan', 'admin_sistem'].includes(role)
		);
		goto(bolehDashboard ? '/dashboard' : '/absensi', { replaceState: true });
	});
</script>

<div class="flex min-h-screen items-center justify-center text-sm text-slate-500">Memuat…</div>
