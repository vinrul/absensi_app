const BASE = '/api/v1';

let accessToken: string | null = null;
let sedangSegarkan: Promise<boolean> | null = null;

export class ApiError extends Error {
	status: number;
	kode: string;

	constructor(status: number, kode: string, message: string) {
		super(message);
		this.status = status;
		this.kode = kode;
	}
}

/** Access token hanya berada di memori; refresh memakai cookie httpOnly. */
export function simpanToken(akses: string) {
	accessToken = akses;
}

export function hapusToken() {
	accessToken = null;
}

export function adaToken() {
	return Boolean(accessToken);
}

function beriTahuSesiBerakhir() {
	if (typeof window !== 'undefined') {
		window.dispatchEvent(new CustomEvent('personel:sesi-berakhir'));
	}
}

async function segarkan(): Promise<boolean> {
	if (sedangSegarkan) return sedangSegarkan;

	sedangSegarkan = (async () => {
		try {
			const res = await fetch(`${BASE}/auth/refresh`, {
				method: 'POST',
				credentials: 'include',
				signal: AbortSignal.timeout(30_000)
			});
			if (!res.ok) {
				hapusToken();
				return false;
			}
			const data = (await res.json()) as { access_token: string };
			simpanToken(data.access_token);
			return true;
		} catch {
			return false;
		}
	})();

	const hasil = await sedangSegarkan;
	sedangSegarkan = null;
	return hasil;
}

type Opsi = Omit<RequestInit, 'body'> & {
	body?: unknown;
	retry?: boolean;
};

async function kirim(path: string, opsi: Opsi, body: unknown): Promise<Response> {
	const { retry: _retry, ...rest } = opsi;
	const headers = new Headers(rest.headers);

	if (!(body instanceof FormData) && body !== undefined) {
		headers.set('content-type', 'application/json');
	}
	if (accessToken) headers.set('authorization', `Bearer ${accessToken}`);

	try {
		return await fetch(`${BASE}${path}`, {
			...rest,
			headers,
			credentials: 'include',
			signal: rest.signal ?? AbortSignal.timeout(45_000),
			body:
				body === undefined
					? undefined
					: body instanceof FormData
						? body
						: JSON.stringify(body)
		});
	} catch (err) {
		if (err instanceof DOMException && err.name === 'AbortError') {
			throw new ApiError(0, 'jaringan', 'Permintaan melebihi batas waktu. Periksa jaringan.');
		}
		throw new ApiError(0, 'jaringan', 'Tidak dapat terhubung ke server. Periksa jaringan Anda.');
	}
}

async function bacaJson(res: Response): Promise<unknown> {
	const teks = await res.text();
	if (!teks) return null;
	try {
		return JSON.parse(teks);
	} catch {
		return teks;
	}
}

export async function api<T>(path: string, opsi: Opsi = {}): Promise<T> {
	const { body, retry = true, ...rest } = opsi;
	let res = await kirim(path, rest, body);

	if (res.status === 401 && retry) {
		const pulih = await segarkan();
		if (pulih) res = await kirim(path, rest, body);
		else beriTahuSesiBerakhir();
	}

	const data = await bacaJson(res);

	if (!res.ok) {
		const error = (data as { error?: { code?: string; message?: string } } | null)?.error;
		if (res.status === 401) beriTahuSesiBerakhir();
		throw new ApiError(
			res.status,
			error?.code ?? 'error',
			error?.message ?? `Permintaan gagal (${res.status})`
		);
	}

	return data as T;
}

/** Petakan path berkas server menjadi URL absolut. */
export function urlBerkas(path: string): string {
	if (path.startsWith('/api/v1/') || path.startsWith('http')) return path;
	if (path.startsWith('auth/foto/')) return `/api/v1/${path}`;
	if (path.startsWith('lampiran/')) return `/api/v1/${path}`;
	if (path.startsWith('/files/')) return `/api/v1${path}`;
	return `/api/v1/lampiran/raw/${path}`;
}

/** Ambil berkas dengan bearer token, lalu jadikan object URL. */
export async function apiBlobUrl(path: string): Promise<string> {
	const url = urlBerkas(path);

	const ambil = async () => {
		const headers = new Headers();
		if (accessToken) headers.set('authorization', `Bearer ${accessToken}`);
		return fetch(url, { headers, signal: AbortSignal.timeout(45_000) });
	};

	let res = await ambil();
	if (res.status === 401 && (await segarkan())) res = await ambil();
	if (!res.ok) throw new ApiError(res.status, 'berkas', 'Berkas tidak dapat dimuat');
	return URL.createObjectURL(await res.blob());
}

/** Unggah lampiran/bukti ke `POST /lampiran`. */
export async function unggahLampiran(
	ownerType: 'rencana' | 'hasil' | 'output' | 'absensi',
	ownerId: string,
	kategori: 'lampiran' | 'bukti' | 'foto',
	file: File
): Promise<void> {
	const form = new FormData();
	form.append('owner_type', ownerType);
	form.append('owner_id', ownerId);
	form.append('kategori', kategori);
	form.append('file', file, file.name);
	await api('/lampiran', { method: 'POST', body: form });
}

/** Unggah foto profil ke `POST /auth/foto`. */
export async function unggahFotoProfil(file: File): Promise<unknown> {
	const form = new FormData();
	form.append('file', file, file.name);
	return api('/auth/foto', { method: 'POST', body: form });
}
