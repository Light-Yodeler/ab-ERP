# Anda ERP - Desktop ERP & Sistem Pelaporan Pajak Daerah Ultra-Ringan

Sistem Desktop ERP Ultra-Ringan berbasis **Tauri v2 + Native Rust + SQLite C-Native (`rusqlite`)** yang direkayasa balik (*reverse engineering*) dari aplikasi pelaporan pajak lama berbasis Electron (~138 MB installer, ~200 MB RAM). Aplikasi ini dirancang modular untuk unit usaha **Anda Bungalows & Restaurant**, mencakup pelaporan pajak daerah (Dispenda/Bapenda Lombok Tengah), manajemen data transaksi mentah, pengarsipan otomatis berkas Excel, autentikasi PIN kriptografis, pencadangan database, dan modul rekonsiliasi audit target (Subset-Sum Optimizer).

---

## ⚡ Keunggulan & Benchmark

| Metrik | Hasil Aktual (Anda ERP) | Aplikasi Lama (Electron) | Peningkatan |
|---|---|---|---|
| **Ukuran Installer (.dmg macOS)** | **~3.4 - 4.2 MB** | ~138 MB | **~35x Lebih Ringan** |
| **Ukuran Bundle Aplikasi (.app)** | **~7.8 MB** | ~280 MB | **~36x Lebih Kompak** |
| **Konsumsi RAM (Idle / Aktif)** | **~25 - 35 MB** | 180 - 320 MB | **~8x Lebih Hemat RAM** |
| **Waktu Peluncuran (Cold Startup)**| **Instan (< 0.2 detik)** | 2.5 - 4.5 detik | **> 15x Lebih Cepat** |
| **Engine Database Lokal** | **SQLite C-Native (WAL Mode)** | In-Memory `sql.js` <!-- living-docs-ignore --> (raw RAM) | **Crash-Proof & ACID** |
| **Kecepatan Solver Audit Target** | **< 1 ms (0.80 ms)** | N/A (Manual kalkulator) | **Sub-milidetik** |

---
## 📚 Peta Dokumentasi

Dokumentasi proyek dikelola secara terstruktur menggunakan panduan living-docs:
* [README.md](README.md): Pengenalan sistem, benchmark performa, dan petunjuk lokal.
* [CHANGELOG.md](CHANGELOG.md): Riwayat rilis dan perubahan fitur berbasis Semantic Versioning.
* [docs/FEATURES.md](docs/FEATURES.md): Katalog lengkap fitur fungsional (`FEAT-01` s/d `FEAT-09`).
* [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): Arsitektur teknis, kontrak IPC, skema database, ADR, dan runbook pemulihan bencana.

---


## 🏗️ Arsitektur Proyek

Aplikasi dibangun tanpa beban runtime Node.js/Chromium ganda menggunakan **Tauri v2**, backend **Rust murni**, dan antarmuka **HTML5 / CSS / Vanilla JS** berstandar Financial Desktop Utility (WCAG AA kontras tinggi):

### 1. Backend Rust (`src-tauri/`)
- `src/db.rs`: Manajemen koneksi SQLite C-native (`rusqlite`) dengan mode WAL, migrasi skema tabel (`units`, `transactions`, `import_logs`, `settings`, `users`), auto-migrasi data lama dari `laporan_bapenda.sqlite`, paginasi SQL efisien, serta backup/restore online crash-safe.
- `src/auth.rs`: Hashing PIN berbasis PBKDF2 HMAC-SHA512 (10.000 iterasi) yang 100% kompatibel dengan hash database lama, verifikasi waktu-konstan, serta kontrol hak akses peran (`admin` dan `staff`).
- `src/excel_importer.rs`: Mesin spreadsheet cepat berbasis `calamine`, deteksi unit bisnis otomatis (Bungalows vs Restaurant), normalisasi tanggal (serial date Excel 1900, ISO, Indonesia) dan nominal rupiah, mode ganti periode / skip duplicate, serta auto-archiving berkas fisik ke `arsip_excel/`.
- `src/tax_report.rs`: Generator rekapitulasi kalender harian bulanan (hari 1 s/d 31), kalkulasi omset kotor, pajak daerah terutang 10%, pendapatan bersih, metrik hari aktif vs nihil, dan generator nomor dokumen resmi.
- `src/optimizer.rs` & `src/excel.rs`: Mesin rekonsiliasi subset transaksi acak target baris 80 (Subset Sum solver) berbasis *Parallel Multi-Start Randomized Greedy + 2-Opt Local Search* dan injeksi formula `=SUM(...)` via `umya-spreadsheet`.
- `src/commands.rs` & `src/lib.rs`: Registrasi seluruh Tauri IPC handler yang menghubungkan backend Rust ke frontend UI.

