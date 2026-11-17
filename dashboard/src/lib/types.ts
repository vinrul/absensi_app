export type Role =
	| 'admin_sistem'
	| 'admin_polda'
	| 'admin_satker_polda'
	| 'admin_polres'
	| 'admin_satker_polres'
	| 'pimpinan'
	| 'pimpinan_polda'
	| 'pimpinan_satker_polda'
	| 'pimpinan_polres'
	| 'pimpinan_satker_polres'
	| 'atasan'
	| 'personel';

export type SatkerLevel = 'polda' | 'polres' | 'satker';

export interface RoleScope {
	role: Role;
	scope_satker_id: string | null;
	scope_satker_nama: string | null;
	scope_level: SatkerLevel | null;
}

export interface Satker {
	id: string;
	parent_id: string | null;
	level: SatkerLevel;
	nama: string;
	kode: string;
	alamat: string | null;
	timezone: string;
	hari_kerja: number[];
	jam_masuk: string;
	jam_pulang: string;
	jam_pulang_sabtu: string | null;
	is_active: boolean;
}

export interface AdminAkun {
	id: string;
	username: string;
	nama: string;
	satker_id: string;
	satker_nama: string;
	satker_level: SatkerLevel;
	is_active: boolean;
	must_change_password: boolean;
	role: RoleScope | null;
}

export interface Periode {
	id: string;
	nama: string;
	tanggal_mulai: string;
	tanggal_selesai: string;
	status: 'open' | 'closed';
	dibuka_kembali_at: string | null;
	alasan_buka_kembali: string | null;
}

export interface KesiapanSatker {
	satker_id: string;
	satker_nama: string;
	satker_level: SatkerLevel;
	jumlah_personel: number;
	sudah_dinilai: number;
	belum_dinilai: number;
	siap_ditutup: boolean;
	ditandai_oleh: string | null;
	ditandai_oleh_nama: string | null;
	ditandai_at: string | null;
	catatan: string | null;
}

export interface KesiapanResponse {
	periode_id: string;
	periode_nama: string;
	periode_status: 'open' | 'closed';
	data: KesiapanSatker[];
	ringkasan: {
		total_satker: number;
		siap: number;
		belum_siap: number;
		total_personel: number;
		sudah_dinilai: number;
		belum_dinilai: number;
	};
}

export type JenisLibur = 'nasional' | 'cuti_bersama' | 'khusus';

export interface HariLibur {
	id: string;
	tanggal: string;
	nama: string;
	jenis: JenisLibur;
	satker_id: string | null;
	satker_nama: string | null;
	is_active: boolean;
}

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
	personel_id?: string;
	satker_id?: string;
	personel_nama?: string;
	personel_nrp?: string;
	personel_pangkat?: string;
	unit?: string | null;
	tanggal: string;
	jenis: 'masuk' | 'pulang';
	status_kehadiran: StatusKehadiran;
	jam: string;
	jarak_m?: number | null;
	dalam_radius?: boolean;
	is_offline?: boolean;
	is_mock_suspect?: boolean;
	perlu_verifikasi?: boolean;
	verified_at?: string | null;
	catatan_verifikator?: string | null;
	foto_url?: string | null;
}

/** Satu baris per personel per tanggal: absen masuk dan pulang digabung. */
export interface AbsensiHarian {
	personel_id: string;
	satker_id: string;
	personel_nama: string;
	personel_nrp: string;
	personel_pangkat: string;
	unit: string | null;
	tanggal: string;
	masuk_jam: string | null;
	pulang_jam: string | null;
	status_masuk: StatusKehadiran | null;
	status_pulang: StatusKehadiran | null;
	is_offline: boolean;
	is_mock_suspect: boolean;
	luar_radius: boolean;
	perlu_verifikasi: boolean;
	perlu_verifikasi_ids: string[];
	verified_at: string | null;
	catatan_verifikator: string | null;
	menit_telat: number;
	foto_url: string | null;
}

export interface RekapAbsensi {
	personel_id: string;
	nama: string;
	nrp: string;
	pangkat: string;
	jabatan: string;
	unit: string | null;
	hadir: number;
	dinas_luar: number;
	piket: number;
	telat: number;
	menit_telat: number;
	cuti: number;
	izin: number;
	sakit: number;
	libur_nasional: number;
	tanpa_keterangan: number;
	menunggu_verifikasi: number;
}

