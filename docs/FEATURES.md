# Katalog Fitur: Anda ERP

Daftar seluruh fitur aplikasi beserta kegunaan, aturan kerja, berkas yang mengimplementasikan, dan pengaturan yang memengaruhinya. Setiap fitur memiliki ID tetap `FEAT-NN` yang tidak pernah diubah.

---

## Modul 1: Autentikasi Pengguna

### FEAT-01: Autentikasi PIN Kriptografis
* **Kegunaan:** Mengamankan akses aplikasi desktop dengan keypad PIN numerik cepat tanpa memerlukan koneksi internet.
* **Pemicu:** Pengguna memilih akun pada layar login (`admin` atau `staff`) dan menekan PIN lalu menekan tombol Masuk ke Sistem.
* **Aturan Kerja:**
  1. PIN minimal 4 digit dan maksimal 8 digit numerik.
  2. Verifikasi menggunakan algoritma PBKDF2 HMAC-SHA512 dengan 10.000 iterasi dan salt acak hex 32 karakter.
  3. Verifikasi dilakukan dengan pembandingan waktu konstan (*constant-time comparison*) untuk mencegah serangan *timing attack*.
  4. Pengguna dengan peran `admin` memiliki akses penuh termasuk manajemen akun dan pengaturan basis data. Pengguna dengan peran `staff` dibatasi pada operasional harian.
* **Berkas Terkait:**
  * Backend: `src-tauri/src/auth.rs` (`verify_pin()`, `hash_pin()`), `src-tauri/src/commands.rs` (`auth_login()`, `auth_logout()`)
  * Frontend: `src/app.js` (`submitLogin()`, `logout()`, `handlePinInput()`)
* **Pengaturan:** Tabel `users` pada basis data lokal SQLite.

---

## Modul 2: Rekapitulasi Pajak Daerah

### FEAT-02: Rekapitulasi Kalender Pajak Bulanan & PB1 10%
* **Kegunaan:** Menghitung total omset kotor, pajak daerah terutang 10%, dan pendapatan bersih selama 1 s/d 31 hari kalender untuk pelaporan Bapenda.
* **Pemicu:** Pengguna membuka tab Rekapitulasi Pajak atau memilih periode bulan tertentu.
* **Aturan Kerja:**
  1. Mengumpulkan transaksi harian pada unit aktif untuk periode bulan `YYYY-MM`.
  2. Memetakan omset per hari kalender (1 s/d 28, 29, 30, atau 31 tergantung jumlah hari pada bulan tersebut).
  3. Menghitung tarif pajak terutang tepat 10% (`total_revenue * 0.10`).
  4. Menghitung pendapatan bersih (`total_revenue - tax_due`).
  5. Menghasilkan nomor dokumen unik berformat `No: ANDA-<UNIT>/<TAHUN>-<BULAN>/<HASH>`.
* **Berkas Terkait:**
  * Backend: `src-tauri/src/tax_report.rs` (`generate_tax_recap()`)
  * Frontend: `src/app.js` (`loadRecapData()`, `renderRecapTable()`, `renderRecapChart()`)
* **Pengaturan:** Tabel `settings` (`tax_rate`, default 0.10).

### FEAT-03: Cetak Formulir Resmi A4 Landscape & Penamaan Berkas Laporan
* **Kegunaan:** Menyediakan formulir cetak resmi standar Bapenda siap tanda tangan serta ekspor berkas PDF dan CSV dengan penamaan nomor dokumen otomatis.
* **Pemicu:** Pengguna menekan tombol Cetak Formulir Resmi pada tab rekapitulasi.
* **Aturan Kerja:**
  1. Membuka modal pratinjau A4 landscape yang memuat kop surat usaha, nomor dokumen, tabel 31 hari, grafik batang omset, ringkasan setoran pajak, dan kolom tanda tangan pimpinan/petugas.
  2. Saat tombol Cetak ke Printer / Simpan PDF ditekan, aplikasi menyinkronkan judul jendela sistem (`NSWindow.title`) dan `document.title` ke nomor laporan yang telah disanitasi (contoh: `ANDA-BUNGALOWS-2026-08-9A4F82C1`).
  3. Karakter yang tidak diizinkan sistem berkas seperti garis miring (`/`) dan titik dua (`:`) otomatis diubah menjadi tanda hubung (`-`).
  4. Saat pengguna menekan Ekspor CSV, berkas diunduh dengan nama `<nomor_laporan>.csv`.
  5. Saat modal ditutup atau proses cetak selesai, judul jendela otomatis dikembalikan ke judul bawaan aplikasi.
