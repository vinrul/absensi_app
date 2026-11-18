import { api, ApiError, hapusToken, simpanToken } from '#lib/api/client';
import type { LoginResponse, Pengguna } from '#lib/types';

const KUNCI_PENGGUNA = 'personel.pengguna';

export const session = $state<{ user: Pengguna | null; siap: boolean }>({
	user: null,
	siap: false
});

function penyimpanan(): Storage | null {
	try {
		return typeof localStorage === 'undefined' ? null : localStorage;
	} catch {
		return null;
	}
}

function simpanPenggunaLokal(user: Pengguna | null) {
	const simpan = penyimpanan();
	if (!simpan) return;
	if (user) simpan.setItem(KUNCI_PENGGUNA, JSON.stringify(user));
	else simpan.removeItem(KUNCI_PENGGUNA);
}

function bacaPenggunaLokal(): Pengguna | null {
	const simpan = penyimpanan();
	const teks = simpan?.getItem(KUNCI_PENGGUNA);
	if (!teks) return null;
	try {
		return JSON.parse(teks) as Pengguna;
	} catch {
		return null;
	}
}

export function simpanPengguna(user: Pengguna) {
	session.user = user;
	simpanPenggunaLokal(user);
}

export async function login(nrp: string, password: string): Promise<Pengguna> {
	const data = await api<LoginResponse>('/auth/login', {
		method: 'POST',
		body: {
			nrp: nrp.trim(),
			password,
			device_id: 'web-personel',
			client: 'personel_web'
		},
		retry: false
	});
	simpanToken(data.access_token);
	simpanPengguna(data.user);
	return data.user;
}

/** Pulihkan sesi memakai cookie refresh httpOnly. Data pengguna tersimpan dipakai saat offline. */
export async function pulihkanSesi(): Promise<boolean> {
	const tersimpan = bacaPenggunaLokal();
	if (tersimpan) session.user = tersimpan;

	try {
		const user = await api<Pengguna>('/auth/me');
		simpanPengguna(user);
		return true;
	} catch (err) {
		if (err instanceof ApiError && err.status === 401) {
			hapusSesiLokal();
			return false;
		}
		return Boolean(tersimpan);
	} finally {
		session.siap = true;
	}
}

export function hapusSesiLokal() {
	hapusToken();
	session.user = null;
	simpanPenggunaLokal(null);
}

export async function logout() {
	try {
		await api('/auth/logout', { method: 'POST', retry: false });
	} catch {
		// Token lokal tetap dibersihkan meski server tidak terjangkau.
	} finally {
		hapusSesiLokal();
	}
}

export function punyaRole(...roles: string[]): boolean {
	return roles.some((role) => (session.user?.roles ?? []).includes(role));
}

export function bisaMenyetujui(): boolean {
	return punyaRole('atasan') || (session.user?.roles ?? []).some((r) => r.startsWith('pimpinan'));
}

export function peranUtama(user = session.user): string {
	const roles = user?.roles ?? [];
	if (roles.some((r) => r.startsWith('pimpinan'))) return 'Pimpinan';
	if (roles.includes('atasan')) return 'Atasan Langsung';
	return 'Personel';
}
