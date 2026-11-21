# Rencana Arsitektur & Rekayasa Balik: Desktop ERP Ultra-Ringan (Tauri v2 + Rust + SQLite)

## Context
Pengguna meminta untuk merekayasa balik (*reverse engineering*) seluruh fitur aplikasi pelaporan pajak dan rekapitulasi pendapatan (*Laporan dispenda*) yang sebelumnya berbasis Electron (~138 MB installer, konsumsi RAM ~200 MB) ke dalam fondasi arsitektur desktop ultra-ringan yang telah dibuat (Tauri v2 + Native Rust + SQLite). Aplikasi ini ditransformasikan menjadi **Sistem ERP Desktop Modular ("Anda ERP")** untuk perusahaan (*Anda Bungalows & Restaurant*), yang dirancang modular agar dapat diekspansi dengan mudah untuk modul-modul bisnis berikutnya (seperti Pengeluaran/Biaya, Inventaris/Stok, Pembelian, dan Akuntansi Buku Besar) dengan konsumsi memori sangat rendah (~25–35 MB RAM), ukuran installer kecil (~8–12 MB), dan peluncuran instan (<0.2 detik).

Sesuai arahan pengguna, perencanaan ini menerapkan prinsip **Karpathy Guidelines** (kesederhanaan mutlak, perubahan bedah, tanpa abstraksi spekulatif, dan eksekusi berbasis tujuan terverifikasi) serta aturan **Anti-Slop** (desain fungsional murni, tanpa ornamen buatan, kontras tinggi WCAG AA, dan dukungan tema terang/gelap).

---

## Approach & Tahapan Implementasi

### 1. Fondasi Arsitektur Database & Penyimpanan Lokal (`core/db.rs`)
Menggantikan `sql.js` (JavaScript in-memory) dengan **`rusqlite`** (engine SQLite3 native C terkompilasi langsung ke dalam binary):
- **Mode Konkurensi & Integritas:** Aktifkan `PRAGMA journal_mode = WAL;`, `PRAGMA synchronous = NORMAL;`, dan `PRAGMA foreign_keys = ON;`.
- **Lokasi Penyimpanan Standar:** Direktori `app_data_dir("com.andabungalows.erp")`, dengan auto-migrasi dari folder legacy `Laporan dispenda` / `laporan-bapenda-dispenda` jika ditemukan file `laporan_bapenda.sqlite`.
- **Skema Relasional Kompatibel & Modular:**
  - `units`: Master unit bisnis (`id`, `code`, `name`, `tax_rate`, `address`, `npwpd`, `is_active`). Bawaan: `Bungalows` dan `Restaurant`, siap ditambah unit baru kapan saja.
  - `transactions`: Transaksi pendapatan mentah (`id`, `unit`, `date`, `amount`, `notes`, `source_file`, `created_at`).
  - `import_logs`: Log riwayat impor berkas Excel (`id`, `unit`, `file_name`, `records_count`, `total_amount`, `stored_path`, `imported_at`).
  - `settings`: Konfigurasi sistem key-value (`key`, `value`).
  - `users`: Autentikasi dan hak akses (`id`, `username`, `pin_hash`, `salt`, `full_name`, `role`, `created_at`, `last_login`).

### 2. Modul Autentikasi & Keamanan Kriptografis (`core/auth.rs`)
- **Algoritma Hashing PIN:** PBKDF2 HMAC-SHA512 dengan 10.000 iterasi dan salt acak 16 byte (`sha2`, `pbkdf2`, `rand`), identik 100% dengan implementasi Node.js crypto sebelumnya sehingga database SQLite lama langsung dapat digunakan tanpa reset password.
- **Peran & Hak Akses (RBAC):**
  - `admin`: Akses menyeluruh, kelola profil usaha, kelola pengguna (tambah/edit/hapus), serta cadangkan dan pulihkan database.
  - `staff`: Akses operasional harian (impor Excel, input/edit data mentah, rekapitulasi, cetak laporan resmi, dan ubah PIN akun sendiri).
