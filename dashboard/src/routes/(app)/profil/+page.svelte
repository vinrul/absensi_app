<script lang="ts">
	import { onDestroy } from 'svelte';
	import { goto } from '$app/navigation';
	import { api, ApiError } from '#lib/api/client';
	import { logout, session, type Pengguna } from '#lib/stores/session.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Avatar from '#lib/components/Avatar.svelte';

	let versiAvatar = $state(0);
	let berkasFoto = $state<File | null>(null);
	let pratinjau = $state<string | null>(null);
	let inputFoto = $state<HTMLInputElement | null>(null);
	let mengunggah = $state(false);
	let infoFoto = $state<string | null>(null);
	let errorFoto = $state<string | null>(null);

	let lama = $state('');
	let baru = $state('');
	let ulang = $state('');
	let tampilLama = $state(false);
	let tampilBaru = $state(false);
	let tampilUlang = $state(false);
	let menyimpan = $state(false);
	let infoPassword = $state<string | null>(null);
	let errorPassword = $state<string | null>(null);

	const pengguna = $derived(session.user);

	const LABEL_PERAN: Record<string, string> = {
		admin_sistem: 'Admin Sistem',
		admin_polda: 'Admin Polda',
		admin_satker_polda: 'Admin Satker Polda',
		admin_polres: 'Admin Polres',
		admin_satker_polres: 'Admin Satker Polres',
		pimpinan: 'Pimpinan',
		pimpinan_polda: 'Pimpinan Polda',
		pimpinan_satker_polda: 'Pimpinan Satker Polda',
		pimpinan_polres: 'Pimpinan Polres',
		pimpinan_satker_polres: 'Pimpinan Satker Polres',
		atasan: 'Atasan Langsung',
		personel: 'Personel'
	};

	function labelPeran(peran: string) {
		return LABEL_PERAN[peran] ?? peran;
	}

	function pilihBerkas(event: Event) {
		errorFoto = null;
		infoFoto = null;
		const input = event.currentTarget as HTMLInputElement;
		const berkas = input.files?.[0] ?? null;
		if (pratinjau) URL.revokeObjectURL(pratinjau);
		pratinjau = null;
		berkasFoto = null;

		if (!berkas) return;
		if (!['image/jpeg', 'image/png'].includes(berkas.type)) {
			errorFoto = 'Foto harus berformat JPG atau PNG.';
			input.value = '';
			return;
		}
		if (berkas.size > 5 * 1024 * 1024) {
			errorFoto = 'Ukuran foto maksimal 5 MB.';
			input.value = '';
			return;
		}
		berkasFoto = berkas;
		pratinjau = URL.createObjectURL(berkas);
	}

	async function unggahFoto() {
		if (!berkasFoto) return;
		mengunggah = true;
		errorFoto = null;
		infoFoto = null;
		try {
			const form = new FormData();
			form.append('file', berkasFoto);
			const hasil = await api<Pengguna>('/auth/foto', { method: 'POST', body: form });
			session.user = hasil;
			infoFoto = 'Foto profil berhasil diperbarui.';
			berkasFoto = null;
			if (pratinjau) URL.revokeObjectURL(pratinjau);
			pratinjau = null;
			if (inputFoto) inputFoto.value = '';
			versiAvatar += 1;
		} catch (err) {
			errorFoto = err instanceof ApiError ? err.message : 'Gagal mengunggah foto profil';
		} finally {
			mengunggah = false;
		}
	}

	async function ubahPassword(event: SubmitEvent) {
		event.preventDefault();
		errorPassword = null;
		infoPassword = null;

		if (baru.length < 8) {
			errorPassword = 'Password baru minimal 8 karakter.';
			return;
		}
		if (!/[A-Za-z]/.test(baru) || !/\d/.test(baru)) {
			errorPassword = 'Password baru harus mengandung huruf dan angka.';
			return;
		}
		if (baru !== ulang) {
			errorPassword = 'Konfirmasi password baru tidak sama.';
			return;
		}
		if (baru === lama) {
			errorPassword = 'Password baru harus berbeda dari password lama.';
			return;
		}

		menyimpan = true;
		try {
			await api('/auth/change-password', {
				method: 'POST',
				body: { old_password: lama, new_password: baru }
			});
			infoPassword = 'Password berhasil diubah. Silakan login kembali.';
			await logout().catch(() => {});
			await goto('/login?ubah=password', { replaceState: true });
		} catch (err) {
			errorPassword = err instanceof ApiError ? err.message : 'Gagal mengubah password';
		} finally {
			menyimpan = false;
		}
	}

	onDestroy(() => {
		if (pratinjau) URL.revokeObjectURL(pratinjau);
	});
</script>