* **Berkas Terkait:**
  * Backend: `src-tauri/src/commands.rs` (`print_report()`, `set_window_title()`)
  * Frontend: `src/app.js` (`openPrintReportModal()`, `getCleanReportFileName()`, `exportRecapToCsv()`)
* **Pengaturan:** Opsi sertakan tanda tangan (`chk-toggle-signatures`).

---

## Modul 3: Buku Besar Data Mentah (Ledger)

### FEAT-04: Ledger Transaksi, Rincian Reservasi 13 Kolom & Paginasi SQL
* **Kegunaan:** Mengelola data transaksi mentah harian maupun melihat rincian tamu reservasi kamar format 13 kolom lengkap dengan pencarian, metrik ringkasan, dan pagination berbasis database.
* **Pemicu:** Pengguna membuka tab Data Mentah (Ledger), memilih mode tampilan (`Ringkas (3 Kolom)` atau `Rincian Reservasi (13 Kolom)`), atau melakukan pencarian/filter.
* **Aturan Kerja:**
  1. Mengambil data transaksi atau rincian reservasi kamar dengan filter unit, filter bulan, dan query pencarian multi-atribut (nama tamu, nomor kamar, kode booking, negara, remarks/OTA, tanggal).
  2. Paginasi SQL menggunakan `LIMIT` dan `OFFSET` dengan opsi 25, 50, atau 100 baris per halaman.
  3. Pada mode Rincian Reservasi (13 Kolom), menampilkan 4 kartu metrik ringkasan (`Total Reservasi`, `Total Tamu (Pax)`, `Total Malam`, `Total Payment`) serta seluruh 13 kolom (`DATE`, `NAME`, `Total Person`, `Booking Number`, `Nationality`, `Room Number`, `Total Kamar`, `Date Check In`, `Date Check Out`, `Total Night`, `Rate`, `Total Payment`, `Remarks`).
  4. Menyediakan fungsi tambah, ubah, dan hapus transaksi serta tombol pembersihan transaksi duplikat pada mode Ringkas (3 Kolom), serta fungsi edit/hapus rincian reservasi 13 kolom khusus Admin (disinkronkan atomik dengan tabel `transactions`, Rekap Pajak 10%, dan Form 1 Pariwisata) dan ekspor `.xlsx` 13 kolom pada mode Rincian Reservasi (`FEAT-04`).
* **Berkas Terkait:**
  * Backend: `src-tauri/src/db.rs` (`get_transactions()`, `get_paginated_room_reservations()`, `add_transaction()`, `update_transaction()`, `delete_transaction()`, `update_room_reservation()`, `delete_room_reservation()`), `src-tauri/src/excel.rs` (`export_room_reservations_to_excel()`), `src-tauri/src/commands.rs` (`get_room_reservations()`, `update_room_reservation()`, `delete_room_reservation()`, `export_room_reservations_excel()`, `get_origin_reference_list()`)
  * Frontend: `src/app.js` (`loadLedgerData()`, `updateLedgerModeUI()`, `openReservationModal()`, `saveReservation()`, `loadOriginDatalist()`)
* **Pengaturan:** Indeks komposit `idx_transactions_lookup` dan `idx_room_res_unit_date` pada SQLite.

---

## Modul 4: Pengimpor Spreadsheet Excel

### FEAT-05: Impor Spreadsheet Excel & Auto-Arsip Fisik
* **Kegunaan:** Membaca berkas spreadsheet Excel harian tanpa dependensi Microsoft Office, memvalidasi kolom, dan mengarsipkan salinan berkas fisik.
* **Pemicu:** Pengguna memilih berkas spreadsheet `.xlsx` atau `.xls` pada tab Unggah & Impor Excel.
* **Aturan Kerja:**
  1. Parser Calamine membaca lembar kerja pertama, mendeteksi secara otomatis apakah berkas bertipe 13 kolom reservasi kamar tamu (`DATE`, `NAME`, `Total Payment`, dll.) atau 3 kolom pendapatan ringkas (`Tanggal`, `Total`, `Keterangan`).
  2. Normalisasi tanggal menerima beragam format (`DD/MM/YY`, `DD/MM-YY`, `DD.MM.YYYY`, ISO-8601 + waktu, serial Excel, maupun teks bulan Indonesia/Inggris), menampilkan format seragam `DD/MM/YY` pada pratinjau Optimizer/ekspor Excel, dan menyimpan kanonik `YYYY-MM-DD` di database.
  3. Pada berkas 13 kolom reservasi, sistem melakukan dual-write atomik ke tabel `transactions` (omset harian) dan tabel `room_reservations` (detail 13 atribut tamu dan kamar). Berkas 3 kolom ringkas hanya mengisi tabel `transactions` tanpa membuat baris kosong di tabel reservasi.
  4. Parser mengabaikan baris ringkasan footer (`TOTAL` / `JUMLAH`) dan kebal terhadap eror formula Excel (`#DIV/0!`).
  5. Melakukan hashing SHA-256 berkas sumber untuk mendeteksi berkas duplikat yang pernah diimpor.
  6. Opsi penanganan data mencakup Timpa Periode / Timpa Unit (*Replace*, khusus Admin) atau Lewati Duplikat (*Append Skip*).
  7. Salinan fisik berkas disimpan secara otomatis ke direktori arsip lokal (`arsip_excel/`).
