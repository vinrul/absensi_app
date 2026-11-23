<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import Icon from '#lib/components/Icon.svelte';
	import Avatar from '#lib/components/Avatar.svelte';
	import JamServer from '#lib/components/JamServer.svelte';
	import { hapusSesiLokal, pulihkanSesi, session } from '#lib/stores/session.svelte';

	let { children } = $props();

	let siap = $state(false);
	let izinSelesai = $state<boolean | null>(null);
	let langkahIzin = $state(0);
	let izinLokasi = $state(false);
	let izinNotif = $state(false);

	const KUNCI_IZIN = 'personel.izin_selesai';

	const tab = [
		{ href: '/beranda', label: 'Beranda', icon: 'home' },
		{ href: '/absensi', label: 'Absensi', icon: 'pin' },
		{ href: '/strong', label: 'Strong', icon: 'calendar' }
	];

	const halamanIzin = [
		{
			judul: 'Selamat datang di Absensi & Strong Point',
			deskripsi:
				'Sebelum masuk, aplikasi perlu menjelaskan izin yang digunakan. Izin hanya diminta sesuai kebutuhan dan tidak dipakai untuk melacak Anda di luar absensi.',
			alasan:
				'Penjelasan ini muncul sekali setelah aplikasi dipasang. Izin dapat diubah kapan saja melalui pengaturan browser.',
			ikon: 'checkCircle',
			iminta: null as 'lokasi' | 'notifikasi' | null,
			label: 'Mulai'
		},
		{
			judul: 'Akses Lokasi',
			deskripsi:
				'Lokasi dipakai saat Anda menekan tombol absen untuk menghitung jarak ke titik dinas dan memvalidasi radius.',
			alasan:
				'Aplikasi tidak melacak lokasi di latar belakang. Lokasi diambil hanya ketika Anda membuka halaman absensi atau menekan Perbarui Lokasi.',
			ikon: 'pin',
			iminta: 'lokasi' as const,
			label: 'Izinkan Lokasi'
		},
		{
			judul: 'Kamera',
			deskripsi:
				'Kamera dipakai untuk mengambil foto bukti kehadiran dan dokumentasi strong point. Foto bersifat opsional dan dikirim hanya saat Anda menyimpan data.',
			alasan:
				'Aplikasi hanya membuka kamera ketika Anda menekan tombol Ambil Foto. Tidak ada pengambilan foto diam-diam.',
			ikon: 'camera',
			iminta: null,
			label: 'Mengerti'
		},
		{
			judul: 'Notifikasi',
			deskripsi:
				'Notifikasi membantu mengingatkan jadwal strong point dan antrean absensi yang belum terkirim.',
			alasan:
				'Jika izin notifikasi tidak diberikan, pengingat tetap tampil di dalam aplikasi tetapi tidak muncul sebagai notifikasi browser.',
			ikon: 'info',
			iminta: 'notifikasi' as const,
			label: 'Aktifkan Notifikasi'
		},
		{
			judul: 'Penyimpanan Offline',
			deskripsi:
				'Antrean absensi dan strong point disimpan di penyimpanan browser bila tidak ada sinyal, lalu dikirim saat Anda menekan Kirim Antrean.',
			alasan:
				'Data antrean tidak diunggah ke pihak lain. Antrean tetap tersimpan sampai berhasil dikirim atau Anda keluar dari akun.',
			ikon: 'shield',
			iminta: null,
			label: 'Mengerti'
		},
		{
			judul: 'Siap Digunakan',
			deskripsi:
				'Semua penjelasan selesai. Anda dapat mengubah izin kapan saja melalui pengaturan browser.',
			alasan:
				'Jika izin lokasi belum diberikan, absensi berbasis radius tidak dapat dilakukan sampai izin diaktifkan.',
			ikon: 'checkCircle',
			iminta: null,
			label: 'Selesai'
		}
	];

	const aktif = $derived(halamanIzin[Math.min(langkahIzin, halamanIzin.length - 1)]);

	function periksaIzin() {
		if (typeof navigator !== 'undefined' && navigator.permissions) {
			void navigator.permissions
				.query({ name: 'geolocation' })
				.then((hasil) => (izinLokasi = hasil.state === 'granted'))
				.catch(() => {});
		}
		if (typeof Notification !== 'undefined') izinNotif = Notification.permission === 'granted';
	}

	function selesaikanIzin() {
		try {
			localStorage.setItem(KUNCI_IZIN, '1');
		} catch {
			// abaikan
		}
		izinSelesai = true;
	}

	async function mintaLokasi() {
		if (typeof navigator === 'undefined' || !navigator.geolocation) {
			selesaikanLangkah();
			return;
		}
		navigator.geolocation.getCurrentPosition(
			() => {
				izinLokasi = true;
				selesaikanLangkah();
			},
			() => selesaikanLangkah(),
			{ enableHighAccuracy: true, timeout: 20_000, maximumAge: 0 }
		);
	}

	async function mintaNotifikasi() {
		if (typeof Notification === 'undefined') {
			selesaikanLangkah();
			return;
		}
		try {
			const hasil = await Notification.requestPermission();
			izinNotif = hasil === 'granted';
		} catch {
			// abaikan
		}
		selesaikanLangkah();
	}

	function selesaikanLangkah() {
		if (langkahIzin >= halamanIzin.length - 1) selesaikanIzin();
		else langkahIzin += 1;
	}

	function tombolIzin() {
		if (aktif.iminta === 'lokasi') void mintaLokasi();
		else if (aktif.iminta === 'notifikasi') void mintaNotifikasi();
		else selesaikanLangkah();
	}

	function judulHalaman(): string {
		const jalur = page.url.pathname;
		if (jalur.startsWith('/absensi/riwayat')) return 'Riwayat Absensi';
		if (jalur.startsWith('/absensi')) return 'Absensi';
		if (jalur.startsWith('/strong/riwayat')) return 'Riwayat Strong Point';
		if (jalur.startsWith('/strong/wizard')) return 'Strong Point';
		if (jalur.startsWith('/strong')) return 'Strong Point';
		if (jalur.startsWith('/rencana/persetujuan')) return 'Persetujuan Rencana';
		if (jalur.startsWith('/rencana')) return 'Rencana Kerja';
		if (jalur.startsWith('/hasil/verifikasi')) return 'Verifikasi Hasil';
		if (jalur.startsWith('/hasil')) return 'Hasil Kerja';
		if (jalur.startsWith('/output/verifikasi')) return 'Verifikasi Output';
		if (jalur.startsWith('/output')) return 'Output';
		if (jalur.startsWith('/profil')) return 'Profil & Pengaturan';
		return 'Absensi Kinerja';
	}

	function tabAktif(href: string): boolean {
		const jalur = page.url.pathname;
		if (href === '/beranda') return jalur === '/beranda';
		return jalur === href || jalur.startsWith(`${href}/`);
	}

	onMount(() => {
		const tanganiSesi = () => {
			hapusSesiLokal();
			void goto('/login', { replaceState: true });
		};
		window.addEventListener('personel:sesi-berakhir', tanganiSesi);

		void (async () => {
			const masuk = await pulihkanSesi();
			if (!masuk) {
				await goto('/login', { replaceState: true });
				return;
			}
			try {
				izinSelesai = localStorage.getItem(KUNCI_IZIN) === '1';
			} catch {
				izinSelesai = true;
			}
			periksaIzin();
			siap = true;
		})();

		return () => window.removeEventListener('personel:sesi-berakhir', tanganiSesi);
	});
