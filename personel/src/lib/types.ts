export type SatkerLevel = 'polda' | 'polres' | 'satker';

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
	roles: string[];
	role_scopes: { role: string; scope_satker_id: string | null; scope_satker_nama: string | null }[];
	must_change_password: boolean;
	atasan_id: string | null;
	email?: string | null;
	phone?: string | null;
	foto_url?: string | null;
}

export interface LoginResponse {
	access_token: string;
	refresh_token: string;
	token_type: string;
	expires_in: number;
	user: Pengguna;
}

export interface TokenResponse {
	access_token: string;
	refresh_token: string;
	token_type: string;
	expires_in: number;
}

export interface Meta {
	page: number;
	per_page: number;
	total: number;
}

export interface ListResponse<T> {
	data: T[];
	meta: Meta;
}

// --------------------------------------------------------------------- absensi

export type JenisAbsen = 'masuk' | 'pulang';

export type StatusKehadiran =
	| 'dinas_kantor'
	| 'dinas_luar'
	| 'ph'
	| 'piket'
	| 'libur_nasional'
	| 'cuti'
	| 'izin'
	| 'sakit'
	| 'tanpa_keterangan';

export interface Absensi {
	id: string;
	tanggal: string;
	jenis: JenisAbsen;
	status_kehadiran: StatusKehadiran;
	jam: string;
	jarak_m: number | null;
	dalam_radius: boolean;
	is_offline: boolean;
	is_mock_suspect: boolean;
	perlu_verifikasi: boolean;
	catatan_verifikator: string | null;
	foto_url: string | null;
}

export interface AbsensiHariIni {
	tanggal: string;
	masuk: Absensi | null;
	pulang: Absensi | null;
}

export interface Geofence {
	titik_nama: string | null;
	jarak_m: number | null;
	radius_m: number | null;
	dalam_radius: boolean;
}

export interface TitikDinas {
	id: string;
	nama: string;
	jenis: string;
	latitude: number;
	longitude: number;
	radius_m: number;
	jarak_m: number | null;
	is_active: boolean;
}

export interface TitikTerdekatResponse {
	titik: TitikDinas[];
	terdekat: Geofence;
}

export interface AbsensiResponse {
	data: Absensi;
	geofence: Geofence;
	server_time: string;
}

export interface AbsensiRequest {
	client_uuid: string;
	jenis: JenisAbsen;
	status_kehadiran: StatusKehadiran;
	captured_at: string;
	latitude: number | null;
	longitude: number | null;
	akurasi_m: number | null;
	is_mock_suspect: boolean;
	is_offline: boolean;
	device_id: string | null;
	foto_base64: string | null;
}

export interface AbsensiAntrean extends AbsensiRequest {
	dibuat_pada: number;
}

// ------------------------------------------------------------------- strong point

export interface JadwalPhSaya {
	id: string;
	satker_id: string;
	strong_point_id: string;
	strong_point_nama: string;
	latitude: number;
	longitude: number;
	radius_m: number;
	tanggal: string;
	jam_mulai: string;
	jam_selesai: string;
	regu: string | null;
	catatan: string | null;
	peran: string | null;
}

export interface AbsensiPh {
	id: string;
	jadwal_ph_id: string;
	personel_id: string;
	personel_nama: string;
	personel_nrp: string;
	strong_point_id: string;
	strong_point_nama: string;
	satker_id: string;
	jenis: JenisAbsen;
	captured_at: string;
	jam: string;
	latitude: number | null;
	longitude: number | null;
	akurasi_m: number | null;
	jarak_m: number | null;
	dalam_radius: boolean;
	foto_url: string | null;
	is_offline: boolean;
	is_mock_suspect: boolean;
	perlu_verifikasi: boolean;
	verified_at: string | null;
	catatan_verifikator: string | null;
}

export interface AbsensiPhRequest {
	client_uuid: string;
	jadwal_ph_id: string;
	jenis: JenisAbsen;
	captured_at: string;
	latitude: number | null;
	longitude: number | null;
	akurasi_m: number | null;
	is_mock_suspect: boolean;
	is_offline: boolean;
	foto_base64: string | null;
}

export interface AbsensiPhAntrean extends AbsensiPhRequest {
	dibuat_pada: number;
}

// ------------------------------------------------------------------ rencana kerja

export type StatusRencana = 'draft' | 'diajukan' | 'disetujui' | 'perlu_revisi' | 'ditolak';

