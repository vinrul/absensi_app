<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { logout, pulihkanSesi, session } from '#lib/stores/session.svelte';
	import Icon from '#lib/components/Icon.svelte';

	let { children }: { children?: Snippet } = $props();

	let menuTerbuka = $state(false);
	let siap = $state(false);

	type Item = { href: string; label: string; icon: string; roles: string[] };

	const adminRole = [
		'admin_sistem',
		'admin_polda',
		'admin_satker_polda',
		'admin_polres',
		'admin_satker_polres'
	];
	const pimpinanRole = [
		'pimpinan',
		'pimpinan_polda',
		'pimpinan_satker_polda',
		'pimpinan_polres',
		'pimpinan_satker_polres'
	];
	const semuaRole = ['personel', 'atasan', ...pimpinanRole, ...adminRole];
	const pengelolaRole = ['atasan', ...adminRole];
	const dashboardRole = ['atasan', ...pimpinanRole, ...adminRole];

	const menu: Item[] = [
		{ href: '/dashboard', label: 'Dashboard', icon: 'dashboard', roles: dashboardRole },
		{
			href: '/admin',
			label: 'Akun Admin',
			icon: 'users',
			roles: ['admin_sistem', 'admin_polda', 'admin_polres']
		},
		{ href: '/personel', label: 'Data Personel', icon: 'users', roles: adminRole },
		{ href: '/satker', label: 'Satker & Jam Kerja', icon: 'pin', roles: adminRole },
		{ href: '/kesiapan', label: 'Kesiapan Periode', icon: 'check', roles: adminRole },
		{ href: '/libur', label: 'Hari Libur', icon: 'absensi', roles: semuaRole },
		{ href: '/ph', label: 'PH & Strong Point', icon: 'clock', roles: dashboardRole },
		{ href: '/absensi', label: 'Absensi Saya', icon: 'absensi', roles: semuaRole },
		{ href: '/absensi/rekap', label: 'Rekap Kehadiran', icon: 'users', roles: dashboardRole },
		{ href: '/absensi/titik-dinas', label: 'Titik Dinas', icon: 'pin', roles: pengelolaRole },
		{ href: '/rencana-kerja', label: 'Rencana Kerja', icon: 'rencana', roles: semuaRole },
		{ href: '/persetujuan', label: 'Persetujuan', icon: 'check', roles: dashboardRole },
		{ href: '/hasil-kerja', label: 'Hasil Kerja', icon: 'hasil', roles: semuaRole },
		{ href: '/output', label: 'Output', icon: 'output', roles: semuaRole },
		{ href: '/penilaian', label: 'Penilaian', icon: 'penilaian', roles: dashboardRole },
		{ href: '/laporan', label: 'Laporan', icon: 'laporan', roles: dashboardRole },
		{ href: '/pengaturan', label: 'Pengaturan', icon: 'pengaturan', roles: ['admin_sistem'] }
	];

	const menuTampil = $derived(
		menu.filter((item) => item.roles.some((role) => (session.user?.roles ?? []).includes(role as never)))
	);

	const judulHalaman = $derived(
		menu.find((item) => page.url.pathname === item.href)?.label ??
			(page.url.pathname.startsWith('/rencana-kerja')
				? 'Rencana Kerja'
				: page.url.pathname.startsWith('/absensi')
					? 'Absensi'
					: page.url.pathname.startsWith('/personel')
						? 'Data Personel'
						: page.url.pathname.startsWith('/admin')
							? 'Akun Admin'
							: page.url.pathname.startsWith('/satker')
								? 'Satker & Jam Kerja'
								: page.url.pathname.startsWith('/kesiapan')
									? 'Kesiapan Periode'
									: page.url.pathname.startsWith('/libur')
										? 'Hari Libur'
										: page.url.pathname.startsWith('/ph')
									? 'PH & Strong Point'
									: 'Penilaian Kinerja')
	);

	function aktif(href: string) {
		if (href === '/absensi') return page.url.pathname === '/absensi';
		return page.url.pathname === href || page.url.pathname.startsWith(href + '/');
	}

	onMount(async () => {
		const masuk = session.user ? true : await pulihkanSesi();
		if (!masuk) {
			goto('/login', { replaceState: true });
			return;
		}
		siap = true;
	});

	async function keluar() {
		await logout();
		await goto('/login', { replaceState: true });
	}
