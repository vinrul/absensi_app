<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import { login, pulihkanSesi, session } from '#lib/stores/session.svelte';

	let nrp = $state('');
	let password = $state('');
	let tampilPassword = $state(false);
	let memuat = $state(false);
	let galat = $state<string | null>(null);
	const berhasilUbah = $derived(page.url.searchParams.get('ubah') === 'password');

	onMount(async () => {
		if (session.user) {
			await goto('/beranda', { replaceState: true });
			return;
		}
		if (await pulihkanSesi()) await goto('/beranda', { replaceState: true });
	});

	async function masuk(event: SubmitEvent) {
		event.preventDefault();
		if (!nrp.trim() || !password) return;
		memuat = true;
		galat = null;
		try {
			const user = await login(nrp, password);
			await goto(user.must_change_password ? '/profil?wajib=1' : '/beranda', {
				replaceState: true
			});
		} catch (err) {
			galat =
				err instanceof ApiError
					? err.message
					: 'Tidak dapat masuk. Periksa NRP dan password Anda.';
		} finally {
			memuat = false;
		}
	}
</script>

<svelte:head><title>Masuk — Absensi Kinerja</title></svelte:head>

<div class="relative flex min-h-dvh flex-col overflow-hidden">
	<div
		class="pointer-events-none absolute inset-x-0 top-0 h-[46vh] rounded-b-[3rem] bg-gradient-to-br from-brand-800 via-brand-700 to-brand-500"
	></div>
	<div class="pointer-events-none absolute -top-24 -right-16 size-72 rounded-full bg-white/10 blur-2xl"></div>
	<div class="pointer-events-none absolute top-32 -left-20 size-56 rounded-full bg-sun-400/20 blur-2xl"></div>

	<main class="relative z-10 mx-auto flex w-full max-w-md flex-1 flex-col px-5 py-8">
		<div class="flex flex-col items-center text-center text-white">
			<div class="rounded-2xl bg-white p-2 shadow-lg">
				<img src="/logo-polda-bali.png" alt="Logo Polda Bali" class="h-16 w-auto" />
			</div>
			<h1 class="mt-4 text-2xl font-bold tracking-tight">Absensi Kinerja</h1>
			<p class="mt-1 text-sm text-brand-100">Personel Polda Bali dan Polres jajaran</p>
			<span
				class="mt-3 rounded-full bg-white/15 px-3 py-1 text-[10px] font-bold uppercase tracking-widest text-brand-50 ring-1 ring-white/20"
			>
				Personel &amp; Atasan Langsung
			</span>
		</div>

		<form class="kartu mt-8 space-y-4 p-5" onsubmit={masuk}>
			<div>
				<h2 class="text-lg font-bold text-ink-900">Masuk</h2>
				<p class="text-xs text-ink-500">Gunakan NRP dan password dari admin satker.</p>
			</div>

			{#if berhasilUbah}
				<Pesan
					jenis="sukses"
					pesan="Password berhasil diubah. Silakan masuk dengan password baru Anda."
				/>
			{/if}
			{#if galat}
				<Pesan jenis="galat" pesan={galat} />
			{/if}

			<div>
				<label class="label" for="nrp">NRP</label>
				<div class="relative">
					<span class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-ink-400">
						<Icon name="user" size={17} />
					</span>
					<input
						id="nrp"
						class="input pl-10"
						inputmode="numeric"
						autocomplete="username"
						placeholder="Contoh: 85000001"
						bind:value={nrp}
						oninput={(e) => (nrp = (e.currentTarget as HTMLInputElement).value.replace(/\D/g, ''))}
						required
					/>
				</div>
			</div>

			<div>
				<label class="label" for="password">Password</label>
				<div class="relative">
					<span class="pointer-events-none absolute top-1/2 left-3 -translate-y-1/2 text-ink-400">
						<Icon name="lock" size={17} />
					</span>
					<input
						id="password"
						class="input pr-11 pl-10"
						type={tampilPassword ? 'text' : 'password'}
						autocomplete="current-password"
						placeholder="Password"
						bind:value={password}
						required
					/>
					<button
						type="button"
						class="absolute top-1/2 right-2 -translate-y-1/2 rounded-lg p-1.5 text-ink-400 hover:text-ink-700"
						aria-label={tampilPassword ? 'Sembunyikan password' : 'Tampilkan password'}
						onclick={() => (tampilPassword = !tampilPassword)}
					>
						<Icon name={tampilPassword ? 'eyeOff' : 'eye'} size={17} />
					</button>
				</div>
			</div>

			<button class="btn btn-utama w-full py-3" type="submit" disabled={memuat || !nrp || !password}>
				{#if memuat}
					<span class="size-4 animate-spin rounded-full border-2 border-white/40 border-t-white"></span>
					Memproses…
				{:else}
					Masuk
					<Icon name="right" size={16} />
				{/if}
			</button>

			<p class="text-center text-[11px] leading-relaxed text-ink-400">
				Akun admin dan dashboard pimpinan memakai aplikasi dashboard terpisah.
			</p>
		</form>

		<p class="mt-auto pt-6 text-center text-[11px] leading-relaxed text-ink-400">
			Data personel bersifat rahasia. Gunakan perangkat dinas dan jangan bagikan password.
		</p>
	</main>
</div>
