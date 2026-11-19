<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { logout, pulihkanSesi, session, bolehMasukDashboard } from '#lib/stores/session.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Avatar from '#lib/components/Avatar.svelte';

	let { children }: { children?: Snippet } = $props();

	let menuTerbuka = $state(false);
	let siap = $state(false);

	type Item = { href: string; label: string; icon: string; roles: string[] };
	type SubItem = { href: string; label: string; roles: string[] };
	type Grup = { label: string; icon: string; items: SubItem[]; basis?: string };

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
		{ href: '/dashboard', label: 'Dashboard', icon: 'dashboard', roles: dashboardRole }
	];

	const grupMenu: Grup[] = [
		{
			label: 'Master Data',
			icon: 'users',
			items: [
				{ href: '/satker', label: 'Satker & Jam Kerja', roles: adminRole },
				{ href: '/absensi/titik-dinas', label: 'Titik Dinas', roles: pengelolaRole },
				{ href: '/libur', label: 'Hari Libur', roles: semuaRole },
				{ href: '/personel', label: 'Data Personel', roles: adminRole }
			]
		},
		{
			label: 'Absensi',
			icon: 'absensi',
			items: [
				{ href: '/absensi/rekap?tab=rekap', label: 'Rekap Kehadiran', roles: dashboardRole },
				{ href: '/absensi/rekap?tab=detail', label: 'Detail Rekap', roles: dashboardRole },
				{
					href: '/absensi/rekap?tab=verifikasi',
					label: 'Verifikasi Offline',
					roles: dashboardRole
				}
			]
		},
		{
			label: 'Strong Point',
			icon: 'pin',
			basis: '/ph',
			items: [
				{ href: '/ph?tab=jadwal', label: 'Jadwal Strong Point', roles: dashboardRole },
				{ href: '/ph?tab=rekap', label: 'Rekap Strong Point', roles: dashboardRole }
			]
		},
		{
			label: 'Kinerja',
			icon: 'trend',
			items: [
				{ href: '/rencana-kerja', label: 'Rencana Kerja', roles: semuaRole },
				{ href: '/hasil-kerja', label: 'Hasil Kerja', roles: semuaRole },
				{ href: '/output', label: 'Output', roles: semuaRole }
			]
		},
		{
			label: 'Laporan',
			icon: 'laporan',
			items: [
				{ href: '/penilaian', label: 'Penilaian', roles: dashboardRole },
				{ href: '/laporan', label: 'Print Laporan', roles: dashboardRole }
			]
		},
		{
			label: 'Pengaturan',
			icon: 'pengaturan',
			items: [
				{
					href: '/admin',
					label: 'Akun Admin',
					roles: ['admin_sistem', 'admin_polda', 'admin_polres']
				},
				{ href: '/kesiapan', label: 'Kesiapan Periode', roles: adminRole },
				{ href: '/pengaturan', label: 'Periode', roles: ['admin_sistem'] }
			]
		}
	];

	function punyaAkses(roles: string[]) {
		return roles.some((role) => (session.user?.roles ?? []).includes(role as never));
	}

	/** Pimpinan murni (tanpa admin) tidak melihat Master Data. */
	const pimpinanSaja = $derived(punyaAkses(pimpinanRole) && !punyaAkses(adminRole));

	const menuTampil = $derived(menu.filter((item) => punyaAkses(item.roles)));

	const grupTampil = $derived(
		grupMenu
			.filter((grup) => !(grup.label === 'Master Data' && pimpinanSaja))
			.map((grup) => ({
				...grup,
				items: grup.items
					.filter((item) => punyaAkses(item.roles))
					.filter(
						(item) =>
							!(pimpinanSaja && item.href.includes('/absensi/rekap?tab=verifikasi'))
					)
			}))
			.filter((grup) => grup.items.length > 0)
	);

	let grupTerbuka = $state<Record<string, boolean>>({});

	function tabBawaan(jalur: string) {
		if (jalur === '/absensi/rekap') return 'rekap';
		if (jalur === '/ph') return 'jadwal';
		return null;
	}

	function aktif(href: string) {
		const [jalur, kueri] = href.split('?');
		if (page.url.pathname !== jalur && !page.url.pathname.startsWith(jalur + '/')) return false;
		if (!kueri) return true;

		const harap = new URLSearchParams(kueri);
		for (const [kunci, nilai] of harap) {
			const aktual =
				kunci === 'tab'
					? (page.url.searchParams.get('tab') ?? tabBawaan(jalur))
					: page.url.searchParams.get(kunci);
			if (aktual !== nilai) return false;
		}
		return true;
	}

	$effect(() => {
		for (const grup of grupTampil) {
			if (grupSedangAktif(grup)) grupTerbuka[grup.label] = true;
		}
	});

	function toggleGrup(label: string) {
		grupTerbuka[label] = !grupTerbuka[label];
	}

	function grupSedangAktif(grup: Grup) {
		if (grup.items.some((item) => aktif(item.href))) return true;
		return grup.basis
			? page.url.pathname === grup.basis || page.url.pathname.startsWith(grup.basis + '/')
			: false;
	}

	const judulHalaman = $derived.by(() => {
		const semua = [...menuTampil, ...grupTampil.flatMap((grup) => grup.items)];
		const cocok = semua.find((item) => aktif(item.href))?.label;
		if (cocok) return cocok;
		if (page.url.pathname.startsWith('/profil')) return 'Profil Saya';
		if (page.url.pathname.startsWith('/ph')) return 'Strong Point';
		return 'Penilaian Kinerja';
	});

	/** Konteks satker: admin global melihat semua satker, bukan satker asalnya. */
	const konteksSatker = $derived.by(() => {
		const user = session.user;
		if (!user) return '';
		if (user.roles.includes('admin_sistem')) return 'Semua Satker';
		for (const level of ['polda', 'polres', 'satker'] as const) {
			const scope = user.role_scopes.find(
				(item) => item.scope_level === level && item.scope_satker_nama
			);
			if (scope?.scope_satker_nama) return scope.scope_satker_nama;
		}
		return user.satker_nama;
	});

	onMount(async () => {
		const masuk = session.user ? true : await pulihkanSesi();
		if (!masuk) {
			goto('/login', { replaceState: true });
			return;
		}
		if (!bolehMasukDashboard()) {
			await logout().catch(() => {});
			goto('/login', { replaceState: true });
			return;
		}
		const jalurMaster = ['/satker', '/personel', '/libur', '/absensi/titik-dinas'];
		if (
			pimpinanSaja &&
			jalurMaster.some(
				(jalur) =>
					page.url.pathname === jalur || page.url.pathname.startsWith(jalur + '/')
			)
		) {
			await goto('/dashboard', { replaceState: true });
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
			class="fixed inset-y-0 left-0 z-40 flex w-64 -translate-x-full flex-col bg-navy-800 text-navy-100 transition-transform lg:sticky lg:top-0 lg:h-screen lg:translate-x-0 {menuTerbuka
				? 'translate-x-0'
				: ''}"
		>
			<div class="flex h-16 shrink-0 items-center gap-3 border-b border-white/10 px-5">
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

			<nav class="min-h-0 flex-1 space-y-1 overflow-y-auto px-3 py-4">
				{#each menuTampil as item (item.href)}
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

				{#each grupTampil as grup (grup.label)}
					{@const terbuka = grupTerbuka[grup.label] ?? false}
					{@const grupAktif = grupSedangAktif(grup)}
					<div class="pt-1">
						<button
							type="button"
							class="flex w-full items-center justify-between gap-2 rounded-lg px-3 py-2 text-sm transition-colors {grupAktif
								? 'text-gold-400'
								: 'text-navy-100'} hover:bg-white/10"
							onclick={() => toggleGrup(grup.label)}
							aria-expanded={terbuka}
						>
							<span class="flex min-w-0 items-center gap-3">
								<Icon name={grup.icon} size={17} />
								<span class="truncate">{grup.label}</span>
							</span>
							<Icon
								name="right"
								size={14}
								class="shrink-0 transition-transform {terbuka ? 'rotate-90' : ''}"
							/>
						</button>

						{#if terbuka}
							<div class="mt-1 ml-5 space-y-0.5 border-l border-white/10 pl-3">
								{#each grup.items as sub (sub.href)}
									<a
										href={sub.href}
										class="flex items-center rounded-lg px-3 py-1.5 text-[13px] transition-colors {aktif(
											sub.href
										)
											? 'bg-gold-500 font-medium text-navy-900'
											: 'text-navy-200 hover:bg-white/10 hover:text-white'}"
										onclick={() => (menuTerbuka = false)}
									>
										<span class="truncate">{sub.label}</span>
									</a>
								{/each}
							</div>
						{/if}
					</div>
				{/each}
			</nav>

			<div class="shrink-0 space-y-1 border-t border-white/10 p-3">
				<a
					href="/profil"
					title="Profil Saya"
					class="flex items-center gap-3 rounded-lg px-2 py-2 transition-colors {page.url
						.pathname === '/profil'
						? 'bg-white/10'
						: 'hover:bg-white/10'}"
					onclick={() => (menuTerbuka = false)}
				>
					<Avatar pengguna={session.user} size={32} class="ring-2 ring-white/20" />
					<span class="min-w-0 flex-1">
						<span class="block truncate text-sm font-medium text-white">
							{session.user?.nama ?? '-'}
						</span>
						<span class="block truncate text-[11px] text-navy-300">Profil Saya</span>
					</span>
					<Icon name="right" size={14} class="shrink-0 text-navy-300" />
				</a>
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
						<p class="text-[11px] text-slate-500">{konteksSatker}</p>
					</div>
				</div>

				<div class="flex items-center gap-3">
					<a href="/profil" class="hidden text-right hover:underline sm:block">
						<p class="text-sm font-medium text-slate-800">{session.user?.nama}</p>
						<p class="text-[11px] text-slate-500">
							{session.user?.pangkat} · {session.user?.jenis_akun === 'admin'
								? `@${session.user?.username ?? '-'}`
								: `NRP ${session.user?.nrp ?? '-'}`}
						</p>
					</a>
					<a href="/profil" title="Profil Saya" class="rounded-full hover:opacity-90">
						<Avatar pengguna={session.user} size={36} class="ring-2 ring-slate-200" />
					</a>
				</div>
			</header>

			<main class="flex-1 px-4 py-5 lg:px-6 lg:py-6">
				{@render children?.()}
			</main>
		</div>
	</div>
{/if}