export interface TitikDinas {
	id: string;
	satker_id: string;
	nama: string;
	jenis: 'kantor' | 'luar';
	latitude: number;
	longitude: number;
	radius_m: number;
	is_active: boolean;
}

export type StatusRencana = 'draft' | 'diajukan' | 'disetujui' | 'perlu_revisi' | 'ditolak';

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

export interface Tahapan {
	id: string;
	urutan: number;
	uraian: string;
	target_kuantitas: number | null;
	satuan: string | null;
	bobot: number | null;
	program?: string | null;
	kegiatan?: string | null;
	tujuan?: string | null;
	indikator_kinerja?: string | null;
	target_kualitas?: string | null;
	tanggal_mulai?: string | null;
	tanggal_selesai?: string | null;
}

export interface RencanaDetail extends Rencana {
	tahapan: Tahapan[];
	lampiran: Lampiran[];
}

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

export type Kategori = 'sangat_baik' | 'baik' | 'cukup' | 'perlu_pembinaan';

export interface Penilaian {
	id: string;
	personel_id: string;
	periode_id: string;
	satker_id: string;
	personel_nama: string;
	personel_nrp: string;
	pangkat: string;
	jabatan: string;
	unit: string | null;
	skor_absensi: number;
	skor_rencana_kerja: number;
	skor_hasil_kerja: number;
	skor_output: number;
	nilai_akhir: number;
	kategori: Kategori;
	catatan_pimpinan: string | null;
	keputusan: 'disetujui' | 'perlu_perbaikan' | null;
	computed_at: string;
}

export interface Personel {
	id: string;
	nrp: string;
	nama: string;
	pangkat: string;
	jabatan: string;
	unit: string | null;
	personel_nama: string;
	personel_nrp: string;
	satker_id: string;
	satker_nama: string;
	satker_level: SatkerLevel;
	atasan_id: string | null;
	atasan_nama: string | null;
	email: string | null;
	phone: string | null;
	is_active: boolean;
	roles: RoleScope[];
}

export interface RingkasanDashboard {
	periode_id: string;
	periode_nama: string;
	periode: string;
	total_personel: number;
	hadir_hari_ini: number;
	persen_hadir: number;
	total_rencana: number;
	total_hasil: number;
	total_output: number;
	rata_rata_nilai: number;
}

export interface ListResponse<T> {
	data: T[];
	meta: { page: number; per_page: number; total: number };
}

export const LABEL_STATUS_KEHADIRAN: Record<StatusKehadiran, string> = {
	dinas_kantor: 'Dinas Kantor',
	dinas_luar: 'Dinas Luar',
	ph: 'PH (Perintah Hadir)',
	piket: 'Piket/Shift',
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
	triwulan: 'Triwulan',
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

export const LABEL_KATEGORI: Record<Kategori, string> = {
	sangat_baik: 'Sangat Baik',
	baik: 'Baik',
	cukup: 'Cukup',
	perlu_pembinaan: 'Perlu Pembinaan'
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

// ------------------------------------------------------------------ PH module

export interface StrongPoint {
	id: string;
	satker_id: string;
	nama: string;
	alamat: string | null;
	latitude: number;
	longitude: number;
	radius_m: number;
	is_active: boolean;
}

export interface PersonelJadwalPh {
	personel_id: string;
	nama: string;
	nrp: string;
	pangkat: string;
	unit: string | null;
	peran: string | null;
}

export interface JadwalPh {
	id: string;
	satker_id: string;
	strong_point_id: string;
	strong_point_nama: string;
	tanggal: string;
	jam_mulai: string;
	jam_selesai: string;
	catatan: string | null;
	jumlah_personel: number;
	personel: PersonelJadwalPh[];
}

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
	jenis: 'masuk' | 'pulang';
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

export interface RekapPh {
	personel_id: string;
	nama: string;
	nrp: string;
	pangkat: string;
	unit: string | null;
	jumlah_jadwal: number;
	hadir: number;
	telat: number;
	tidak_hadir: number;
	menunggu_verifikasi: number;
}
