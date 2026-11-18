<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, ApiError, unggahFotoProfil } from '#lib/api/client';
	import Avatar from '#lib/components/Avatar.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Pesan from '#lib/components/Pesan.svelte';
	import Sheet from '#lib/components/Sheet.svelte';
	import { kosongkanAntrean } from '#lib/offline';
	import {
		hapusSesiLokal,
		logout,
		peranUtama,
		simpanPengguna,
		session
	} from '#lib/stores/session.svelte';
	import { PANGKAT_POLRI, type Pengguna } from '#lib/types';

	let versiAvatar = $state(0);
	let memproses = $state(false);
	let pesan = $state<string | null>(null);
	let galat = $state<string | null>(null);
	let infoFoto = $state<string | null>(null);
	let galatFoto = $state<string | null>(null);

	let bukaEdit = $state(false);
	let bukaPassword = $state(false);
	let nama = $state('');
	let pangkat = $state('');
	let jabatan = $state('');
	let phone = $state('');
	let email = $state('');

	let lama = $state('');
	let baru = $state('');
	let ulang = $state('');
	let tampilLama = $state(false);
	let tampilBaru = $state(false);
	let tampilUlang = $state(false);

	const pengguna = $derived(session.user);
	const wajib = $derived(page.url.searchParams.get('wajib') === '1');

	async function muatProfil() {
		try {
			const user = await api<Pengguna>('/auth/me');
			simpanPengguna(user);
		} catch {
			// Layout menangani sesi berakhir.
		}
	}

	function isiForm(user: Pengguna | null) {
		nama = user?.nama ?? '';
		pangkat = user?.pangkat ?? PANGKAT_POLRI[0];
		jabatan = user?.jabatan ?? '';
		phone = user?.phone ?? '';
		email = user?.email ?? '';
	}

	function bukaFormEdit() {
		isiForm(session.user);
		galat = null;
		bukaEdit = true;
	}

	async function pilihFoto(event: Event) {
		const input = event.currentTarget as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		galatFoto = null;
		infoFoto = null;
		if (!['image/jpeg', 'image/png'].includes(file.type)) {
			galatFoto = 'Foto harus berformat JPG atau PNG.';
			input.value = '';
			return;
		}
		if (file.size > 5 * 1024 * 1024) {
			galatFoto = 'Ukuran foto maksimal 5 MB.';
			input.value = '';
			return;
		}
		memproses = true;
		try {
			const hasil = (await unggahFotoProfil(file)) as Pengguna;
			simpanPengguna(hasil);
			versiAvatar += 1;
			infoFoto = 'Foto profil berhasil diperbarui.';
		} catch (err) {
			galatFoto = err instanceof ApiError ? err.message : 'Gagal mengunggah foto.';
		} finally {
			memproses = false;
			input.value = '';
		}
	}

	async function simpanProfil() {
		if (memproses) return;
		if (!nama.trim() || !jabatan.trim()) {
			galat = 'Nama dan jabatan wajib diisi.';
			return;
		}
		memproses = true;
		galat = null;
		try {
			const hasil = await api<Pengguna>('/auth/profile', {
				method: 'PATCH',
				body: {
					nama: nama.trim(),
					pangkat,
					jabatan: jabatan.trim(),
					phone: phone.trim() || null,
					email: email.trim() || null
				}
			});
			simpanPengguna(hasil);
			pesan = 'Profil berhasil diperbarui.';
			bukaEdit = false;
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Profil gagal disimpan.';
		} finally {
			memproses = false;
		}
	}

	async function ubahPassword() {
		if (memproses) return;
		galat = null;
		pesan = null;
		if (baru.length < 8) {
			galat = 'Password baru minimal 8 karakter.';
			return;
		}
		if (!/[A-Za-z]/.test(baru) || !/\d/.test(baru)) {
			galat = 'Password baru harus mengandung huruf dan angka.';
			return;
		}
		if (baru !== ulang) {
			galat = 'Konfirmasi password baru tidak sama.';
			return;
		}
		if (baru === lama) {
			galat = 'Password baru harus berbeda dari password lama.';
			return;
		}
		memproses = true;
		try {
			const user = session.user;
			await api('/auth/change-password', {
				method: 'POST',
				body: { old_password: lama, new_password: baru }
			});
			if (user) kosongkanAntrean(user.id);
			hapusSesiLokal();
			await goto('/login?ubah=password', { replaceState: true });
		} catch (err) {
			galat = err instanceof ApiError ? err.message : 'Gagal mengubah password.';
		} finally {
			memproses = false;
		}
	}

	async function keluar() {
		const user = session.user;
		if (user) kosongkanAntrean(user.id);
		await logout();
		await goto('/login', { replaceState: true });
	}

	onMount(() => {
		isiForm(session.user);
		void muatProfil();
	});
