@echo off
setlocal
cd /d "%~dp0"

if not exist "keystore.properties" (
    echo [GAGAL] android\keystore.properties tidak ditemukan.
    echo Salin keystore.properties.example menjadi keystore.properties lalu isi nilainya.
    exit /b 1
)

echo == Build APK release (signed) dan AAB ==
call gradlew.bat clean :app:assembleRelease :app:bundleRelease
if errorlevel 1 (
    echo.
    echo [GAGAL] Build release. Periksa pesan Gradle di atas.
    exit /b 1
)

echo.
echo Selesai. Berkas hasil build:
echo   APK : app\build\outputs\apk\release\app-release.apk
echo   AAB : app\build\outputs\bundle\release\app-release.aab