- **Akun Bawaan:** `admin` (PIN: `123456`) dan `staff` (PIN: `1234`).

### 3. Modul Mesin Spreadsheet & Deteksi Unit Otomatis (`modules/excel_importer.rs`)
- **Parser Cepat Calamine:**
  - Membaca berkas `.xlsx`, `.xls`, dan `.xlsm` tanpa beban memori.
  - Normalisasi tanggal cerdas: Mendukung serial number Excel (1900 leap year bug fix), format ISO `YYYY-MM-DD`, format Indonesia `DD/MM/YYYY`, dan varian pemisah titik/garis miring.
  - Normalisasi nominal angka: Membersihkan awalan `Rp`, titik ribuan, koma desimal, dan spasi.
- **Deteksi Unit Cerdas & Validasi:**
  - Analisis nama sheet dan nama file untuk mencocokkan kata kunci (`resto`, `restaurant`, `f&b`, `makan` $\to$ Restaurant; `bungalow`, `kamar`, `villa`, `room` $\to$ Bungalows).
  - Peringatan antarmuka jika file yang diunggah terdeteksi milik unit berbeda dari tab aktif.
- **Mode Impor Transaksi:**
  - `replace_period`: Mengganti transaksi pada bulan-bulan yang terdapat dalam berkas (standar teraman).
  - `replace_all_unit`: Menghapus dan menulis ulang seluruh transaksi unit terkait.
  - `skip_duplicates`: Melewati baris dengan kombinasi unit, tanggal, amount, dan notes yang sama persis.
- **Pengarsipan Fisik Otomatis:**
  - Salinan fisik berkas Excel otomatis disimpan ke direktori `arsip_excel/` dengan nama timestamp aman `[{timestamp}]_[{unit}]_{filename}`.

### 4. Modul Rekapitulasi & Kalkulasi Perpajakan Daerah (`modules/tax_report.rs`)
- **Agregasi Kalender Harian (Hari 1 s/d Hari Terakhir Bulan):**
  - Query ringkas `SELECT date, SUM(amount) FROM transactions WHERE unit = ? AND date BETWEEN ? AND ? GROUP BY date`.
  - Mengisi otomatis seluruh 28–31 hari kalender: hari aktif pendapatan, hari nihil pendapatan (`amount = 0`), nama hari dalam bahasa Indonesia (Senin–Minggu), dan saldo pendapatan kumulatif harian.
- **Kalkulasi Metrik Pajak & Finansial:**
  - `totalRevenue`: Akumulasi omset kotor bulan berjalan.
  - `taxRate`: Tarif pajak daerah dari konfigurasi (bawaan: 10% / 0.10).
  - `taxDue`: Pajak terutang $= \text{round}(totalRevenue \times taxRate)$.
  - `netRevenue`: Pendapatan bersih $= totalRevenue - taxDue$.
  - `activeDaysCount` & `zeroDaysCount`.
  - `maxRevenue` & `minRevenueNonZero`.
  - `avgPerDay`: Rata-rata per hari kalender.
- **Generator Nomor Dokumen Unik:**
  - Format resmi: `No: ANDA-{UNIT_CODE}/{YEAR}-{MONTH}/{HEX_HASH}`.

### 5. Modul Tinjauan Data Mentah & Paginasi Database (`modules/ledger.rs`)
- **Paginasi SQL Berkinerja Tinggi:**
  - Mengambil total cacah dan total nominal terlebih dahulu (`COUNT(*)`, `SUM(amount)`).
  - Menarik data per halaman menggunakan `LIMIT ? OFFSET ?` (pilihan ukuran halaman: 25, 50, 100 baris) dengan pengurutan `ORDER BY date DESC, id DESC`.
- **Fitur CRUD & Pembersihan Data:**
  - Tambah transaksi manual langsung.
  - Edit dan hapus transaksi perorangan.
  - Fitur pembersihan otomatis transaksi duplikat (`clean_duplicate_transactions`).