export interface Lampiran {
	id: string;
	owner_type: string;
	owner_id: string;
	kategori: string;
	file_name: string;
	mime: string;
	size_bytes: number;
	storage_key: string;
	created_at: string;
}

export interface Tahapan {
	id: string;
	urutan: number;
	uraian: string;
	target_kuantitas: number | null;
	satuan: string | null;
	bobot: number | null;
	program: string | null;
	kegiatan: string | null;
	tujuan: string | null;
	indikator_kinerja: string | null;
	target_kualitas: string | null;
	tanggal_mulai: string | null;
	tanggal_selesai: string | null;
}

export interface Rencana {
	id: string;
	personel_id: string;
	personel_nama: string | null;
	personel_nrp: string | null;
	unit: string | null;
	program: string;
	kegiatan: string;
	uraian: string;
	tujuan: string | null;
	indikator_kinerja: string | null;
	jenis: string;
	dasar_jenis: 'sp' | 'tor' | 'perintah_lisan' | null;
	dasar_nomor: string | null;
	dasar_tanggal: string | null;
	dasar_perihal: string | null;
	target_kuantitas: number | null;
	satuan: string | null;
	target_kualitas: string | null;
	target_waktu: string | null;
	tanggal_mulai: string;
	tanggal_selesai: string;
	status: StatusRencana;
	catatan_atasan: string | null;
	jumlah_revisi: number;
}

export interface RencanaDetail extends Rencana {
	tahapan: Tahapan[];
	lampiran: Lampiran[];
}

export interface TahapanRequest {
	uraian: string;
	target_kuantitas: number | null;
	satuan: string | null;
	bobot: number | null;
	program: string | null;
	kegiatan: string | null;
	tujuan: string | null;
	indikator_kinerja: string | null;
	target_kualitas: string | null;
	tanggal_mulai: string | null;
	tanggal_selesai: string | null;
}

export interface RencanaRequest {
	periode_id: string | null;
	program: string;
	kegiatan: string;
	uraian: string;
	tujuan: string | null;
	indikator_kinerja: string | null;
	jenis: string;
	dasar_jenis: string | null;
	dasar_nomor: string | null;
	dasar_tanggal: string | null;
	dasar_perihal: string | null;
	target_kuantitas: number | null;
	satuan: string | null;
	target_kualitas: string | null;
	target_waktu: string | null;
	tanggal_mulai: string;
	tanggal_selesai: string;
	tahapan: TahapanRequest[];
	ajukan: boolean;
}

export type UbahRencanaRequest = Omit<RencanaRequest, 'periode_id' | 'ajukan'>;

// ------------------------------------------------------------------- hasil kerja

export type StatusHasil = 'draft' | 'diajukan' | 'terverifikasi' | 'perlu_revisi';

export interface Hasil {
	id: string;
	rencana_id: string;
	personel_id: string;
	personel_nama: string | null;
	personel_nrp: string | null;
	rencana_kegiatan: string | null;
	uraian_pelaksanaan: string;
	progres_persen: number;
	jumlah_realisasi: number | null;
	kendala: string | null;
	tanggal_realisasi: string;
	status: StatusHasil;
	skor_kuantitas: number | null;
	skor_kualitas: number | null;
	skor_waktu: number | null;
	skor_total: number | null;
	catatan_verifikator: string | null;
}

export interface HasilDetail extends Hasil {
	lampiran: Lampiran[];
}

export interface HasilRequest {
	rencana_id: string;
	uraian_pelaksanaan: string;
	progres_persen: number;
	jumlah_realisasi: number | null;
	kendala: string | null;
	tanggal_realisasi: string;
	ajukan: boolean;
}

export type UbahHasilRequest = Omit<HasilRequest, 'rencana_id' | 'ajukan'>;

// ------------------------------------------------------------------------ output

export type StatusOutput =
	| 'draft'
	| 'menunggu_verifikasi'
	| 'dalam_verifikasi'
	| 'disetujui'
	| 'perlu_revisi';

export interface Output {
	id: string;
	hasil_kerja_id: string;
	rencana_id: string;
	personel_id: string;
	personel_nama: string | null;
	personel_nrp: string | null;
	rencana_kegiatan: string | null;
	jenis_output: string;
	judul: string;
	uraian: string | null;
	tanggal_hasil: string;
	url_eksternal: string | null;
	status: StatusOutput;
	skor_kelengkapan: number | null;
	skor_kesesuaian: number | null;
	skor_keabsahan: number | null;
	skor_total: number | null;
	catatan_verifikator: string | null;
}

