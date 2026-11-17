<script lang="ts">
	import { onMount } from 'svelte';
	import { api, ApiError } from '#lib/api/client';
	import { adalahAdmin, session } from '#lib/stores/session.svelte';
	import type { ListResponse, Personel, Role, RoleScope, Satker } from '#lib/types';
	import Badge from '#lib/components/Badge.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import Modal from '#lib/components/Modal.svelte';

	type FormPersonel = {
		id: string | null;
		nrp: string;
		nama: string;
		pangkat: string;
		jabatan: string;
		satker_id: string;
		atasan_id: string;
		phone: string;
		password: string;
		is_active: boolean;
	};

	const ROLE_LABEL: Record<Role, string> = {
		admin_sistem: 'Admin Global',
		admin_polda: 'Admin Polda',
		admin_satker_polda: 'Admin Satker Polda',
		admin_polres: 'Admin Polres',
		admin_satker_polres: 'Admin Satker Polres',
		pimpinan: 'Pimpinan Global',
		pimpinan_polda: 'Pimpinan Polda',
		pimpinan_satker_polda: 'Pimpinan Satker Polda',
		pimpinan_polres: 'Pimpinan Polres',
		pimpinan_satker_polres: 'Pimpinan Satker Polres',
		atasan: 'Atasan Langsung',
		personel: 'Personel'
	};

	const ROLE_ADMIN: Role[] = [
		'admin_sistem',
		'admin_polda',
		'admin_satker_polda',
		'admin_polres',
		'admin_satker_polres'
	];

	const PANGKAT_POLRI = [
		{ value: 'BHARADA', label: 'Bhayangkara Dua (Bharada)' },
		{ value: 'BHARATU', label: 'Bhayangkara Satu (Bharatu)' },
		{ value: 'BHARAKA', label: 'Bhayangkara Kepala (Bharaka)' },
		{ value: 'ABRIPDA', label: 'Ajun Brigadir Polisi Dua (Abripda)' },
		{ value: 'ABRIPTU', label: 'Ajun Brigadir Polisi Satu (Abriptu)' },
		{ value: 'ABRIPKA', label: 'Ajun Brigadir Polisi Kepala (Abripka)' },
		{ value: 'BRIPDA', label: 'Brigadir Polisi Dua (Bripda)' },
		{ value: 'BRIPTU', label: 'Brigadir Polisi Satu (Briptu)' },
		{ value: 'BRIPKA', label: 'Brigadir Polisi Kepala (Bripka)' },
		{ value: 'AIPDA', label: 'Ajun Inspektur Polisi Dua (Aipda)' },
		{ value: 'AIPTU', label: 'Ajun Inspektur Polisi Satu (Aiptu)' },
		{ value: 'IPDA', label: 'Inspektur Polisi Dua (Ipda)' },
		{ value: 'IPTU', label: 'Inspektur Polisi Satu (Iptu)' },
		{ value: 'AKP', label: 'Ajun Komisaris Polisi (AKP)' },
		{ value: 'KOMPOL', label: 'Komisaris Polisi (Kompol)' },
		{ value: 'AKBP', label: 'Ajun Komisaris Besar Polisi (AKBP)' },
		{ value: 'KOMBES POL', label: 'Komisaris Besar Polisi (Kombes Pol)' },
		{ value: 'BRIGJEN POL', label: 'Brigadir Jenderal Polisi (Brigjen Pol)' },
		{ value: 'IRJEN POL', label: 'Inspektur Jenderal Polisi (Irjen Pol)' },
		{ value: 'KOMJEN POL', label: 'Komisaris Jenderal Polisi (Komjen Pol)' },
		{ value: 'JENDERAL POL', label: 'Jenderal Polisi' }
	];

	let data = $state<Personel[]>([]);
	let semuaPersonel = $state<Personel[]>([]);
	let satkerList = $state<Satker[]>([]);
	let meta = $state({ page: 1, per_page: 10, total: 0 });
	let cari = $state('');
	let filterSatker = $state('');
	let memuat = $state(true);
	let error = $state<string | null>(null);
	let info = $state<string | null>(null);

	let formTerbuka = $state(false);
	let form = $state<FormPersonel>(kosong());
	let selectedRoles = $state<Role[]>(['personel']);
	let resetTarget = $state<Personel | null>(null);
	let resetPassword = $state('');

	const admin = $derived(adalahAdmin());
	const globalAdmin = $derived(session.user?.roles.includes('admin_sistem') ?? false);
	const scopePolda = $derived(
		session.user?.role_scopes.find((r) => r.role === 'admin_polda')?.scope_satker_id ?? ''
	);
	const scopePolres = $derived(
		session.user?.role_scopes.find((r) => r.role === 'admin_polres')?.scope_satker_id ?? ''
	);
	const scopeSatker = $derived(
		session.user?.role_scopes.find((r) =>
			['admin_satker_polda', 'admin_satker_polres'].includes(r.role)
		)?.scope_satker_id ?? ''
	);

	const satkerOptions = $derived(satkerList.filter((s) => s.level === 'satker' && s.is_active));
	const satkerTerpilih = $derived(satkerList.find((s) => s.id === form.satker_id) ?? null);
	const atasanOptions = $derived(
		semuaPersonel.filter(
			(p) =>
				p.satker_id === form.satker_id &&
				p.is_active &&
				p.id !== form.id
		)
	);
	const totalHalaman = $derived(Math.max(1, Math.ceil(meta.total / meta.per_page)));

	function kosong(): FormPersonel {
		return {
			id: null,
			nrp: '',
			nama: '',
			pangkat: 'BRIPKA',
			jabatan: 'Penyidik',
			satker_id: '',
			atasan_id: '',
			phone: '',
			password: '',
			is_active: true
		};
	}


	function poldaDari(satker: Satker | null): string | null {
		if (!satker) return null;
		if (satker.level === 'polda') return satker.id;
		const induk = satkerList.find((s) => s.id === satker.parent_id);
		if (induk?.level === 'polda') return induk.id;
		if (induk?.level === 'polres') return induk.parent_id;
		return null;
	}

	function polresDari(satker: Satker | null): string | null {
		if (!satker) return null;
		const induk = satkerList.find((s) => s.id === satker.parent_id);
		return induk?.level === 'polres' ? induk.id : null;
	}

	function scopeUntuk(role: Role): string | null {
		const satker = satkerTerpilih;
		if (!satker) return null;
		switch (role) {
			case 'admin_sistem':
			case 'pimpinan':
			case 'personel':
			case 'atasan':
				return null;
			case 'admin_polda':
			case 'pimpinan_polda':
				return poldaDari(satker);
			case 'admin_polres':
			case 'pimpinan_polres':
				return polresDari(satker);
			case 'admin_satker_polda':
			case 'pimpinan_satker_polda':
				return satker.parent_id && satkerList.find((s) => s.id === satker.parent_id)?.level === 'polda'
					? satker.id
					: null;
			case 'admin_satker_polres':
			case 'pimpinan_satker_polres':
				return satker.parent_id && satkerList.find((s) => s.id === satker.parent_id)?.level === 'polres'
					? satker.id
					: null;
			default:
				return null;
		}
	}

	function roleBoleh(role: Role): boolean {
		if (!satkerTerpilih) return false;
		if (role === 'admin_sistem' || role === 'pimpinan') return globalAdmin;
		if (['personel', 'atasan'].includes(role)) return true;
		if (globalAdmin) return Boolean(scopeUntuk(role));
		if (role === 'admin_polda' || role === 'pimpinan_polda') {
			return Boolean(scopePolda && poldaDari(satkerTerpilih) === scopePolda);
		}
		if (role === 'admin_polres' || role === 'pimpinan_polres') {
			return Boolean(scopePolres && polresDari(satkerTerpilih) === scopePolres);
		}
		if (role === 'admin_satker_polda' || role === 'pimpinan_satker_polda') {
			return Boolean(
				(scopePolda && poldaDari(satkerTerpilih) === scopePolda) ||
				(scopeSatker && scopeUntuk(role) === scopeSatker)
			);
		}
		if (role === 'admin_satker_polres' || role === 'pimpinan_satker_polres') {
			return Boolean(
				(scopePolres && polresDari(satkerTerpilih) === scopePolres) ||
				(scopeSatker && scopeUntuk(role) === scopeSatker)
			);
		}
		return false;
	}

	const rolePilihan = $derived(
		(Object.keys(ROLE_LABEL) as Role[]).filter(
			(role) => !ROLE_ADMIN.includes(role) && roleBoleh(role)
		)
	);



	function labelScope(role: Role): string {
		const id = scopeUntuk(role);
		if (!id) return '';
		return satkerList.find((s) => s.id === id)?.nama ?? '';
	}

	function roleBadge(row: Personel): string {
		return row.roles
			.map((r) => {
				const label = ROLE_LABEL[r.role] ?? r.role;
				return r.scope_satker_nama ? `${label} (${r.scope_satker_nama})` : label;
			})
			.join(', ');
	}

	async function muat(page = 1) {
		memuat = true;
		error = null;
		try {
			const params = new URLSearchParams({
				page: String(page),
				per_page: String(meta.per_page)
			});
			if (cari.trim()) params.set('q', cari.trim());
			if (filterSatker) params.set('satker_id', filterSatker);
			const hasil = await api<ListResponse<Personel>>(`/personel?${params}`);
			data = hasil.data;
			meta = hasil.meta;
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal memuat personel';
		} finally {
			memuat = false;
		}
	}

	async function muatPilihan() {
		const [s, p] = await Promise.all([
			api<Satker[]>('/satker'),
			api<ListResponse<Personel>>('/personel?per_page=200')
		]);
		satkerList = s;
		semuaPersonel = p.data;
		if (!form.satker_id) form.satker_id = satkerAwal();
	}

	function satkerAwal(): string {
		if (scopeSatker) return scopeSatker;
		if (scopePolda) {
			return satkerOptions.find((s) => s.parent_id === scopePolda)?.id ?? satkerOptions[0]?.id ?? '';
		}
		if (scopePolres) {
			return satkerOptions.find((s) => s.parent_id === scopePolres)?.id ?? satkerOptions[0]?.id ?? '';
		}
		return satkerOptions[0]?.id ?? '';
	}

	function bukaTambah() {
		form = kosong();
		form.satker_id = satkerAwal();
		selectedRoles = ['personel'];
		formTerbuka = true;
	}

	function bukaEdit(row: Personel) {
		form = {
			id: row.id,
			nrp: row.nrp,
			nama: row.nama,
			pangkat: row.pangkat,
			jabatan: row.jabatan,
			satker_id: row.satker_id,
			atasan_id: row.atasan_id ?? '',
			phone: row.phone ?? '',
			password: '',
			is_active: row.is_active
		};
		selectedRoles = row.roles.map((r) => r.role);
		formTerbuka = true;
	}

	async function simpan() {
		error = null;
		info = null;
		try {
			const roles: RoleScope[] = selectedRoles.map((role) => ({
				role,
				scope_satker_id: scopeUntuk(role),
				scope_satker_nama: null,
				scope_level: null
			}));
			if (selectedRoles.some((role) => role !== 'personel' && role !== 'atasan' && !scopeUntuk(role))) {
				throw new Error('Cakupan role tidak valid untuk satker yang dipilih.');
			}
			const body = {
				nama: form.nama,
				pangkat: form.pangkat,
				jabatan: form.jabatan,
				satker_id: form.satker_id,
				atasan_id: form.atasan_id || null,
				hapus_atasan: Boolean(form.id) && !form.atasan_id,
				phone: form.phone || null,
				is_active: form.is_active
			};
			if (form.id) {
				await api(`/personel/${form.id}`, { method: 'PATCH', body });
				await api(`/personel/${form.id}/roles`, {
					method: 'POST',
					body: { roles: roles.map(({ role, scope_satker_id }) => ({ role, scope_satker_id })) }
				});
				info = `Personel ${form.nama} diperbarui.`;
			} else {
				await api('/personel', {
					method: 'POST',
					body: {
						...body,
						nrp: form.nrp,
						password: form.password || undefined,
						roles: roles.map(({ role, scope_satker_id }) => ({ role, scope_satker_id }))
					}
				});
				info = `Personel ${form.nama} ditambahkan.`;
			}
			formTerbuka = false;
			await muat(1);
			await muatPilihan();
		} catch (err) {
			error = err instanceof ApiError || err instanceof Error ? err.message : 'Gagal menyimpan personel';
		}
	}

	async function toggleAktif(row: Personel) {
		try {
			await api(`/personel/${row.id}`, {
				method: 'PATCH',
				body: { is_active: !row.is_active }
			});
			await muat(meta.page);
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal mengubah status';
		}
	}

	async function simpanReset() {
		if (!resetTarget) return;
		try {
			const hasil = await api<{ password_sementara?: string; message: string }>(
				`/personel/${resetTarget.id}/reset-password`,
				{ method: 'POST', body: { password: resetPassword || null } }
			);
			info = hasil.password_sementara
				? `Password sementara: ${hasil.password_sementara}`
				: hasil.message;
			resetTarget = null;
			resetPassword = '';
		} catch (err) {
			error = err instanceof ApiError ? err.message : 'Gagal mereset password';
		}
	}

	onMount(async () => {
		await muatPilihan();
		await muat(1);
	});
