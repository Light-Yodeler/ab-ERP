# Anda ERP

Anda ERP (v1.6.5) adalah aplikasi desktop untuk pencatatan pendapatan harian, pelaporan pajak daerah, dan pendataan wisatawan pada unit usaha Anda Bungalows dan Anda Restaurant di Lombok Tengah. Aplikasi berjalan lokal tanpa server eksternal di atas Tauri v2, backend Rust, SQLite via `rusqlite` dalam mode WAL, serta antarmuka HTML5, CSS, dan JavaScript.

---

## Fitur Utama

* Rekapitulasi pajak daerah bulanan menyusun kalender harian dari tanggal 1 hingga akhir bulan, menghitung pajak daerah sesuai tarif unit atau bawaan 10%, menampilkan grafik batang SVG, dan menyediakan cetak A4 landscape serta ekspor CSV.
* Pendataan Wisatawan Form 1 Dinas Pariwisata membagi periode bulan ke empat minggu, memisahkan tamu domestik dan mancanegara, serta menghitung okupansi malam kamar (*room-night*) dan tempat tidur (*bed-night*).
* Buku besar data mentah mendukung tampilan ringkas 3 kolom dan rincian reservasi 13 kolom dengan paginasi SQL, penyuntingan serta penghapusan reservasi khusus admin yang tersinkron ke transaksi dan Form 1, serta ekspor `.xlsx` 13 kolom.
* Pengimpor spreadsheet berbasis `calamine` membaca berkas `.xlsx`, `.xls`, dan `.xlsm`, mendeteksi unit bisnis, memeriksa duplikat lewat hash SHA-256, menormalkan beragam format tanggal (`DD/MM/YY`, `DD/MM-YY`, `DD.MM.YYYY`, `YYYY-MM-DD`, ISO timestamp, serial Excel, dan teks bulan Indonesia atau Inggris) ke penyimpanan kanonik `YYYY-MM-DD`, membatasi mode timpa hanya bagi admin, dan mengarsipkan salinan berkas ke folder data lokal `arsip_excel/`.
* Audit Target Solver menggunakan algoritma subset-sum paralel (Rayon) untuk berkas 2 kolom lama maupun 13 kolom reservasi, menjaga agar dua transaksi pada kamar dan malam yang sama tidak terpilih bersamaan tanpa mengubah nomor kamar asli, menampilkan peringatan konflik sumber serta anomali kamar atau tanggal dengan tampilan seragam `DD/MM/YY`, dan mengekspor `.xlsx` berumus dinamis `H`/`I`/`K` berformat `dd/mm/yy` beserta sheet `Referensi & Petunjuk`.
* Autentikasi lokal memverifikasi PIN pengguna dengan PBKDF2-HMAC-SHA512 (10.000 iterasi) dan kontrol peran `admin` serta `staff`, dilengkapi backup online SQLite dan pemulihan database yang membuat cadangan `.pre_restore.bak` sebelum menimpa database aktif.

---

## Struktur Proyek

| Berkas / Direktori | Peran dalam Sistem |
|---|---|
| [src-tauri/src/db.rs](src-tauri/src/db.rs) | Koneksi SQLite WAL (`rusqlite`), skema 6 tabel (`units`, `transactions`, `settings`, `import_logs`, `users`, `room_reservations`), paginasi SQL, CRUD tersinkronisasi, dan backup/restore. |
| [src-tauri/src/auth.rs](src-tauri/src/auth.rs) | Hashing dan verifikasi PIN PBKDF2-HMAC-SHA512 waktu-konstan serta pengelolaan akun peran `admin` dan `staff`. |
| [src-tauri/src/excel_importer.rs](src-tauri/src/excel_importer.rs) | Pratinjau impor, deteksi duplikat SHA-256, parser tanggal fleksibel multi-format, dan penulisan atomik ke SQLite. |
| [src-tauri/src/tax_report.rs](src-tauri/src/tax_report.rs) | Kalkulasi rekap pajak bulanan dan statistik Form 1 Dinas Pariwisata untuk asal domestik maupun mancanegara. |
| [src-tauri/src/optimizer.rs](src-tauri/src/optimizer.rs) | Solver subset-sum paralel (Rayon), deteksi konflik malam-kamar, dan pemeriksaan anomali kamar/tanggal. |
| [src-tauri/src/excel.rs](src-tauri/src/excel.rs) | Analisis workbook 2 kolom/13 kolom (`DD/MM/YY`), ekspor bersih 13 kolom dan rekap Ledger via `umya-spreadsheet`, serta generator template unduhan. |
| [src-tauri/src/commands.rs](src-tauri/src/commands.rs) & [src-tauri/src/lib.rs](src-tauri/src/lib.rs) | Registrasi command IPC Tauri, penjagaan sesi dan otorisasi `admin`, serta konfigurasi dialog sistem. |
| [src/index.html](src/index.html), [src/style.css](src/style.css), [src/app.js](src/app.js) | Antarmuka desktop 5 tab modul kerja, layar login PIN, tema terang/gelap, tata letak cetak A4 landscape, dan pemanggilan IPC frontend. |

---

## Dokumentasi

* [CHANGELOG.md](CHANGELOG.md) mencatat riwayat rilis dan perubahan tiap versi berdasarkan Semantic Versioning.
* [docs/FEATURES.md](docs/FEATURES.md) merinci aturan kerja dan berkas terkait untuk fitur `FEAT-01` hingga `FEAT-09`.
* [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) menjelaskan arsitektur runtime, lokasi database lokal, kontrak perintah IPC, skema database, ADR, dan prosedur pemulihan bencana.

---

## Menjalankan dan Membangun

Proyek membutuhkan Rust toolchain (edition 2021) dengan Cargo serta Node.js untuk menjalankan CLI Tauri (`@tauri-apps/cli`).

```bash
# Menjalankan aplikasi dalam mode pengembangan
npm run dev

# Membangun bundel rilis produksi
npm run build
```

Perintah build rilis menulis hasil bundel ke direktori `src-tauri/target/release/bundle/`. Pada rilis `1.6.5`, bundel aplikasi macOS (`Anda ERP.app`) berukuran 11,06 MiB.

---

## Pengujian

```bash
# Memeriksa sintaks frontend JavaScript
node --check src/app.js

# Menjalankan seluruh unit test dan integration test Rust
cargo test --manifest-path src-tauri/Cargo.toml
```

Rangkaian `cargo test` menjalankan 18 kasus uji yang tersebar di unit test [src-tauri/src/lib.rs](src-tauri/src/lib.rs) (2 kasus), [tests/erp_core_tests.rs](tests/erp_core_tests.rs) (8 kasus: autentikasi, CRUD dan sinkronisasi tanggal fleksibel, rekap pajak, impor 13 kolom, Form 1, dan backup/restore), [tests/optimizer_tests.rs](tests/optimizer_tests.rs) (7 kasus: solver subset-sum, ekspor berumus 2 kolom/13 kolom, pencegahan bentrok jadwal kamar termasuk tanggal teks `DD/MM-YY`), serta [tests/smoke_verify.rs](tests/smoke_verify.rs) (1 kasus).

---

## Lisensi

Hak Cipta (c) 2024 Muhammad Syahid Ramadhon. Dilindungi undang-undang.