</script>

{#if !siap}
	<div class="flex min-h-screen items-center justify-center text-sm text-slate-500">
		Memuat sesi…
	</div>
{:else}
	<div class="flex min-h-screen bg-slate-100">
		<!-- Sidebar -->
		<aside
			class="fixed inset-y-0 left-0 z-40 w-64 -translate-x-full bg-navy-800 text-navy-100 transition-transform lg:static lg:translate-x-0 {menuTerbuka
				? 'translate-x-0'
				: ''}"
		>
			<div class="flex h-16 items-center gap-3 border-b border-white/10 px-5">
				<div
					class="flex size-10 shrink-0 items-center justify-center rounded-lg bg-white p-1"
				>
					<img
						src="/logo-polda-bali.png"
						alt="Logo Polda Bali"
						class="h-full w-auto"
					/>
				</div>
				<div class="min-w-0">
					<p class="truncate text-sm font-semibold text-white">Penilaian Kinerja</p>
					<p class="truncate text-[11px] text-navy-300">Polda Bali</p>
				</div>
			</div>

			<nav class="space-y-1 px-3 py-4">
				{#each menuTampil as item}
					<a
						href={item.href}
						class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors {aktif(item.href)
							? 'bg-gold-500 font-medium text-navy-900'
							: 'text-navy-100 hover:bg-white/10'}"
						onclick={() => (menuTerbuka = false)}
					>
						<Icon name={item.icon} size={17} />
						<span class="truncate">{item.label}</span>
					</a>
				{/each}
			</nav>

			<div class="absolute inset-x-0 bottom-0 border-t border-white/10 p-3">
				<button
					class="flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm text-navy-100 hover:bg-white/10"
					onclick={keluar}
				>
					<Icon name="logout" size={17} />
					Keluar
				</button>
			</div>
		</aside>

		{#if menuTerbuka}
			<button
				type="button"
				class="fixed inset-0 z-30 bg-slate-900/40 lg:hidden"
				aria-label="Tutup menu"
				onclick={() => (menuTerbuka = false)}
			></button>
		{/if}

		<!-- Konten -->
		<div class="flex min-w-0 flex-1 flex-col">
			<header
				class="sticky top-0 z-20 flex h-16 items-center justify-between gap-4 border-b border-slate-200 bg-white px-4 lg:px-6 no-print"
			>
				<div class="flex items-center gap-3">
					<button
						class="rounded-lg p-2 text-slate-500 hover:bg-slate-100 lg:hidden"
						onclick={() => (menuTerbuka = true)}
						aria-label="Buka menu"
					>
						<Icon name="menu" size={20} />
					</button>
					<img src="/logo-polda-bali.png" alt="Logo Polda Bali" class="h-8 w-auto lg:hidden" />
					<div>
						<h1 class="text-sm font-semibold text-slate-800">{judulHalaman}</h1>
						<p class="text-[11px] text-slate-500">{session.user?.satker_nama}</p>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<div class="hidden text-right sm:block">
						<p class="text-sm font-medium text-slate-800">{session.user?.nama}</p>
						<p class="text-[11px] text-slate-500">
							{session.user?.pangkat} · {session.user?.jenis_akun === 'admin'
								? `@${session.user?.username ?? '-'}`
								: `NRP ${session.user?.nrp ?? '-'}`}
						</p>
					</div>
					<div
						class="flex size-9 items-center justify-center rounded-full bg-navy-100 text-sm font-semibold text-navy-800"
					>
						{(session.user?.nama ?? '?').slice(0, 1)}
					</div>
				</div>
			</header>

			<main class="flex-1 px-4 py-5 lg:px-6 lg:py-6">
				{@render children?.()}
			</main>
		</div>
	</div>
{/if}