</script>

<svelte:head><title>Data Personel — Penilaian Kinerja</title></svelte:head>

<div class="space-y-4">
	<div class="card flex flex-wrap items-end justify-between gap-3 p-4">
		<div>
			<h2 class="text-sm font-semibold text-slate-800">Data Personel</h2>
			<p class="mt-1 text-xs text-slate-500">
				Kelola data, atasan langsung, role, dan status akun dalam cakupan Anda.
			</p>
		</div>
		{#if admin}
			<button class="btn btn-primary" onclick={bukaTambah}>
				<Icon name="plus" size={16} /> Tambah Personel
			</button>
		{/if}
	</div>

	{#if info}<p class="rounded-lg bg-emerald-50 px-4 py-2 text-sm text-emerald-700 ring-1 ring-emerald-200">{info}</p>{/if}
	{#if error}<p class="rounded-lg bg-rose-50 px-4 py-2 text-sm text-rose-700 ring-1 ring-rose-200">{error}</p>{/if}

	<div class="card overflow-hidden">
		<div class="flex flex-wrap items-end justify-between gap-2 border-b border-slate-200 px-4 py-3">
			<div class="flex flex-wrap items-end gap-2">
				<div class="min-w-56">
					<label class="label" for="cari">Cari</label>
					<input id="cari" class="input" placeholder="Nama / NRP" bind:value={cari} onkeydown={(e) => e.key === 'Enter' && muat(1)} />
				</div>
				<div class="min-w-56">
					<label class="label" for="fsatker">Satker</label>
					<select id="fsatker" class="input" bind:value={filterSatker} onchange={() => muat(1)}>
						<option value="">Semua Satker</option>
						{#each satkerOptions as s}<option value={s.id}>{s.nama}</option>{/each}
					</select>
				</div>
				<button class="btn btn-ghost border border-slate-300" onclick={() => muat(1)}>
					<Icon name="search" size={16} />
				</button>
			</div>
			<span class="text-xs text-slate-500">{meta.total} personel</span>
		</div>

		<div class="overflow-x-auto">
			<table class="w-full">
				<thead class="bg-slate-50">
					<tr>
						<th class="th">NRP</th>
						<th class="th">Nama</th>
						<th class="th">Pangkat</th>
						<th class="th">Jabatan / Unit</th>
						<th class="th">Satker</th>
						<th class="th">Atasan</th>
						<th class="th">Role</th>
						<th class="th">Status</th>
						<th class="th no-print">Aksi</th>
					</tr>
				</thead>
				<tbody class="divide-y divide-slate-100">
					{#each data as row}
						<tr class="hover:bg-slate-50/70">
							<td class="td text-slate-600">{row.nrp}</td>
							<td class="td font-medium text-slate-800">{row.nama}</td>
							<td class="td text-slate-600">{row.pangkat}</td>
							<td class="td text-slate-600">
								{row.jabatan}{row.unit ? ` · ${row.unit}` : ''}
							</td>
							<td class="td text-slate-600">{row.satker_nama}</td>
							<td class="td text-slate-600">{row.atasan_nama ?? '-'}</td>
							<td class="td max-w-56 text-xs text-slate-600">{roleBadge(row)}</td>
							<td class="td"><Badge tone={row.is_active ? 'green' : 'slate'}>{row.is_active ? 'Aktif' : 'Nonaktif'}</Badge></td>
							<td class="td no-print">
								{#if admin}
									<div class="flex flex-wrap gap-1">
										<button class="btn btn-ghost border border-slate-300 px-2 py-1 text-xs" onclick={() => bukaEdit(row)}>Edit</button>
										<button class="btn btn-ghost px-2 py-1 text-xs" onclick={() => { resetTarget = row; resetPassword = ''; }}>Reset</button>
										<button class="btn btn-ghost px-2 py-1 text-xs {row.is_active ? 'text-rose-600' : 'text-emerald-600'}" onclick={() => toggleAktif(row)}>
											{row.is_active ? 'Nonaktifkan' : 'Aktifkan'}
										</button>
									</div>
								{/if}
							</td>
						</tr>
					{:else}
						<tr><td class="td text-center text-slate-500" colspan="9">{memuat ? 'Memuat…' : 'Tidak ada data.'}</td></tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="flex items-center justify-between border-t border-slate-200 px-4 py-2 text-xs text-slate-500">
			<span>Halaman {meta.page} / {totalHalaman}</span>
			<div class="flex gap-1">
				<button class="btn btn-ghost px-2 py-1" disabled={meta.page <= 1} onclick={() => muat(meta.page - 1)}>‹</button>
				<button class="btn btn-ghost px-2 py-1" disabled={meta.page >= totalHalaman} onclick={() => muat(meta.page + 1)}>›</button>
			</div>
		</div>
	</div>
</div>

<Modal open={formTerbuka} title={form.id ? 'Edit Personel' : 'Tambah Personel'} lebar="max-w-3xl" onclose={() => (formTerbuka = false)}>
	<div class="grid gap-3 sm:grid-cols-2">
		<div><label class="label" for="nrp">NRP</label><input id="nrp" class="input" bind:value={form.nrp} disabled={Boolean(form.id)} /></div>
		<div><label class="label" for="nama">Nama</label><input id="nama" class="input" bind:value={form.nama} /></div>
		<div><label class="label" for="pangkat">Pangkat</label>
			<select id="pangkat" class="input" bind:value={form.pangkat}>
				{#each PANGKAT_POLRI as p}
					<option value={p.value}>{p.label}</option>
				{/each}
				{#if !PANGKAT_POLRI.some((p) => p.value === form.pangkat)}
					<option value={form.pangkat}>{form.pangkat} (data lama)</option>
				{/if}
			</select>
		</div>
		<div><label class="label" for="jabatan">Jabatan</label><input id="jabatan" class="input" bind:value={form.jabatan} /></div>
		<div>
			<label class="label" for="psatker">Satker</label>
			<select id="psatker" class="input" bind:value={form.satker_id}>
				{#each satkerOptions as s}<option value={s.id}>{s.nama}</option>{/each}
			</select>
		</div>
		<div>
			<label class="label" for="patasan">Atasan Langsung</label>
			<select id="patasan" class="input" bind:value={form.atasan_id}>
				<option value="">Belum ditetapkan</option>
				{#each atasanOptions as p}
					<option value={p.id}>{p.nama} · {p.nrp}</option>
				{/each}
			</select>
		</div>
		<div><label class="label" for="phone">Telepon</label><input id="phone" class="input" bind:value={form.phone} /></div>
		{#if !form.id}
			<div><label class="label" for="pass">Password Awal (opsional)</label><input id="pass" class="input" placeholder="Default: Polri#2026" bind:value={form.password} /></div>
		{/if}
		<div class="sm:col-span-2">
			<span class="label">Role & Cakupan</span>
			<div class="grid gap-2 rounded-lg border border-slate-200 p-3 sm:grid-cols-2">
				{#each rolePilihan as role}
					<label class="flex items-start gap-2 text-xs text-slate-700">
						<input type="checkbox" checked={selectedRoles.includes(role)} onchange={() => selectedRoles = selectedRoles.includes(role) ? selectedRoles.filter((r) => r !== role) : [...selectedRoles, role]} />
						<span>
							{ROLE_LABEL[role]}
							{#if labelScope(role)}<span class="block text-[11px] text-slate-500">{labelScope(role)}</span>{/if}
						</span>
					</label>
				{/each}
			</div>
		</div>
	</div>
	<div class="mt-4 flex items-center justify-between gap-2">
		<label class="flex items-center gap-2 text-xs text-slate-600">
			<input type="checkbox" bind:checked={form.is_active} /> Akun aktif
		</label>
		<div class="flex gap-2">
			<button class="btn btn-ghost" onclick={() => (formTerbuka = false)}>Batal</button>
			<button class="btn btn-primary" onclick={simpan}>Simpan</button>
		</div>
	</div>
</Modal>

<Modal open={!!resetTarget} title="Reset Password" onclose={() => (resetTarget = null)}>
	{#if resetTarget}
		<div class="space-y-3 text-sm">
			<p class="text-slate-600">Reset password untuk <strong>{resetTarget.nama}</strong>?</p>
			<div>
				<label class="label" for="resetpass">Password Baru (opsional)</label>
				<input id="resetpass" class="input" placeholder="Kosongkan untuk Polri#2026" bind:value={resetPassword} />
			</div>
			<p class="text-xs text-slate-500">Personel wajib mengganti password saat login berikutnya.</p>
			<div class="flex justify-end gap-2">
				<button class="btn btn-ghost" onclick={() => (resetTarget = null)}>Batal</button>
				<button class="btn btn-gold" onclick={simpanReset}>Reset</button>
			</div>
		</div>
	{/if}
</Modal>