</script>

<svelte:head><title>{judulHalaman()} — Absensi Kinerja</title></svelte:head>

{#if !siap}
	<div class="flex min-h-dvh items-center justify-center">
		<div class="flex flex-col items-center gap-3 text-ink-500">
			<span class="size-8 animate-spin rounded-full border-2 border-brand-200 border-t-brand-600"></span>
			<p class="text-xs font-medium">Memuat sesi…</p>
		</div>
	</div>
{:else if izinSelesai === false}
	<div class="relative flex min-h-dvh flex-col overflow-hidden bg-gradient-to-br from-brand-900 via-brand-800 to-brand-600">
		<div class="pointer-events-none absolute -top-24 -right-20 size-72 rounded-full bg-white/10 blur-3xl"></div>
		<div class="mx-auto flex w-full max-w-md flex-1 flex-col px-5 py-8">
			<div class="flex flex-col items-center text-center text-white">
				<div class="rounded-2xl bg-white p-2 shadow-lg">
					<img src="/logo-polda-bali.png" alt="Logo SIAP Bali" class="h-14 w-auto" />
				</div>
				<h1 class="mt-4 text-xl font-bold">Izin Aplikasi</h1>
				<p class="text-sm text-brand-100">Penjelasan sebelum Anda masuk</p>
			</div>

			<div class="kartu mt-6 space-y-4 p-5">
				<div class="flex size-12 items-center justify-center rounded-2xl bg-brand-50 text-brand-700">
					<Icon name={aktif.ikon} size={26} />
				</div>
				<div>
					<h2 class="text-lg font-bold text-ink-900">{aktif.judul}</h2>
					<p class="mt-1 text-sm leading-relaxed text-ink-600">{aktif.deskripsi}</p>
				</div>

				<div class="rounded-xl bg-brand-50 p-3 text-sm text-brand-900">
					<p class="text-[11px] font-bold uppercase tracking-wider text-brand-700">
						Mengapa izin ini dipakai?
					</p>
					<p class="mt-1 text-xs leading-relaxed text-brand-900/80">{aktif.alasan}</p>
				</div>

				{#if aktif.iminta === 'lokasi'}
					<p class="text-xs font-semibold {izinLokasi ? 'text-emerald-600' : 'text-amber-600'}">
						{izinLokasi ? 'Izin lokasi sudah diberikan.' : 'Izin lokasi belum diberikan.'}
					</p>
				{:else if aktif.iminta === 'notifikasi'}
					<p class="text-xs font-semibold {izinNotif ? 'text-emerald-600' : 'text-amber-600'}">
						{izinNotif ? 'Notifikasi sudah diaktifkan.' : 'Notifikasi belum diaktifkan.'}
					</p>
				{/if}

				<button class="btn btn-utama w-full py-3" type="button" onclick={tombolIzin}>
					{aktif.iminta === 'lokasi' && !izinLokasi
						? 'Izinkan Lokasi'
						: aktif.iminta === 'notifikasi' && !izinNotif
							? 'Aktifkan Notifikasi'
							: langkahIzin >= halamanIzin.length - 1
								? 'Selesai'
								: 'Lanjut'}
				</button>

				{#if aktif.iminta}
					<button class="btn btn-hantu w-full" type="button" onclick={selesaikanLangkah}>
						Lewati dulu
					</button>
				{/if}
			</div>

			<p class="mt-auto pt-6 text-center text-xs text-brand-100">
				Langkah {langkahIzin + 1} dari {halamanIzin.length}
			</p>
		</div>
	</div>
{:else}
	<div class="mx-auto flex min-h-dvh w-full max-w-xl flex-col">
		<header
			class="sticky top-0 z-30 flex h-14 items-center justify-between gap-3 border-b border-white/60 bg-ink-50/85 px-4 backdrop-blur-md"
		>
			<div class="flex min-w-0 items-center gap-2.5">
				<img src="/logo-polda-bali.png" alt="Logo SIAP Bali" class="h-8 w-auto" />
				<div class="min-w-0">
					<p class="truncate text-sm font-bold text-ink-900">{judulHalaman()}</p>
					<p class="truncate text-[10px] text-ink-400">{session.user?.satker_nama}</p>
				</div>
			</div>
			<div class="flex shrink-0 items-center gap-2">
				<JamServer />
				<a href="/profil" aria-label="Profil" class="rounded-full ring-2 ring-white shadow-sm">
					<Avatar pengguna={session.user} size={34} />
				</a>
			</div>
		</header>

		<main class="flex-1 px-4 pt-4 pb-aman">
			{@render children?.()}
		</main>

		<nav
			class="fixed inset-x-0 bottom-0 z-40 border-t border-ink-100 bg-white/95 px-2 pt-1.5 backdrop-blur-md"
			style="padding-bottom: max(0.45rem, env(safe-area-inset-bottom))"
			aria-label="Navigasi utama"
		>
			<div class="mx-auto grid max-w-xl grid-cols-6 gap-0.5">
				{#each tab as item (item.href)}
					<a href={item.href} class="nav-tab" data-aktif={tabAktif(item.href)}>
						<Icon name={item.icon} size={20} />
						<span class="truncate">{item.label}</span>
					</a>
				{/each}
			</div>
		</nav>
	</div>
{/if}
