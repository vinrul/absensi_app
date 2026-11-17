<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import { session } from '#lib/stores/session.svelte';
	import type { AdminAkun, ListResponse, Role, Satker } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';

	type FormAdmin = {
		id: string | null;
		username: string;
		nama: string;
		role: Role;
		scope_satker_id: string;
		password: string;
		is_active: boolean;
	};

	const ROLE_ADMIN: Role[] = [
		'admin_sistem',
		'admin_polda',
		'admin_satker_polda',
		'admin_polres',
		'admin_satker_polres'
	];

	const ROLE_LABEL: Record<string, string> = {
		admin_sistem: 'Admin Global',
		admin_polda: 'Admin Polda',
		admin_satker_polda: 'Admin Satker Polda',
		admin_polres: 'Admin Polres',
		admin_satker_polres: 'Admin Satker Polres'
	};

	let data = $state<AdminAkun[]>([]);
	let satkerList = $state<Satker[]>([]);
	let meta = $state({ page: 1, per_page: 10, total: 0 });
	let cari = $state('');
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);

	let formTerbuka = $state(false);
	let form = $state<FormAdmin>(kosong());
	let resetTarget = $state<AdminAkun | null>(null);
	let resetPassword = $state('');

	const superAdmin = $derived(session.user?.roles.includes('admin_sistem') ?? false);

	const scopeAktor = $derived(
		session.user?.role_scopes.find((r) =>
			['admin_polda', 'admin_polres', 'admin_satker_polda', 'admin_satker_polres'].includes(
				r.role
			)
		)?.scope_satker_id ?? ''
	);

	const rolePilihan = $derived(
		ROLE_ADMIN.filter((role) => {
			if (superAdmin) return true;
			if (role === 'admin_satker_polda') return session.user?.roles.includes('admin_polda');
			if (role === 'admin_satker_polres') return session.user?.roles.includes('admin_polres');
			return false;
		})
	);

	const levelScope = $derived.by((): Satker['level'] | null => {
		switch (form.role) {
			case 'admin_polda':
				return 'polda';
			case 'admin_polres':
				return 'polres';
			case 'admin_satker_polda':
			case 'admin_satker_polres':
				return 'satker';
			default:
				return null;
		}
	});

	const scopeOptions = $derived(
		satkerList.filter((s) => {
			if (!s.is_active || s.level !== levelScope) return false;
			if (superAdmin) return true;
			if (!scopeAktor) return false;
			return s.id === scopeAktor || s.parent_id === scopeAktor;
		})
	);

	const poldaId = $derived(satkerList.find((s) => s.level === 'polda')?.id ?? '');

	const totalHalaman = $derived(Math.max(1, Math.ceil(meta.total / meta.per_page)));

	function kosong(): FormAdmin {
		return {
			id: null,
			username: '',
			nama: '',
			role: 'admin_satker_polda',
			scope_satker_id: '',
			password: '',
			is_active: true
		};
	}

	function pesan(err: unknown): string {
		return err instanceof ApiError ? err.message : 'Terjadi kesalahan';
	}

	async function muat() {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams({
				page: String(meta.page),
				per_page: String(meta.per_page)
			});
			if (cari.trim()) params.set('q', cari.trim());
			const hasil = await api<ListResponse<AdminAkun>>(`/admin?${params}`);
			data = hasil.data;
			meta = hasil.meta;
		} catch (err) {
			error = pesan(err);
		} finally {
			memuat = false;
		}
	}

	async function muatSatker() {
		satkerList = await api<Satker[]>('/satker');
	}

	onMount(() => {
		void Promise.all([muat(), muatSatker()]);
	});

	function bukaTambah() {
		form = kosong();
		form.role = rolePilihan[0] ?? 'admin_satker_polda';
		form.scope_satker_id = scopeOptions[0]?.id || scopeAktor || '';
		formTerbuka = true;
	}

	function bukaEdit(row: AdminAkun) {
		form = {
			id: row.id,
			username: row.username,
			nama: row.nama,
			role: (row.role?.role as Role) ?? 'admin_satker_polda',
			scope_satker_id: row.role?.scope_satker_id ?? '',
			password: '',
			is_active: row.is_active
		};
		formTerbuka = true;
	}

	function gantiRole() {
		if (form.role === 'admin_sistem') {
			form.scope_satker_id = '';
		} else if (!form.scope_satker_id || !scopeOptions.some((s) => s.id === form.scope_satker_id)) {
			form.scope_satker_id = scopeOptions[0]?.id || scopeAktor || '';
		}
	}

	async function simpan() {
		error = null;
		info = null;
		try {
			const satkerId =
				form.role === 'admin_sistem' ? poldaId : form.scope_satker_id || scopeAktor;
			if (!satkerId) throw new Error('Pilih satker/unit untuk akun admin.');
			if (form.role !== 'admin_sistem' && !form.scope_satker_id) {
				throw new Error('Peran admin ini membutuhkan cakupan satker.');
			}
			if (form.id) {
				await api(`/admin/${form.id}`, {
					method: 'PATCH',
					body: {
						username: form.username.trim().toLowerCase(),
						nama: form.nama.trim(),
						satker_id: satkerId,
						is_active: form.is_active,
						role: form.role,
						scope_satker_id: form.role === 'admin_sistem' ? null : form.scope_satker_id
					}
				});
				info = 'Akun admin diperbarui.';
			} else {
				await api('/admin', {
					method: 'POST',
					body: {
						username: form.username.trim().toLowerCase(),
						nama: form.nama.trim(),
						satker_id: satkerId,
						role: form.role,
						scope_satker_id: form.role === 'admin_sistem' ? null : form.scope_satker_id,
						password: form.password || undefined
					}
				});
				info = 'Akun admin dibuat.';
			}
			formTerbuka = false;
			await muat();
		} catch (err) {
			error = pesan(err);
		}
	}

	async function reset() {
		if (!resetTarget) return;
		error = null;
		info = null;
		try {
			const hasil = await api<{ password_sementara: string }>(
				`/admin/${resetTarget.id}/reset-password`,
				{ method: 'POST', body: { password: resetPassword || undefined } }
			);
			info = `Password ${resetTarget.username} direset: ${hasil.password_sementara}`;
			resetTarget = null;
			resetPassword = '';
			await muat();
		} catch (err) {
			error = pesan(err);
		}
	}

	function scopeLabel(id: string | null): string {
		if (!id) return 'Global';
		return satkerList.find((s) => s.id === id)?.nama ?? '—';
	}

	function roleBadge(row: AdminAkun): string {
		const role = row.role?.role ?? '—';
		return ROLE_LABEL[role] ?? role;
	}

	function ubahHalaman(delta: number) {
		meta.page = Math.min(totalHalaman, Math.max(1, meta.page + delta));
		void muat();
	}