<svelte:head><title>Profil Saya — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card p-5">
		<div class="flex flex-wrap items-center gap-5">
			<div class="shrink-0">
				<Avatar pengguna={pengguna} size={96} versi={versiAvatar} class="ring-4 ring-white shadow" />
			</div>
			<div class="min-w-0 flex-1">
				<h2 class="text-lg font-semibold text-slate-800">{pengguna?.nama ?? '-'}</h2>
				<p class="text-sm text-slate-500">
					{pengguna?.pangkat ?? '-'} · {pengguna?.jabatan ?? '-'}
				</p>
				<p class="text-xs text-slate-500">
					{pengguna?.nrp ? `NRP ${pengguna.nrp}` : `Username ${pengguna?.username ?? '-'}`} ·
					{pengguna?.satker_nama ?? '-'}
				</p>
				<div class="mt-2 flex flex-wrap gap-1">
					{#each pengguna?.roles ?? [] as peran}
						<span class="rounded-full bg-slate-100 px-2 py-0.5 text-[11px] text-slate-600">
							{labelPeran(peran)}
						</span>
					{/each}
				</div>
			</div>
		</div>

		{#if pengguna?.must_change_password}
			<p class="mt-4 rounded-lg bg-amber-50 px-3 py-2 text-sm text-amber-800 ring-1 ring-amber-200">
				Anda diwajibkan mengganti password.
			</p>
		{/if}
	</div>

	<div class="grid gap-4 lg:grid-cols-2">
		<div class="card space-y-3 p-5">
			<div>
				<h3 class="text-sm font-semibold text-slate-800">Foto Profil</h3>
				<p class="text-xs text-slate-500">Format JPG atau PNG, maksimal 5 MB.</p>
			</div>

			{#if infoFoto}
				<p class="rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">
					{infoFoto}
				</p>
			{/if}
			{#if errorFoto}
				<p class="rounded-lg bg-rose-50 px-3 py-2 text-sm text-rose-700 ring-1 ring-rose-200">
					{errorFoto}
				</p>
			{/if}

			{#if pratinjau}
				<img
					src={pratinjau}
					alt="Pratinjau foto profil"
					class="h-48 w-full rounded-lg object-contain ring-1 ring-slate-200"
				/>
			{/if}

			<input
				bind:this={inputFoto}
				type="file"
				accept="image/jpeg,image/png"
				class="block w-full text-sm text-slate-600 file:mr-3 file:rounded-lg file:border-0 file:bg-navy-800 file:px-3 file:py-2 file:text-sm file:font-medium file:text-white hover:file:bg-navy-700"
				onchange={pilihBerkas}
			/>

			<button class="btn btn-primary" disabled={!berkasFoto || mengunggah} onclick={unggahFoto}>
				<Icon name="upload" size={16} />
				{mengunggah ? 'Mengunggah…' : 'Unggah Foto'}
			</button>
		</div>

		<div class="card space-y-3 p-5">
			<div>
				<h3 class="text-sm font-semibold text-slate-800">Ganti Password</h3>
				<p class="text-xs text-slate-500">
					Minimal 8 karakter dan mengandung huruf serta angka. Setelah berhasil, Anda akan
					diminta login kembali.
				</p>
			</div>

			{#if infoPassword}
				<p class="rounded-lg bg-emerald-50 px-3 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">
					{infoPassword}
				</p>
			{/if}
			{#if errorPassword}
				<p class="rounded-lg bg-rose-50 px-3 py-2 text-sm text-rose-700 ring-1 ring-rose-200">
					{errorPassword}
				</p>
			{/if}

			<form class="space-y-3" onsubmit={ubahPassword}>
				<div>
					<label class="label" for="pass-lama">Password Lama</label>
					<div class="relative">
						<input
							id="pass-lama"
							class="input pr-10"
							type={tampilLama ? 'text' : 'password'}
							autocomplete="current-password"
							bind:value={lama}
							required
						/>
						<button
							type="button"
							class="absolute top-1/2 right-2 -translate-y-1/2 rounded p-1 text-slate-400 hover:text-slate-600"
							onclick={() => (tampilLama = !tampilLama)}
							aria-label={tampilLama ? 'Sembunyikan password lama' : 'Tampilkan password lama'}
						>
							<Icon name={tampilLama ? 'eyeOff' : 'eye'} size={16} />
						</button>
					</div>
				</div>

				<div>
					<label class="label" for="pass-baru">Password Baru</label>
					<div class="relative">
						<input
							id="pass-baru"
							class="input pr-10"
							type={tampilBaru ? 'text' : 'password'}
							autocomplete="new-password"
							bind:value={baru}
							required
						/>
						<button
							type="button"
							class="absolute top-1/2 right-2 -translate-y-1/2 rounded p-1 text-slate-400 hover:text-slate-600"
							onclick={() => (tampilBaru = !tampilBaru)}
							aria-label={tampilBaru ? 'Sembunyikan password baru' : 'Tampilkan password baru'}
						>
							<Icon name={tampilBaru ? 'eyeOff' : 'eye'} size={16} />
						</button>
					</div>
				</div>

				<div>
					<label class="label" for="pass-ulang">Ulangi Password Baru</label>
					<div class="relative">
						<input
							id="pass-ulang"
							class="input pr-10"
							type={tampilUlang ? 'text' : 'password'}
							autocomplete="new-password"
							bind:value={ulang}
							required
						/>
						<button
							type="button"
							class="absolute top-1/2 right-2 -translate-y-1/2 rounded p-1 text-slate-400 hover:text-slate-600"
							onclick={() => (tampilUlang = !tampilUlang)}
							aria-label={tampilUlang ? 'Sembunyikan konfirmasi password' : 'Tampilkan konfirmasi password'}
						>
							<Icon name={tampilUlang ? 'eyeOff' : 'eye'} size={16} />
						</button>
					</div>
				</div>

				<button class="btn btn-primary w-full" type="submit" disabled={menyimpan}>
					<Icon name="check" size={16} />
					{menyimpan ? 'Menyimpan…' : 'Simpan Password Baru'}
				</button>
			</form>
		</div>
	</div>
</div>