### 6. Modul Pencadangan & Pemulihan Bencana (`core/backup.rs`)
- **Cadangkan (Backup):**
  - Menyalin database aktif ke file tujuan yang dipilih pengguna (`.sqlite` / `.bak`) dengan verifikasi integritas ukuran file.
- **Pulihkan (Restore):**
  - Uji keabsahan file cadangan terlebih dahulu (memeriksa header SQLite dan ketersediaan tabel `transactions`).
  - Pembuatan cadangan darurat otomatis `laporan_bapenda.sqlite.pre_restore.bak` sebelum database aktif digantikan.
  - Muat ulang koneksi database tanpa perlu me-restart aplikasi.

### 7. Integrasi Modul Subset Target Solver (Akuntansi & Audit)
- Mengintegrasikan mesin *Subset Sum Target Solver* native Rust yang telah dibangun sebelumnya ke dalam menu khusus **Audit & Rekonsiliasi Target**.
- Memungkinkan staf akuntansi/keuangan membagi atau memilah transaksi acak mendekati target nominal pembagian (misal 50% untuk rekonsiliasi kas/pajak) secara instan.

### 8. Desain Antarmuka Pengguna ERP (Native HTML5/CSS/Vanilla JS)
Mengusung estetika **Financial Desktop Utility** (ENERGY 1 / RHYTHM 1 / MOTION 1):
- **Struktur Layar Tunggal Modular:**
  - **Layar Masuk (Login Screen):** Kartu pemilihan profil pengguna, display 4–6 digit PIN, numpad interaktif dan input keyboard langsung.
  - **Bilah Navigasi Atas (Header ERP):**
    - Pengalih Unit Bisnis: `[ Bungalows ]` | `[ Restaurant ]` | `[ + Tambah Unit ]`.
    - Tab Modul ERP:
      1. `[ Rekapitulasi Pajak ]` (Kartu metrik, grafik tren harian, tabel 31 hari, tombol Cetak PDF & Ekspor Excel).
      2. `[ Data Mentah ]` (Tabel transaksi dengan paginasi SQL, filter periode, pencarian teks, CRUD manual).
      3. `[ Riwayat Impor ]` (Daftar berkas terimpor, tombol buka file arsip asli).
      4. `[ Audit Target (Subset Solver) ]` (Mesin optimasi target B80 otomatis).
    - Status Akun Pengguna, Tombol Pengaturan, Tombol Mode Gelap/Terang, dan Keluar (Logout).
  - **Pratinjau Cetak Formulir Resmi (Print Report View):**
    - Formulir A4 Landscape resmi siap cetak ke printer sistem atau simpan PDF, lengkap dengan kop surat perusahaan, nomor NPWPD, grafik bar harian SVG, dan kolom tanda tangan pimpinan/petugas.
  - **Modal Pengaturan (Settings Modal):**
    - Tab Profil Usaha (NPWPD, alamat, tarif pajak, pimpinan).
    - Tab Lokasi Penyimpanan (Jalur database, ukuran di disk, total transaksi, tombol buka di Finder/Explorer).
    - Tab Kelola Pengguna (Admin: tambah/edit/hapus user dan peran).
    - Tab Cadangan & Pemulihan (Backup/restore database, reset unit, reset total).

---

## Critical Files & Lokasi Implementasi

1. `excel-subset-app/src-tauri/Cargo.toml`
   - Menambahkan dependensi `rusqlite = { version = "0.32", features = ["bundled"] }`, `pbkdf2 = "0.12"`, `sha2 = "0.10"`.
2. `excel-subset-app/src-tauri/src/db.rs`
   - Definisi koneksi SQLite, migrasi skema tabel, CRUD transaksi, paginasi SQL, backup/restore, dan inisialisasi default users & settings.
3. `excel-subset-app/src-tauri/src/auth.rs`
   - Logika PBKDF2 HMAC-SHA512, verifikasi PIN sesi, dan manajemen pengguna.
