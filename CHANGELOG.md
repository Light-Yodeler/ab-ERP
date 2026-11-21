# Changelog

Format mengacu pada [Keep a Changelog](https://keepachangelog.com/id/1.0.0/) dan [Semantic Versioning](https://semver.org/lang/id/).

Kategori yang dipakai: `Added` (fitur baru), `Changed` (perilaku berubah), `Fixed` (perbaikan bug), `Removed` (fitur dihapus), `Security` (perbaikan atau celah keamanan yang diketahui).

---

## [Unreleased]


---

## [1.6.5] - 2026-11-21

### Fixed
* Penyelesaian seluruh temuan audit `/open-code-review-delegate` dan penyeragaman format tanggal **`DD/MM/YY`** multi-format:
  * **Parser Tanggal Universal** (`src-tauri/src/excel_importer.rs`, `src-tauri/src/excel.rs`, `src-tauri/src/db.rs`): Menerima seluruh format umum (`DD/MM/YY`, `DD/MM-YY`, `DD.MM.YYYY`, `YYYY-MM-DD`, ISO datetime bertimestamp, nama bulan Indonesia/Inggris, serial Excel teks, serta `MM/DD/YY` saat hari > 12) dengan prioritas urutan `DD/MM/YY`. Pratinjau Optimizer kini menampilkan seluruh tanggal dalam format `DD/MM/YY`, sedangkan database menyimpannya secara kanonik sebagai `YYYY-MM-DD`.
  * **Ekspor 13 Kolom Berbasis Teks & `Total Night` Kosong** (`src-tauri/src/excel.rs`): `export_solution_to_excel()` kini mengonversi tanggal teks pada sumber menjadi serial tanggal Excel `dd/mm/yy`, mengisi `Total Night` kosong dari selisih `Check Out - Check In`, mengaktifkan rumus `H`/`I`/`K` dengan *cached result* akurat, serta menjaga seleksi indeks invalid pada cabang Legacy 2-kolom agar tidak menulis `SUM(F2:F1)`.
  * **Keamanan RBAC & Otorisasi Sesi** (`src-tauri/src/commands.rs`, `src/app.js`): `import_excel_file()` membatasi mode destruktif (`replace_period` dan `replace_all_unit`) hanya untuk Administrator; `auth_update_pin()` mewajibkan sesi aktif dan kepemilikan akun kecuali admin; `export_room_reservations_excel()` dan `download_excel_template()` menyaring pemisah jalur pada nama berkas bawaan.
  * **Sinkronisasi Tanggal Reservasi & Anomali** (`src-tauri/src/db.rs`, `src-tauri/src/optimizer.rs`): `db::update_transaction()` menggeser `check_in_date` dan `check_out_date` terkait secara atomik saat tanggal transaksi diubah; `analyze_room_issues()` menandai ketidakcocokan atau kekosongan `Total Night`; pratinjau footer `TARGET` dan gaya tabel di `src/app.js` / `src/style.css` diperjelas serta dilindungi `escapeHtml()`.

---

## [1.6.4] - 2026-11-21

### Fixed
* Perbaikan deteksi tanggal teks pada pencegahan bentrok kamar modul **Audit & Rekonsiliasi (Optimizer)** (`src-tauri/src/optimizer.rs`, `src-tauri/src/excel_importer.rs`):
  * `parse_flex_date()` kini memakai ulang parser tanggal fleksibel `excel_importer::parse_date_string` sehingga format tanggal teks campuran seperti `DD/MM-YY` (`01/10-26`, `11/10-26`), `DD.MM.YYYY`, maupun nama bulan Indonesia ikut dikenali penuh oleh detektor konflik kamar dan kendala anti-bentrok solver.
  * `item_stay_nights()` kini memprioritaskan selisih tanggal kalender `[check_in, check_out)` di atas kolom `Total Night` agar pemesanan multi-kamar yang mencatat *room-nights* tidak memicu bentrok semu pada malam berikutnya.
  * `analyze_room_issues()` kini melaporkan anomali eksplisit jika format tanggal Check-In suatu baris tetap tidak dapat dikenali alih-alih melewatinya tanpa peringatan.

---

## [1.6.3] - 2026-11-21

### Fixed
* Pencegahan bentrok jadwal kamar dan tanggal pada modul **Audit & Rekonsiliasi (Audit Target Solver)** (`solve_subset()` di `src-tauri/src/optimizer.rs`):
  * Menambahkan deteksi okupansi malam-kamar `(Room Number, Check In..Check Out)` sehingga dua transaksi pada kamar dan tanggal menginap yang tumpang tindih (termasuk notasi multi-kamar seperti `"07, 13"`) tidak dapat dipilih bersamaan, sementara check-out di hari yang sama dengan check-in baru tetap diizinkan dan nomor kamar asli tetap utuh.
  * `analyze_excel_file()` di `src-tauri/src/excel.rs` mengekspos daftar `room_conflicts` dan `room_anomalies` (mis. format kamar desimal `"5.1"` atau ketidaksesuaian `Total Kamar`), serta menjaga tanggal `Date Check in` yang berbeda dari kolom `DATE` tetap utuh saat ekspor template bersih 13 kolom.
  * Antarmuka Optimizer (`src/app.js`, `src/index.html`) kini mereset hasil solver lama saat pengguna mengganti berkas dan menampilkan kotak peringatan `#opt-conflict-warning` jika berkas sumber mengandung bentrok atau anomali kamar.

---

## [1.6.2] - 2026-11-21

### Changed
* Penyelarasan penuh modul **Audit & Rekonsiliasi (Audit Target Solver)** dan generator template 13 kolom dengan format template terbaru:
  * Tabel pratinjau Optimizer kini menampilkan ke-13 kolom (`DATE`, `NAME`, `Pax`, `Booking No`, `Nationality`, `Kamar`, `Jml Kamar`, `Check In`, `Check Out`, `Malam`, `Rate`, `Total Payment`, `Remarks`) serta nomor baris sumber dan label dinamis (`Total Payment (Kolom L)`, `50% Total Payment`).
  * Ekspor Optimizer (`export_solution_to_excel`), ekspor Ledger 13 kolom (`export_room_reservations_to_excel`), dan template unduhan Bungalows (`generate_import_template`) kini menulis tanggal serial `dd/mm/yy` serta rumus dinamis baris `H` (`=A{r}`), `I` (`=H{r}+J{r}`), dan `K` (`=IF(J{r}*G{r}>0,L{r}/J{r}/G{r},0)`) lengkap dengan cached result.
  * Deteksi header 13 kolom (`is_13_col_header`) kini menerima variasi nama kolom `Tanggal` di kolom A.

---

## [1.6.1] - 2026-11-21

### Fixed
* Sinkronisasi otomatis nilai `rate` dan validasi tanggal/nominal pada `db::update_transaction()`, `db::add_transaction()`, serta transaksi SQLite atomik pada `db::delete_period_transactions()`.
* Ekspor CSV pada rekapitulasi pajak (`exportRecapToCsv`) kini menempelkan elemen tautan sementara ke `document.body` dan membersihkan object URL setelah unduhan.
* Pelestarian format tanggal Excel (`dd/mm/yy`), cached result rumus `SUM`, dan penyertaan sheet `Referensi & Petunjuk` pada ekspor standalone 13 kolom di Optimizer serta pemformatan tanggal Check-In/Check-Out di `analyze_excel_file`.
* Perlindungan parsing `excel_importer` (batas serial tanggal valid, slicing ISO-8601 aman UTF-8, invarian fallback `check_out_date > check_in_date`, dan propagasi error hash/arsip) serta pelaporan `tax_report` (validasi tahun `1900..=2100` dan normalisasi label asal tahan huruf besar-kecil).
* Pemulihan/migrasi database (`restore_database`, `init_db_with_migration`) menggunakan SQLite Online Backup API agar aman terhadap mode WAL.
* Perbaikan UI `app.js`/`style.css`/`index.html`: sinkronisasi filter periode saat ganti unit, penghapusan awalan ganda `NIP. `, antrean dialog in-DOM, tarif pajak & nama perusahaan dinamis pada modal cetak, penyalinan teks angka Form 1 (`user-select: text`), pemindahan dependensi `objc2` khusus macOS di `Cargo.toml`, dan portabilitas suite pengujian.

### Security
* Penegakan `require_admin` pada perintah hapus massal `delete_period_transactions` dan `clean_duplicate_transactions` (serta penyembunyian tombol pembersihan duplikat untuk peran `staff` di UI), dan pengambilan ID admin langsung dari sesi `AppState` pada `auth_delete_user`.

---

## [1.6.0] - 2026-11-21

### Added
* Fitur **Edit & Hapus Rincian Reservasi (13 Kolom)** khusus Admin di tab Data Mentah (Ledger) melalui modal 13 kolom lengkap (`#modal-reservation`) dengan autocomplete daerah Indonesia & seluruh 198 negara dunia (`<datalist id="nationality-options">`) serta kalkulasi otomatis `Total Night` dan `Rate`.
* Sinkronisasi atomik dua arah antara tabel `room_reservations` dan `transactions` saat Admin mengubah atau menghapus baris reservasi/transaksi (`db::update_room_reservation()`, `db::delete_room_reservation()`, `update_transaction()`, `delete_transaction()`), sehingga Rekap Pajak 10% dan Form 1 Pariwisata tetap akurat.
* Tombol **Ekspor Excel (13 Kolom)** pada mode Rincian Reservasi (`export_room_reservations_excel`, `excel::export_room_reservations_to_excel()`) untuk mengekspor data hasil suntingan ke berkas `.xlsx` 13 kolom siap pakai beserta sheet `Referensi & Petunjuk` dan dropdown kolom `E`.

### Security
* Penegakan kontrol akses khusus Administrator (`require_admin`) pada `update_transaction`, `delete_transaction`, `update_room_reservation`, dan `delete_room_reservation` di backend Rust dan antarmuka tabel Ledger.

---

## [1.5.3] - 2026-11-21

### Added
* Daftar lengkap seluruh ~198 negara/wilayah dunia (A-Z) dan ~60 provinsi/kota utama Indonesia pada sheet `Referensi & Petunjuk` dan dropdown kolom `E` (`E2:E1000`) untuk template Excel 13 kolom bawaan aplikasi (`excel.rs`) maupun seluruh berkas template bulanan.
* Penonaktifan pemblokiran input pada validasi dropdown (`showErrorMessage = false`, `allowBlank = true`) agar operator tetap bebas mengetik nama negara/wilayah atau kota yang belum ada di dropdown.
* Penambahan `FOREIGN_OVERRIDES` pada `is_domestic_origin()` (`Papua Nugini`, `Papua New Guinea`) agar tidak bertabrakan dengan kata kunci provinsi `papua`.

---

## [1.5.2] - 2026-11-21

### Added
* Dukungan notasi awalan eksplisit (`IDN - Mataram`, `Domestik - Surabaya`, `WNI - Bali`, `INTL - Australia`, `Mancanegara - Prancis`, `WNA - Jerman`) pada `is_domestic_origin()` beserta normalisasi label asal di `normalize_origin_label()`.
* Perluasan cakupan kata kunci `DOMESTIC_KEYWORDS` mencakup 38 provinsi dan kota/kabupaten utama Indonesia pada Form 1 Dinas Pariwisata.
* Pembaruan template resmi Excel 13 kolom pada `generate_import_template()` dengan penambahan lembar kedua `Referensi & Petunjuk` dan contoh daerah asal spesifik (`Mataram`).

### Changed
* Parser header importer 13 kolom pada `excel_importer.rs` menerima variasi nama header `Nationality / Daerah Asal`, `Negara / Daerah Asal`, `Asal`, dan `Daerah Asal`.

---

## [1.5.1] - 2026-11-21

### Fixed
* Fallback perhitungan tanggal keluar pada `generate_tourism_report()` ketika `check_out_date <= check_in_date` sehingga durasi malam menginap (`total_nights`) tetap terhitung pada metrik kamar dan tempat tidur Form 1 Dinas Pariwisata.
* Pencocokan asal wisatawan domestik pada `is_domestic_origin()` menggunakan pencocokan batas kata untuk kata kunci pendek agar nama negara mancanegara seperti `Solomon Islands` tidak salah diklasifikasikan sebagai domestik.
* Query `get_available_months()` menyertakan periode bulan dari tabel `room_reservations` (`date`, `check_in_date`, dan `check_out_date`) bersama tabel `transactions`.
* Penghapusan fungsi tidak terpakai `db::record_import_log()`, pemindahan inline style statis pada `#view-tourism` ke class utilitas CSS, dan penggantian fixture path eksternal pada integration test agar mandiri.

### Security
* Penambahan perintah IPC `auth_logout` untuk mengosongkan status sesi pengguna `AppState.current_user` di backend Rust saat pengguna menekan tombol keluar dari frontend.

---

## [1.5.0] - 2026-11-21

### Added
* Modul baru **Pendataan Wisatawan (Form 1 Dinas Pariwisata Kabupaten Lombok Tengah)** pada menu **Pajak Daerah** untuk menampilkan seluruh data siap salin bagi formulir resmi bulanan (`FEAT-09`):
  * Tabel 4 minggu (`Minggu 1` s/d `Minggu 4` + `TOTAL`) mencakup Check-In/Check-Out Wisatawan Domestik & Mancanegara, Kamar `Occupied` & `Vacant` (*room-night*), dan Tempat Tidur `Check Out (Bed)` & `Occupied (Bed)` (*bed-night*).
  * Tabel peringkat 10 besar kunjungan **Mancanegara** vs **Nusantara** beserta daftar lengkap seluruh negara/daerah jika melebihi 10 entri.
  * Kalkulasi otomatis **Rata-rata Tinggal (Malam)** beserta indikator kategori `1-2 MALAM`, `3 MALAM`, dan `> 3 MALAM`, serta pengaturan kapasitas kamar (default 14) dan jenis akomodasi.
* Perintah Tauri IPC `get_tourism_report` dan fungsi agregasi `tax_report::generate_tourism_report`.

---

## [1.4.0] - 2026-11-20

### Added
* Mode tampilan toggle **Rincian Reservasi (13 Kolom)** di dalam tab Data Mentah (Ledger) untuk menampilkan seluruh kolom format unggahan terbaru (`DATE`, `NAME`, `Total Person`, `Booking Number`, `Nationality`, `Room Number`, `Total Kamar`, `Date Check In`, `Date Check Out`, `Total Night`, `Rate`, `Total Payment`, dan `Remarks`) lengkap dengan kartu ringkasan metrik (`Total Reservasi`, `Total Tamu`, `Total Malam`, `Total Payment`) serta filter periode, pencarian multi-atribut, dan paginasi SQL (`FEAT-04`).
* Perintah Tauri IPC `get_room_reservations` beserta fungsi paginasi & agregasi `db::get_paginated_room_reservations` di backend Rust.

---

## [1.3.1] - 2026-11-20

### Added
* Pemuatan dinamis seluruh akun pengguna terdaftar pada layar login (`refreshLoginUsers()`) serta indikator 8-digit dot PIN.

### Fixed
* Pemulihan database (`restore_database`) menggunakan SQLite Online Backup API yang aman terhadap mode WAL serta memvalidasi keberhasilan pembuatan salinan darurat `.pre_restore.bak`.
* Pemotongan kode unit aman UTF-8 pada nomor dokumen laporan pajak (`tax_report.rs`), guard pengecekan `imported_at` saat migrasi startup (`db.rs`), batas pembersihan baris dinamis pada ekspor Excel (`excel.rs`), dan konsistensi dialog konfirmasi in-DOM (`appConfirm`).

### Security
* Mitigasi DOM XSS melalui sanitasi `escapeHtml()` pada seluruh rendering tabel dan pratinjau di antarmuka.
* Pengaktifan Content Security Policy (CSP) pada Webview Tauri dan penghapusan inline form event handlers.
* Penegakan kontrol akses peran (`admin` vs `staff`) pada `AppState` backend Rust (`commands.rs`) dan pembatasan akses pengaturan pada antarmuka pengguna.

---

## [1.3.0] - 2026-11-07

### Added
* Tampilan pratinjau tabel hasil rekonsiliasi acak di layar aplikasi kini menampilkan seluruh kolom sesuai template 13 kolom baru (`Tanggal`, `Nama`, `Kamar`, `Check In`, `Check Out`, `Malam`, `Rate`, `Total Payment`, dan `Remarks`).
* Ekspor hasil rekonsiliasi format 13 kolom kini menghasilkan berkas `.xlsx` baru bersih terpisah yang langsung siap pakai dan 100% mengikuti struktur template 13 kolom (`FEAT-06`).

---

## [1.2.0] - 2026-11-07

### Added
* Dukungan format Excel 13 kolom reservasi kamar tamu (`Book1.xlsx` / `Rekap Tamu`) pada modul Audit Target Solver (Optimizer).
* Deteksi otomatis target 50% dari baris footer dan kalkulasi subset acak berbasis multi-start randomized greedy.
* Ekspor hasil acak 13 kolom ke Kolom O s/d AA pada lembar kerja sumber serta pembuatan lembar kerja bersih baru `Rekap Tamu (Hasil Acak)`.

### Changed
* Modul Audit Target Solver (`FEAT-06`) kini mendukung format ganda (Format 2 Kolom Legacy dan Format 13 Kolom Reservasi Kamar) secara otomatis tanpa mengubah antarmuka maupun logika solver lama.

---

## [1.1.0] - 2026-11-07

### Added
* Tabel basis data baru `room_reservations` untuk menyimpan struktur 13 kolom reservasi kamar tamu (`FEAT-05`).
* Dukungan pengenalan format tanggal teks 2-digit tahun `DD/MM-YY`, `DD/MM/YY`, `DD-MM-YY`, dan `DD.MM.YY`.
* Generator template resmi unduh Excel 13 kolom (`Rekap Tamu`) untuk unit Bungalows.

### Changed
* Modul impor Excel kini mendeteksi berkas 13 kolom reservasi kamar secara otomatis dan melakukan dual-write atomik ke tabel `transactions` dan `room_reservations`. Format 3 kolom lama tetap didukung normal ke tabel `transactions` tanpa membuat baris kosong di tabel reservasi.

---

## [1.0.1] - 2026-11-06

### Changed
* Penamaan berkas default pada dialog cetak laporan (Simpan sebagai PDF) dan ekspor CSV kini otomatis menggunakan nomor laporan resmi yang disanitasi (`FEAT-03`).
* Perintah Tauri IPC `print_report` kini menerima argumen opsional `title` untuk sinkronisasi judul native window saat mencetak.
* Penambahan perintah Tauri IPC `set_window_title` untuk mengatur judul native window secara terprogram.

### Fixed
* Memperbaiki judul dialog cetak PDF pada macOS yang sebelumnya tetap menggunakan judul aplikasi default karena WKWebView membaca judul jendela sistem (`NSWindow.title`).

---

## [1.0.0] - 2026-11-06

### Added
* Rilis perdana Anda ERP desktop berbasis Tauri v2 dan Rust.
* Modul autentikasi pengguna berbasis PIN kriptografis PBKDF2 HMAC-SHA512 (`FEAT-01`).
* Modul rekapitulasi kalender pajak bulanan dan PB1 10% Dispenda Lombok Tengah (`FEAT-02`).
* Modul cetak formulir resmi A4 landscape dengan pratinjau interaktif (`FEAT-03`).
* Modul pencatatan transaksi harian (ledger data mentah) dan paginasi SQL (`FEAT-04`).
* Modul impor berkas spreadsheet Excel via parser Calamine dan auto-arsip fisik (`FEAT-05`).
* Modul rekonsiliasi audit target subset-sum optimizer (`FEAT-06`).
* Manajemen multi-unit usaha untuk Bungalows dan Restaurant (`FEAT-07`).
* Pencadangan database online SQLite dan prosedur pemulihan bencana (`FEAT-08`).
