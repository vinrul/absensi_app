import { api, setAccessToken } from '#lib/api/client';
import type { Role, RoleScope, SatkerLevel } from '#lib/types';

export type { Role } from '#lib/types';

export interface Pengguna {
	id: string;
	nrp: string | null;
	username: string | null;
	jenis_akun: 'admin' | 'personel';
	nama: string;
	pangkat: string;
	jabatan: string;
	satker_id: string;
	satker_nama: string;
	satker_level: SatkerLevel;
	roles: Role[];
	role_scopes: RoleScope[];
	must_change_password: boolean;
	atasan_id: string | null;
}

export const session = $state<{ user: Pengguna | null; siap: boolean }>({
	user: null,
	siap: false
});

export async function login(identitas: string, password: string): Promise<Pengguna> {
	const data = await api<{ access_token: string; user: Pengguna }>('/auth/login', {
		method: 'POST',
		body: { username: identitas, password, device_id: 'dashboard', client: 'web' },
		retry: false
	});
	setAccessToken(data.access_token);
	session.user = data.user;
	session.siap = true;
	return data.user;
}

/** Coba pulihkan sesi dari refresh cookie (dipakai saat halaman dimuat ulang). */
export async function pulihkanSesi(): Promise<boolean> {
	if (session.user) {
		session.siap = true;
		return true;
	}
	try {
		const user = await api<Pengguna>('/auth/me');
		session.user = user;
		return true;
	} catch {
		session.user = null;
		return false;
	} finally {
		session.siap = true;
	}
}

export async function logout() {
	try {
		await api('/auth/logout', { method: 'POST' });
	} finally {
		setAccessToken(null);
		session.user = null;
	}
}

export function punyaRole(...roles: Role[]): boolean {
	const dimiliki = session.user?.roles ?? [];
	return roles.some((role) => dimiliki.includes(role));
}

export function adalahAdmin() {
	return punyaRole(
		'admin_sistem',
		'admin_polda',
		'admin_satker_polda',
		'admin_polres',
		'admin_satker_polres'
	);
}

export function adalahAdminGlobal() {
	return punyaRole('admin_sistem');
}

export function adalahPimpinan() {
	return punyaRole(
		'pimpinan',
		'pimpinan_polda',
		'pimpinan_satker_polda',
		'pimpinan_polres',
		'pimpinan_satker_polres'
	);
}

export function adalahAtasan() {
	return punyaRole(
		'atasan',
		'admin_sistem',
		'admin_polda',
		'admin_satker_polda',
		'admin_polres',
		'admin_satker_polres'
	);
}

export function adalahPengelolaPh() {
	return adalahAtasan() || adalahPimpinan();
}

/** Pimpinan tanpa role admin: dashboard tidak menampilkan Master Data. */
export function adalahPimpinanSaja() {
	return adalahPimpinan() && !adalahAdmin();
}

/** Hanya akun dengan role admin/pimpinan yang boleh masuk dashboard web. */
export function bolehMasukDashboard(user = session.user) {
	const roles = user?.roles ?? [];
	return roles.some((role) => role.startsWith('admin_') || role.startsWith('pimpinan'));
}

export function bisaLihatManajemen() {
	return adalahAdmin() || adalahPimpinan() || punyaRole('atasan');
}
