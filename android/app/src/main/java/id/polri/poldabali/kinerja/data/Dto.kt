package id.polri.poldabali.kinerja.data

import kotlinx.serialization.Serializable

// ---------------------------------------------------------------- autentikasi

@Serializable
data class LoginRequest(
    val nrp: String,
    val password: String,
    val deviceId: String? = null,
    val client: String = "android"
)

@Serializable
data class Pengguna(
    val id: String,
    val nrp: String? = null,
    val username: String? = null,
    val jenisAkun: String = "personel",
    val nama: String,
    val pangkat: String,
    val jabatan: String,
    val satkerId: String,
    val satkerNama: String,
    val satkerLevel: String,
    val roles: List<String> = emptyList(),
    val mustChangePassword: Boolean = false,
    val atasanId: String? = null,
    val email: String? = null,
    val phone: String? = null,
    val fotoUrl: String? = null
) {
    val peranUtama: String
        get() = when {
            roles.any { it.startsWith("pimpinan") } -> "Pimpinan"
            roles.any { it.startsWith("admin") } -> "Admin"
            roles.contains("atasan") -> "Atasan Langsung"
            else -> "Personel"
        }
}

@Serializable
data class LoginResponse(
    val accessToken: String,
    val refreshToken: String,
    val tokenType: String = "Bearer",
    val expiresIn: Long = 900,
    val user: Pengguna
)

@Serializable
data class TokenResponse(
    val accessToken: String,
    val refreshToken: String,
    val tokenType: String = "Bearer",
    val expiresIn: Long = 900
)

@Serializable
data class UbahProfilRequest(
    val nama: String? = null,
    val pangkat: String? = null,
    val jabatan: String? = null,
    val phone: String? = null,
    val email: String? = null
)

@Serializable
data class UbahPasswordRequest(
    val oldPassword: String,
    val newPassword: String
)

@Serializable
data class ApiErrorBody(val error: ApiErrorDetail)

@Serializable
data class ApiErrorDetail(val code: String = "error", val message: String = "Terjadi kesalahan")

// ------------------------------------------------------------------- absensi

@Serializable
data class Absensi(
    val id: String,
    val tanggal: String,
    val jenis: String,
    val statusKehadiran: String,
    val jam: String,
    val jarakM: Double? = null,
    val dalamRadius: Boolean = true,
    val isOffline: Boolean = false,
    val isMockSuspect: Boolean = false,
    val perluVerifikasi: Boolean = false,
    val catatanVerifikator: String? = null,
    val fotoUrl: String? = null
)

@Serializable
data class AbsensiRequest(
    val clientUuid: String,
    val jenis: String,
    val statusKehadiran: String,
    val capturedAt: String,
    val latitude: Double? = null,
    val longitude: Double? = null,
    val akurasiM: Double? = null,
    val isMockSuspect: Boolean = false,
    val isOffline: Boolean = false,
    val deviceId: String? = null,
    val fotoBase64: String? = null
)

@Serializable
data class AbsensiHariIni(
    val tanggal: String,
    val masuk: Absensi? = null,
    val pulang: Absensi? = null
)

@Serializable
data class AbsensiResponse(
    val data: Absensi,
    val geofence: Geofence,
    val serverTime: String
)

@Serializable
data class Geofence(
    val titikNama: String? = null,
    val jarakM: Double? = null,
    val radiusM: Int? = null,
    val dalamRadius: Boolean = false
)

@Serializable
data class TitikDinas(
    val id: String,
    val nama: String,
    val jenis: String = "kantor",
    val latitude: Double,
    val longitude: Double,
    val radiusM: Int = 100,
    val jarakM: Double? = null,
    val isActive: Boolean = true
)

@Serializable
data class TitikTerdekatResponse(
    val titik: List<TitikDinas> = emptyList(),
    val terdekat: Geofence = Geofence()
)

/** Entri antrean absensi offline. */
@Serializable
data class AbsensiAntrean(
    val clientUuid: String,
    val jenis: String,
    val statusKehadiran: String,
    val capturedAt: String,
    val latitude: Double? = null,
    val longitude: Double? = null,
    val akurasiM: Double? = null,
    val isMockSuspect: Boolean = false,
    val fotoBase64: String? = null,
    val dibuatPada: Long = System.currentTimeMillis()
)

// --------------------------------------------------------------- rencana kerja

