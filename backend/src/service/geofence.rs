use uuid::Uuid;

const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Jarak dua koordinat dengan rumus Haversine (meter).
pub fn haversine_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let d_phi = (lat2 - lat1).to_radians();
    let d_lambda = (lon2 - lon1).to_radians();

    let a = (d_phi / 2.0).sin().powi(2) + phi1.cos() * phi2.cos() * (d_lambda / 2.0).sin().powi(2);

    2.0 * EARTH_RADIUS_M * a.sqrt().asin()
}

#[derive(Debug, Clone)]
pub struct Titik {
    pub id: Uuid,
    pub nama: String,
    pub latitude: f64,
    pub longitude: f64,
    pub radius_m: i32,
    pub jenis: crate::domain::enums::TitikDinasJenis,
}

#[derive(Debug, Clone)]
pub struct HasilGeofence {
    pub titik_id: Option<Uuid>,
    pub titik_nama: Option<String>,
    pub jarak_m: Option<f64>,
    pub radius_m: Option<i32>,
    pub dalam_radius: bool,
}

/// Titik terdekat dan status radius.
///
/// Akurasi GPS (meter) dipakai sebagai toleransi: personel dianggap berada di
/// dalam radius bila `jarak - akurasi <= radius`. Ini mencegah penolakan palsu
/// saat GPS di dalam gedung menghasilkan akurasi buruk.
pub fn evaluasi(titik: &[Titik], lat: f64, lng: f64, akurasi_m: f64) -> HasilGeofence {
    let Some(terdekat) = titik
        .iter()
        .map(|t| {
            let jarak = haversine_m(lat, lng, t.latitude, t.longitude);
            (t, jarak)
        })
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    else {
        return HasilGeofence {
            titik_id: None,
            titik_nama: None,
            jarak_m: None,
            radius_m: None,
            dalam_radius: false,
        };
    };

    let (titik_terdekat, jarak) = terdekat;
    let toleransi = if akurasi_m.is_finite() && akurasi_m > 0.0 {
        akurasi_m
    } else {
        0.0
    };

    let dalam_radius = (jarak - toleransi) <= f64::from(titik_terdekat.radius_m);

    HasilGeofence {
        titik_id: Some(titik_terdekat.id),
        titik_nama: Some(titik_terdekat.nama.clone()),
        jarak_m: Some((jarak * 100.0).round() / 100.0),
        radius_m: Some(titik_terdekat.radius_m),
        dalam_radius,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::enums::TitikDinasJenis;

    fn titik(radius: i32) -> Titik {
        // Markas Polda Bali
        Titik {
            id: Uuid::nil(),
            nama: "Markas".into(),
            latitude: -8.65629,
            longitude: 115.22144,
            radius_m: radius,
            jenis: TitikDinasJenis::Kantor,
        }
    }

    #[test]
    fn jarak_titik_sama_adalah_nol() {
        let jarak = haversine_m(-8.65629, 115.22144, -8.65629, 115.22144);
        assert!(jarak < 0.001, "jarak = {jarak}");
    }

    #[test]
    fn di_dalam_radius() {
        // ~55 meter ke utara
        let hasil = evaluasi(&[titik(100)], -8.65579, 115.22144, 5.0);
        assert!(hasil.dalam_radius);
        assert!(hasil.jarak_m.unwrap() > 50.0 && hasil.jarak_m.unwrap() < 60.0);
    }

    #[test]
    fn di_luar_radius_tanpa_toleransi() {
        // ~1.1 km ke utara
        let hasil = evaluasi(&[titik(100)], -8.64629, 115.22144, 0.0);
        assert!(!hasil.dalam_radius);
        assert!(hasil.jarak_m.unwrap() > 1000.0);
    }

    #[test]
    fn akurasi_buruk_memberi_toleransi() {
        // ~150 meter ke utara, radius 100 m, akurasi 80 m => dianggap dalam radius
        let hasil = evaluasi(&[titik(100)], -8.65494, 115.22144, 80.0);
        assert!(hasil.dalam_radius);
    }

    #[test]
    fn tanpa_titik_tidak_pernah_dalam_radius() {
        let hasil = evaluasi(&[], -8.65629, 115.22144, 5.0);
        assert!(!hasil.dalam_radius);
        assert!(hasil.titik_id.is_none());
    }

    #[test]
    fn memilih_titik_terdekat() {
        let jauh = Titik {
            id: Uuid::from_u128(2),
            nama: "Luar".into(),
            latitude: -8.70,
            longitude: 115.22144,
            radius_m: 100,
            jenis: TitikDinasJenis::Kantor,
        };
        let hasil = evaluasi(&[jauh, titik(100)], -8.65629, 115.22144, 0.0);
        assert_eq!(hasil.titik_id, Some(Uuid::nil()));
    }
}