* **Berkas Terkait:**
  * Backend: `src-tauri/src/excel_importer.rs` (`import_excel()`, `preview_excel_import()`)
  * Frontend: `src/app.js` (`handleFileValidationAndPreview()`)
* **Pengaturan:** Direktori penyimpanan arsip pada `app_data_dir/arsip_excel`.

---

## Modul 5: Audit Target Solver (Optimizer)

### FEAT-06: Subset-Sum Optimizer & In-Place Write
* **Kegunaan:** Menemukan kombinasi baris transaksi harian agar totalnya mendekati nilai target pelaporan tertentu (misalnya baris 80 target nominal) dalam waktu sub-milidetik.
* **Pemicu:** Pengguna mengunggah berkas Excel target pada tab Audit Target Solver dan menekan tombol Jalankan Solver.
* **Aturan Kerja:**
  1. Mendeteksi secara otomatis struktur lembar kerja: format 2 kolom lama (`Tanggal`, `Total`) atau format 13 kolom reservasi kamar (`DATE`, `NAME`, `Total Payment`).
  2. Mendeteksi target secara dinamis dari baris footer (contoh: baris formula `=TOTAL / 2` pada baris 79) atau default 50% dari total omset.
  3. Menjalankan algoritma kombinatorial *Multi-Start Randomized Greedy + 2-Opt Local Search* secara paralel via Rayon.
  4. Memastikan selisih total subset terhadap target berada dalam ambang toleransi kecil.
  5. Tombol Buat Variasi Baru menghasilkan variasi kombinasi transaksi berbeda dengan nilai selisih yang setara.
  6. Pada format 2 kolom lama, hasil acak ditulis ke kolom E & F file Excel beserta formula `=SUM(F2:F...)`.
  7. Pada format 13 kolom, tabel pratinjau antarmuka menampilkan seluruh 13 kolom lengkap (`DATE`, `NAME`, `Pax`, `Booking No`, `Nationality`, `Kamar`, `Jml Kamar`, `Check In`, `Check Out`, `Malam`, `Rate`, `Total Payment`, `Remarks`), dan tombol ekspor menghasilkan berkas `.xlsx` baru bersih terpisah yang langsung siap pakai dengan rumus dinamis `H`/`I`/`K`, format tanggal `dd/mm/yy`, serta lembar `Referensi & Petunjuk` dan dropdown kolom `E`.
  8. Solver memvalidasi jadwal kamar `(Room Number, Check In..Check Out)` sebagai kendala keras anti-bentrok agar dua tamu tidak pernah menempati kamar yang sama pada malam yang sama, serta menampilkan peringatan pada antarmuka jika berkas sumber mengandung bentrok kamar atau anomali data kamar (tanpa mengubah nomor kamar asli).
* **Berkas Terkait:**
  * Backend: `src-tauri/src/optimizer.rs` (`solve_subset()`, `analyze_room_issues()`), `src-tauri/src/excel.rs` (`analyze_excel_file()`, `export_solution_to_excel()`)
  * Frontend: `src/app.js` (`runOptimizerSolver()`, `exportOptimizerExcel()`)
* **Pengaturan:** Parameter `tolerance` (default 100.0) dan `max_iterations` (default 50.000).

---

## Modul 6: Multi-Unit Usaha & Manajemen Sistem