</script>

<svelte:head><title>Profil &amp; Pengaturan — Absensi Kinerja</title></svelte:head>

<div class="space-y-3">
	<a href="/beranda" class="btn btn-hantu -ml-2 px-2 py-1"><Icon name="left" size={16} /> Kembali</a>

	{#if pesan}<Pesan jenis="sukses" pesan={pesan} />{/if}
	{#if galat && !bukaEdit && !bukaPassword}<Pesan jenis="galat" pesan={galat} />{/if}
	{#if wajib || pengguna?.must_change_password}
		<Pesan jenis="info" pesan="Anda diwajibkan mengganti password. Gunakan tombol Ganti Password di bawah." />
	{/if}

	<section class="kartu p-5">
		<div class="flex items-center gap-4">
			<Avatar pengguna={pengguna} size={76} versi={versiAvatar} class="ring-4 ring-brand-50" />
			<div class="min-w-0 flex-1">
				<h1 class="truncate text-lg font-bold text-ink-900">{pengguna?.nama ?? '-'}</h1>
				<p class="text-xs text-ink-500">{pengguna?.pangkat} · {pengguna?.jabatan}</p>
				<p class="text-[11px] text-ink-400">
					{pengguna?.nrp ? `NRP ${pengguna.nrp}` : `@${pengguna?.username ?? '-'}`} ·
					{pengguna?.satker_nama}
				</p>
				<span class="chip mt-1 bg-brand-50 text-brand-700">{peranUtama()}</span>
			</div>
		</div>

		<div class="mt-3 grid grid-cols-2 gap-2">
			<label class="btn btn-kedua cursor-pointer {memproses ? 'pointer-events-none opacity-50' : ''}">
				<Icon name="camera" size={16} /> Ganti Foto
				<input type="file" accept="image/jpeg,image/png" class="sr-only" onchange={pilihFoto} />
			</label>
			<button class="btn btn-kedua" onclick={bukaFormEdit}>
				<Icon name="edit" size={16} /> Edit Profil
			</button>
		</div>

		{#if infoFoto}<Pesan jenis="sukses" pesan={infoFoto} class="mt-2" />{/if}
		{#if galatFoto}<Pesan jenis="galat" pesan={galatFoto} class="mt-2" />{/if}
	</section>

	<section class="kartu p-4">
		<p class="judul-bagian">Data Profil</p>
		<dl class="mt-1">
			<div class="baris-info"><dt>Nama</dt><dd>{pengguna?.nama ?? '-'}</dd></div>
			<div class="baris-info"><dt>Pangkat</dt><dd>{pengguna?.pangkat ?? '-'}</dd></div>
			<div class="baris-info"><dt>Jabatan</dt><dd>{pengguna?.jabatan ?? '-'}</dd></div>
			<div class="baris-info"><dt>NRP</dt><dd>{pengguna?.nrp ?? '-'}</dd></div>
			<div class="baris-info"><dt>Satker</dt><dd>{pengguna?.satker_nama ?? '-'}</dd></div>
			<div class="baris-info"><dt>Telepon</dt><dd>{pengguna?.phone ?? '-'}</dd></div>
			<div class="baris-info"><dt>Email</dt><dd>{pengguna?.email ?? '-'}</dd></div>
		</dl>
	</section>

	<section class="kartu space-y-2 p-4">
		<button class="btn btn-kedua w-full" onclick={() => { galat = null; bukaPassword = true; }}>
			<Icon name="lock" size={16} /> Ganti Password
		</button>
		<button class="btn btn-hantu w-full text-rose-600" onclick={keluar}>
			<Icon name="logout" size={16} /> Keluar
		</button>
	</section>

	<p class="px-1 text-center text-[10px] leading-relaxed text-ink-400">
		Sesi ini terhubung ke server absensi Polda Bali. Jangan bagikan password kepada siapa pun.
	</p>
</div>

{#if bukaEdit}
	<Sheet judul="Edit Profil" onTutup={() => (bukaEdit = false)}>
		<div class="space-y-3">
			{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}
			<div>
				<label class="label" for="p-nama">Nama</label>
				<input id="p-nama" class="input" bind:value={nama} />
			</div>
			<div>
				<label class="label" for="p-pangkat">Pangkat</label>
				<select id="p-pangkat" class="input" bind:value={pangkat}>
					{#each PANGKAT_POLRI as item (item)}
						<option value={item}>{item}</option>
					{/each}
				</select>
			</div>
			<div>
				<label class="label" for="p-jabatan">Jabatan</label>
				<input id="p-jabatan" class="input" bind:value={jabatan} />
			</div>
			<div>
				<label class="label" for="p-phone">Telepon</label>
				<input id="p-phone" class="input" inputmode="tel" bind:value={phone} placeholder="Opsional" />
			</div>
			<div>
				<label class="label" for="p-email">Email</label>
				<input id="p-email" type="email" class="input" bind:value={email} placeholder="Opsional" />
			</div>
		</div>

		{#snippet footer()}
			<div class="flex gap-2">
				<button class="btn btn-kedua flex-1" onclick={() => (bukaEdit = false)}>Batal</button>
				<button class="btn btn-utama flex-1" disabled={memproses} onclick={simpanProfil}>
					{memproses ? 'Menyimpan…' : 'Simpan'}
				</button>
			</div>
		{/snippet}
	</Sheet>
{/if}

{#if bukaPassword}
	<Sheet judul="Ganti Password" onTutup={() => (bukaPassword = false)}>
		<div class="space-y-3">
			{#if galat}<Pesan jenis="galat" pesan={galat} />{/if}
			<Pesan
				jenis="info"
				pesan="Minimal 8 karakter dan mengandung huruf serta angka. Setelah berhasil, semua sesi dicabut dan Anda harus login kembali."
			/>

			<div>
				<label class="label" for="pw-lama">Password Lama</label>
				<div class="relative">
					<input
						id="pw-lama"
						class="input pr-11"
						type={tampilLama ? 'text' : 'password'}
						autocomplete="current-password"
						bind:value={lama}
					/>
					<button
						type="button"
						class="absolute top-1/2 right-2 -translate-y-1/2 rounded-lg p-1.5 text-ink-400"
						aria-label={tampilLama ? 'Sembunyikan password lama' : 'Tampilkan password lama'}
						onclick={() => (tampilLama = !tampilLama)}
					>
						<Icon name={tampilLama ? 'eyeOff' : 'eye'} size={17} />
					</button>
				</div>
			</div>

			<div>
				<label class="label" for="pw-baru">Password Baru</label>
				<div class="relative">
					<input
						id="pw-baru"
						class="input pr-11"
						type={tampilBaru ? 'text' : 'password'}
						autocomplete="new-password"
						bind:value={baru}
					/>
					<button
						type="button"
						class="absolute top-1/2 right-2 -translate-y-1/2 rounded-lg p-1.5 text-ink-400"
						aria-label={tampilBaru ? 'Sembunyikan password baru' : 'Tampilkan password baru'}
						onclick={() => (tampilBaru = !tampilBaru)}
					>
						<Icon name={tampilBaru ? 'eyeOff' : 'eye'} size={17} />
					</button>
				</div>
			</div>

			<div>
				<label class="label" for="pw-ulang">Ulangi Password Baru</label>
				<div class="relative">
					<input
						id="pw-ulang"
						class="input pr-11"
						type={tampilUlang ? 'text' : 'password'}
						autocomplete="new-password"
						bind:value={ulang}
					/>
					<button
						type="button"
						class="absolute top-1/2 right-2 -translate-y-1/2 rounded-lg p-1.5 text-ink-400"
						aria-label={tampilUlang ? 'Sembunyikan konfirmasi' : 'Tampilkan konfirmasi'}
						onclick={() => (tampilUlang = !tampilUlang)}
					>
						<Icon name={tampilUlang ? 'eyeOff' : 'eye'} size={17} />
					</button>
				</div>
			</div>
		</div>

		{#snippet footer()}
			<button class="btn btn-utama w-full" disabled={memproses} onclick={ubahPassword}>
				{memproses ? 'Menyimpan…' : 'Simpan Password Baru'}
			</button>
		{/snippet}
	</Sheet>
{/if}
