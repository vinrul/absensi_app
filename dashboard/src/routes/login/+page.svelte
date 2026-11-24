<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { bolehMasukDashboard, login, logout, pulihkanSesi } from '#lib/stores/session.svelte';
	import { ApiError } from '#lib/api/client';
	import Icon from '#lib/components/Icon.svelte';

	let nrp = $state('');
	let password = $state('');
	let memuat = $state(false);
	let error = $state<string | null>(null);
	let tampilkanPassword = $state(false);

	onMount(async () => {
		if (await pulihkanSesi()) {
			if (bolehMasukDashboard()) {
				goto('/dashboard', { replaceState: true });
			} else {
				await logout().catch(() => {});
			}
		}
	});

	async function kirim(event: SubmitEvent) {
		event.preventDefault();
		error = null;
		memuat = true;
		try {
			const user = await login(nrp.trim(), password);
			if (!bolehMasukDashboard(user)) {
				await logout().catch(() => {});
				error =
					'Dashboard hanya untuk akun pimpinan dan admin. Personel dan atasan langsung menggunakan aplikasi Android.';
				return;
			}
			await goto('/dashboard', { replaceState: true });
		} catch (err) {
			error =
				err instanceof ApiError
					? err.message
					: 'Tidak dapat terhubung ke server. Periksa koneksi Anda.';
		} finally {
			memuat = false;
		}
	}
</script>

<svelte:head>
	<title>Masuk — SIAP Bali</title>
</svelte:head>

<div class="flex min-h-screen flex-col lg:flex-row">
	<!-- Panel identitas -->
	<div
		class="relative flex flex-col justify-between overflow-hidden bg-navy-800 px-8 py-10 text-white lg:w-1/2 lg:px-14 lg:py-14"
	>
		<div class="absolute inset-0 opacity-10">
			<div class="absolute -top-24 -right-24 size-96 rounded-full border-[24px] border-gold-500"></div>
			<div class="absolute -bottom-32 -left-20 size-96 rounded-full border-[24px] border-white"></div>
		</div>

		<div class="relative">
			<div class="flex items-center gap-3">
				<img src="/logo-polda-bali.png" alt="Logo SIAP Bali" class="h-20 w-auto" />
				<div>
					<p class="text-sm font-semibold tracking-widest text-gold-400 uppercase">Polda Bali</p>
					<p class="text-xs text-navy-200">Kepolisian Daerah Bali</p>
				</div>
			</div>

			<h1 class="mt-12 max-w-md text-3xl leading-tight font-bold lg:text-4xl">SIAP Bali</h1>
			<p class="mt-4 max-w-md text-sm leading-relaxed text-navy-100">
				Pencatatan kehadiran berbasis geolokasi, rencana kerja, hasil kerja, dan output yang
				terintegrasi menjadi nilai kinerja personel Polda Bali dan Polres jajaran.
			</p>

			<ul class="mt-8 grid max-w-md grid-cols-2 gap-3 text-xs text-navy-100">
				<li class="rounded-lg bg-white/5 px-3 py-2">Absensi GPS radius 100 m</li>
				<li class="rounded-lg bg-white/5 px-3 py-2">Bobot 20 / 20 / 30 / 30</li>
				<li class="rounded-lg bg-white/5 px-3 py-2">Verifikasi berjenjang</li>
				<li class="rounded-lg bg-white/5 px-3 py-2">Dashboard pimpinan</li>
			</ul>
		</div>

		<p class="relative mt-10 text-[11px] text-navy-200">
			Data personel bersifat rahasia. Akses hanya untuk personel yang berwenang.
		</p>
	</div>

	<!-- Form login -->
	<div class="flex flex-1 items-center justify-center px-6 py-12">
		<form class="w-full max-w-sm" onsubmit={kirim}>
			<h2 class="text-xl font-semibold text-slate-800">Masuk ke aplikasi</h2>
			<p class="mt-1 text-sm text-slate-500">
				Personel memakai NRP; admin memakai username dan password yang diberikan.
			</p>

			{#if error}
				<div
					class="mt-5 flex items-start gap-2 rounded-lg bg-rose-50 px-3 py-2 text-sm text-rose-700 ring-1 ring-rose-200"
				>
					<Icon name="alert" size={16} class="mt-0.5 shrink-0" />
					<span>{error}</span>
				</div>
			{/if}

			<div class="mt-6">
				<label class="label" for="nrp">Username / NRP</label>
				<input
					id="nrp"
					class="input"
					autocomplete="username"
					inputmode="numeric"
					placeholder="contoh: 85000001"
					bind:value={nrp}
					required
				/>
			</div>

			<div class="mt-4">
				<label class="label" for="password">Password</label>
				<div class="relative">
					<input
						id="password"
						class="input pr-10"
						type={tampilkanPassword ? 'text' : 'password'}
						autocomplete="current-password"
						placeholder="••••••••"
						bind:value={password}
						required
					/>
					<button
						type="button"
						class="absolute top-1/2 right-2 -translate-y-1/2 rounded p-1 text-slate-400 hover:text-slate-600"
						onclick={() => (tampilkanPassword = !tampilkanPassword)}
						aria-label="Tampilkan password"
					>
						<Icon name="eye" size={16} />
					</button>
				</div>
			</div>

			<button class="btn btn-primary mt-6 w-full" type="submit" disabled={memuat}>
				{memuat ? 'Memproses…' : 'Masuk'}
			</button>

			<p class="mt-6 text-center text-xs text-slate-400">
				Lupa password? Hubungi admin sistem satker Anda.
			</p>
		</form>
	</div>
</div>