### FEAT-07: Manajemen Multi-Unit Usaha
* **Kegunaan:** Memisahkan pembukuan dan pelaporan pajak antara unit Bungalows dan Restaurant.
* **Pemicu:** Pengguna memilih switcher unit di header aplikasi atau mengelola unit di modal pengaturan.
* **Aturan Kerja:**
  1. Setiap transaksi terikat pada `unit` tertentu.
  2. Seluruh filter rekapitulasi, buku besar, dan impor berkas terisolasi sesuai unit aktif.
  3. Kode unit otomatis disingkat (contoh: `BGW` untuk Bungalows, `RST` untuk Restaurant) pada nomor dokumen resmi.
* **Berkas Terkait:**
  * Backend: `src-tauri/src/db.rs` (`get_units()`, `add_unit()`, `update_unit()`)
  * Frontend: `src/app.js` (`initNav()`, `loadActiveView()`)
* **Pengaturan:** Tabel `units` pada SQLite.

### FEAT-08: Backup Online SQLite & Pemulihan Bencana
* **Kegunaan:** Menjamin keamanan data operasional dengan fasilitas pencadangan hot-backup tanpa menghentikan aplikasi.
* **Pemicu:** Pengguna menekan tombol Cadangkan Database Sekarang atau Pulihkan Database dari modal pengaturan.
* **Aturan Kerja:**
  1. Pencadangan menggunakan SQLite Online Backup API (`rusqlite::backup`), menghasilkan berkas `.sqlite` mandiri yang konsisten.
  2. Sebelum memulihkan data dari berkas cadangan, aplikasi membuat salinan darurat otomatis `.pre_restore.bak` dari database aktif.
  3. Setelah pemulihan selesai, aplikasi memuat ulang seluruh data memori dan menyegarkan tampilan modul aktif.
* **Berkas Terkait:**
  * Backend: `src-tauri/src/db.rs` (`backup_database()`, `restore_database()`)
  * Frontend: `src/app.js` (`initSettingsModal()`)
* **Pengaturan:** Lokasi penyimpanan database SQLite lokal di folder data aplikasi pengguna.

---

## Modul 7: Pendataan Wisatawan Dinas Pariwisata

### FEAT-09: Rekap Bulanan Pendataan Wisatawan (Form 1 Dinas Pariwisata)
* **Kegunaan:** Menyediakan seluruh angka siap salin untuk mengisi formulir resmi **Pendataan Wisatawan Per Bulan (Form 1: Dinas Pariwisata Kabupaten Lombok Tengah)** secara otomatis dari data reservasi 13 kolom.
* **Pemicu:** Pengguna memilih menu **Pajak Daerah → Pendataan Wisatawan (Form 1)**, lalu memilih periode bulan, kapasitas kamar, dan jenis akomodasi.
* **Aturan Kerja:**
  1. Memecah periode bulan ke dalam 4 baris mingguan (`Minggu 1 (1-7)`, `Minggu 2 (8-14)`, `Minggu 3 (15-21)`, `Minggu 4 (22-akhir bulan)`).
  2. Mengklasifikasikan tamu **Domestik (Nusantara)** vs **Mancanegara** berdasarkan kolom `nationality` (mendukung kota/38 provinsi Indonesia, notasi eksplisit `IDN - Kota` / `INTL - Negara`, serta seluruh ~198 negara/wilayah dunia termasuk `Papua Nugini` dan input negara kustom tak terdaftar) serta menghitung jumlah tamu Check-In dan Check-Out per minggu.
  3. Menghitung `Occupied` dan `Vacant` kamar dalam satuan *room-night* serta `Check Out (Bed)` dan `Occupied (Bed)` tempat tidur dalam satuan *bed-night* berdasarkan rentang hari menginap aktif (`check_in_date <= tanggal < check_out_date`).
  4. Mengagregasi kunjungan wisatawan per negara/daerah asal yang diurutkan dari jumlah tamu terbanyak (menampilkan 10 baris sesuai blanko cetak beserta daftar lengkap bila melebihi 10 asal).
  5. Menghitung rata-rata lama tinggal (malam) beserta kategori indikator `1-2 MALAM`, `3 MALAM`, atau `> 3 MALAM`.
* **Berkas Terkait:**
  * Backend: `src-tauri/src/tax_report.rs` (`generate_tourism_report()`, `is_domestic_origin()`, `normalize_origin_label()`), `src-tauri/src/excel.rs` (`generate_import_template()`), `src-tauri/src/commands.rs` (`get_tourism_report()`)
  * Frontend: `src/app.js` (`initTourismModule()`, `loadTourismData()`), `src/index.html` (`#view-tourism`)