</script>

<div class="space-y-4">
	<div class="flex flex-wrap items-end justify-between gap-3">
		<div>
			<h1 class="text-xl font-semibold text-slate-800">Akun Admin</h1>
			<p class="mt-1 text-sm text-slate-500">
				Akun pengelola sistem. Login hanya melalui dashboard web dan tidak memerlukan absensi.
			</p>
		</div>
		<button class="btn btn-primary" onclick={bukaTambah}>
			<Icon name="plus" size={16} /> Tambah Admin
		</button>
	</div>

	{#if info}
		<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>
	{/if}
	{#if error}
		<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>
	{/if}

	<div class="card flex flex-wrap items-end gap-3 p-4">
		<div class="min-w-56 flex-1">
			<label class="label" for="cari">Cari</label>
			<input
				id="cari"
				class="input"
				placeholder="Username atau nama"
				bind:value={cari}
				onkeydown={(e) => e.key === 'Enter' && muat()}
			/>
		</div>
		<button class="btn btn-ghost" onclick={() => muat()}>Cari</button>
	</div>

	<div class="card overflow-hidden">
		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">Username</th>
						<th class="th">Nama</th>
						<th class="th">Peran</th>
						<th class="th">Cakupan</th>
						<th class="th">Status</th>
						<th class="th text-right">Aksi</th>
					</tr>
				</thead>
				<tbody>
					{#each data as row (row.id)}
						<tr>
							<td class="td font-mono text-xs">{row.username}</td>
							<td class="td">
								{row.nama}
								<span class="block text-[11px] text-slate-500">{row.satker_nama}</span>
							</td>
							<td class="td text-xs">{roleBadge(row)}</td>
							<td class="td text-xs">{scopeLabel(row.role?.scope_satker_id ?? null)}</td>
							<td class="td">
								<Badge tone={row.is_active ? 'green' : 'slate'}>
									{row.is_active ? 'Aktif' : 'Nonaktif'}
								</Badge>
							</td>
							<td class="td text-right">
								<div class="flex justify-end gap-1">
									<button class="btn btn-ghost px-2 py-1 text-xs" onclick={() => bukaEdit(row)}>
										Edit
									</button>
									<button
										class="btn btn-ghost px-2 py-1 text-xs"
										onclick={() => (resetTarget = row)}
									>
										Reset
									</button>
								</div>
							</td>
						</tr>
					{:else}
						<tr>
							<td class="td text-slate-500" colspan="6">
								{memuat ? 'Memuat…' : 'Belum ada akun admin.'}
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</div>
		<div class="flex items-center justify-between border-t border-slate-100 px-4 py-3 text-sm">
			<span class="text-slate-500">Total {meta.total} akun</span>
			<div class="flex items-center gap-2">
				<button class="btn btn-ghost px-2 py-1" disabled={meta.page <= 1} onclick={() => ubahHalaman(-1)}>
					Sebelumnya
				</button>
				<span class="text-slate-600">{meta.page} / {totalHalaman}</span>
				<button
					class="btn btn-ghost px-2 py-1"
					disabled={meta.page >= totalHalaman}
					onclick={() => ubahHalaman(1)}
				>
					Berikutnya
				</button>
			</div>
		</div>
	</div>
</div>

<Modal open={formTerbuka} title={form.id ? 'Edit Akun Admin' : 'Tambah Akun Admin'} onclose={() => (formTerbuka = false)}>
	<form class="space-y-4" onsubmit={(e) => { e.preventDefault(); void simpan(); }}>
		<div class="grid gap-4 sm:grid-cols-2">
			<div>
				<label class="label" for="username">Username</label>
				<input
					id="username"
					class="input font-mono"
					bind:value={form.username}
					placeholder="mis. admin_polres"
					autocomplete="off"
				/>
			</div>
			<div>
				<label class="label" for="nama">Nama</label>
				<input id="nama" class="input" bind:value={form.nama} />
			</div>
			<div>
				<label class="label" for="role">Peran</label>
				<select id="role" class="input" bind:value={form.role} onchange={gantiRole}>
					{#each rolePilihan as role}
						<option value={role}>{ROLE_LABEL[role]}</option>
					{/each}
				</select>
			</div>
			{#if form.role !== 'admin_sistem'}
				<div>
					<label class="label" for="scope">Cakupan satker</label>
					<select id="scope" class="input" bind:value={form.scope_satker_id}>
						<option value="">— pilih —</option>
						{#each scopeOptions as node}
							<option value={node.id}>{node.nama}</option>
						{/each}
					</select>
				</div>
			{/if}
			{#if !form.id}
				<div>
					<label class="label" for="password">Password awal</label>
					<input
						id="password"
						class="input"
						type="password"
						bind:value={form.password}
						placeholder="Kosongkan untuk Admin#2026"
						autocomplete="new-password"
					/>
				</div>
			{/if}
		</div>
		{#if form.id}
			<label class="flex items-center gap-2 text-sm text-slate-700">
				<input type="checkbox" bind:checked={form.is_active} /> Akun aktif
			</label>
		{/if}
		<div class="flex justify-end gap-2">
			<button type="button" class="btn btn-ghost" onclick={() => (formTerbuka = false)}>Batal</button>
			<button type="submit" class="btn btn-primary">Simpan</button>
		</div>
	</form>
</Modal>

<Modal open={Boolean(resetTarget)} title="Reset Password Admin" onclose={() => (resetTarget = null)}>
	<form class="space-y-4" onsubmit={(e) => { e.preventDefault(); void reset(); }}>
		<p class="text-sm text-slate-600">
			Akun <span class="font-mono font-semibold">{resetTarget?.username}</span> akan wajib
			mengganti password saat login berikutnya.
		</p>
		<div>
			<label class="label" for="reset">Password baru (opsional)</label>
			<input
				id="reset"
				class="input"
				type="password"
				bind:value={resetPassword}
				placeholder="Kosongkan untuk Admin#2026"
				autocomplete="new-password"
			/>
		</div>
		<div class="flex justify-end gap-2">
			<button type="button" class="btn btn-ghost" onclick={() => (resetTarget = null)}>Batal</button>
			<button type="submit" class="btn btn-primary">Reset</button>
		</div>
	</form>
</Modal>