4. `excel-subset-app/src-tauri/src/excel_importer.rs`
   - Parser Calamine, normalisasi tanggal/nominal, auto-detect unit bisnis, dan archiving fisik berkas Excel.
5. `excel-subset-app/src-tauri/src/tax_report.rs`
   - Kalkulasi kalender bulanan (1..31 hari), perhitungan pajak 10%, akumulasi harian, dan agregasi metrik.
6. `excel-subset-app/src-tauri/src/commands.rs` & `lib.rs`
   - Registrasi seluruh Tauri IPC commands yang menghubungkan Rust ke frontend UI.
7. `excel-subset-app/src/index.html`, `style.css`, `app.js`
   - Frontend UI modular ERP lengkap dengan antarmuka login PIN, switcher unit, tab modul, data mentah terpaginasi, modal impor, pratinjau cetak A4 landscape, dan modal pengaturan.

---

## Verifikasi & Kriteria Keberhasilan

1. **Uji Keutuhan Database & Autentikasi:**
   - Database SQLite otomatis diinisialisasi dengan akun default (`admin`: 123456, `staff`: 1234).
   - Verifikasi bahwa hashing PBKDF2 di Rust 100% cocok dengan hash di database `laporan_bapenda.sqlite` yang ada pada aplikasi lama.
2. **Uji Impor Data Excel Nyata:**
   - Impor file `Contoh_Pendapatan_Bungalows2.xlsx` dan `Contoh_Pendapatan_Restaurant.xlsx`.
   - Validasi deteksi unit otomatis, normalisasi tanggal (format YYYY-MM-DD), pembersihan nominal, dan salinan berkas tersimpan di `arsip_excel/`.
3. **Uji Rekapitulasi Pajak & Metrik:**
   - Pilih periode bulan yang memiliki transaksi.
   - Periksa perhitungan omset kotor, pajak terutang 10%, dan pendapatan bersih tepat hingga digit rupiah tanpa pembulatan liar.
   - Periksa 31 hari kalender terpetakan dengan benar (hari aktif vs hari nihil).
4. **Uji Pratinjau Cetak & PDF:**
   - Buka pratinjau cetak laporan: nomor NPWPD, kop surat, grafik SVG tren omset harian, dan kolom tanda tangan tampil proporsional pada format A4 Landscape.
5. **Uji Paginasi Data Mentah:**
   - Navigasi halaman 1, 2, dst. dengan batas 25/50 baris per halaman. Filter pencarian teks dan filter bulan berfungsi secara instan di level SQL.
6. **Uji Cadangan & Pemulihan (Backup & Restore):**
   - Ekspor backup ke file cadangan baru. Uji restore file cadangan tersebut dan verifikasi file pengaman `.pre_restore.bak` dibuat secara otomatis.
7. **Uji Bobot & Penggunaan Memori (Lightweight Benchmark):**
   - Ukuran installer macOS `.dmg` tetap $\le 10 \text{ MB}$.
   - Penggunaan RAM saat aplikasi berjalan aktif $\le 35 \text{ MB}$ (dibandingkan Electron 180–300 MB).
   - Waktu startup aplikasi $< 0.3 \text{ detik}$.

---

## Asumsi & Kebijakan Fallback

1. **Migrasi Data Lama Otomatis:**
   - *Asumsi:* Pengguna memiliki riwayat transaksi di file `laporan_bapenda.sqlite` aplikasi lama.
   - *Fallback:* Pada saat pertama kali diluncurkan, jika file database lokal baru masih kosong, sistem secara otomatis mendeteksi dan mengimpor data transaksi dan arsip dari direktori aplikasi Electron lama tanpa perlu input manual.
2. **Kompatibilitas Format Excel:**
   - *Asumsi:* Format kolom file pendapatan harian bervariasi (beberapa berkas menempatkan kolom tanggal di baris/kolom berbeda).
   - *Fallback:* Algoritma `excel_importer.rs` memindai 25 baris pertama untuk mendeteksi header secara dinamis (`tanggal`/`tgl` dan `total`/`jumlah`/`amount`/`omset`).