* **Pengaturan:** Jumlah kamar (`anda_erp_room_capacity`, default `14`) dan jenis akomodasi (`anda_erp_acc_type`, default `Hotel Non Bintang`).

---

## Modul 8: Sertifikat Pelatihan Properti

### FEAT-10: Generator & Arsip Sertifikat Training (`ANDA BUNGALOWS & RESTAURANT`)
* **Kegunaan:** Menerbitkan, menyimpan, mengelola, dan mengekspor sertifikat penyelesaian program *training* secara konsisten sesuai desain Canva properti tanpa aplikasi desain eksternal.
* **Pemicu:** Pengguna memilih menu langsung **Sertifikat Training** pada bilah navigasi utama (tersedia global untuk seluruh properti).
* **Aturan Kerja:**
  1. Mendukung pilihan ukuran kertas **F4 / Folio Horizontal (`330 x 215 mm`, bawaan)** dan **A4 Horizontal (`297 x 210 mm`)** dengan bingkai emas `#C67D24` berjarak `10 mm` di keempat sisi serta tipografi terbundel luring (`Nunito Sans Condensed`, `Source Serif Pro`, `Libre Baskerville`, `Roboto`).
  2. Menomori sertifikat secara otomatis dengan format `000/ABR/YYYY` (`001/ABR/2026`, `002/ABR/2026`, di-reset otomatis menjadi `001` saat pergantian tahun terbit), tetap dapat disunting manual, dan dijamin unik (*case-insensitive*) di tabel SQLite `training_certificates`.
  3. Menerima format tanggal fleksibel (`YYYY-MM-DD`, `DD/MM/YY`, dll.), menyarankan durasi berbahasa Inggris (`three-month`, `six-month`), mendukung input bidang/divisi tunggal maupun rotasi lintas divisi beserta durasinya dengan penyisipan otomatis `"the ... Department"` (mis. `in the Housekeeping Department (3 months) and the Food & Beverage Department (3 months)`), template predikat/kompetensi opsional, serta menulis rentang tanggal lengkap bertahun pada kedua tanggal (`from June 1, 2026 to November 21, 2026.`) sambil menghilangkan baris berulang `Awarded on` apabila tanggal terbit sama dengan tanggal selesai; tombol **Simpan & Tambah Berikutnya** tetap mempertahankan periode, divisi, predikat, program, sekolah, penandatangan, dan kertas.
  4. Menggunakan satu sumber SVG kanonik untuk **pratinjau langsung di layar**, **ekspor PDF vektor** (`svg2pdf`), **unduhan PNG 300 DPI** (`resvg` + `tiny-skia`), serta **dialog cetak sistem borderless**.
  5. Kontrol akses berbasis peran: `admin` dan `staff` dapat membuat sertifikat baru, memantau riwayat, mencari/memfilter berdasarkan tahun atau status, mencetak, dan mengekspor PDF/PNG; hanya `admin` yang dapat menyunting data tersimpan, membatalkan/mengaktifkan status (`active` / `revoked`), menghapus rekaman, mengganti gambar logo PNG atau tanda tangan digital opsional, serta mengubah pengaturan penandatangan bawaan.
* **Berkas Terkait:**
  * Backend: `src-tauri/src/certificate.rs` (`render_certificate_svg()`, `render_certificate_pdf_bytes()`, `render_certificate_png_bytes()`, `validate_and_save_asset()`), `src-tauri/src/db.rs` (`create_certificate()`, `update_certificate()`, `get_paginated_certificates()`, `get_next_certificate_number()`), `src-tauri/src/commands.rs` (`create_certificate()`, `preview_certificate_svg()`, `export_certificate_pdf()`, `export_certificate_png()`, `print_certificate()`)
  * Frontend: `src/app.js` (`initCertificatesModule()`, `submitCertificateForm()`, `exportCertificateFormat()`, `printCertificateDocument()`), `src/index.html` (`#view-certificates`, `#dialog-cert-settings`), `src/style.css`, `src/assets/fonts/`
* **Pengaturan:** Kunci `certSignerName` (bawaan `Agus Sumardi Wijaya`), `certSignerTitle` (bawaan `General Manager`), `certProgramName` (bawaan `hospitality training program`), `certPaperSize` (`F4`), `certShowSignature` (`0`), `certLogoFile`, dan `certSignatureFile` pada tabel `settings` serta folder `cert_assets/`.