@Serializable
data class Rencana(
    val id: String,
    val personelId: String = "",
    val program: String,
    val kegiatan: String,
    val uraian: String = "",
    val tujuan: String? = null,
    val indikatorKinerja: String? = null,
    val jenis: String = "bulanan",
    val dasarJenis: String? = null,
    val dasarNomor: String? = null,
    val dasarTanggal: String? = null,
    val dasarPerihal: String? = null,
    val targetKuantitas: Double? = null,
    val satuan: String? = null,
    val targetKualitas: String? = null,
    val targetWaktu: String? = null,
    val tanggalMulai: String,
    val tanggalSelesai: String,
    val status: String = "draft",
    val catatanAtasan: String? = null,
    val jumlahRevisi: Int = 0,
    val personelNama: String? = null,
    val personelNrp: String? = null,
    val unit: String? = null,
    val tahapan: List<Tahapan> = emptyList(),
    val lampiran: List<Lampiran> = emptyList()
)

@Serializable
data class Tahapan(
    val id: String,
    val urutan: Int = 0,
    val uraian: String,
    val targetKuantitas: Double? = null,
    val satuan: String? = null,
    val bobot: Double? = null,
    val program: String? = null,
    val kegiatan: String? = null,
    val tujuan: String? = null,
    val indikatorKinerja: String? = null,
    val targetKualitas: String? = null,
    val tanggalMulai: String? = null,
    val tanggalSelesai: String? = null
)

@Serializable
data class Lampiran(
    val id: String,
    val ownerType: String = "",
    val ownerId: String = "",
    val kategori: String = "",
    val fileName: String = "",
    val mime: String = "",
    val sizeBytes: Long = 0,
    val storageKey: String = "",
    val createdAt: String = ""
)

@Serializable
data class RencanaRequest(
    val periodeId: String? = null,
    val program: String,
    val kegiatan: String,
    val uraian: String,
    val tujuan: String? = null,
    val indikatorKinerja: String? = null,
    val jenis: String = "bulanan",
    val dasarJenis: String? = null,
    val dasarNomor: String? = null,
    val dasarTanggal: String? = null,
    val dasarPerihal: String? = null,
    val targetKuantitas: Double? = null,
    val satuan: String? = null,
    val targetKualitas: String? = null,
    val targetWaktu: String? = null,
    val tanggalMulai: String,
    val tanggalSelesai: String,
    val tahapan: List<TahapanRequest> = emptyList(),
    val ajukan: Boolean = true
)

@Serializable
data class KeputusanRequest(
    val status: String,
    val catatan: String? = null
)

@Serializable
data class UbahRencanaRequest(
    val program: String,
    val kegiatan: String,
    val uraian: String,
    val tujuan: String? = null,
    val indikatorKinerja: String? = null,
    val jenis: String = "bulanan",
    val dasarJenis: String? = null,
    val dasarNomor: String? = null,
    val dasarTanggal: String? = null,
    val dasarPerihal: String? = null,
    val targetKuantitas: Double? = null,
    val satuan: String? = null,
    val targetKualitas: String? = null,
    val targetWaktu: String? = null,
    val tanggalMulai: String,
    val tanggalSelesai: String,
    val tahapan: List<TahapanRequest> = emptyList()
)

@Serializable
data class TahapanRequest(
    val uraian: String,
    val targetKuantitas: Double? = null,
    val satuan: String? = null,
    val bobot: Double? = null,
    val program: String? = null,
    val kegiatan: String? = null,
    val tujuan: String? = null,
    val indikatorKinerja: String? = null,
    val targetKualitas: String? = null,
    val tanggalMulai: String? = null,
    val tanggalSelesai: String? = null
)

// ----------------------------------------------------------------- hasil kerja

@Serializable
data class Hasil(
    val id: String,
    val rencanaId: String,
    val uraianPelaksanaan: String,
    val progresPersen: Double = 0.0,
    val jumlahRealisasi: Double? = null,
    val kendala: String? = null,
    val tanggalRealisasi: String,
    val status: String = "diajukan",
    val skorKuantitas: Double? = null,
    val skorKualitas: Double? = null,
    val skorWaktu: Double? = null,
    val skorTotal: Double? = null,
    val catatanVerifikator: String? = null,
    val rencanaKegiatan: String? = null,
    val personelNama: String? = null,
    val personelNrp: String? = null,
    val lampiran: List<Lampiran> = emptyList()
)

@Serializable
data class HasilRequest(
    val rencanaId: String,
    val uraianPelaksanaan: String,
    val progresPersen: Double,
    val jumlahRealisasi: Double? = null,
    val kendala: String? = null,
    val tanggalRealisasi: String,
    val ajukan: Boolean = true
)

@Serializable
data class UbahHasilRequest(
    val uraianPelaksanaan: String,
    val progresPersen: Double,
    val jumlahRealisasi: Double? = null,
    val kendala: String? = null,
    val tanggalRealisasi: String
)

