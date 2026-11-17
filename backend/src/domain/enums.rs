use serde::{Deserialize, Serialize};

macro_rules! pg_enum {
    ($name:ident, $type_name:literal, [$($variant:ident => $label:literal),* $(,)?]) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            Serialize,
            Deserialize,
            sqlx::Type,
        )]
        #[serde(rename_all = "snake_case")]
        #[sqlx(type_name = $type_name, rename_all = "snake_case")]
        pub enum $name {
            $($variant),*
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),*];

            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $label),*
                }
            }
        }
    };
}

pg_enum!(SatkerLevel, "satker_level", [Polda => "polda", Polres => "polres", Satker => "satker"]);

/// Jenis akun pada tabel `personel`.
///
/// `Admin` adalah pengelola sistem (login lewat dashboard dengan username),
/// `Personel` adalah pegawai yang wajib absen (login dengan NRP, termasuk
/// pimpinan dan atasan).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type, Default)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum JenisAkun {
    Admin,
    #[default]
    Personel,
}

impl JenisAkun {
    pub fn is_admin(self) -> bool {
        matches!(self, Self::Admin)
    }
}

/// Kanal login yang dikirim klien.
///
/// Akun admin hanya boleh masuk lewat `web`; aplikasi Android hanya menerima
/// akun personel (termasuk pimpinan/atasan).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KanalLogin {
    #[default]
    Web,
    Android,
}

pg_enum!(
    AppRole,
    "app_role",
    [
        AdminSistem => "admin_sistem",
        AdminPolda => "admin_polda",
        AdminSatkerPolda => "admin_satker_polda",
        AdminPolres => "admin_polres",
        AdminSatkerPolres => "admin_satker_polres",
        Personel => "personel",
        Atasan => "atasan",
        Pimpinan => "pimpinan",
        PimpinanPolda => "pimpinan_polda",
        PimpinanSatkerPolda => "pimpinan_satker_polda",
        PimpinanPolres => "pimpinan_polres",
        PimpinanSatkerPolres => "pimpinan_satker_polres",
    ]
);

impl AppRole {
    /// Admin global (super admin) yang boleh mengelola seluruh data.
    pub fn admin_global(self) -> bool {
        matches!(self, Self::AdminSistem)
    }

    /// Semua peran admin, termasuk admin ber-scope.
    pub fn admin_any(self) -> bool {
        matches!(
            self,
            Self::AdminSistem
                | Self::AdminPolda
                | Self::AdminSatkerPolda
                | Self::AdminPolres
                | Self::AdminSatkerPolres
        )
    }

    /// Semua peran pimpinan, termasuk pimpinan ber-scope.
    pub fn pimpinan_any(self) -> bool {
        matches!(
            self,
            Self::Pimpinan
                | Self::PimpinanPolda
                | Self::PimpinanSatkerPolda
                | Self::PimpinanPolres
                | Self::PimpinanSatkerPolres
        )
    }

    pub fn butuh_scope(self) -> bool {
        matches!(
            self,
            Self::AdminPolda
                | Self::AdminSatkerPolda
                | Self::AdminPolres
                | Self::AdminSatkerPolres
                | Self::PimpinanPolda
                | Self::PimpinanSatkerPolda
                | Self::PimpinanPolres
                | Self::PimpinanSatkerPolres
        )
    }
}

pg_enum!(AbsensiJenis, "absensi_jenis", [Masuk => "masuk", Pulang => "pulang"]);

pg_enum!(
    StatusKehadiran,
    "status_kehadiran",
    [
        DinasKantor => "dinas_kantor",
        DinasLuar => "dinas_luar",
        Ph => "ph",
        Piket => "piket",
        LiburNasional => "libur_nasional",
        Cuti => "cuti",
        Izin => "izin",
        Sakit => "sakit",
        TanpaKeterangan => "tanpa_keterangan",
    ]
);

impl StatusKehadiran {
    /// Kehadiran yang dihitung sebagai hari kerja efektif (masuk penyebut).
    pub fn dihitung_hadir(&self) -> bool {
        matches!(
            self,
            Self::DinasKantor | Self::DinasLuar | Self::Ph | Self::Piket
        )
    }