### 2. Frontend UI (`src/`)
- `index.html`: Layout aplikasi ERP desktop modern modular (Layar login PIN dengan keypad numerik, switcher unit bisnis global, 4 tab modul kerja, modal formulir A4 landscape resmi, dan modal pengaturan sistem).
- `style.css`: Estetika **Financial Desktop Utility** dengan dukungan penuh Dark & Light theme (WCAG AA kontras tinggi, dial ENERGY 1 / RHYTHM 1 / MOTION 1, serta media query `@media print` A4 landscape presisi).
- `app.js`: Logika interaktif modular, keyboard listener untuk PIN, rendering grafik bar SVG tren omset harian, dan paginasi data mentah.

---

## 🚀 Petunjuk Menjalankan & Membangun

### Prasyarat Mesin
- **Rust Toolchain:** `rustc` & `cargo` $\ge 1.75$ (`rustup update stable`)
- **Node.js:** $\ge 18$ (hanya untuk menjalankan script pembungkus Tauri CLI)

### 1. Menjalankan Mode Pengembangan (Dev Mode)
```bash
# Jalankan mode dev Tauri:
npm run dev
# atau langsung via Cargo:
cargo tauri dev
```

### 2. Membangun Installer Aplikasi (.dmg / .exe)
```bash
# Membangun bundle produksi teroptimasi:
npm run build
```
File installer otomatis dihasilkan di:
- **macOS:** `src-tauri/target/release/bundle/dmg/` (Ukuran ~3.4 - 4.2 MB)
- **Windows:** `src-tauri/target/release/bundle/nsis/` (Ukuran ~4 - 6 MB)

---

## 🧪 Pengujian & Verifikasi Otomatis

Seluruh logika kritis diuji menggunakan suite pengujian otomatis:
```bash
cd src-tauri
# Menjalankan seluruh test suite:
cargo test -- --nocapture

# Atau dalam mode rilis berkinerja tinggi:
cargo test --release -- --nocapture
```

### Lingkup Verifikasi:
1. `erp_core_tests`:
   - `test_auth_pbkdf2_legacy_compatibility`: Memverifikasi kecocokan 100% hash PBKDF2 HMAC-SHA512 dengan database lama.
   - `test_db_lifecycle_and_crud`: Menguji inisialisasi tabel, unit bawaan, pengaturan, operasi CRUD transaksi, dan paginasi SQL.
   - `test_tax_recap_report_computation`: Menguji pemetaan 31 hari kalender harian, perhitungan omset, pajak 10%, dan nomor dokumen resmi.
   - `test_backup_and_disaster_recovery`: Menguji backup online SQLite, pembuatan salinan darurat `.pre_restore.bak`, dan pemulihan data utuh.
2. `optimizer_tests` & `smoke_verify`:
   - Solver rekonsiliasi subset sum, ekspor in-place ke kolom E & F file Excel, serta injeksi formula dinamis `=SUM(F2:F...)`.

---

## 📄 Lisensi
Hak Cipta (c) 2024 Muhammad Syahid Ramadhon. Dilindungi undang-undang.