@Serializable
data class VerifikasiHasilRequest(
    val setuju: Boolean = true,
    val skorKualitas: Double? = null,
    val catatan: String? = null
)

// ---------------------------------------------------------------------- output

@Serializable
data class Output(
    val id: String,
    val hasilKerjaId: String = "",
    val rencanaId: String = "",
    val rencanaKegiatan: String? = null,
    val personelNama: String? = null,
    val personelNrp: String? = null,
    val jenisOutput: String,
    val judul: String,
    val uraian: String? = null,
    val tanggalHasil: String,
    val urlEksternal: String? = null,
    val status: String = "menunggu_verifikasi",
    val skorKelengkapan: Double? = null,
    val skorKesesuaian: Double? = null,
    val skorKeabsahan: Double? = null,
    val skorTotal: Double? = null,
    val catatanVerifikator: String? = null,
    val lampiran: List<Lampiran> = emptyList()
)

@Serializable
data class OutputRequest(
    val hasilKerjaId: String,
    val jenisOutput: String,
    val judul: String,
    val uraian: String? = null,
    val tanggalHasil: String,
    val urlEksternal: String? = null,
    val ajukan: Boolean = true
)

@Serializable
data class UbahOutputRequest(
    val jenisOutput: String,
    val judul: String,
    val uraian: String? = null,
    val tanggalHasil: String,
    val urlEksternal: String? = null
)

@Serializable
data class VerifikasiOutputRequest(
    val setuju: Boolean = true,
    val skorKesesuaian: Double? = null,
    val skorKeabsahan: Double? = null,
    val catatan: String? = null
)

// ------------------------------------------------------------------- penilaian

@Serializable
data class Periode(
    val id: String,
    val nama: String,
    val tanggalMulai: String,
    val tanggalSelesai: String,
    val status: String = "open"
)

@Serializable
data class Penilaian(
    val id: String,
    val personelId: String = "",
    val personelNama: String = "",
    val personelNrp: String = "",
    val pangkat: String = "",
    val jabatan: String = "",
    val unit: String? = null,
    val skorAbsensi: Double = 0.0,
    val skorRencanaKerja: Double = 0.0,
    val skorHasilKerja: Double = 0.0,
    val skorOutput: Double = 0.0,
    val nilaiAkhir: Double = 0.0,
    val kategori: String = "cukup",
    val catatanPimpinan: String? = null,
    val keputusan: String? = null,
    val computedAt: String = ""
)

@Serializable
data class PenilaianDetailResponse(
    val data: Penilaian,
    val komponen: List<KomponenNilai> = emptyList()
)

@Serializable
data class KomponenNilai(
    val indikator: String,
    val skor: Double = 0.0
)

// ----------------------------------------------------------------------- PH

@Serializable
data class JadwalPhSaya(
    val id: String,
    val satkerId: String,
    val strongPointId: String,
    val strongPointNama: String,
    val latitude: Double,
    val longitude: Double,
    val radiusM: Int = 100,
    val tanggal: String,
    val jamMulai: String,
    val jamSelesai: String,
    val regu: String? = null,
    val catatan: String? = null,
    val peran: String? = null
)

@Serializable
data class AbsensiPh(
    val id: String,
    val jadwalPhId: String,
    val personelId: String,
    val personelNama: String = "",
    val personelNrp: String = "",
    val strongPointId: String,
    val strongPointNama: String = "",
    val satkerId: String = "",
    val jenis: String,
    val capturedAt: String = "",
    val jam: String,
    val latitude: Double? = null,
    val longitude: Double? = null,
    val akurasiM: Double? = null,
    val jarakM: Double? = null,
    val dalamRadius: Boolean = false,
    val fotoUrl: String? = null,
    val isOffline: Boolean = false,
    val isMockSuspect: Boolean = false,
    val perluVerifikasi: Boolean = false,
    val verifiedAt: String? = null,
    val catatanVerifikator: String? = null
)

@Serializable
data class AbsensiPhRequest(
    val clientUuid: String,
    val jadwalPhId: String,
    val jenis: String,
    val capturedAt: String,
    val latitude: Double? = null,
    val longitude: Double? = null,
    val akurasiM: Double? = null,
    val isMockSuspect: Boolean = false,
    val isOffline: Boolean = false,
    val fotoBase64: String? = null
)

// --------------------------------------------------------------------- umum

@Serializable
data class Meta(val page: Int = 1, val perPage: Int = 20, val total: Long = 0)

@Serializable
data class ListResponse<T>(
    val data: List<T> = emptyList(),
    val meta: Meta = Meta()
)