    /// Status yang menetralkan hari tersebut (dikeluarkan dari penyebut).
    pub fn netral(&self) -> bool {
        matches!(
            self,
            Self::LiburNasional | Self::Cuti | Self::Izin | Self::Sakit
        )
    }

    pub fn butuh_radius(&self) -> bool {
        matches!(self, Self::DinasKantor | Self::Piket)
    }
}

pg_enum!(TitikDinasJenis, "titik_dinas_jenis", [Kantor => "kantor", Luar => "luar"]);

pg_enum!(
    JenisLibur,
    "jenis_libur",
    [
        Nasional => "nasional",
        CutiBersama => "cuti_bersama",
        Khusus => "khusus",
    ]
);

pg_enum!(
    JenisRencana,
    "jenis_rencana",
    [
        Harian => "harian",
        Mingguan => "mingguan",
        Bulanan => "bulanan",
        Triwulan => "triwulan",
        PenugasanKhusus => "penugasan_khusus",
    ]
);

pg_enum!(
    DasarTugasJenis,
    "dasar_tugas_jenis",
    [Sp => "sp", Tor => "tor", PerintahLisan => "perintah_lisan"]
);

pg_enum!(
    StatusRencana,
    "status_rencana",
    [
        Draft => "draft",
        Diajukan => "diajukan",
        Disetujui => "disetujui",
        PerluRevisi => "perlu_revisi",
        Ditolak => "ditolak",
    ]
);

pg_enum!(
    StatusHasil,
    "status_hasil",
    [
        Draft => "draft",
        Diajukan => "diajukan",
        Terverifikasi => "terverifikasi",
        PerluRevisi => "perlu_revisi",
    ]
);

pg_enum!(
    JenisOutput,
    "jenis_output",
    [
        Laporan => "laporan",
        DokumenPendukung => "dokumen_pendukung",
        Foto => "foto",
        Video => "video",
        DataDiagram => "data_diagram",
        BeritaAcara => "berita_acara",
        SuratNotaDinas => "surat_nota_dinas",
        LinkSistem => "link_sistem",
    ]
);

impl JenisOutput {
    /// Jenis output yang wajib menyertakan berkas unggahan.
    pub fn wajib_berkas(&self) -> bool {
        !matches!(self, Self::LinkSistem)
    }
}

pg_enum!(
    StatusOutput,
    "status_output",
    [
        Draft => "draft",
        MenungguVerifikasi => "menunggu_verifikasi",
        DalamVerifikasi => "dalam_verifikasi",
        Disetujui => "disetujui",
        PerluRevisi => "perlu_revisi",
    ]
);

pg_enum!(
    BerkasOwner,
    "berkas_owner",
    [
        Rencana => "rencana",
        Hasil => "hasil",
        Output => "output",
        Absensi => "absensi",
    ]
);

pg_enum!(
    BerkasKategori,
    "berkas_kategori",
    [Lampiran => "lampiran", Bukti => "bukti", Foto => "foto"]
);

pg_enum!(PeriodeStatus, "periode_status", [Open => "open", Closed => "closed"]);

pg_enum!(
    IndikatorKinerja,
    "indikator_kinerja",
    [
        Absensi => "absensi",
        RencanaKerja => "rencana_kerja",
        HasilKerja => "hasil_kerja",
        Output => "output",
    ]
);

pg_enum!(
    KategoriKinerja,
    "kategori_kinerja",
    [
        SangatBaik => "sangat_baik",
        Baik => "baik",
        Cukup => "cukup",
        PerluPembinaan => "perlu_pembinaan",
    ]
);

impl KategoriKinerja {
    pub fn dari_nilai(nilai: f64) -> Self {
        if nilai >= 95.0 {
            Self::SangatBaik
        } else if nilai >= 90.0 {
            Self::Baik
        } else if nilai >= 80.0 {
            Self::Cukup
        } else {
            Self::PerluPembinaan
        }
    }
}

pg_enum!(
    KeputusanPenilaian,
    "keputusan_penilaian",
    [Disetujui => "disetujui", PerluPerbaikan => "perlu_perbaikan"]
);
