const BASE = '/api/v1';

let accessToken: string | null = null;
let refreshPromise: Promise<boolean> | null = null;

export class ApiError extends Error {
	status: number;
	code: string;

	constructor(status: number, code: string, message: string) {
		super(message);
		this.status = status;
		this.code = code;
	}
}

export function setAccessToken(token: string | null) {
	accessToken = token;
}

export function getAccessToken() {
	return accessToken;
}

async function refreshSession(): Promise<boolean> {
	if (refreshPromise) return refreshPromise;

	refreshPromise = (async () => {
		try {
			const res = await fetch(`${BASE}/auth/refresh`, {
				method: 'POST',
				credentials: 'include'
			});
			if (!res.ok) return false;
			const data = (await res.json()) as { access_token: string };
			accessToken = data.access_token;
			return true;
		} catch {
			return false;
		}
	})();

	const hasil = await refreshPromise;
	refreshPromise = null;
	return hasil;
}

type Opsi = Omit<RequestInit, 'body'> & {
	body?: unknown;
	retry?: boolean;
};

export async function api<T>(path: string, opsi: Opsi = {}): Promise<T> {
	const { body, retry = true, ...rest } = opsi;

	const kirim = async (): Promise<Response> => {
		const headers = new Headers(rest.headers);

		if (body instanceof FormData) {
			// biarkan browser menentukan boundary
		} else if (body !== undefined) {
			headers.set('content-type', 'application/json');
		}

		if (accessToken) headers.set('authorization', `Bearer ${accessToken}`);

		return fetch(`${BASE}${path}`, {
			...rest,
			headers,
			credentials: 'include',
			body:
				body === undefined
					? undefined
					: body instanceof FormData
						? body
						: JSON.stringify(body)
		});
	};

	let res = await kirim();

	if (res.status === 401 && retry) {
		const pulih = await refreshSession();
		if (pulih) res = await kirim();
	}

	const teks = await res.text();
	let data: unknown = null;
	if (teks) {
		try {
			data = JSON.parse(teks);
		} catch {
			data = teks;
		}
	}

	if (!res.ok) {
		const error = (data as { error?: { code?: string; message?: string } } | null)?.error;
		throw new ApiError(
			res.status,
			error?.code ?? 'error',
			error?.message ?? `Permintaan gagal (${res.status})`
		);
	}

	return data as T;
}

/** Ambil berkas (foto/bukti) dengan header Authorization lalu jadikan object URL. */
export async function apiBlobUrl(path: string): Promise<string> {
	const url =
		path.startsWith('/api/v1/') || path.startsWith('/files/')
			? path
			: path.startsWith('auth/foto/')
				? `/api/v1/${path}`
				: `/api/v1/lampiran/raw/${path}`;

	const ambil = async () => {
		const headers = new Headers();
		if (accessToken) headers.set('authorization', `Bearer ${accessToken}`);
		return fetch(url, { headers, credentials: 'include' });
	};

	let res = await ambil();
	if (res.status === 401 && (await refreshSession())) res = await ambil();
	if (!res.ok) throw new ApiError(res.status, 'error', 'Berkas tidak dapat dimuat');

	const blob = await res.blob();
	return URL.createObjectURL(blob);
}

/** Unduh berkas biner (mis. PDF) dengan bearer token dan refresh otomatis. */
export async function apiBlob(path: string): Promise<Blob> {
	const ambil = async () => {
		const headers = new Headers();
		if (accessToken) headers.set('authorization', `Bearer ${accessToken}`);
		return fetch(`${BASE}${path}`, { headers, credentials: 'include' });
	};

	let res = await ambil();
	if (res.status === 401 && (await refreshSession())) res = await ambil();

	if (!res.ok) {
		const teks = await res.text();
		let pesan = `Permintaan gagal (${res.status})`;
		try {
			const data = JSON.parse(teks) as { error?: { code?: string; message?: string } };
			pesan = data.error?.message ?? pesan;
		} catch {
			// biarkan pesan bawaan
		}
		throw new ApiError(res.status, 'error', pesan);
	}

	return res.blob();
}