export interface OutputDetail extends Output {
	lampiran: Lampiran[];
}

export interface OutputRequest {
	hasil_kerja_id: string;
	jenis_output: string;
	judul: string;
	uraian: string | null;
	tanggal_hasil: string;
	url_eksternal: string | null;
	ajukan: boolean;
}

export type UbahOutputRequest = Omit<OutputRequest, 'hasil_kerja_id' | 'ajukan'>;

// --------------------------------------------------------------------- penilaian

export interface Periode {
	id: string;
	nama: string;
	tanggal_mulai: string;
	tanggal_selesai: string;
	status: 'open' | 'closed';
}

export interface Penilaian {
	id: string;
	personel_id: string;
	periode_id: string;
	personel_nama: string;
	personel_nrp: string;
	pangkat: string;
	jabatan: string;
	skor_absensi: number;
	skor_rencana_kerja: number;
	skor_hasil_kerja: number;
	skor_output: number;
	nilai_akhir: number;
	kategori: string;
	catatan_pimpinan: string | null;
	keputusan: string | null;
	computed_at: string;
}

export interface PenilaianDetailResponse {
	data: Penilaian;
	komponen: { indikator: string; skor: number }[];
}

// ------------------------------------------------------------------------ label

export const LABEL_STATUS_KEHADIRAN: Record<StatusKehadiran, string> = {
	dinas_kantor: 'Dinas Kantor',
	dinas_luar: 'Dinas Luar',
	ph: 'PH (Perintah Hadir)',
	piket: 'Piket / Shift',
	libur_nasional: 'Libur Nasional',
	cuti: 'Cuti',
	izin: 'Izin',
	sakit: 'Sakit',
	tanpa_keterangan: 'Tanpa Keterangan'
};

export const LABEL_STATUS_RENCANA: Record<StatusRencana, string> = {
	draft: 'Draft',
	diajukan: 'Diajukan',
	disetujui: 'Disetujui',
	perlu_revisi: 'Perlu Revisi',
	ditolak: 'Ditolak'
};

export const LABEL_JENIS_RENCANA: Record<string, string> = {
	harian: 'Harian',
	mingguan: 'Mingguan',
	bulanan: 'Bulanan',
	triwulan: 'Triwulan / Semester',
	penugasan_khusus: 'Penugasan Khusus'
};

export const LABEL_STATUS_HASIL: Record<StatusHasil, string> = {
	draft: 'Draft',
	diajukan: 'Menunggu Verifikasi',
	terverifikasi: 'Terverifikasi',
	perlu_revisi: 'Perlu Revisi'
};

export const LABEL_STATUS_OUTPUT: Record<StatusOutput, string> = {
	draft: 'Draft',
	menunggu_verifikasi: 'Menunggu Verifikasi',
	dalam_verifikasi: 'Dalam Verifikasi',
	disetujui: 'Disetujui',
	perlu_revisi: 'Perlu Revisi'
};

export const LABEL_JENIS_OUTPUT: Record<string, string> = {
	laporan: 'Laporan',
	dokumen_pendukung: 'Dokumen Pendukung',
	foto: 'Foto Kegiatan',
	video: 'Video Kegiatan',
	data_diagram: 'Data / Diagram',
	berita_acara: 'Berita Acara',
	surat_nota_dinas: 'Surat / Nota Dinas',
	link_sistem: 'Link / Sistem'
};

export const LABEL_KATEGORI: Record<string, string> = {
	sangat_baik: 'Sangat Baik',
	baik: 'Baik',
	cukup: 'Cukup',
	perlu_pembinaan: 'Perlu Pembinaan'
};

export const PANGKAT_POLRI = [
	'BHARADA',
	'BHARATU',
	'BHARAKA',
	'ABRIPDA',
	'ABRIPTU',
	'ABRIPKA',
	'BRIPDA',
	'BRIPTU',
	'BRIPKA',
	'AIPDA',
	'AIPTU',
	'IPDA',
	'IPTU',
	'AKP',
	'KOMPOL',
	'AKBP',
	'KOMBES POL',
	'BRIGJEN POL',
	'IRJEN POL',
	'KOMJEN POL',
	'JENDERAL POL'
];
