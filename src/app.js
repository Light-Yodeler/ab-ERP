// ==========================================================================
// ANDA ERP - Desktop Frontend Logic (Vanilla JS)
// ==========================================================================

(function () {
  'use strict';

  // Tauri IPC invoke wrapper with graceful fallback for development/testing
  const tauriInvoke = window.__TAURI__?.core?.invoke;
  const isTauri = typeof tauriInvoke === 'function';
  const APP_DEFAULT_TITLE = document.title || 'Anda ERP - Sistem Pelaporan Pendapatan & Pajak Daerah';
  function getCleanReportFileName(docNum) {
    if (!docNum) return '';
    return String(docNum)
      .replace(/^No:\s*/i, '')
      .replace(/[\/\\:*?"<>|]/g, '-');
  }

  async function invoke(cmd, args = {}) {
    if (isTauri) {
      return await tauriInvoke(cmd, args);
    }
    console.log(`[IPC Mock] ${cmd}:`, args);

    // Mock implementations for offline browser preview
    if (cmd === 'auth_login') {
      if (args.pin === '123456' || args.pin === '1234') {
        return {
          id: args.username === 'admin' ? 1 : 2,
          username: args.username,
          full_name: args.username === 'admin' ? 'Administrator' : 'Staff Pelaporan',
          role: args.username === 'admin' ? 'admin' : 'staff',
        };
      }
      throw new Error('PIN yang Anda masukkan salah.');
    }
    if (cmd === 'get_available_months') {
      return ['2026-08', '2026-07'];
    }
    if (cmd === 'get_tax_recap') {
      const days = 31;
      const items = Array.from({ length: days }, (_, i) => {
        const d = i + 1;
        const rev = d % 3 === 0 ? 0 : 2500000 + (d * 120000);
        return {
          day: d,
          date: `2026-08-${String(d).padStart(2, '0')}`,
          day_name: ['Senin', 'Selasa', 'Rabu', 'Kamis', 'Jumat', 'Sabtu', 'Minggu'][i % 7],
          revenue: rev,
          is_active: rev > 0,
          cumulative_revenue: 0,
        };
      });
      let cum = 0;
      let active = 0;
      for (const it of items) {
        cum += it.revenue;
        it.cumulative_revenue = cum;
        if (it.is_active) active++;
      }
      const taxDue = Math.round(cum * 0.10);
      return {
        unit: args.unit,
        unit_name: args.unit === 'Restaurant' ? 'ANDA RESTAURANT' : 'ANDA BUNGALOWS',
        month: args.month,
        month_name: 'Agustus 2026',
        year: 2026,
        month_number: 8,
        days_in_month: 31,
        total_revenue: cum,
        tax_rate: 0.10,
        tax_due: taxDue,
        net_revenue: cum - taxDue,
        active_days: active,
        zero_days: 31 - active,
        max_daily_revenue: 6220000,
        min_daily_revenue_non_zero: 2620000,
        avg_per_day: cum / 31,
        document_number: `No: ANDA-${args.unit.toUpperCase()}/2026-08/9A4F82C1`,
        npwpd: 'P.2.0001234.01.23',
        business_name: 'ANDA BUNGALOWS & RESTAURANT',
        business_address: 'Jalan Pariwisata Pantai Kuta, Kecamatan Pujut, Lombok Tengah, NTB',
        sign_position: 'Wajib Pajak Daerah',
        sign_name: 'Pimpinan / Pengelola',
        officer_name: 'Petugas Bapenda Lombok Tengah',
        officer_nip: 'NIP. 19800101 200501 1 001',
        generated_at: '02-11-2026 14:00:00',
        daily_items: items,
      };
    }
    if (cmd === 'get_transactions') {
      return {
        items: [
          { id: 101, unit: 'Bungalows', date: '2026-08-31', amount: 3500000, notes: 'Kamar 01 & 02', source_file: 'import.xlsx', created_at: '2026-08-31' },
          { id: 102, unit: 'Bungalows', date: '2026-08-30', amount: 4200000, notes: 'Kamar VIP', source_file: 'import.xlsx', created_at: '2026-08-30' },
        ],
        total_count: 2,
        total_amount: 7700000,
        page: 1,
        page_size: 25,
        total_pages: 1,
      };
    }
    if (cmd === 'get_room_reservations') {
      return {
        items: [
          {
            id: 1,
            unit: 'Bungalows',
            transaction_id: 101,
            date: '2026-10-01',
            guest_name: 'Paul Hutson',
            total_person: 2,
            booking_number: '-',
            nationality: 'Australia',
            room_number: '02',
            total_rooms: 1,
            check_in_date: '2026-10-01',
            check_out_date: '2026-10-03',
            total_nights: 2,
            rate: 1250000,
            total_payment: 2500000,
            remarks: 'Walk in',
            source_file: 'Rekap_Tamu_Oktober_2026.xlsx',
            created_at: '2026-10-01',
          },
        ],
        total_count: 1,
        total_person: 2,
        total_rooms: 1,
        total_nights: 2,
        total_payment: 2500000,
        page: 1,
        page_size: 25,
        total_pages: 1,
      };
    }
    if (cmd === 'get_settings') {
      return {
        businessName: 'ANDA BUNGALOWS & RESTAURANT',
        businessAddress: 'Jalan Pariwisata Pantai Kuta, Lombok Tengah, NTB',
        npwpd: 'P.2.0001234.01.23',
        signPosition: 'Wajib Pajak Daerah',
        signName: 'Pimpinan / Pengelola',
        officerName: 'Petugas Bapenda Lombok Tengah',
        officerNip: 'NIP. 19800101 200501 1 001',
      };
    }
    if (cmd === 'get_storage_info') {
      return {
        db_path: '/Users/.../com.andabungalows.erp/laporan_bapenda.sqlite',
        db_size_bytes: 192512,
        archive_dir: '/Users/.../com.andabungalows.erp/arsip_excel',
        archive_size_bytes: 420100,
        total_transactions: 455,
        total_units: 2,
        total_import_logs: 14,
      };
    }
    if (cmd === 'preview_import_file') {
      const fname = args.filePath.split('/').pop().split('\\').pop();
      const ext = fname.includes('.') ? fname.split('.').pop().toLowerCase() : '';
      if (!['xlsx', 'xls', 'xlsm'].includes(ext)) {
        throw new Error(`Format berkas '.${ext}' tidak didukung. Harap pilih berkas Excel (.xlsx, .xls, .xlsm).`);
      }
      return {
        file_name: fname,
        file_size_bytes: 45210,
        file_hash: 'mockhash1234567890abcdef',
        detected_unit: args.targetUnit || 'Bungalows',
        target_sheet: 'Bungalows',
        total_sheet_rows: 360,
        valid_rows_count: 358,
        total_amount: 436217456,
        periods_detected: ['2026-08'],
        period_string: '2026-08',
        is_duplicate_file: false,
        duplicate_log: null,
        existing_period_transactions_count: 358,
      };
    }
    if (cmd === 'import_excel_file') {
      const fname = args.filePath.split('/').pop().split('\\').pop();
      return {
        unit: args.options?.targetUnit || 'Bungalows',
        detected_unit: 'Bungalows',
        file_name: fname,
        imported_records: 358,
        skipped_records: 0,
        total_amount: 436217456,
        months_affected: ['2026-08'],
        period: '2026-08',
        file_hash: 'mockhash1234567890abcdef',
        stored_path: '/path/to/archive/import.xlsx',
      };
    }
    if (cmd === 'get_import_logs') {
      return [
        { id: 1, unit: 'Bungalows', file_name: 'Contoh_Pendapatan_Bungalows.xlsx', records_count: 358, total_amount: 436217456, period: '2026-08', file_hash: 'mockhash1', imported_at: '2026-08-31 14:14:31', stored_path: '' }
      ];
    }
    if (cmd === 'auth_list_users') {
      return [
        { id: 1, username: 'admin', full_name: 'Administrator', role: 'admin', last_login: 'Hari ini' },
        { id: 2, username: 'staff', full_name: 'Staff Pelaporan', role: 'staff', last_login: 'Kemarin' },
      ];
    }
    if (cmd === 'download_excel_template') {
      return `Template Excel ${args.unit || 'Bungalows'} berhasil disimpan.`;
    }
    if (cmd === 'print_report') {
      window.print();
      return;
    }
    if (cmd === 'set_window_title') {
      return;
    }
    if (cmd === 'get_origin_reference_list') {
      return {
        domestic_regions: ['Mataram', 'Jakarta', 'Surabaya', 'Bali', 'Bandung'],
        world_countries: ['Australia', 'Prancis', 'Jerman', 'Malaysia', 'Singapura'],
      };
    }
    if (cmd === 'export_room_reservations_excel') {
      return 'Berhasil mengekspor data reservasi (13 kolom) [Mock].';
    }
    if (cmd === 'get_tourism_report') {
      const cap = args.roomCapacity || 14;
      return {
        unit: args.unit || 'Bungalows',
        accommodation_name: 'ANDA BUNGALOWS',
        accommodation_type: 'Hotel Non Bintang',
        room_capacity: cap,
        month: args.month || '2026-10',
        month_name: 'OKTOBER',
        year: 2026,
        days_in_month: 31,
        total_reservations: 55,
        total_guests: 108,
        domestic_guests: 4,
        foreign_guests: 104,
        total_room_nights: 112,
        total_bed_nights: 210,
        occupancy_rate_pct: (112 / (cap * 31)) * 100,
        avg_stay_nights: 1.93,
        stay_category: '1-2 MALAM',
        weekly_rows: [
          { week_number: 1, label: 'Minggu 1 (1-7)', days_count: 7, domestic_check_in: 0, domestic_check_out: 0, foreign_check_in: 14, foreign_check_out: 10, room_occupied: 25, room_vacant: cap * 7 - 25, bed_check_out: 10, bed_occupied: 42 },
          { week_number: 2, label: 'Minggu 2 (8-14)', days_count: 7, domestic_check_in: 2, domestic_check_out: 2, foreign_check_in: 22, foreign_check_out: 20, room_occupied: 28, room_vacant: cap * 7 - 28, bed_check_out: 22, bed_occupied: 50 },
          { week_number: 3, label: 'Minggu 3 (15-21)', days_count: 7, domestic_check_in: 0, domestic_check_out: 0, foreign_check_in: 18, foreign_check_out: 17, room_occupied: 26, room_vacant: cap * 7 - 26, bed_check_out: 17, bed_occupied: 49 },
          { week_number: 4, label: 'Minggu 4 (22-31)', days_count: 10, domestic_check_in: 2, domestic_check_out: 2, foreign_check_in: 50, foreign_check_out: 48, room_occupied: 33, room_vacant: cap * 10 - 33, bed_check_out: 50, bed_occupied: 69 },
        ],
        weekly_total: { week_number: 0, label: 'TOTAL', days_count: 31, domestic_check_in: 4, domestic_check_out: 4, foreign_check_in: 104, foreign_check_out: 95, room_occupied: 112, room_vacant: cap * 31 - 112, bed_check_out: 99, bed_occupied: 210 },
        foreign_origins: [
          { rank: 1, origin_name: 'Prancis', guest_count: 20, reservation_count: 8 },
          { rank: 2, origin_name: 'Australia', guest_count: 14, reservation_count: 7 },
        ],
        domestic_origins: [
          { rank: 1, origin_name: 'Indonesia', guest_count: 4, reservation_count: 2 },
        ],
      };
    }
    if (cmd === 'get_certificate_settings') {
      return {
        signer_name: 'Agus Sumardi Wijaya',
        signer_title: 'General Manager',
        program_name: 'hospitality training program',
        paper_size: 'F4',
        issued_city: 'Kuta, Lombok',
        body_template: 'for successfully completing the {durasi} {jenis_program} in the {departemen} at {properti}, from {mulai} to {selesai}.',
        show_signature: false,
        has_custom_logo: false,
        has_custom_signature: false,
        logo_data_uri: '',
        signature_data_uri: null,
      };
    }
    if (cmd === 'get_departments') {
      if (!window.__mockDepartments) {
        window.__mockDepartments = [
          { id: 1, code: 'KIT', name: 'Kitchen', program_description: 'The program covered food preparation, kitchen hygiene, and menu production.', competencies: 'Basic cooking techniques\nFood safety (HACCP)\nIngredient handling\nFood plating', is_active: true, sort_order: 10 },
          { id: 2, code: 'FB', name: 'Food & Beverage Service', program_description: 'The program covered restaurant service, guest handling, and table setup.', competencies: 'Table service\nMenu knowledge\nGuest complaint handling\nUpselling techniques', is_active: true, sort_order: 20 },
          { id: 3, code: 'BAR', name: 'Bar', program_description: 'The program covered beverage preparation and bar operations.', competencies: 'Beverage preparation\nProduct knowledge\nBar inventory control', is_active: true, sort_order: 30 },
          { id: 4, code: 'FO', name: 'Front Office', program_description: 'The program covered guest reception, reservations, and check-in/check-out procedures.', competencies: 'Guest registration\nReservation systems\nGuest communication\nPayment handling', is_active: true, sort_order: 40 },
          { id: 5, code: 'HK', name: 'Housekeeping', program_description: 'The program covered room preparation, cleanliness standards, and laundry operations.', competencies: 'Room cleanliness standards\nBed making\nLinen handling\nPublic area maintenance', is_active: true, sort_order: 50 },
          { id: 6, code: 'MNT', name: 'Maintenance / Engineering', program_description: 'The program covered facility upkeep and basic technical repairs.', competencies: 'Electrical maintenance\nPlumbing repairs\nAir conditioning maintenance\nPool & garden upkeep', is_active: true, sort_order: 60 },
          { id: 7, code: 'SM', name: 'Sales & Marketing', program_description: 'The program covered promotional campaigns, online booking channels, and guest engagement.', competencies: 'Social media management\nOTA channel management (Booking.com, Agoda)\nGuest inquiry handling', is_active: true, sort_order: 70 },
          { id: 8, code: 'ADM', name: 'Administration / Accounting', program_description: 'The program covered hotel administration, cashiering, and basic bookkeeping.', competencies: 'Transaction recording\nInvoicing\nDaily financial reporting', is_active: true, sort_order: 80 },
          { id: 9, code: 'SEC', name: 'Security', program_description: 'The program covered guest safety, emergency response, and property security procedures.', competencies: 'Property patrols\nEmergency procedures\nAccess control', is_active: true, sort_order: 90 },
          { id: 10, code: 'SPA', name: 'Spa & Recreation', program_description: 'The program covered spa treatments and guest activity coordination.', competencies: 'Spa treatment techniques\nEquipment sanitation\nGuest activity coordination', is_active: true, sort_order: 100 },
        ];
      }
      return [...window.__mockDepartments];
    }
    if (cmd === 'save_department') {
      const p = args.payload || {};
      const list = window.__mockDepartments || [];
      if (p.id) {
        const idx = list.findIndex((d) => d.id === p.id);
        if (idx >= 0) {
          list[idx] = { ...list[idx], ...p, code: String(p.code || '').toUpperCase() };
          return list[idx];
        }
      }
      const nextId = list.reduce((m, d) => Math.max(m, d.id), 0) + 1;
      const created = { id: nextId, code: String(p.code || '').toUpperCase(), name: p.name || '', program_description: p.program_description || '', competencies: p.competencies || '', is_active: true, sort_order: nextId * 10 };
      list.push(created);
      return created;
    }
    if (cmd === 'delete_department') {
      window.__mockDepartments = (window.__mockDepartments || []).filter((d) => d.id !== args.id);
      return;
    }
    if (cmd === 'get_program_types') {
      if (!window.__mockProgramTypes) {
        window.__mockProgramTypes = [
          { id: 1, code: 'OJT', name: 'on-the-job training program', default_duration_months: 6, description: 'Belajar langsung sambil bekerja di bawah supervisor', is_active: true, sort_order: 10 },
          { id: 2, code: 'INT', name: 'internship program', default_duration_months: 3, description: 'Magang terstruktur, sering bagian dari kurikulum kampus', is_active: true, sort_order: 20 },
          { id: 3, code: 'APP', name: 'apprenticeship program', default_duration_months: 6, description: 'Pelatihan jangka panjang dengan jenjang keterampilan', is_active: true, sort_order: 30 },
          { id: 4, code: 'MT', name: 'management trainee program', default_duration_months: 6, description: 'Rotasi antar departemen untuk calon pemimpin', is_active: true, sort_order: 40 },
          { id: 5, code: 'WS', name: 'short course / workshop', default_duration_months: 1, description: 'Pelatihan singkat topik tertentu (1-5 hari)', is_active: true, sort_order: 50 },
          { id: 6, code: 'REF', name: 'refresher / skill upgrade program', default_duration_months: 1, description: 'Penyegaran keterampilan atau standar baru', is_active: true, sort_order: 60 },
          { id: 7, code: 'CERT', name: 'certification course', default_duration_months: 1, description: 'Pelatihan dengan ujian (food safety, first aid, fire safety)', is_active: true, sort_order: 70 },
        ];
      }
      return [...window.__mockProgramTypes];
    }
    if (cmd === 'save_program_type') {
      const p = args.payload || {};
      const list = window.__mockProgramTypes || [];
      if (p.id) {
        const idx = list.findIndex((d) => d.id === p.id);
        if (idx >= 0) {
          list[idx] = { ...list[idx], ...p, code: String(p.code || '').toUpperCase() };
          return list[idx];
        }
      }
      const nextId = list.reduce((m, d) => Math.max(m, d.id), 0) + 1;
      const created = { id: nextId, code: String(p.code || '').toUpperCase(), name: p.name || '', default_duration_months: Number(p.default_duration_months) || 3, description: p.description || '', is_active: true, sort_order: nextId * 10 };
      list.push(created);
      return created;
    }
    if (cmd === 'delete_program_type') {
      window.__mockProgramTypes = (window.__mockProgramTypes || []).filter((d) => d.id !== args.id);
      return;
    }
    if (cmd === 'get_next_certificate_no') {
      const romans = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII', 'VIII', 'IX', 'X', 'XI', 'XII'];
      const y = args.year || new Date().getFullYear();
      const mIdx = Math.max(1, Math.min(12, Number(args.month) || (new Date().getMonth() + 1))) - 1;
      const dept = args.departmentCode ? `-${String(args.departmentCode).toUpperCase()}` : '';
      return `001/ABR${dept}/${romans[mIdx]}/${y}`;
    }
    if (cmd === 'get_certificate_years') {
      return [String(new Date().getFullYear())];
    }
    if (cmd === 'get_certificates') {
      return {
        items: [],
        total_count: 0,
        active_count: 0,
        revoked_count: 0,
        page: args.page || 1,
        page_size: args.pageSize || 25,
        total_pages: 1,
      };
    }
    if (cmd === 'preview_certificate_svg') {
      const p = args.payload || {};
      const isA4 = String(p.paper_size || 'F4').toUpperCase() === 'A4';
      const w = isA4 ? 297 : 330;
      const h = isA4 ? 210 : 215;
      const cx = w / 2;
      const fmtMockDept = (s) => {
        const clean = String(s).trim().replace(/^the\s+/i, '');
        const m = /^(.*?)\s*\(([^()]+)\)\s*$/.exec(clean);
        const baseRaw = m ? m[1].trim() : clean;
        const base = /department$|dept\.?$/i.test(baseRaw) ? baseRaw : `${baseRaw} Department`;
        return m ? `the ${base} (${m[2].trim()})` : `the ${base}`;
      };
      const deptItems = String(p.department || '').split(';').map(s => s.trim()).filter(Boolean).map(fmtMockDept);
      const divLine = deptItems.length ? `in ${escapeHtml(deptItems.join(' and '))}` : '';
      const achLine = p.achievement_text ? `<text x="${cx}" y="156" text-anchor="middle" font-size="3.6" fill="#000000">${escapeHtml(p.achievement_text)}</text>` : '';
      const issuedLine = p.issued_date
        ? `<text x="${cx}" y="164" text-anchor="middle" font-size="3.3" fill="#3F3F46">${p.issued_city ? `Issued in ${escapeHtml(p.issued_city)}, on ${escapeHtml(p.issued_date)}` : `Issued on ${escapeHtml(p.issued_date)}`}</text>`
        : '';
      return `<svg xmlns="http://www.w3.org/2000/svg" width="${w}mm" height="${h}mm" viewBox="0 0 ${w} ${h}"><rect x="0" y="0" width="${w}" height="${h}" fill="#FFFFFF"/><rect x="10" y="10" width="${w - 20}" height="${h - 20}" fill="none" stroke="#C67D24" stroke-width="0.53"/><text x="${w - 20}" y="21.25" text-anchor="end" font-size="3.53" fill="#545454">Certificate No. ${escapeHtml(p.certificate_no || '[000/ABR/I/YYYY]')}</text><text x="${cx}" y="86" text-anchor="middle" font-size="14" fill="#1F1F1F">Certificate of Completion</text><text x="${cx}" y="130" text-anchor="middle" font-weight="700" font-size="12" fill="#000000">${escapeHtml(p.recipient_name || '[Recipient Name]')}</text><text x="${cx}" y="148" text-anchor="middle" font-size="3.8" fill="#000000">${divLine}</text>${achLine}${issuedLine}</svg>`;
    }
    if (cmd === 'print_certificate') {
      window.print();
      return;
    }
    if (cmd === 'export_certificate_pdf' || cmd === 'export_certificate_png') {
      return 'Berkas sertifikat berhasil diekspor (Mode Mock).';
    }
    throw new Error(`Perintah '${cmd}' belum didukung dalam mode mock.`);
  }

  // Application State
  const state = {
    currentUser: null,
    selectedLoginUser: 'admin',
    pinBuffer: [],
    activeUnit: 'Bungalows',
    activeTab: 'recap',
    recapMonth: '',
    availableMonths: [],
    currentRecap: null,
    ledgerPage: 1,
    ledgerPageSize: 25,
    ledgerMonth: 'all',
    ledgerSearch: '',
    ledgerMode: 'summary',
    ledgerReservations: [],
    tourismMonth: '',
    tourismRoomCapacity: Number(localStorage.getItem('anda_erp_room_capacity')) || 14,
    tourismAccType: localStorage.getItem('anda_erp_acc_type') || 'Hotel Non Bintang',
    // Certificates Module State
    certPage: 1,
    certTotalPages: 1,
    certPageSize: 25,
    certYear: 'all',
    certStatus: 'all',
    certSearch: '',
    certEditingId: null,
    certEditingStatus: 'active',
    certManualNo: false,
    certManualDuration: false,
    certManualDates: false,
    certStaffSavedLock: false,
    certInitialized: false,
    certSettings: null,
    certDepartments: [],
    certProgramTypes: [],
    certRows: [],
    certPreviewTimer: null,
    certPreviewSeq: 0,
    certCleanSnapshot: '',
    certPreviewRow: null,
    // Optimizer Module State
    optFilePath: null,
    optAnalysis: null,
    optResult: null,
    optSeenSignatures: [],
    theme: localStorage.getItem('anda_erp_theme') || 'light',
  };

  // HTML Escape Helper
  function escapeHtml(val) {
    if (val === null || val === undefined) return '';
    return String(val)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  // Currency & Number Formatter
  function formatRupiah(num) {
    if (num === null || num === undefined || isNaN(num)) return 'Rp 0';
    const parts = num.toFixed(2).split('.');
    const integerPart = parseInt(parts[0], 10).toLocaleString('id-ID');
    const decimalPart = parts[1];
    return decimalPart === '00' ? `Rp ${integerPart}` : `Rp ${integerPart},${decimalPart}`;
  }

  function formatBytes(bytes) {
    if (!bytes || bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
  // System Local Date & Time Helpers
  function getSystemLocalDateStr(d = new Date()) {
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }

  function getMonthsAgoLocalDateStr(monthsBack) {
    const now = new Date();
    const d = new Date(now.getFullYear(), now.getMonth() - monthsBack, 1);
    const maxDay = new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
    d.setDate(Math.min(now.getDate(), maxDay));
    return getSystemLocalDateStr(d);
  }

  function getSystemLocalMonthStr(d = new Date()) {
    const year = d.getFullYear();
    const month = String(d.getMonth() + 1).padStart(2, '0');
    return `${year}-${month}`;
  }

  function formatDateTime(val) {
    if (!val || val === '-') return '-';
    if (typeof val === 'string' && (val.includes('T') || val.endsWith('Z'))) {
      const dt = new Date(val);
      if (!isNaN(dt.getTime())) {
        const y = dt.getFullYear();
        const m = String(dt.getMonth() + 1).padStart(2, '0');
        const d = String(dt.getDate()).padStart(2, '0');
        const hh = String(dt.getHours()).padStart(2, '0');
        const mm = String(dt.getMinutes()).padStart(2, '0');
        const ss = String(dt.getSeconds()).padStart(2, '0');
        return `${y}-${m}-${d} ${hh}:${mm}:${ss}`;
      }
    }
    return String(val);
  }

  function initSystemClock() {
    const clockEl = document.getElementById('system-clock-time');
    if (!clockEl) return;
    const updateClock = () => {
      const now = new Date();
      const timeStr = now.toLocaleTimeString('id-ID', { hour12: false });
      const dateStr = now.toLocaleDateString('id-ID', { day: '2-digit', month: 'short', year: 'numeric' });
      clockEl.textContent = `${dateStr} · ${timeStr}`;
    };
    updateClock();
    setInterval(updateClock, 1000);
  }


  // =========================================================================
  // 1. PIN Authentication Logic
  // =========================================================================
  function bindLoginUserButtons() {
    const errorBox = document.getElementById('login-error');
    const userBtns = document.querySelectorAll('.btn-user-select');
    userBtns.forEach(btn => {
      btn.addEventListener('click', () => {
        userBtns.forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        state.selectedLoginUser = btn.dataset.username;
        state.pinBuffer = [];
        updatePinDots();
        if (errorBox) errorBox.classList.add('hidden');
      });
    });
  }

  async function refreshLoginUsers() {
    const container = document.querySelector('.user-selector');
    if (!container) return;

    try {
      const users = await invoke('auth_list_users');
      if (Array.isArray(users) && users.length > 0) {
        if (!users.some(u => u.username === state.selectedLoginUser)) {
          state.selectedLoginUser = users[0].username;
        }
        container.innerHTML = users.map(u => {
          const isActive = u.username === state.selectedLoginUser;
          return `
            <button type="button" class="btn-user-select${isActive ? ' active' : ''}" data-username="${escapeHtml(u.username)}">
              <span class="user-role-badge">${escapeHtml(u.role)}</span>
              <span class="user-name">${escapeHtml(u.full_name)}</span>
            </button>
          `;
        }).join('');
      }
    } catch (err) {
      console.warn('Gagal memuat daftar akun login, menggunakan default:', err);
    }

    bindLoginUserButtons();
  }

  function applyRolePermissions(role) {
    const isAdmin = role === 'admin';
    const usersTabBtn = document.querySelector('.settings-tab[data-stab="users"]');
    const usersPane = document.getElementById('stab-users');
    const btnRestore = document.getElementById('btn-restore-now');
    const btnSaveProfile = document.getElementById('btn-save-profile-settings');

    if (usersTabBtn) usersTabBtn.classList.toggle('hidden', !isAdmin);
    if (!isAdmin && usersPane && usersPane.classList.contains('active')) {
      document.querySelectorAll('.settings-tab').forEach(t => t.classList.remove('active'));
      document.querySelectorAll('.settings-pane').forEach(p => p.classList.remove('active'));
      document.querySelector('.settings-tab[data-stab="profile"]')?.classList.add('active');
      document.getElementById('stab-profile')?.classList.add('active');
    }
    if (btnRestore) btnRestore.classList.toggle('hidden', !isAdmin);
    if (btnSaveProfile) btnSaveProfile.disabled = !isAdmin;

    const replacePeriodRadio = document.querySelector('input[name="import-mode"][value="replace_period"]');
    const replaceAllRadio = document.querySelector('input[name="import-mode"][value="replace_all_unit"]');
    const skipDupRadio = document.querySelector('input[name="import-mode"][value="skip_duplicates"]');
    [replacePeriodRadio, replaceAllRadio].forEach(el => {
      if (!el) return;
      el.disabled = !isAdmin;
      el.closest('.radio-label')?.classList.toggle('is-disabled', !isAdmin);
    });
    if (!isAdmin && skipDupRadio) {
      skipDupRadio.checked = true;
    }

    const btnCertSettings = document.getElementById('btn-cert-settings');
    if (btnCertSettings) {
      btnCertSettings.classList.toggle('hidden', !isAdmin);
    }
    const btnCertCatalog = document.getElementById('btn-cert-catalog');
    if (btnCertCatalog) {
      btnCertCatalog.classList.toggle('hidden', !isAdmin);
    }
    if (state.certInitialized) {
      renderCertificateHistoryRows();
    }

    updateLedgerModeUI();
  }

  function initAuth() {
    const keyBtns = document.querySelectorAll('.btn-key');
    const btnSubmit = document.getElementById('btn-submit-pin');

    bindLoginUserButtons();
    refreshLoginUsers();

    keyBtns.forEach(btn => {
      btn.addEventListener('click', () => {
        const key = btn.dataset.key;
        handlePinInput(key);
      });
    });

    // Keyboard support for login PIN
    window.addEventListener('keydown', (e) => {
      const loginOverlay = document.getElementById('login-screen');
      if (!loginOverlay || loginOverlay.classList.contains('hidden')) return;

      if (e.key >= '0' && e.key <= '9') {
        handlePinInput(e.key);
      } else if (e.key === 'Backspace') {
        handlePinInput('backspace');
      } else if (e.key === 'Enter') {
        submitLogin();
      } else if (e.key === 'Escape') {
        handlePinInput('clear');
      }
    });

    if (btnSubmit) {
      btnSubmit.addEventListener('click', submitLogin);
    }
  }

  function handlePinInput(key) {
    const errorBox = document.getElementById('login-error');
    if (errorBox) errorBox.classList.add('hidden');

    if (key === 'clear') {
      state.pinBuffer = [];
    } else if (key === 'backspace') {
      state.pinBuffer.pop();
    } else if (state.pinBuffer.length < 8) {
      state.pinBuffer.push(key);
    }
    updatePinDots();
  }

  function updatePinDots() {
    const dots = document.querySelectorAll('#pin-dots .dot');
    dots.forEach((dot, index) => {
      if (index < state.pinBuffer.length) {
        dot.classList.add('filled');
      } else {
        dot.classList.remove('filled');
      }
    });
  }

  async function submitLogin() {
    const errorBox = document.getElementById('login-error');
    const pin = state.pinBuffer.join('');

    if (pin.length < 4) {
      if (errorBox) {
        errorBox.textContent = 'Silakan masukkan minimal 4 digit PIN.';
        errorBox.classList.remove('hidden');
      }
      return;
    }

    try {
      const profile = await invoke('auth_login', {
        username: state.selectedLoginUser,
        pin: pin,
      });

      state.currentUser = profile;
      applyRolePermissions(profile.role);
      document.getElementById('login-screen').classList.add('hidden');
      document.getElementById('app-layout').classList.remove('hidden');

      // Update Header user badge
      const badgeName = document.getElementById('badge-name');
      const badgeRole = document.getElementById('badge-role');
      if (badgeName) badgeName.textContent = profile.full_name;
      if (badgeRole) badgeRole.textContent = profile.role;

      // Initialize Main ERP views
      await initializeERP();
    } catch (err) {
      if (errorBox) {
        errorBox.textContent = typeof err === 'string' ? err : (err.message || 'PIN salah.');
        errorBox.classList.remove('hidden');
      }
      state.pinBuffer = [];
      updatePinDots();
    }
  }

  async function logout() {
    if (isCertFormDirty()) {
      saveCertDraftToStorage();
      const proceed = await appConfirm(
        'Ada draf sertifikat yang belum disimpan ke database. Tetap keluar? (Draf tetap disimpan sementara di perangkat ini untuk akun Anda.)',
        'Draf Sertifikat Belum Disimpan',
        'warning',
        'Tetap Keluar',
        'Kembali'
      );
      if (!proceed) return;
    }
    resetCertificatesSessionState();
    document.title = APP_DEFAULT_TITLE;
    if (isTauri) {
      invoke('set_window_title', { title: APP_DEFAULT_TITLE }).catch(() => {});
      invoke('auth_logout').catch(() => {});
    }
    state.currentUser = null;
    state.pinBuffer = [];
    updatePinDots();
    document.getElementById('app-layout').classList.add('hidden');
    document.getElementById('login-screen').classList.remove('hidden');
    const errorBox = document.getElementById('login-error');
    if (errorBox) errorBox.classList.add('hidden');
    refreshLoginUsers();
  }

  // =========================================================================
  // 2. Global Initialization & Unit / Tab Switcher
  // =========================================================================
  function isAdminUser() {
    return state.currentUser?.role === 'admin';
  }

  async function loadOriginDatalist() {
    const datalist = document.getElementById('nationality-options');
    if (!datalist) return;
    try {
      const ref = await invoke('get_origin_reference_list');
      const items = [
        ...(ref?.domestic_regions || []),
        ...(ref?.world_countries || []),
      ];
      datalist.innerHTML = '';
      items.forEach((val) => {
        const opt = document.createElement('option');
        opt.value = val;
        datalist.appendChild(opt);
      });
    } catch (err) {
      console.warn('Gagal memuat daftar referensi asal wisatawan:', err);
    }
  }

  async function initializeERP() {
    await loadOriginDatalist();

    // 1. Fetch available months for active unit
    await refreshAvailableMonths();

    // 2. Load active view
    await loadActiveView();
  }

  async function refreshAvailableMonths() {
    try {
      const months = await invoke('get_available_months', { unit: state.activeUnit });
      state.availableMonths = months || [];

      // Populate Recap month dropdown
      const recapSelect = document.getElementById('recap-month-select');
      const ledgerSelect = document.getElementById('ledger-month-select');

      if (recapSelect) {
        recapSelect.innerHTML = '';
        if (state.availableMonths.length === 0) {
          const nowMonth = getSystemLocalMonthStr();
          recapSelect.innerHTML = `<option value="${nowMonth}">${nowMonth} (Bulan Berjalan)</option>`;
          state.recapMonth = nowMonth;
        } else {
          const keepRecap = state.availableMonths.includes(state.recapMonth);
          state.availableMonths.forEach((m, idx) => {
            const opt = document.createElement('option');
            opt.value = m;
            opt.textContent = m;
            if ((keepRecap && m === state.recapMonth) || (!keepRecap && idx === 0)) {
              opt.selected = true;
              state.recapMonth = m;
            }
            recapSelect.appendChild(opt);
          });
        }
      }

      if (ledgerSelect) {
        ledgerSelect.innerHTML = '<option value="all">Semua Periode</option>';
        if (state.ledgerMonth !== 'all' && !state.availableMonths.includes(state.ledgerMonth)) {
          state.ledgerMonth = 'all';
        }
        state.availableMonths.forEach(m => {
          const opt = document.createElement('option');
          opt.value = m;
          opt.textContent = m;
          ledgerSelect.appendChild(opt);
        });
        ledgerSelect.value = state.ledgerMonth;
      }

      const tourismSelect = document.getElementById('tourism-month-select');
      if (tourismSelect) {
        tourismSelect.innerHTML = '';
        if (state.availableMonths.length === 0) {
          const nowMonth = getSystemLocalMonthStr();
          tourismSelect.innerHTML = `<option value="${nowMonth}">${nowMonth} (Bulan Berjalan)</option>`;
          state.tourismMonth = nowMonth;
        } else {
          const keepCurrent = state.availableMonths.includes(state.tourismMonth);
          state.availableMonths.forEach((m, idx) => {
            const opt = document.createElement('option');
            opt.value = m;
            opt.textContent = m;
            if ((keepCurrent && m === state.tourismMonth) || (!keepCurrent && idx === 0)) {
              opt.selected = true;
              state.tourismMonth = m;
            }
            tourismSelect.appendChild(opt);
          });
        }
      }
    } catch (err) {
      console.error('Gagal mengambil daftar periode:', err);
    }
  }

  async function loadActiveView() {
    if (state.activeTab === 'recap') {
      await loadRecapData();
    } else if (state.activeTab === 'ledger') {
      await loadLedgerData();
    } else if (state.activeTab === 'imports') {
      await loadImportLogs();
    } else if (state.activeTab === 'tourism') {
      await loadTourismData();
    } else if (state.activeTab === 'certificates') {
      await loadCertificatesModuleView();
    }
  }

  function switchTab(tabName) {
    state.activeTab = tabName;

    document.querySelectorAll('.module-view').forEach(view => {
      view.classList.remove('active');
    });

    const activeView = document.getElementById(`view-${tabName}`);
    if (activeView) activeView.classList.add('active');

    // Update active dropdown items
    document.querySelectorAll('.dropdown-item').forEach(item => {
      if (item.dataset.action === 'view-tab' && item.dataset.tab === tabName) {
        item.classList.add('active');
      } else {
        item.classList.remove('active');
      }
    });

    const navCertBtn = document.getElementById('nav-tab-certificates');
    if (navCertBtn) {
      navCertBtn.classList.toggle('active', tabName === 'certificates');
    }

    // Highlight parent dropdown toggle
    const tabToDropdown = {
      recap: 'pajak',
      tourism: 'pajak',
      ledger: 'transaksi',
      imports: 'transaksi',
      optimizer: 'audit',
    };

    const targetGroup = tabToDropdown[tabName] || null;
    document.querySelectorAll('.nav-dropdown').forEach(dropdown => {
      const toggle = dropdown.querySelector('.nav-dropdown-toggle');
      if (targetGroup && dropdown.dataset.dropdown === targetGroup) {
        toggle?.classList.add('active');
      } else {
        toggle?.classList.remove('active');
      }
    });
  }

  function closeAllDropdowns() {
    document.querySelectorAll('.nav-dropdown').forEach(dd => {
      dd.classList.remove('open');
      const toggle = dd.querySelector('.nav-dropdown-toggle');
      if (toggle) toggle.setAttribute('aria-expanded', 'false');
    });
  }

  function initNav() {
    // Unit Switcher
    const unitTabs = document.querySelectorAll('.unit-tab');
    unitTabs.forEach(tab => {
      tab.addEventListener('click', async () => {
        unitTabs.forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        state.activeUnit = tab.dataset.unit;
        state.ledgerPage = 1;
        state.ledgerMonth = 'all';
        await refreshAvailableMonths();
        await loadActiveView();
      });
    });

    // Categorized Dropdowns
    const dropdowns = document.querySelectorAll('.nav-dropdown');
    dropdowns.forEach(dd => {
      const toggle = dd.querySelector('.nav-dropdown-toggle');
      if (toggle) {
        toggle.addEventListener('click', (e) => {
          e.stopPropagation();
          const isOpen = dd.classList.contains('open');
          closeAllDropdowns();
          if (!isOpen) {
            dd.classList.add('open');
            toggle.setAttribute('aria-expanded', 'true');
          }
        });
      }
    });

    // Dropdown Action Items
    document.querySelectorAll('.dropdown-item').forEach(item => {
      item.addEventListener('click', async (e) => {
        e.stopPropagation();
        closeAllDropdowns();

        const action = item.dataset.action;
        if (action === 'view-tab') {
          const tab = item.dataset.tab;
          switchTab(tab);
          await loadActiveView();

          if (item.dataset.focus === 'logs') {
            const logsCard = document.querySelector('.table-card');
            if (logsCard) logsCard.scrollIntoView({ behavior: 'smooth' });
          }
        } else if (action === 'open-print') {
          openPrintReportModal();
        } else if (action === 'open-add-transaction') {
          switchTab('ledger');
          await loadActiveView();
          openTransactionModal();
        } else if (action === 'download-template') {
          downloadTemplate();
        }
      });
    });

    // Direct Top-Level Nav Buttons (Sertifikat Training)
    const navCertBtn = document.getElementById('nav-tab-certificates');
    if (navCertBtn) {
      navCertBtn.addEventListener('click', async () => {
        closeAllDropdowns();
        switchTab('certificates');
        await loadActiveView();
      });
    }

    // Close dropdowns on outside click or Escape key
    document.addEventListener('click', () => {
      closeAllDropdowns();
    });

    document.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        closeAllDropdowns();
      }
    });

    // Logout Button
    const btnLogout = document.getElementById('btn-logout');
    if (btnLogout) btnLogout.addEventListener('click', logout);

    // Theme Toggle
    const themeToggle = document.getElementById('theme-toggle');
    if (themeToggle) {
      themeToggle.addEventListener('click', () => {
        state.theme = state.theme === 'light' ? 'dark' : 'light';
        applyTheme(state.theme);
      });
    }
    applyTheme(state.theme);
  }

  function applyTheme(theme) {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('anda_erp_theme', theme);
    const sunIcon = document.getElementById('theme-icon-sun');
    const moonIcon = document.getElementById('theme-icon-moon');
    if (sunIcon && moonIcon) {
      if (theme === 'dark') {
        sunIcon.classList.add('hidden');
        moonIcon.classList.remove('hidden');
      } else {
        sunIcon.classList.remove('hidden');
        moonIcon.classList.add('hidden');
      }
    }
  }

  // =========================================================================
  // 3. Module 1: Rekapitulasi Pajak (Bapenda)
  // =========================================================================
  function initRecapModule() {
    const recapSelect = document.getElementById('recap-month-select');
    if (recapSelect) {
      recapSelect.addEventListener('change', async () => {
        state.recapMonth = recapSelect.value;
        await loadRecapData();
      });
    }

    const btnRefresh = document.getElementById('btn-refresh-recap');
    if (btnRefresh) {
      btnRefresh.addEventListener('click', async () => {
        await refreshAvailableMonths();
        await loadRecapData();
      });
    }

    const btnPrint = document.getElementById('btn-print-recap');
    if (btnPrint) {
      btnPrint.addEventListener('click', openPrintReportModal);
    }

    const btnExportCsv = document.getElementById('btn-export-recap-csv');
    if (btnExportCsv) {
      btnExportCsv.addEventListener('click', exportRecapToCsv);
    }
  }

  async function loadRecapData() {
    const month = state.recapMonth || document.getElementById('recap-month-select')?.value;
    if (!month) return;

    try {
      const data = await invoke('get_tax_recap', {
        unit: state.activeUnit,
        month: month,
      });

      state.currentRecap = data;

      // Update KPI Cards
      const taxPct = `${(data.tax_rate * 100).toFixed(0)}%`;
      document.getElementById('kpi-total-revenue').textContent = formatRupiah(data.total_revenue);
      document.getElementById('kpi-doc-no').textContent = data.document_number;
      document.getElementById('kpi-tax-rate').textContent = taxPct;
      const kpiTaxDueLabel = document.getElementById('kpi-tax-due-label');
      if (kpiTaxDueLabel) kpiTaxDueLabel.textContent = `Pajak Terutang (${taxPct})`;
      document.getElementById('kpi-tax-due').textContent = formatRupiah(data.tax_due);
      document.getElementById('kpi-net-revenue').textContent = formatRupiah(data.net_revenue);
      document.getElementById('kpi-active-days').textContent = `${data.active_days} / ${data.days_in_month} Hari`;
      document.getElementById('kpi-zero-days').textContent = `${data.zero_days} Hari Nihil`;
      document.getElementById('kpi-avg-day').textContent = formatRupiah(data.avg_per_day);
      document.getElementById('kpi-max-day').textContent = `Tertinggi: ${formatRupiah(data.max_daily_revenue)}`;

      // Update Chart Label
      document.getElementById('chart-month-label').textContent = `${data.unit_name} — ${data.month_name}`;
      document.getElementById('table-record-count').textContent = `${data.days_in_month} Hari Kalender`;

      // Render SVG Bar Chart
      renderRecapChart(data.daily_items, data.max_daily_revenue);

      // Render Table
      renderRecapTable(data);
    } catch (err) {
      console.error('Gagal memuat rekapitulasi pajak:', err);
      const tbody = document.getElementById('recap-table-body');
      if (tbody) {
        tbody.innerHTML = `<tr><td colspan="6" class="text-center text-warning">Gagal memuat data rekapitulasi: ${escapeHtml(err)}</td></tr>`;
      }
    }
  }

  function renderRecapChart(items, maxRev) {
    const svg = document.getElementById('recap-bar-chart');
    if (!svg) return;

    svg.innerHTML = '';
    const width = 1000;
    const height = 160;
    const paddingBottom = 24;
    const chartHeight = height - paddingBottom;
    const n = items.length;
    if (n === 0) return;

    const barWidth = Math.max(12, Math.floor((width - 40) / n) - 6);
    const gap = (width - 40 - (n * barWidth)) / (n + 1);
    const ceiling = maxRev > 0 ? maxRev * 1.1 : 1000000;

    // Baseline
    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    line.setAttribute('x1', '10');
    line.setAttribute('y1', String(chartHeight));
    line.setAttribute('x2', String(width - 10));
    line.setAttribute('y2', String(chartHeight));
    line.setAttribute('class', 'chart-axis-line');
    svg.appendChild(line);

    const tooltip = document.getElementById('chart-tooltip');

    items.forEach((item, i) => {
      const x = 20 + i * (barWidth + gap);
      const barH = item.revenue > 0 ? (item.revenue / ceiling) * (chartHeight - 15) : 3;
      const y = chartHeight - barH;

      const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      rect.setAttribute('x', String(x));
      rect.setAttribute('y', String(y));
      rect.setAttribute('width', String(barWidth));
      rect.setAttribute('height', String(barH));
      rect.setAttribute('rx', '2');
      rect.setAttribute('class', item.is_active ? 'chart-bar' : 'chart-bar zero-bar');

      rect.addEventListener('mouseenter', (e) => {
        if (!tooltip) return;
        tooltip.textContent = `Tgl ${item.day} (${item.day_name}): ${formatRupiah(item.revenue)}`;
        tooltip.classList.remove('hidden');
        const bbox = rect.getBoundingClientRect();
        const parentBbox = svg.getBoundingClientRect();
        tooltip.style.left = `${bbox.left - parentBbox.left + (barWidth / 2)}px`;
        tooltip.style.top = `${bbox.top - parentBbox.top - 28}px`;
      });

      rect.addEventListener('mouseleave', () => {
        if (tooltip) tooltip.classList.add('hidden');
      });

      svg.appendChild(rect);

      // Label day number below bar
      if (item.day % 2 !== 0 || n <= 15) {
        const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        text.setAttribute('x', String(x + barWidth / 2));
        text.setAttribute('y', String(chartHeight + 16));
        text.setAttribute('class', 'chart-label');
        text.textContent = String(item.day);
        svg.appendChild(text);
      }
    });
  }

  function renderRecapTable(data) {
    const tbody = document.getElementById('recap-table-body');
    const tfoot = document.getElementById('recap-table-foot');
    if (!tbody) return;

    tbody.innerHTML = '';
    data.daily_items.forEach(item => {
      const tr = document.createElement('tr');
      const parts = item.date.split('-');
      const formattedDate = `${parts[2]}/${parts[1]}/${parts[0]}`;

      tr.innerHTML = `
        <td class="font-mono text-muted">Hari ${escapeHtml(item.day)}</td>
        <td class="font-mono">${escapeHtml(formattedDate)}</td>
        <td>${escapeHtml(item.day_name)}</td>
        <td class="font-mono text-right ${item.revenue > 0 ? 'text-accent' : 'text-muted'}">
          ${escapeHtml(formatRupiah(item.revenue))}
        </td>
        <td class="text-center">
          <span class="status-badge ${item.is_active ? 'active' : 'zero'}">
            ${item.is_active ? 'Aktif' : 'Nihil'}
          </span>
        </td>
        <td class="font-mono text-right">${escapeHtml(formatRupiah(item.cumulative_revenue))}</td>
      `;
      tbody.appendChild(tr);
    });

    if (tfoot) {
      const taxPct = `${(data.tax_rate * 100).toFixed(0)}%`;
      tfoot.innerHTML = `
        <tr>
          <th colspan="3">TOTAL OMSET KOTOR</th>
          <th class="font-mono text-accent text-right">${escapeHtml(formatRupiah(data.total_revenue))}</th>
          <th colspan="2"></th>
        </tr>
        <tr>
          <th colspan="3">PAJAK TERUTANG (${escapeHtml(taxPct)})</th>
          <th class="font-mono text-warning text-right">${escapeHtml(formatRupiah(data.tax_due))}</th>
          <th colspan="2"></th>
        </tr>
        <tr>
          <th colspan="3">PENDAPATAN BERSIH</th>
          <th class="font-mono text-success text-right">${escapeHtml(formatRupiah(data.net_revenue))}</th>
          <th colspan="2"></th>
        </tr>
      `;
    }
  }

  function renderPrintChart(items, maxRev, daysInMonth) {
    const svg = document.getElementById('p-svg-chart');
    if (!svg) return;
    svg.innerHTML = '';
    const width = 460;
    const height = 110;
    const paddingBottom = 18;
    const chartHeight = height - paddingBottom;
    const n = items.length;
    if (n === 0) return;

    const ceiling = maxRev > 0 ? maxRev * 1.1 : 1000000;
    const barWidth = Math.max(5, Math.floor((width - 45) / n) - 3);
    const gap = (width - 45 - (n * barWidth)) / (n + 1);

    // Baseline
    const line = document.createElementNS('http://www.w3.org/2000/svg', 'line');
    line.setAttribute('x1', '35');
    line.setAttribute('y1', String(chartHeight));
    line.setAttribute('x2', String(width - 5));
    line.setAttribute('y2', String(chartHeight));
    line.setAttribute('stroke', '#d1d5db');
    line.setAttribute('stroke-width', '1');
    svg.appendChild(line);

    // Y Axis labels
    const formatY = (v) => (v >= 1e6 ? `${(v / 1e6).toFixed(1)}M` : (v >= 1e3 ? `${(v / 1e3).toFixed(0)}k` : `${v}`));
    const yTop = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    yTop.setAttribute('x', '30');
    yTop.setAttribute('y', '12');
    yTop.setAttribute('text-anchor', 'end');
    yTop.setAttribute('font-size', '8');
    yTop.setAttribute('fill', '#6b7280');
    yTop.textContent = formatY(maxRev);
    svg.appendChild(yTop);

    const yMid = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    yMid.setAttribute('x', '30');
    yMid.setAttribute('y', String(chartHeight / 2 + 3));
    yMid.setAttribute('text-anchor', 'end');
    yMid.setAttribute('font-size', '8');
    yMid.setAttribute('fill', '#6b7280');
    yMid.textContent = formatY(maxRev / 2);
    svg.appendChild(yMid);

    const yZero = document.createElementNS('http://www.w3.org/2000/svg', 'text');
    yZero.setAttribute('x', '30');
    yZero.setAttribute('y', String(chartHeight));
    yZero.setAttribute('text-anchor', 'end');
    yZero.setAttribute('font-size', '8');
    yZero.setAttribute('fill', '#6b7280');
    yZero.textContent = '0';
    svg.appendChild(yZero);

    // Bars
    items.forEach((item, i) => {
      const x = 38 + i * (barWidth + gap);
      const barH = item.revenue > 0 ? (item.revenue / ceiling) * (chartHeight - 12) : 2;
      const y = chartHeight - barH;

      const rect = document.createElementNS('http://www.w3.org/2000/svg', 'rect');
      rect.setAttribute('x', String(x));
      rect.setAttribute('y', String(y));
      rect.setAttribute('width', String(barWidth));
      rect.setAttribute('height', String(barH));
      rect.setAttribute('rx', '1');
      rect.setAttribute('fill', item.is_active ? '#1e293b' : '#d1d5db');
      svg.appendChild(rect);

      // X labels: 1, 4, 7, 10, 13, 16, 19, 22, 25, 28, last
      const day = item.day;
      if (day === 1 || day === 4 || day === 7 || day === 10 || day === 13 || day === 16 || day === 19 || day === 22 || day === 25 || day === 28 || day === daysInMonth) {
        const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
        text.setAttribute('x', String(x + barWidth / 2));
        text.setAttribute('y', String(chartHeight + 12));
        text.setAttribute('text-anchor', 'middle');
        text.setAttribute('font-size', '7.5');
        text.setAttribute('fill', '#4b5563');
        text.textContent = String(day);
        svg.appendChild(text);
      }
    });
  }

  function openPrintReportModal() {
    if (!state.currentRecap) return;
    const r = state.currentRecap;
    const taxPct = `${(r.tax_rate * 100).toFixed(0)}%`;

    // Header Top
    document.getElementById('p-doc-number').textContent = r.document_number;
    document.getElementById('p-title-unit').textContent = `LAPORAN PENDAPATAN ${r.unit.toUpperCase()}`;
    const pCompanyName = document.getElementById('p-company-name');
    if (pCompanyName) pCompanyName.textContent = r.business_name || 'ANDA BUNGALOWS & RESTAURANT';
    document.getElementById('p-npwpd-head').textContent = r.npwpd;
    document.getElementById('p-company-address').textContent = r.business_address;
    document.getElementById('p-period-badge').textContent = `Periode: ${r.month_name}`;

    // Subline
    document.getElementById('p-npwpd-sub').textContent = r.npwpd;
    const monthOnly = r.month_name.split(' ')[0] || r.month;
    document.getElementById('p-meta-month').textContent = monthOnly;
    document.getElementById('p-meta-year').textContent = String(r.year);
    const pTaxRateInline = document.getElementById('p-tax-rate-inline');
    if (pTaxRateInline) pTaxRateInline.textContent = taxPct;
    const pKpiTaxTitle = document.getElementById('p-kpi-tax-title');
    if (pKpiTaxTitle) pKpiTaxTitle.textContent = `PAJAK TERUTANG (${taxPct})`;

    // Top 3 KPI Cards
    document.getElementById('p-kpi-gross').textContent = formatRupiah(r.total_revenue);
    document.getElementById('p-kpi-tax').textContent = formatRupiah(r.tax_due);
    document.getElementById('p-kpi-net').textContent = formatRupiah(r.net_revenue);

    // Stats Summary Line
    document.getElementById('p-stat-days').textContent = `${r.days_in_month} Hari`;
    document.getElementById('p-stat-active').textContent = `${r.active_days} Hari`;
    document.getElementById('p-stat-zero').textContent = `${r.zero_days} Hari`;
    document.getElementById('p-stat-avg').textContent = formatRupiah(r.avg_per_day);
    document.getElementById('p-stat-max').textContent = formatRupiah(r.max_daily_revenue);
    document.getElementById('p-stat-min').textContent = formatRupiah(r.min_daily_revenue_non_zero);

    // Left Column: Daily 1..31 Table
    const pBody = document.getElementById('p-table-body');
    const pFoot = document.getElementById('p-table-foot');
    pBody.innerHTML = '';

    r.daily_items.forEach(it => {
      const tr = document.createElement('tr');
      const statusHtml = it.is_active
        ? '<span class="p-status-active">Ada Pendapatan</span>'
        : '<span class="p-status-zero">Tidak Ada Pendapatan</span>';
      const revHtml = it.revenue > 0 ? escapeHtml(formatRupiah(it.revenue)) : '<span class="text-muted">Rp 0</span>';

      tr.innerHTML = `
        <td class="text-center">${escapeHtml(it.day)}</td>
        <td class="font-mono">${escapeHtml(it.date)}</td>
        <td>${escapeHtml(it.day_name)}</td>
        <td class="font-mono text-right">${revHtml}</td>
        <td class="text-center">${statusHtml}</td>
        <td class="font-mono text-right">${escapeHtml(formatRupiah(it.cumulative_revenue))}</td>
      `;
      pBody.appendChild(tr);
    });

    pFoot.innerHTML = `
      <tr>
        <th colspan="3" class="text-center">TOTAL PERIODE</th>
        <th class="font-mono text-right">${escapeHtml(formatRupiah(r.total_revenue))}</th>
        <th class="text-center text-danger">PAJAK ${escapeHtml(taxPct)}</th>
        <th class="font-mono text-right text-danger">${escapeHtml(formatRupiah(r.tax_due))}</th>
      </tr>
    `;

    // Right Column: Chart + Summary
    document.getElementById('p-chart-range-label').textContent = `1 - ${r.days_in_month} ${monthOnly}`;
    renderPrintChart(r.daily_items, r.max_daily_revenue, r.days_in_month);

    document.getElementById('p-sum-gross').textContent = formatRupiah(r.total_revenue);
    const pSumTaxRate = document.getElementById('p-sum-tax-rate');
    if (pSumTaxRate) pSumTaxRate.textContent = taxPct;
    document.getElementById('p-sum-tax').textContent = formatRupiah(r.tax_due);
    document.getElementById('p-sum-net').textContent = formatRupiah(r.net_revenue);

    // Signatures
    document.getElementById('p-note-unit').textContent = r.unit;
    const pNoteTaxRate = document.getElementById('p-note-tax-rate');
    if (pNoteTaxRate) pNoteTaxRate.textContent = taxPct;
    document.getElementById('p-officer-name').textContent = r.officer_name || '............................................';
    const rawNip = (r.officer_nip || '').trim();
    document.getElementById('p-officer-nip').textContent = rawNip
      ? (/^NIP\.?\s*/i.test(rawNip) ? rawNip : `NIP. ${rawNip}`)
      : 'NIP. ........................................';

    const lastDate = `${r.days_in_month} ${monthOnly} ${r.year}`;
    document.getElementById('p-sig-date').textContent = `Lombok Tengah, ${lastDate}`;
    document.getElementById('p-sign-name').textContent = r.sign_name || 'Pimpinan / Pengelola';

    // Signature Toggle state check
    const chkSig = document.getElementById('chk-toggle-signatures');
    const sigWrap = document.getElementById('p-signatures-wrapper');
    if (chkSig && sigWrap) {
      if (chkSig.checked) {
        sigWrap.classList.remove('hidden');
      } else {
        sigWrap.classList.add('hidden');
      }
    }

    document.getElementById('modal-print').classList.remove('hidden');
    if (r.document_number) {
      const cleanTitle = getCleanReportFileName(r.document_number);
      document.title = cleanTitle;
      if (isTauri) {
        invoke('set_window_title', { title: cleanTitle }).catch(() => {});
      }
    }
  }
  function exportRecapToCsv() {
    if (!state.currentRecap) return;
    const r = state.currentRecap;
    const taxPct = `${(r.tax_rate * 100).toFixed(0)}%`;
    let csv = `Laporan Rekapitulasi Pendapatan & Pajak Daerah\n`;
    csv += `Unit Usaha,"${r.unit_name}"\nMasa Pajak,"${r.month_name}"\nNomor Dokumen,"${r.document_number}"\n\n`;
    csv += `Hari Ke,Tanggal,Hari,Omset Harian,Status,Akumulasi Saldo\n`;

    r.daily_items.forEach(it => {
      csv += `${it.day},${it.date},${it.day_name},${it.revenue},${it.is_active ? 'Aktif' : 'Nihil'},${it.cumulative_revenue}\n`;
    });

    csv += `\nTotal Omset Kotor,,,${r.total_revenue}\n`;
    csv += `Pajak Terutang (${taxPct}),,,${r.tax_due}\n`;
    csv += `Pendapatan Bersih,,,${r.net_revenue}\n`;

    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.setAttribute('href', url);
    const downloadName = (r.document_number ? getCleanReportFileName(r.document_number) : `Rekap_Pajak_${r.unit}_${r.month}`) + '.csv';
    link.setAttribute('download', downloadName);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  }

  // =========================================================================
  // 4. Module 2: Data Mentah (Ledger)
  // =========================================================================
  function updateLedgerModeUI() {
    const isRes = state.ledgerMode === 'reservations';
    const kpiGrid = document.getElementById('ledger-res-kpi-grid');
    const searchInput = document.getElementById('ledger-search-input');
    const btnExportRes = document.getElementById('btn-export-reservations');
    const btnClean = document.getElementById('btn-clean-duplicates');
    const btnAdd = document.getElementById('btn-add-transaction');
    const cardTitle = document.getElementById('ledger-card-title');

    if (kpiGrid) kpiGrid.classList.toggle('hidden', !isRes);
    if (btnExportRes) btnExportRes.classList.toggle('hidden', !isRes);
    if (btnClean) btnClean.classList.toggle('hidden', isRes || !isAdminUser());
    if (btnAdd) btnAdd.classList.toggle('hidden', isRes);
    if (searchInput) {
      searchInput.placeholder = isRes
        ? 'Cari tamu, kamar, booking, negara, OTA...'
        : 'Cari keterangan, nomor, tanggal...';
    }
    if (cardTitle) {
      cardTitle.textContent = isRes
        ? 'Daftar Rincian Reservasi Tamu (13 Kolom)'
        : 'Daftar Transaksi Pendapatan';
    }
  }

  function initLedgerModule() {
    const modeTabs = document.querySelectorAll('.ledger-mode-tab');
    modeTabs.forEach(tab => {
      tab.addEventListener('click', () => {
        modeTabs.forEach(t => t.classList.remove('active'));
        tab.classList.add('active');
        state.ledgerMode = tab.dataset.ledgerMode || 'summary';
        state.ledgerPage = 1;
        updateLedgerModeUI();
        loadLedgerData();
      });
    });

    const monthSelect = document.getElementById('ledger-month-select');
    if (monthSelect) {
      monthSelect.addEventListener('change', () => {
        state.ledgerMonth = monthSelect.value;
        state.ledgerPage = 1;
        loadLedgerData();
      });
    }

    const searchInput = document.getElementById('ledger-search-input');
    const filterBtn = document.getElementById('btn-ledger-filter');
    if (filterBtn && searchInput) {
      filterBtn.addEventListener('click', () => {
        state.ledgerSearch = searchInput.value.trim();
        state.ledgerPage = 1;
        loadLedgerData();
      });
      searchInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
          state.ledgerSearch = searchInput.value.trim();
          state.ledgerPage = 1;
          loadLedgerData();
        }
      });
    }

    const pageSizeSelect = document.getElementById('ledger-page-size');
    if (pageSizeSelect) {
      pageSizeSelect.addEventListener('change', () => {
        state.ledgerPageSize = parseInt(pageSizeSelect.value, 10);
        state.ledgerPage = 1;
        loadLedgerData();
      });
    }

    const btnPrev = document.getElementById('btn-ledger-prev');
    const btnNext = document.getElementById('btn-ledger-next');
    if (btnPrev) {
      btnPrev.addEventListener('click', () => {
        if (state.ledgerPage > 1) {
          state.ledgerPage--;
          loadLedgerData();
        }
      });
    }
    if (btnNext) {
      btnNext.addEventListener('click', () => {
        state.ledgerPage++;
        loadLedgerData();
      });
    }

    // Add transaction button
    const btnAdd = document.getElementById('btn-add-transaction');
    if (btnAdd) {
      btnAdd.addEventListener('click', () => {
        openTransactionModal();
      });
    }

    // Clean duplicates button
    const btnClean = document.getElementById('btn-clean-duplicates');
    if (btnClean) {
      btnClean.addEventListener('click', async () => {
        const confirmed = await appConfirm(`Bersihkan transaksi duplikat untuk unit ${state.activeUnit}?`);
        if (!confirmed) return;
        try {
          const removed = await invoke('clean_duplicate_transactions', { unit: state.activeUnit });
          alert(`Berhasil menghapus ${removed} transaksi duplikat.`);
          await loadLedgerData();
          await loadRecapData();
        } catch (err) {
          alert(`Gagal: ${err}`);
        }
      });
    }

    // Modal save transaction
    const btnSaveTrans = document.getElementById('btn-save-transaction');
    if (btnSaveTrans) {
      btnSaveTrans.addEventListener('click', saveTransaction);
    }

    // Export 13-column Excel button
    const btnExportRes = document.getElementById('btn-export-reservations');
    if (btnExportRes) {
      btnExportRes.addEventListener('click', async () => {
        const monthArg = state.ledgerMonth === 'all' ? null : state.ledgerMonth;
        try {
          const msg = await invoke('export_room_reservations_excel', {
            unit: state.activeUnit,
            month: monthArg,
          });
          alert(msg);
        } catch (err) {
          alert(`Gagal mengekspor Excel 13 kolom: ${err}`);
        }
      });
    }

    // Modal save reservation & live derived fields (nights, rate)
    const btnSaveRes = document.getElementById('btn-save-reservation');
    if (btnSaveRes) {
      btnSaveRes.addEventListener('click', saveReservation);
    }
    ['res-checkin', 'res-checkout', 'res-rooms', 'res-payment'].forEach((elId) => {
      const el = document.getElementById(elId);
      if (el) {
        el.addEventListener('input', recalcReservationModalDerived);
        el.addEventListener('change', recalcReservationModalDerived);
      }
    });
  }

  async function loadLedgerData() {
    try {
      const thead = document.getElementById('ledger-table-head');
      const tbody = document.getElementById('ledger-table-body');
      const monthArg = state.ledgerMonth === 'all' ? null : state.ledgerMonth;
      const searchArg = state.ledgerSearch || null;
      const canAdmin = isAdminUser();

      if (state.ledgerMode === 'reservations') {
        if (thead) {
          thead.innerHTML = `
            <tr>
              <th style="width: 96px;">DATE</th>
              <th>NAME</th>
              <th style="width: 72px; text-align: center;">Total Person</th>
              <th style="width: 120px;">Booking Number</th>
              <th style="width: 105px;">Nationality</th>
              <th style="width: 85px; text-align: center;">Room Number</th>
              <th style="width: 75px; text-align: center;">Total Kamar</th>
              <th style="width: 96px;">Date Check In</th>
              <th style="width: 96px;">Date Check Out</th>
              <th style="width: 72px; text-align: center;">Total Night</th>
              <th style="text-align: right; width: 120px;">Rate</th>
              <th style="text-align: right; width: 135px;">Total Payment</th>
              <th style="width: 110px;">Remarks</th>
              ${canAdmin ? '<th style="width: 110px; text-align: center;">Aksi</th>' : ''}
            </tr>
          `;
        }

        const data = await invoke('get_room_reservations', {
          unit: state.activeUnit,
          month: monthArg,
          search: searchArg,
          page: state.ledgerPage,
          pageSize: state.ledgerPageSize,
        });
        state.ledgerReservations = data.items || [];

        const kpiCount = document.getElementById('res-kpi-count');
        const kpiRooms = document.getElementById('res-kpi-rooms');
        const kpiPax = document.getElementById('res-kpi-pax');
        const kpiNights = document.getElementById('res-kpi-nights');
        const kpiPayment = document.getElementById('res-kpi-payment');
        if (kpiCount) kpiCount.textContent = `${data.total_count} Reservasi`;
        if (kpiRooms) kpiRooms.textContent = `${data.total_rooms} Kamar Terjual`;
        if (kpiPax) kpiPax.textContent = `${data.total_person} Orang`;
        if (kpiNights) kpiNights.textContent = `${data.total_nights} Malam`;
        if (kpiPayment) kpiPayment.textContent = formatRupiah(data.total_payment);

        tbody.innerHTML = '';
        if (state.ledgerReservations.length === 0) {
          tbody.innerHTML = `<tr><td colspan="${canAdmin ? 14 : 13}" class="text-center text-muted">Belum ada data rincian reservasi 13 kolom yang cocok.</td></tr>`;
        } else {
          state.ledgerReservations.forEach(r => {
            const tr = document.createElement('tr');
            const remarksHtml = r.remarks
              ? `<span class="badge badge-period">${escapeHtml(r.remarks)}</span>`
              : '-';
            const rateHtml = (r.rate !== null && r.rate !== undefined) ? escapeHtml(formatRupiah(r.rate)) : '-';
            const actionTd = canAdmin
              ? `<td style="text-align: center;">
                   <button class="btn-link btn-edit-res" data-id="${escapeHtml(r.id)}">Edit</button>
                   <button class="btn-link text-warning btn-del-res" data-id="${escapeHtml(r.id)}" style="margin-left: 6px;">Hapus</button>
                 </td>`
              : '';

            tr.innerHTML = `
              <td class="font-mono">${escapeHtml(r.date)}</td>
              <td><strong>${escapeHtml(r.guest_name)}</strong></td>
              <td style="text-align: center;" class="font-mono">${escapeHtml(r.total_person ?? '-')}</td>
              <td class="font-mono text-muted">${escapeHtml(r.booking_number || '-')}</td>
              <td>${escapeHtml(r.nationality || '-')}</td>
              <td style="text-align: center;" class="font-mono"><span class="badge badge-period">${escapeHtml(r.room_number || '-')}</span></td>
              <td style="text-align: center;" class="font-mono">${escapeHtml(r.total_rooms ?? 1)}</td>
              <td class="font-mono text-muted">${escapeHtml(r.check_in_date)}</td>
              <td class="font-mono text-muted">${escapeHtml(r.check_out_date)}</td>
              <td style="text-align: center;" class="font-mono">${escapeHtml(r.total_nights ?? '-')}</td>
              <td style="text-align: right;" class="font-mono">${rateHtml}</td>
              <td style="text-align: right;" class="font-mono text-accent"><strong>${escapeHtml(formatRupiah(r.total_payment))}</strong></td>
              <td>${remarksHtml}</td>
              ${actionTd}
            `;
            tbody.appendChild(tr);
          });

          if (canAdmin) {
            tbody.querySelectorAll('.btn-edit-res').forEach(btn => {
              btn.addEventListener('click', () => {
                const id = parseInt(btn.dataset.id, 10);
                const row = state.ledgerReservations.find(item => item.id === id);
                if (row) openReservationModal(row);
              });
            });

            tbody.querySelectorAll('.btn-del-res').forEach(btn => {
              btn.addEventListener('click', async () => {
                const id = parseInt(btn.dataset.id, 10);
                const row = state.ledgerReservations.find(item => item.id === id);
                const label = row ? `${row.guest_name} (${row.date})` : `ID #${id}`;
                const confirmed = await appConfirm(`Hapus reservasi ${label}? Transaksi pendapatan terkait juga akan ikut dihapus.`);
                if (!confirmed) return;
                try {
                  await invoke('delete_room_reservation', { id });
                  await refreshAvailableMonths();
                  await loadLedgerData();
                  await loadRecapData();
                  await loadTourismData();
                } catch (err) {
                  alert(`Gagal menghapus reservasi: ${err}`);
                }
              });
            });
          }
        }

        document.getElementById('ledger-summary-info').textContent =
          `Total: ${data.total_count} Reservasi | ${formatRupiah(data.total_payment)}`;
        document.getElementById('ledger-page-info').textContent =
          `Halaman ${data.page} dari ${data.total_pages || 1} (${data.total_count} data)`;

        const btnPrev = document.getElementById('btn-ledger-prev');
        const btnNext = document.getElementById('btn-ledger-next');
        if (btnPrev) btnPrev.disabled = data.page <= 1;
        if (btnNext) btnNext.disabled = data.page >= data.total_pages;
        return;
      }

      if (thead) {
        thead.innerHTML = `
          <tr>
            <th style="width: 70px;">ID</th>
            <th style="width: 110px;">Unit</th>
            <th style="width: 120px;">Tanggal</th>
            <th style="text-align: right; width: 170px;">Nominal (Rp)</th>
            <th>Keterangan</th>
            <th style="width: 180px;">Berkas Sumber</th>
            <th style="width: 110px; text-align: center;">Aksi</th>
          </tr>
        `;
      }

      const data = await invoke('get_transactions', {
        unit: state.activeUnit,
        month: monthArg,
        search: searchArg,
        page: state.ledgerPage,
        pageSize: state.ledgerPageSize,
      });

      tbody.innerHTML = '';

      if (data.items.length === 0) {
        tbody.innerHTML = `<tr><td colspan="7" class="text-center text-muted">Tidak ada transaksi yang cocok.</td></tr>`;
      } else {
        data.items.forEach(t => {
          const tr = document.createElement('tr');
          const actionHtml = canAdmin
            ? `<button class="btn-link btn-edit-trans" data-id="${escapeHtml(t.id)}" data-date="${escapeHtml(t.date)}" data-amount="${escapeHtml(t.amount)}" data-notes="${escapeHtml(t.notes || '')}">Edit</button>
               <button class="btn-link text-warning btn-del-trans" data-id="${escapeHtml(t.id)}" style="margin-left: 6px;">Hapus</button>`
            : '<span class="text-muted">-</span>';
          tr.innerHTML = `
            <td class="font-mono text-muted">${escapeHtml(t.id)}</td>
            <td><strong>${escapeHtml(t.unit)}</strong></td>
            <td class="font-mono">${escapeHtml(t.date)}</td>
            <td style="text-align: right;" class="font-mono text-accent">${escapeHtml(formatRupiah(t.amount))}</td>
            <td>${escapeHtml(t.notes || '-')}</td>
            <td class="text-muted" style="font-size: 11px;">${escapeHtml(t.source_file || 'Input Manual')}</td>
            <td style="text-align: center;">${actionHtml}</td>
          `;
          tbody.appendChild(tr);
        });

        if (canAdmin) {
          // Wire edit & delete buttons
          tbody.querySelectorAll('.btn-edit-trans').forEach(btn => {
            btn.addEventListener('click', () => {
              openTransactionModal({
                id: btn.dataset.id,
                unit: state.activeUnit,
                date: btn.dataset.date,
                amount: btn.dataset.amount,
                notes: btn.dataset.notes,
              });
            });
          });

          tbody.querySelectorAll('.btn-del-trans').forEach(btn => {
            btn.addEventListener('click', async () => {
              const id = parseInt(btn.dataset.id, 10);
              const confirmed = await appConfirm(`Hapus transaksi ID #${id}?`);
              if (!confirmed) return;
              try {
                await invoke('delete_transaction', { id });
                await refreshAvailableMonths();
                await loadLedgerData();
                await loadRecapData();
                await loadTourismData();
              } catch (err) {
                alert(`Gagal menghapus: ${err}`);
              }
            });
          });
        }
      }

      // Summary Info
      document.getElementById('ledger-summary-info').textContent =
        `Total: ${data.total_count} Data | ${formatRupiah(data.total_amount)}`;

      // Pagination Controls
      document.getElementById('ledger-page-info').textContent =
        `Halaman ${data.page} dari ${data.total_pages || 1} (${data.total_count} data)`;

      const btnPrev = document.getElementById('btn-ledger-prev');
      const btnNext = document.getElementById('btn-ledger-next');
      if (btnPrev) btnPrev.disabled = data.page <= 1;
      if (btnNext) btnNext.disabled = data.page >= data.total_pages;

    } catch (err) {
      console.error('Gagal memuat data transaksi:', err);
      const tbody = document.getElementById('ledger-table-body');
      if (tbody) {
        tbody.innerHTML = `<tr><td colspan="7" class="text-center text-warning">Gagal memuat data transaksi: ${escapeHtml(err)}</td></tr>`;
      }
    }
  }

  function openTransactionModal(existing = null) {
    const modal = document.getElementById('modal-transaction');
    const title = document.getElementById('modal-trans-title');
    const idInput = document.getElementById('trans-id');
    const unitSelect = document.getElementById('trans-unit');
    const dateInput = document.getElementById('trans-date');
    const amountInput = document.getElementById('trans-amount');
    const notesInput = document.getElementById('trans-notes');

    if (existing) {
      title.textContent = 'Edit Transaksi Pendapatan';
      idInput.value = existing.id;
      unitSelect.value = existing.unit;
      dateInput.value = existing.date;
      amountInput.value = existing.amount;
      notesInput.value = existing.notes;
    } else {
      title.textContent = 'Tambah Transaksi Manual';
      idInput.value = '';
      unitSelect.value = state.activeUnit;
      dateInput.value = getSystemLocalDateStr();
      amountInput.value = '';
      notesInput.value = '';
    }
    modal.classList.remove('hidden');
  }

  async function saveTransaction() {
    const id = document.getElementById('trans-id').value;
    const unit = document.getElementById('trans-unit').value;
    const date = document.getElementById('trans-date').value;
    const amount = parseFloat(document.getElementById('trans-amount').value);
    const notes = document.getElementById('trans-notes').value.trim();

    if (!unit || !date || isNaN(amount) || amount < 0) {
      alert('Mohon isi tanggal dan nominal pendapatan dengan benar.');
      return;
    }

    try {
      if (id) {
        await invoke('update_transaction', {
          id: parseInt(id, 10),
          unit,
          date,
          amount,
          notes: notes || null,
        });
      } else {
        await invoke('add_transaction', {
          unit,
          date,
          amount,
          notes: notes || null,
          sourceFile: 'Input Manual',
        });
      }

      document.getElementById('modal-transaction').classList.add('hidden');
      await refreshAvailableMonths();
      await loadLedgerData();
      await loadRecapData();
      await loadTourismData();
    } catch (err) {
      alert(`Gagal menyimpan: ${err}`);
    }
  }

  function recalcReservationModalDerived() {
    const ciStr = document.getElementById('res-checkin')?.value || '';
    const coStr = document.getElementById('res-checkout')?.value || '';
    const roomsVal = Math.max(1, parseInt(document.getElementById('res-rooms')?.value, 10) || 1);
    const payVal = parseFloat(document.getElementById('res-payment')?.value);
    const nightsEl = document.getElementById('res-nights');
    const rateEl = document.getElementById('res-rate');

    let nights = 0;
    if (ciStr && coStr) {
      const ciMs = Date.parse(`${ciStr}T00:00:00Z`);
      const coMs = Date.parse(`${coStr}T00:00:00Z`);
      if (!Number.isNaN(ciMs) && !Number.isNaN(coMs)) {
        nights = Math.round((coMs - ciMs) / 86400000);
      }
    }
    if (nightsEl) {
      nightsEl.value = nights > 0 ? String(nights) : '0';
    }
    if (rateEl) {
      if (nights > 0 && roomsVal > 0 && !Number.isNaN(payVal) && payVal >= 0) {
        rateEl.value = String(Math.round(payVal / (nights * roomsVal)));
      } else {
        rateEl.value = '';
      }
    }
  }

  function openReservationModal(r) {
    if (!r) return;
    document.getElementById('res-id').value = String(r.id);
    document.getElementById('res-date').value = r.date || '';
    document.getElementById('res-guest-name').value = r.guest_name || '';
    document.getElementById('res-person').value = String(r.total_person ?? 1);
    document.getElementById('res-booking').value = r.booking_number || '';
    document.getElementById('res-nationality').value = r.nationality || '';
    document.getElementById('res-room').value = r.room_number || '';
    document.getElementById('res-rooms').value = String(r.total_rooms ?? 1);
    document.getElementById('res-checkin').value = r.check_in_date || r.date || '';
    document.getElementById('res-checkout').value = r.check_out_date || '';
    document.getElementById('res-payment').value = String(r.total_payment ?? 0);
    document.getElementById('res-remarks').value = r.remarks || '';
    recalcReservationModalDerived();
    document.getElementById('modal-reservation')?.classList.remove('hidden');
  }

  async function saveReservation() {
    const id = parseInt(document.getElementById('res-id').value, 10);
    const date = document.getElementById('res-date').value.trim();
    const guestName = document.getElementById('res-guest-name').value.trim();
    const totalPerson = parseInt(document.getElementById('res-person').value, 10);
    const bookingNumber = document.getElementById('res-booking').value.trim();
    const nationality = document.getElementById('res-nationality').value.trim();
    const roomNumber = document.getElementById('res-room').value.trim();
    const totalRooms = parseInt(document.getElementById('res-rooms').value, 10);
    const checkInDate = document.getElementById('res-checkin').value.trim();
    const checkOutDate = document.getElementById('res-checkout').value.trim();
    const totalPayment = parseFloat(document.getElementById('res-payment').value);
    const remarks = document.getElementById('res-remarks').value.trim();

    if (!id || !guestName || !date || !checkInDate || !checkOutDate) {
      alert('Tanggal pembukuan, nama tamu, Check In, dan Check Out wajib diisi.');
      return;
    }
    if (checkOutDate <= checkInDate) {
      alert('Tanggal Check Out harus setelah tanggal Check In.');
      return;
    }
    if (Number.isNaN(totalPerson) || totalPerson < 1) {
      alert('Total person minimal 1 orang.');
      return;
    }
    if (Number.isNaN(totalRooms) || totalRooms < 1) {
      alert('Total Kamar minimal 1 kamar.');
      return;
    }
    if (Number.isNaN(totalPayment) || totalPayment < 0) {
      alert('Total Payment harus bernilai 0 atau lebih.');
      return;
    }

    recalcReservationModalDerived();

    try {
      await invoke('update_room_reservation', {
        payload: {
          id,
          date,
          guest_name: guestName,
          total_person: totalPerson,
          booking_number: bookingNumber || null,
          nationality: nationality || null,
          room_number: roomNumber || null,
          total_rooms: totalRooms,
          check_in_date: checkInDate,
          check_out_date: checkOutDate,
          total_payment: totalPayment,
          remarks: remarks || null,
        },
      });
      document.getElementById('modal-reservation')?.classList.add('hidden');
      await refreshAvailableMonths();
      await loadLedgerData();
      await loadRecapData();
      await loadTourismData();
    } catch (err) {
      alert(`Gagal memperbarui reservasi: ${err}`);
    }
  }

  // =========================================================================
  // 5. In-DOM Dialog & Alert System (Bulletproof for WKWebView Desktop App)
  // =========================================================================
  let dialogQueue = Promise.resolve();

  function showAppDialog(options) {
    const run = () => displayAppDialog(options);
    const next = dialogQueue.then(run, run);
    dialogQueue = next.catch(() => {});
    return next;
  }

  function displayAppDialog({
    title = 'Pemberitahuan',
    message = '',
    type = 'info',
    confirmText = 'OK',
    cancelText = null,
  }) {
    return new Promise((resolve) => {
      const modal = document.getElementById('modal-app-dialog');
      const titleEl = document.getElementById('app-dialog-title');
      const msgEl = document.getElementById('app-dialog-message');
      const btnConfirm = document.getElementById('app-dialog-btn-confirm');
      const btnCancel = document.getElementById('app-dialog-btn-cancel');
      const closeBtn = document.getElementById('app-dialog-close-btn');
      const iconContainer = document.getElementById('app-dialog-icon-container');

      if (!modal || !titleEl || !msgEl || !btnConfirm) {
        console.warn('Dialog elements missing in DOM:', message);
        resolve(true);
        return;
      }

      titleEl.textContent = title;
      msgEl.textContent = message;
      btnConfirm.textContent = confirmText;

      btnConfirm.className = 'btn btn-primary';
      let iconHtml = '';
      if (type === 'error') {
        btnConfirm.className = 'btn btn-danger';
        iconHtml = `<div class="dialog-icon-circle error"><svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line></svg></div>`;
      } else if (type === 'warning') {
        btnConfirm.className = 'btn btn-warning';
        iconHtml = `<div class="dialog-icon-circle warning"><svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"></path><line x1="12" y1="9" x2="12" y2="13"></line><line x1="12" y1="17" x2="12.01" y2="17"></line></svg></div>`;
      } else if (type === 'success') {
        btnConfirm.className = 'btn btn-success';
        iconHtml = `<div class="dialog-icon-circle success"><svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg></div>`;
      } else {
        iconHtml = `<div class="dialog-icon-circle info"><svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg></div>`;
      }
      if (iconContainer) iconContainer.innerHTML = iconHtml;

      if (cancelText && btnCancel) {
        btnCancel.textContent = cancelText;
        btnCancel.classList.remove('hidden');
      } else if (btnCancel) {
        btnCancel.classList.add('hidden');
      }

      modal.classList.remove('hidden');

      function cleanup(res) {
        modal.classList.add('hidden');
        btnConfirm.removeEventListener('click', onConfirm);
        if (btnCancel) btnCancel.removeEventListener('click', onCancel);
        if (closeBtn) closeBtn.removeEventListener('click', onCancel);
        document.removeEventListener('keydown', onKeyDown);
        resolve(res);
      }

      function onConfirm() { cleanup(true); }
      function onCancel() { cleanup(false); }
      function onKeyDown(e) {
        if (e.key === 'Escape') onCancel();
      }

      btnConfirm.addEventListener('click', onConfirm);
      if (btnCancel) btnCancel.addEventListener('click', onCancel);
      if (closeBtn) closeBtn.addEventListener('click', onCancel);
      document.addEventListener('keydown', onKeyDown);
      btnConfirm.focus();
    });
  }

  const appAlert = (message, title = 'Pemberitahuan', type = 'info') =>
    showAppDialog({ title, message, type, confirmText: 'Mengerti' });
  const appConfirm = (message, title = 'Konfirmasi', type = 'warning', confirmText = 'Lanjutkan', cancelText = 'Batal') =>
    showAppDialog({ title, message, type, confirmText, cancelText });
  const appSuccess = (message, title = 'Berhasil') =>
    showAppDialog({ title, message, type: 'success', confirmText: 'OK' });
  const appError = (message, title = 'Format Berkas Tidak Sesuai') =>
    showAppDialog({ title, message, type: 'error', confirmText: 'Tutup' });

  // Hook global window.alert so no alert in the entire app is ever silently swallowed
  window.alert = (msg) => { appAlert(String(msg)); };

  function formatPeriodLabel(periodStr, fallbackDate = null) {
    if (!periodStr && fallbackDate) {
      const s = String(fallbackDate).trim();
      if (s.length >= 7) {
        periodStr = s.substring(0, 7);
      }
    }
    if (!periodStr || periodStr === '-') return '-';

    const monthNames = [
      '', 'Januari', 'Februari', 'Maret', 'April', 'Mei', 'Juni',
      'Juli', 'Agustus', 'September', 'Oktober', 'November', 'Desember'
    ];
    const parts = String(periodStr).split(',').map(s => s.trim()).filter(Boolean);
    if (parts.length === 0) return '-';

    const formatted = parts.map(p => {
      const segs = p.split('-');
      if (segs.length >= 2) {
        const y = segs[0];
        const mNum = parseInt(segs[1], 10);
        if (y && mNum >= 1 && mNum <= 12) {
          return `${monthNames[mNum]} ${y}`;
        }
      }
      return p;
    });

    if (formatted.length === 1) return formatted[0];
    if (formatted.length === 2) return `${formatted[0]} & ${formatted[1]}`;
    return `${formatted[0]} - ${formatted[formatted.length - 1]}`;
  }

  function initImportsModule() {
    const dropzone = document.getElementById('import-dropzone');
    const hiddenInput = document.getElementById('import-hidden-input');
    const fileSelectedBox = document.getElementById('import-file-selected');
    const fileNameSpan = document.getElementById('import-file-name');
    const btnCancelFile = document.getElementById('btn-cancel-import-file');
    const statusBadge = document.getElementById('import-status-badge');
    const warningBox = document.getElementById('import-warning-box');
    const duplicateConfirmWrap = document.getElementById('import-duplicate-confirm-wrap');
    const chkDuplicateAck = document.getElementById('chk-duplicate-ack');
    const metaUnit = document.getElementById('import-meta-unit');
    const metaPeriod = document.getElementById('import-meta-period');
    const metaCount = document.getElementById('import-meta-count');
    const metaTotal = document.getElementById('import-meta-total');
    const existingInfo = document.getElementById('import-existing-info');
    const progressBox = document.getElementById('import-progress-box');
    const btnExecute = document.getElementById('btn-execute-import');
    const targetUnitSelect = document.getElementById('import-target-unit');

    let pendingImportPath = null;
    let currentPreview = null;

    function resetImportSelection() {
      pendingImportPath = null;
      currentPreview = null;
      if (fileSelectedBox) fileSelectedBox.classList.add('hidden');
      if (fileNameSpan) fileNameSpan.textContent = '-';
      if (warningBox) {
        warningBox.classList.add('hidden');
        warningBox.textContent = '';
      }
      if (duplicateConfirmWrap) {
        duplicateConfirmWrap.classList.add('hidden');
      }
      if (chkDuplicateAck) {
        chkDuplicateAck.checked = false;
      }
      if (existingInfo) {
        existingInfo.classList.add('hidden');
        existingInfo.textContent = '';
      }
      if (progressBox) {
        progressBox.classList.add('hidden');
      }
      if (hiddenInput) hiddenInput.value = '';
    }

    async function handleFileValidationAndPreview(filePath) {
      if (!filePath) return;

      const cleanName = filePath.split('/').pop().split('\\').pop();
      const ext = cleanName.includes('.') ? cleanName.split('.').pop().toLowerCase() : '';

      // 1. Immediate file extension validation with in-DOM Alert Dialog
      if (!['xlsx', 'xls', 'xlsm'].includes(ext)) {
        await appError(
          `Sistem hanya menerima berkas spreadsheet Excel (.xlsx, .xls, .xlsm).\n\n` +
          `Berkas yang dipilih: "${cleanName}" (${ext ? '.' + ext : 'tanpa ekstensi'})\n\n` +
          `Status: Berkas TIDAK diunggah dan TIDAK masuk ke sistem.`,
          '⚠️ Format Berkas Ditolak'
        );
        resetImportSelection();
        return;
      }

      // 2. Staging visual state
      if (fileNameSpan) fileNameSpan.textContent = cleanName;
      if (statusBadge) {
        statusBadge.className = 'status-badge';
        statusBadge.textContent = 'Memverifikasi struktur berkas...';
      }
      if (fileSelectedBox) fileSelectedBox.classList.remove('hidden');
      if (btnExecute) {
        btnExecute.disabled = true;
        btnExecute.textContent = 'Memverifikasi berkas...';
      }

      const targetUnitVal = targetUnitSelect?.value || 'auto';
      const targetUnit = targetUnitVal === 'auto' ? null : targetUnitVal;

      try {
        const preview = await invoke('preview_import_file', {
          filePath,
          targetUnit,
        });

        currentPreview = preview;
        pendingImportPath = filePath;

        // Display metadata
        if (metaUnit) metaUnit.textContent = preview.detected_unit || '-';
        if (metaPeriod) metaPeriod.textContent = formatPeriodLabel(preview.period_string);
        if (metaCount) metaCount.textContent = `${preview.valid_rows_count} baris valid`;
        if (metaTotal) metaTotal.textContent = formatRupiah(preview.total_amount);

        // Security safeguard: Duplicate file check
        if (preview.is_duplicate_file) {
          if (statusBadge) {
            statusBadge.className = 'status-badge zero';
            statusBadge.textContent = '⚠️ Peringatan: Berkas Duplikat';
          }
          const dup = preview.duplicate_log;
          const dupTime = formatDateTime(dup?.imported_at) || 'waktu sebelumnya';
          const dupUnit = dup?.unit || preview.detected_unit;
          const dupRecords = dup?.records_count || preview.valid_rows_count;
          const dupAmt = dup ? formatRupiah(dup.total_amount) : '';

          if (warningBox) {
            warningBox.innerHTML = `
              <strong>⚠️ Peringatan Keamanan Berkas Duplikat:</strong><br>
              Berkas dengan konten identik ini sudah pernah diimpor pada <u>${escapeHtml(dupTime)}</u> untuk unit <strong>${escapeHtml(dupUnit)}</strong> (${escapeHtml(dupRecords)} baris, ${escapeHtml(dupAmt)}).<br>
              <em>Catatan:</em> Mengimpor ulang berkas yang sama berisiko menimpa atau menduplikasi transaksi jika mode yang dipilih tidak tepat.
            `;
            warningBox.classList.remove('hidden');
          }
          if (duplicateConfirmWrap) {
            duplicateConfirmWrap.classList.remove('hidden');
            if (chkDuplicateAck) chkDuplicateAck.checked = false;
          }
        } else {
          if (statusBadge) {
            statusBadge.className = 'status-badge active';
            statusBadge.textContent = '✓ Berkas Terverifikasi & Siap Diimpor';
          }
          if (warningBox) {
            warningBox.classList.add('hidden');
            warningBox.innerHTML = '';
          }
          if (duplicateConfirmWrap) {
            duplicateConfirmWrap.classList.add('hidden');
          }
        }

        // Period conflict check
        if (existingInfo) {
          if (preview.existing_period_transactions_count > 0) {
            existingInfo.textContent = `Catatan: Periode ${formatPeriodLabel(preview.period_string)} saat ini sudah memiliki ${preview.existing_period_transactions_count} transaksi di database. Mode pengimporan yang dipilih akan diterapkan saat Anda mengklik Mulai Impor.`;
            existingInfo.classList.remove('hidden');
          } else {
            existingInfo.classList.add('hidden');
            existingInfo.textContent = '';
          }
        }

        if (btnExecute) {
          btnExecute.disabled = false;
          btnExecute.textContent = 'Mulai Proses Impor ke Database';
        }
      } catch (err) {
        await appError(
          `${err}\n\nStatus: Berkas TIDAK diunggah dan database tetap aman.`,
          '⚠️ Format Berkas Tidak Sesuai'
        );
        resetImportSelection();
      } finally {
        if (btnExecute && !currentPreview) {
          btnExecute.disabled = false;
          btnExecute.textContent = 'Mulai Proses Impor ke Database';
        }
      }
    }

    if (dropzone) {
      dropzone.addEventListener('click', async (e) => {
        if (e.target === hiddenInput) return;

        try {
          const filePath = await invoke('pick_excel_file');
          if (filePath) {
            await handleFileValidationAndPreview(filePath);
          }
        } catch (err) {
          await appError(`Gagal memilih berkas: ${err}`, 'Kesalahan Pemilihan Berkas');
        }
      });

      // Drag and Drop support
      dropzone.addEventListener('dragover', (e) => {
        e.preventDefault();
        e.stopPropagation();
        dropzone.classList.add('dragover');
      });

      dropzone.addEventListener('dragleave', (e) => {
        e.preventDefault();
        e.stopPropagation();
        dropzone.classList.remove('dragover');
      });

      dropzone.addEventListener('drop', async (e) => {
        e.preventDefault();
        e.stopPropagation();
        dropzone.classList.remove('dragover');

        const files = e.dataTransfer?.files;
        if (files && files.length > 0) {
          const file = files[0];
          const cleanName = file.name || 'berkas';
          const ext = cleanName.includes('.') ? cleanName.split('.').pop().toLowerCase() : '';

          // Validate extension first
          if (!['xlsx', 'xls', 'xlsm'].includes(ext)) {
            await appError(
              `Sistem hanya menerima berkas spreadsheet Excel (.xlsx, .xls, .xlsm).\n\n` +
              `Berkas yang ditarik: "${cleanName}" (${ext ? '.' + ext : 'tanpa ekstensi'})\n\n` +
              `Status: Berkas TIDAK diunggah ke sistem.`,
              '⚠️ Format Berkas Ditolak'
            );
            resetImportSelection();
            return;
          }

          if (file.path) {
            await handleFileValidationAndPreview(file.path);
          } else {
            await appAlert(
              `Berkas Excel "${cleanName}" terdeteksi. Silakan klik area kotak unggah untuk memilih berkas melalui dialog desktop.`,
              'Informasi Pengunggahan'
            );
          }
        }
      });
    }

    if (btnCancelFile) {
      btnCancelFile.addEventListener('click', (e) => {
        e.stopPropagation();
        resetImportSelection();
      });
    }

    if (targetUnitSelect) {
      targetUnitSelect.addEventListener('change', async () => {
        if (pendingImportPath) {
          await handleFileValidationAndPreview(pendingImportPath);
        }
      });
    }

    if (btnExecute) {
      btnExecute.addEventListener('click', async () => {
        if (!pendingImportPath || !currentPreview) {
          await appAlert('Silakan pilih berkas Excel yang valid terlebih dahulu.', 'Peringatan', 'warning');
          return;
        }

        // Duplicate confirmation prompt with in-DOM dialog
        if (currentPreview.is_duplicate_file) {
          const isAck = chkDuplicateAck?.checked;
          if (!isAck) {
            const proceed = await appConfirm(
              `Berkas ini terdeteksi sudah pernah diimpor sebelumnya ke database (Log #${currentPreview.duplicate_log?.id || 'lama'} pada ${formatDateTime(currentPreview.duplicate_log?.imported_at) || 'sebelumnya'}).\n\n` +
              `Jika Anda melanjutkan, data periode terkait akan diproses sesuai mode impor yang dipilih.\n\n` +
              `Apakah Anda yakin ingin tetap memproses pengimporan berkas ini?`,
              '⚠️ Konfirmasi Impor Berkas Duplikat',
              'warning',
              'Tetap Impor Berkas',
              'Batal'
            );
            if (!proceed) return;
          }
        }

        const targetUnitVal = targetUnitSelect?.value || 'auto';
        const targetUnit = targetUnitVal === 'auto' ? null : targetUnitVal;
        const mode = document.querySelector('input[name="import-mode"]:checked')?.value || 'replace_period';

        btnExecute.disabled = true;
        btnExecute.innerHTML = '<span class="spinner-sm"></span> Menyimpan transaksi ke database...';
        if (progressBox) progressBox.classList.remove('hidden');

        try {
          const result = await invoke('import_excel_file', {
            filePath: pendingImportPath,
            options: {
              targetUnit,
              mode,
            },
          });

          const periodDesc = formatPeriodLabel(result.period || result.months_affected?.join(', '), result.months_affected?.[0]);
          resetImportSelection();

          await refreshAvailableMonths();
          await loadImportLogs();
          await loadRecapData();

          // In-DOM Success Alert Dialog
          await appSuccess(
            `Data pendapatan berhasil diimpor dan disimpan ke database!\n\n` +
            `• Unit Bisnis: ${result.unit}\n` +
            `• Periode Data: ${periodDesc}\n` +
            `• Transaksi Diimpor: ${result.imported_records} baris\n` +
            `• Transaksi Dilewati: ${result.skipped_records} baris\n` +
            `• Total Nominal: ${formatRupiah(result.total_amount)}\n\n` +
            `Status: Seluruh transaksi tersimpan aman di database dan berkas asli telah diarsipkan.`,
            '✅ Impor Berkas Berhasil'
          );
        } catch (err) {
          await appError(
            `Gagal menyimpan transaksi ke database:\n\n${err}\n\n` +
            `Catatan Pengaman: Seluruh transaksi dibatalkan (rollback). Data di database Anda tetap aman.`,
            '❌ Gagal Mengimpor Data'
          );
        } finally {
          btnExecute.disabled = false;
          btnExecute.textContent = 'Mulai Proses Impor ke Database';
          if (progressBox) progressBox.classList.add('hidden');
        }
      });
    }

    const btnDownloadTpl = document.getElementById('btn-download-import-template');
    if (btnDownloadTpl) {
      btnDownloadTpl.addEventListener('click', downloadTemplate);
    }
  }

  async function downloadTemplate() {
    try {
      const res = await invoke('download_excel_template', {
        unit: state.activeUnit,
      });
      await appSuccess(res, 'Template Diunduh');
    } catch (err) {
      await appError(`Gagal mengunduh template: ${err}`, 'Gagal Mengunduh Template');
    }
  }

  async function loadImportLogs() {
    try {
      const logs = await invoke('get_import_logs', { unit: state.activeUnit });
      const tbody = document.getElementById('import-logs-body');
      tbody.innerHTML = '';

      if (logs.length === 0) {
        tbody.innerHTML = `<tr><td colspan="6" class="text-center text-muted">Belum ada riwayat berkas impor.</td></tr>`;
      } else {
        logs.forEach(l => {
          const tr = document.createElement('tr');
          const periodText = formatPeriodLabel(l.period, l.imported_at);
          tr.innerHTML = `
            <td class="font-mono text-muted">${escapeHtml(formatDateTime(l.imported_at))}</td>
            <td><strong>${escapeHtml(l.unit || '-')}</strong></td>
            <td>${escapeHtml(l.file_name || '-')}</td>
            <td><span class="badge badge-period">${escapeHtml(periodText)}</span></td>
            <td style="text-align: right;" class="font-mono">${escapeHtml(l.records_count)}</td>
            <td style="text-align: right;" class="font-mono text-accent">${escapeHtml(formatRupiah(l.total_amount))}</td>
          `;
          tbody.appendChild(tr);
        });
      }
      document.getElementById('import-logs-count').textContent = `${logs.length} Berkas Terarsip`;
    } catch (err) {
      console.error('Gagal memuat log impor:', err);
      const tbody = document.getElementById('import-logs-body');
      if (tbody) {
        tbody.innerHTML = `<tr><td colspan="6" class="text-center text-warning">Gagal memuat riwayat impor: ${escapeHtml(err)}</td></tr>`;
      }
    }
  }

  // =========================================================================
  // 6. Module 4: Audit Target (Subset Solver)
  // =========================================================================
  function initOptimizerModule() {
    const dropzone = document.getElementById('optimizer-dropzone');
    const btnChangeFile = document.getElementById('btn-opt-change-file');
    const btnRun = document.getElementById('btn-run-solver');
    const btnRegen = document.getElementById('btn-regenerate');
    const btnExport = document.getElementById('btn-export-excel');

    async function handlePick() {
      try {
        const path = await invoke('pick_excel_file');
        if (path) await analyzeOptimizerFile(path);
      } catch (err) {
        alert(`Gagal: ${err}`);
      }
    }

    if (dropzone) dropzone.addEventListener('click', handlePick);
    if (btnChangeFile) btnChangeFile.addEventListener('click', handlePick);

    if (btnRun) {
      btnRun.addEventListener('click', () => runOptimizerSolver(false));
    }
    if (btnRegen) {
      btnRegen.addEventListener('click', () => runOptimizerSolver(true));
    }
    if (btnExport) {
      btnExport.addEventListener('click', exportOptimizerExcel);
    }
  }

  async function analyzeOptimizerFile(filePath) {
    try {
      const analysis = await invoke('analyze_excel', { filePath });
      state.optFilePath = filePath;
      state.optAnalysis = analysis;
      state.optResult = null;
      state.optSeenSignatures = [];

      document.getElementById('optimizer-file-info').classList.remove('hidden');
      document.getElementById('optimizer-dropzone').classList.add('hidden');

      document.getElementById('optimizer-file-name').textContent = filePath.split('/').pop().split('\\').pop();
      document.getElementById('opt-meta-sheet').textContent = analysis.sheet_name;
      document.getElementById('opt-meta-range').textContent = `Baris ${analysis.data_start_row} s/d ${analysis.data_end_row}`;
      document.getElementById('opt-meta-count').textContent = `${analysis.total_rows} Baris`;
      document.getElementById('opt-meta-total').textContent = formatRupiah(analysis.total_sum);

      document.getElementById('opt-input-target').value = analysis.target_detected;
      document.getElementById('opt-kpi-target').textContent = formatRupiah(analysis.target_detected);
      document.getElementById('opt-kpi-formula').textContent = `Formula: =${analysis.target_formula || 'TOTAL / 2'}`;
      document.getElementById('opt-kpi-selected').textContent = 'Rp 0';
      document.getElementById('opt-kpi-count').textContent = `0 dari ${analysis.total_rows} baris`;
      document.getElementById('opt-kpi-diff').textContent = 'Rp 0';
      document.getElementById('opt-kpi-time').textContent = 'Waktu: 0 ms';

      const is13Col = analysis.format_type === 'RoomReservation13Col';
      const totalLabel = document.getElementById('opt-meta-total-label');
      const targetHint = document.getElementById('opt-target-hint');
      const cardTitle = document.getElementById('opt-preview-card-title');
      const btnExport = document.getElementById('btn-export-excel');
      const btnRegen = document.getElementById('btn-regenerate');

      if (btnRegen) btnRegen.classList.add('hidden');
      if (btnExport) btnExport.disabled = true;

      if (totalLabel) {
        totalLabel.textContent = is13Col ? 'Total Payment (Kolom L)' : 'Total Kolom B';
      }
      if (targetHint) {
        targetHint.textContent = is13Col ? '50% Total Payment' : 'Dari B80 / 50%';
      }
      if (cardTitle) {
        cardTitle.textContent = is13Col
          ? 'Pratinjau Hasil Pembagian Template 13 Kolom Rekap Tamu'
          : 'Pratinjau Hasil Pembagian Kolom E & F';
      }
      if (btnExport) {
        btnExport.textContent = is13Col
          ? 'Simpan File Template 13 Kolom Bersih'
          : 'Tulis ke Kolom E & F File Excel';
      }

      const theadTr = document.getElementById('opt-preview-thead-tr');
      const tbody = document.getElementById('opt-preview-body');
      if (theadTr) {
        if (is13Col) {
          theadTr.innerHTML = `
            <th style="width: 40px; text-align: center;">#</th>
            <th style="width: 88px;">DATE</th>
            <th>NAME</th>
            <th style="width: 55px; text-align: center;">Pax</th>
            <th style="width: 110px;">Booking No</th>
            <th style="width: 105px;">Nationality</th>
            <th style="width: 65px; text-align: center;">Kamar</th>
            <th style="width: 65px; text-align: center;">Jml Kamar</th>
            <th style="width: 88px;">Check In</th>
            <th style="width: 88px;">Check Out</th>
            <th style="width: 58px; text-align: center;">Malam</th>
            <th style="width: 105px; text-align: right;">Rate</th>
            <th style="width: 125px; text-align: right;">Total Payment</th>
            <th style="width: 100px;">Remarks</th>
            <th style="width: 65px; text-align: center;">Baris</th>
          `;
        } else {
          theadTr.innerHTML = `
            <th style="width: 80px;">Baris</th>
            <th style="width: 140px;" id="opt-th-date">Kolom E (Tanggal)</th>
            <th style="text-align: right; width: 180px;" id="opt-th-amount">Kolom F (Nominal)</th>
            <th>Keterangan Formula Target</th>
          `;
        }
      }
      if (tbody) {
        tbody.innerHTML = `
          <tr>
            <td colspan="${is13Col ? 15 : 4}" class="text-center text-muted">Klik &ldquo;Jalankan Optimasi Solver&rdquo; untuk menampilkan pratinjau hasil.</td>
          </tr>
        `;
      }

      const warnBox = document.getElementById('opt-conflict-warning');
      if (warnBox) {
        const conflicts = Array.isArray(analysis.room_conflicts) ? analysis.room_conflicts : [];
        const anomalies = Array.isArray(analysis.room_anomalies) ? analysis.room_anomalies : [];
        if (conflicts.length === 0 && anomalies.length === 0) {
          warnBox.classList.add('hidden');
          warnBox.innerHTML = '';
        } else {
          const parts = [];
          if (conflicts.length > 0) {
            const sample = conflicts.slice(0, 8).map(c =>
              `<li>Kamar <strong>${escapeHtml(c.room_number)}</strong> (${escapeHtml(c.date)}): baris ${escapeHtml(c.rows.join(', '))}</li>`
            ).join('');
            const more = conflicts.length > 8 ? `<li>...dan ${conflicts.length - 8} bentrok malam-kamar lainnya</li>` : '';
            parts.push(
              `<div><strong>Peringatan Bentrok Kamar pada File Sumber (${conflicts.length} malam-kamar):</strong><ul style="margin: 4px 0 6px 18px;">${sample}${more}</ul><div>Solver otomatis mencegah baris yang bentrok terpilih bersamaan (nomor kamar asli tidak diubah).</div></div>`
            );
          }
          if (anomalies.length > 0) {
            const sample = anomalies.slice(0, 5).map(a =>
              `<li>Baris ${escapeHtml(a.row)} (Kamar '${escapeHtml(a.room_number || '-')}'): ${escapeHtml(a.message)}</li>`
            ).join('');
            const more = anomalies.length > 5 ? `<li>...dan ${anomalies.length - 5} anomali kamar lainnya</li>` : '';
            parts.push(
              `<div${conflicts.length > 0 ? ' style="margin-top: 8px;"' : ''}><strong>Anomali Data Kamar Terdeteksi (${anomalies.length}):</strong><ul style="margin: 4px 0 0 18px;">${sample}${more}</ul></div>`
            );
          }
          warnBox.innerHTML = parts.join('');
          warnBox.classList.remove('hidden');
        }
      }

      document.getElementById('btn-run-solver').disabled = false;
    } catch (err) {
      alert(`Gagal menganalisis berkas Excel: ${err}`);
    }
  }

  async function runOptimizerSolver(isRegen = false) {
    if (!state.optAnalysis) return;

    const target = parseFloat(document.getElementById('opt-input-target').value);
    if (Number.isNaN(target) || !Number.isFinite(target) || target <= 0) {
      alert('Target nominal harus berupa angka lebih besar dari 0.');
      return;
    }
    const tolerance = parseFloat(document.getElementById('opt-select-tolerance').value);
    const trajectories = parseInt(document.getElementById('opt-select-trajectories').value, 10);

    const options = {
      target,
      tolerance,
      max_trajectories: trajectories,
      seed: Math.floor(Math.random() * 1000000),
      exclude_signatures: isRegen ? state.optSeenSignatures : [],
    };

    try {
      const result = await invoke('solve_subset', {
        items: state.optAnalysis.items,
        options,
      });

      state.optResult = result;
      if (result.signature) {
        state.optSeenSignatures.push(result.signature);
      }

      const is13Col = state.optAnalysis && state.optAnalysis.format_type === 'RoomReservation13Col';
      document.getElementById('opt-kpi-selected').textContent = formatRupiah(result.total_amount);
      document.getElementById('opt-kpi-count').textContent = is13Col
        ? `${result.selected_indices.length} dari ${state.optAnalysis.total_rows} baris (0 bentrok kamar)`
        : `${result.selected_indices.length} dari ${state.optAnalysis.total_rows} baris`;
      document.getElementById('opt-kpi-diff').textContent = formatRupiah(result.difference);
      document.getElementById('opt-kpi-time').textContent = `Waktu: ${result.duration_ms.toFixed(2)} ms`;

      document.getElementById('btn-regenerate').classList.remove('hidden');
      document.getElementById('btn-export-excel').disabled = false;

      // Render Preview Table
      renderOptimizerPreview(result);
    } catch (err) {
      alert(`Solver: ${err}`);
    }
  }

  function renderOptimizerPreview(result) {
    const theadTr = document.getElementById('opt-preview-thead-tr');
    const tbody = document.getElementById('opt-preview-body');
    if (!tbody) return;
    tbody.innerHTML = '';

    const selectedSet = new Set(result.selected_indices);
    const selectedItems = state.optAnalysis.items.filter((_, idx) => selectedSet.has(idx));
    const is13Col = state.optAnalysis && state.optAnalysis.format_type === 'RoomReservation13Col';

    if (theadTr) {
      if (is13Col) {
        theadTr.innerHTML = `
          <th style="width: 40px; text-align: center;">#</th>
          <th style="width: 88px;">DATE</th>
          <th>NAME</th>
          <th style="width: 55px; text-align: center;">Pax</th>
          <th style="width: 110px;">Booking No</th>
          <th style="width: 105px;">Nationality</th>
          <th style="width: 65px; text-align: center;">Kamar</th>
          <th style="width: 65px; text-align: center;">Jml Kamar</th>
          <th style="width: 88px;">Check In</th>
          <th style="width: 88px;">Check Out</th>
          <th style="width: 58px; text-align: center;">Malam</th>
          <th style="width: 105px; text-align: right;">Rate</th>
          <th style="width: 125px; text-align: right;">Total Payment</th>
          <th style="width: 100px;">Remarks</th>
          <th style="width: 65px; text-align: center;">Baris</th>
        `;
      } else {
        theadTr.innerHTML = `
          <th style="width: 80px;">Baris</th>
          <th style="width: 140px;" id="opt-th-date">Kolom E (Tanggal)</th>
          <th style="text-align: right; width: 180px;" id="opt-th-amount">Kolom F (Nominal)</th>
          <th>Keterangan Formula Target</th>
        `;
      }
    }

    if (is13Col) {
      selectedItems.forEach((item, idx) => {
        const tr = document.createElement('tr');
        const det = item.details || {};
        const paxStr = (det.total_person !== null && det.total_person !== undefined) ? det.total_person : '-';
        const bookingStr = det.booking_number || '-';
        const nationalityStr = det.nationality || '-';
        const roomStr = det.room_number ? String(det.room_number) : '-';
        const roomsStr = (det.total_rooms !== null && det.total_rooms !== undefined) ? det.total_rooms : 1;
        const checkIn = det.check_in_date || item.date_str;
        const checkOut = det.check_out_date || item.date_str;
        const nights = (det.total_nights !== null && det.total_nights !== undefined) ? det.total_nights : '-';
        const rateStr = (det.rate !== null && det.rate !== undefined) ? formatRupiah(det.rate) : '-';
        const remarksStr = det.remarks || '-';
        const guestName = item.guest_name || 'Tamu';

        tr.innerHTML = `
          <td class="font-mono text-muted text-center">${escapeHtml(idx + 1)}</td>
          <td class="font-mono">${escapeHtml(item.date_str)}</td>
          <td><strong>${escapeHtml(guestName)}</strong></td>
          <td class="font-mono text-center">${escapeHtml(paxStr)}</td>
          <td class="font-mono text-muted">${escapeHtml(bookingStr)}</td>
          <td>${escapeHtml(nationalityStr)}</td>
          <td class="font-mono text-center"><span class="badge badge-subtle">${escapeHtml(roomStr)}</span></td>
          <td class="font-mono text-center">${escapeHtml(roomsStr)}</td>
          <td class="font-mono text-muted">${escapeHtml(checkIn)}</td>
          <td class="font-mono text-muted">${escapeHtml(checkOut)}</td>
          <td class="font-mono text-center">${escapeHtml(nights)}</td>
          <td class="font-mono text-right">${escapeHtml(rateStr)}</td>
          <td class="font-mono text-accent text-right"><strong>${escapeHtml(formatRupiah(item.amount))}</strong></td>
          <td><span class="badge badge-period">${escapeHtml(remarksStr)}</span></td>
          <td class="text-muted font-mono text-center">#${escapeHtml(item.original_row)}</td>
        `;
        tbody.appendChild(tr);
      });

      const trSum = document.createElement('tr');
      trSum.className = 'row-summary-total';
      trSum.innerHTML = `
        <td class="font-mono text-muted text-center">TOTAL</td>
        <td colspan="11">Total Hasil Acak Terpilih (${escapeHtml(selectedItems.length)} Reservasi)</td>
        <td class="font-mono text-success text-right"><strong>${escapeHtml(formatRupiah(result.total_amount))}</strong></td>
        <td colspan="2" class="font-mono text-muted">=SUM(L2:L${escapeHtml(selectedItems.length + 1)})</td>
      `;
      tbody.appendChild(trSum);

      const trTarget = document.createElement('tr');
      trTarget.className = 'row-summary-target';
      trTarget.innerHTML = `
        <td class="font-mono text-muted text-center">TARGET</td>
        <td colspan="11">Target Nominal Solver</td>
        <td class="font-mono text-accent text-right">${escapeHtml(formatRupiah(result.target_amount))}</td>
        <td colspan="2" class="font-mono text-muted">Selisih: ${escapeHtml(formatRupiah(result.difference))}</td>
      `;
      tbody.appendChild(trTarget);
    } else {
      selectedItems.forEach((item, idx) => {
        const tr = document.createElement('tr');
        const itemDesc = item.guest_name
          ? `${item.guest_name} (Baris asal #${item.original_row})`
          : `Item baris asal #${item.original_row}`;
        tr.innerHTML = `
          <td class="font-mono text-muted">${escapeHtml(idx + 2)}</td>
          <td class="font-mono">${escapeHtml(item.date_str)}</td>
          <td class="font-mono text-accent text-right">${escapeHtml(formatRupiah(item.amount))}</td>
          <td class="text-muted">${escapeHtml(itemDesc)}</td>
        `;
        tbody.appendChild(tr);
      });

      const sumRow = selectedItems.length + 2;
      const trSum = document.createElement('tr');
      trSum.className = 'row-summary-total';
      trSum.innerHTML = `
        <td class="font-mono text-muted">${escapeHtml(sumRow)}</td>
        <td>Jumlah</td>
        <td class="font-mono text-success text-right">${escapeHtml(formatRupiah(result.total_amount))}</td>
        <td class="font-mono text-muted">=SUM(F2:F${escapeHtml(sumRow - 1)})</td>
      `;
      tbody.appendChild(trSum);
    }
  }
  async function exportOptimizerExcel() {
    if (!state.optFilePath || !state.optResult) return;
    try {
      const is13Col = state.optAnalysis && state.optAnalysis.format_type === 'RoomReservation13Col';
      let destPath = state.optFilePath;

      if (isTauri && is13Col) {
        const rawFileName = state.optFilePath.split('/').pop().split('\\').pop().replace(/\.xlsx$/i, '');
        const defaultName = `${rawFileName}_Hasil_Acak_Template_13_Kolom.xlsx`;
        const picked = await invoke('pick_save_file', { defaultName });
        if (!picked) return;
        destPath = picked;
      }

      const msg = await invoke('export_excel', {
        sourcePath: state.optFilePath,
        destinationPath: destPath,
        items: state.optAnalysis.items,
        selectedIndices: state.optResult.selected_indices,
      });
      alert(`Berhasil! ${msg}`);
    } catch (err) {
      alert(`Gagal mengekspor: ${err}`);
    }
  }

  // =========================================================================
  // 7. Settings Modal & Backup/Restore Logic
  // =========================================================================
  function initSettingsModal() {
    const btnOpenSettings = document.getElementById('btn-open-settings');
    const modalSettings = document.getElementById('modal-settings');

    if (btnOpenSettings) {
      btnOpenSettings.addEventListener('click', async () => {
        modalSettings.classList.remove('hidden');
        await loadSettingsData();
      });
    }

    // Tab switcher in settings modal
    const sTabs = document.querySelectorAll('.settings-tab');
    sTabs.forEach(tab => {
      tab.addEventListener('click', async () => {
        sTabs.forEach(t => t.classList.remove('active'));
        tab.classList.add('active');

        document.querySelectorAll('.settings-pane').forEach(p => p.classList.remove('active'));
        const pane = document.getElementById(`stab-${tab.dataset.stab}`);
        if (pane) pane.classList.add('active');

        if (tab.dataset.stab === 'storage') {
          await loadStorageInfo();
        } else if (tab.dataset.stab === 'users') {
          await loadUsersSettings();
        }
      });
    });

    // Save profile settings
    const btnSaveProfile = document.getElementById('btn-save-profile-settings');
    if (btnSaveProfile) {
      btnSaveProfile.addEventListener('click', async () => {
        const settings = {
          businessName: document.getElementById('set-biz-name').value.trim(),
          businessAddress: document.getElementById('set-biz-addr').value.trim(),
          npwpd: document.getElementById('set-npwpd').value.trim(),
          signPosition: document.getElementById('set-sign-pos').value.trim(),
          signName: document.getElementById('set-sign-name').value.trim(),
          officerName: document.getElementById('set-officer-name').value.trim(),
          officerNip: document.getElementById('set-officer-nip').value.trim(),
        };
        try {
          await invoke('update_settings', { settings });
          alert('Profil usaha dan pengaturan berhasil disimpan!');
          await loadRecapData();
        } catch (err) {
          alert(`Gagal: ${err}`);
        }
      });
    }

    // Backup & Restore
    const btnBackup = document.getElementById('btn-backup-now');
    if (btnBackup) {
      btnBackup.addEventListener('click', async () => {
        try {
          const res = await invoke('backup_database_dialog');
          alert(res);
        } catch (err) {
          alert(`Pencadangan gagal: ${err}`);
        }
      });
    }

    const btnRestore = document.getElementById('btn-restore-now');
    if (btnRestore) {
      btnRestore.addEventListener('click', async () => {
        const confirmed = await appConfirm(
          'PERINGATAN: Memulihkan database akan menggantikan seluruh data transaksi aktif dengan isi file cadangan.\n\nSistem akan membuat salinan darurat (.pre_restore.bak) sebelum penggantian.\n\nLanjutkan?',
          'Konfirmasi Pemulihan Database'
        );
        if (!confirmed) return;
        try {
          const res = await invoke('restore_database_dialog');
          alert(res);
          await refreshAvailableMonths();
          await loadActiveView();
        } catch (err) {
          alert(`Pemulihan gagal: ${err}`);
        }
      });
    }

    // User management in settings
    const btnShowAddUser = document.getElementById('btn-show-add-user');
    const boxAddUser = document.getElementById('box-add-user');
    const btnCancelAddUser = document.getElementById('btn-cancel-add-user');
    const btnSaveNewUser = document.getElementById('btn-save-new-user');

    if (btnShowAddUser && boxAddUser) {
      btnShowAddUser.addEventListener('click', () => boxAddUser.classList.remove('hidden'));
    }
    if (btnCancelAddUser && boxAddUser) {
      btnCancelAddUser.addEventListener('click', () => boxAddUser.classList.add('hidden'));
    }
    if (btnSaveNewUser) {
      btnSaveNewUser.addEventListener('click', async () => {
        const username = document.getElementById('new-u-username').value.trim();
        const fullName = document.getElementById('new-u-fullname').value.trim();
        const role = document.getElementById('new-u-role').value;
        const pin = document.getElementById('new-u-pin').value.trim();

        if (!username || !fullName || pin.length < 4) {
          alert('Mohon isi semua data akun dan PIN minimal 4 digit.');
          return;
        }

        try {
          await invoke('auth_create_user', { username, fullName, role, pin });
          alert(`Akun ${username} berhasil dibuat!`);
          boxAddUser.classList.add('hidden');
          await loadUsersSettings();
          await refreshLoginUsers();
        } catch (err) {
          alert(`Gagal: ${err}`);
        }
      });
    }
  }

  async function loadSettingsData() {
    try {
      const s = await invoke('get_settings');
      document.getElementById('set-biz-name').value = s.businessName || '';
      document.getElementById('set-biz-addr').value = s.businessAddress || '';
      document.getElementById('set-npwpd').value = s.npwpd || '';
      document.getElementById('set-sign-pos').value = s.signPosition || '';
      document.getElementById('set-sign-name').value = s.signName || '';
      document.getElementById('set-officer-name').value = s.officerName || '';
      document.getElementById('set-officer-nip').value = s.officerNip || '';
    } catch (err) {
      console.error('Gagal mengambil pengaturan:', err);
    }
  }

  async function loadStorageInfo() {
    try {
      const info = await invoke('get_storage_info');
      document.getElementById('st-db-path').textContent = info.db_path;
      document.getElementById('st-db-size').textContent = formatBytes(info.db_size_bytes);
      document.getElementById('st-archive-dir').textContent = info.archive_dir;
      document.getElementById('st-archive-size').textContent = formatBytes(info.archive_size_bytes);
      document.getElementById('st-total-records').textContent = `${info.total_transactions} Baris Transaksi`;
      document.getElementById('st-total-logs').textContent = `${info.total_import_logs} Riwayat Berkas`;
    } catch (err) {
      console.error('Gagal memuat info storage:', err);
    }
  }

  async function loadUsersSettings() {
    try {
      const users = await invoke('auth_list_users');
      const tbody = document.getElementById('settings-users-body');
      tbody.innerHTML = '';

      users.forEach(u => {
        const tr = document.createElement('tr');
        tr.innerHTML = `
          <td><strong>${escapeHtml(u.username)}</strong></td>
          <td>${escapeHtml(u.full_name)}</td>
          <td><span class="user-role-badge">${escapeHtml(u.role)}</span></td>
          <td class="text-muted" style="font-size: 11px;">${escapeHtml(formatDateTime(u.last_login) || 'Belum pernah')}</td>
          <td style="text-align: center;">
            <button class="btn-link text-warning btn-del-u" data-id="${escapeHtml(u.id)}">Hapus</button>
          </td>
        `;
        tbody.appendChild(tr);
      });

      tbody.querySelectorAll('.btn-del-u').forEach(btn => {
        btn.addEventListener('click', async () => {
          const targetId = parseInt(btn.dataset.id, 10);
          const confirmed = await appConfirm('Hapus akun pengguna ini?');
          if (!confirmed) return;
          try {
            await invoke('auth_delete_user', {
              targetUserId: targetId,
              currentUserId: state.currentUser ? state.currentUser.id : 0,
            });
            await loadUsersSettings();
            await refreshLoginUsers();
          } catch (err) {
            alert(`Gagal: ${err}`);
          }
        });
      });
    } catch (err) {
      console.error('Gagal mengambil data user:', err);
    }
  }

  // =========================================================================
  // 8. Global Modals Dismiss Handler & Print Trigger
  // =========================================================================
  function initModalsDismiss() {
    document.querySelectorAll('[data-close]').forEach(btn => {
      btn.addEventListener('click', () => {
        const modalId = btn.dataset.close;
        const modal = document.getElementById(modalId);
        if (modal) modal.classList.add('hidden');
        if (modalId === 'modal-print') {
          document.title = APP_DEFAULT_TITLE;
          if (isTauri) {
            invoke('set_window_title', { title: APP_DEFAULT_TITLE }).catch(() => {});
          }
        }
      });
    });

    const btnTriggerPrint = document.getElementById('btn-trigger-print');
    if (btnTriggerPrint) {
      btnTriggerPrint.addEventListener('click', async () => {
        cleanupCertificatePrintMode();
        const docNum = state.currentRecap?.document_number;
        const reportTitle = getCleanReportFileName(docNum) || document.title;
        document.title = reportTitle;
        try {
          if (isTauri) {
            await invoke('print_report', { title: reportTitle });
          } else {
            window.print();
          }
        } catch (err) {
          console.warn('Fallback print:', err);
          window.print();
        }
      });
    }

    window.addEventListener('beforeprint', () => {
      if (document.body.classList.contains('printing-certificate')) return;
      const docNum = state.currentRecap?.document_number;
      if (docNum) {
        const cleanTitle = getCleanReportFileName(docNum);
        document.title = cleanTitle;
        if (isTauri) {
          invoke('set_window_title', { title: cleanTitle }).catch(() => {});
        }
      }
    });

    window.addEventListener('afterprint', () => {
      if (
        document.documentElement.classList.contains('printing-certificate') ||
        document.body.classList.contains('printing-certificate')
      ) {
        cleanupCertificatePrintMode();
        return;
      }
      const modal = document.getElementById('modal-print');
      if (!modal || modal.classList.contains('hidden')) {
        document.title = APP_DEFAULT_TITLE;
        if (isTauri) {
          invoke('set_window_title', { title: APP_DEFAULT_TITLE }).catch(() => {});
        }
      }
    });

    const chkSig = document.getElementById('chk-toggle-signatures');
    if (chkSig) {
      chkSig.addEventListener('change', () => {
        const sigWrap = document.getElementById('p-signatures-wrapper');
        if (sigWrap) {
          if (chkSig.checked) {
            sigWrap.classList.remove('hidden');
          } else {
            sigWrap.classList.add('hidden');
          }
        }
      });
    }
  }

  // =========================================================================
  // 9. Module 5: Pendataan Wisatawan (Dinas Pariwisata Form 1)
  // =========================================================================
  function updateTourismAccTypeBoxes() {
    const type = state.tourismAccType || 'Hotel Non Bintang';
    const chkB = document.getElementById('t-chk-bintang');
    const chkNB = document.getElementById('t-chk-nonbintang');
    const chkHS = document.getElementById('t-chk-homestay');
    const setBox = (el, active) => {
      if (!el) return;
      el.classList.toggle('checked', active);
      el.textContent = active ? '✓' : '';
    };
    setBox(chkB, type === 'Hotel Bintang');
    setBox(chkNB, type === 'Hotel Non Bintang');
    setBox(chkHS, type === 'Homestay');
  }

  function initTourismModule() {
    const monthSelect = document.getElementById('tourism-month-select');
    const capInput = document.getElementById('tourism-room-capacity');
    const accSelect = document.getElementById('tourism-acc-type');
    const btnRefresh = document.getElementById('btn-refresh-tourism');

    if (capInput) {
      capInput.value = String(state.tourismRoomCapacity);
      capInput.addEventListener('change', () => {
        const val = Math.max(1, parseInt(capInput.value, 10) || 14);
        capInput.value = String(val);
        state.tourismRoomCapacity = val;
        localStorage.setItem('anda_erp_room_capacity', String(val));
        loadTourismData();
      });
    }

    if (accSelect) {
      accSelect.value = state.tourismAccType;
      updateTourismAccTypeBoxes();
      accSelect.addEventListener('change', () => {
        state.tourismAccType = accSelect.value;
        localStorage.setItem('anda_erp_acc_type', accSelect.value);
        updateTourismAccTypeBoxes();
      });
    }

    if (monthSelect) {
      monthSelect.addEventListener('change', () => {
        state.tourismMonth = monthSelect.value;
        loadTourismData();
      });
    }

    if (btnRefresh) {
      btnRefresh.addEventListener('click', async () => {
        await refreshAvailableMonths();
        await loadTourismData();
      });
    }
  }

  async function loadTourismData() {
    const month = state.tourismMonth || document.getElementById('tourism-month-select')?.value;
    if (!month) return;

    try {
      const report = await invoke('get_tourism_report', {
        unit: state.activeUnit,
        month,
        roomCapacity: state.tourismRoomCapacity,
      });

      document.getElementById('t-acc-name').textContent = report.accommodation_name || '-';
      document.getElementById('t-room-cap').textContent = `${report.room_capacity} Kamar`;
      document.getElementById('t-period-label').textContent = `${report.month_name} ${report.year}`;
      document.getElementById('t-occupancy-rate').textContent = `${report.occupancy_rate_pct.toFixed(1).replace('.', ',')}%`;
      document.getElementById('t-days-badge').textContent = `${report.days_in_month} Hari Kalender`;

      document.getElementById('t-kpi-res').textContent = `${report.total_reservations}`;
      document.getElementById('t-kpi-guests').textContent = `${report.total_guests} Orang`;
      document.getElementById('t-kpi-guests-split').textContent =
        `Domestik: ${report.domestic_guests} | Mancanegara: ${report.foreign_guests}`;
      document.getElementById('t-kpi-room-nights').textContent = `${report.total_room_nights} Room-Night`;
      document.getElementById('t-kpi-room-vacant').textContent =
        `Vacant: ${report.weekly_total.room_vacant} Room-Night`;
      document.getElementById('t-kpi-bed-nights').textContent = `${report.total_bed_nights} Bed-Night`;
      document.getElementById('t-kpi-bed-co').textContent =
        `Check Out (Bed): ${report.weekly_total.bed_check_out}`;

      updateTourismAccTypeBoxes();

      // Render Weekly Table
      const wBody = document.getElementById('tourism-weekly-body');
      const wFoot = document.getElementById('tourism-weekly-foot');
      if (wBody) {
        wBody.innerHTML = '';
        report.weekly_rows.forEach((w) => {
          const tr = document.createElement('tr');
          tr.innerHTML = `
            <td><strong>${escapeHtml(w.label)}</strong></td>
            <td style="text-align: center;" class="font-mono">${escapeHtml(w.domestic_check_in)}</td>
            <td style="text-align: center;" class="font-mono">${escapeHtml(w.domestic_check_out)}</td>
            <td style="text-align: center;" class="font-mono">${escapeHtml(w.foreign_check_in)}</td>
            <td style="text-align: center;" class="font-mono">${escapeHtml(w.foreign_check_out)}</td>
            <td style="text-align: center;" class="font-mono text-accent"><strong>${escapeHtml(w.room_occupied)}</strong></td>
            <td style="text-align: center;" class="font-mono text-muted">${escapeHtml(w.room_vacant)}</td>
            <td style="text-align: center;" class="font-mono">${escapeHtml(w.bed_check_out)}</td>
            <td style="text-align: center;" class="font-mono text-accent"><strong>${escapeHtml(w.bed_occupied)}</strong></td>
            <td class="text-muted" style="font-size: 11px;">${escapeHtml(w.days_count)} hari</td>
          `;
          wBody.appendChild(tr);
        });
      }
      if (wFoot) {
        const tot = report.weekly_total;
        wFoot.innerHTML = `
          <tr>
            <th>TOTAL (${escapeHtml(report.days_in_month)} Hari)</th>
            <th style="text-align: center;" class="font-mono">${escapeHtml(tot.domestic_check_in)}</th>
            <th style="text-align: center;" class="font-mono">${escapeHtml(tot.domestic_check_out)}</th>
            <th style="text-align: center;" class="font-mono">${escapeHtml(tot.foreign_check_in)}</th>
            <th style="text-align: center;" class="font-mono">${escapeHtml(tot.foreign_check_out)}</th>
            <th style="text-align: center;" class="font-mono text-accent">${escapeHtml(tot.room_occupied)}</th>
            <th style="text-align: center;" class="font-mono">${escapeHtml(tot.room_vacant)}</th>
            <th style="text-align: center;" class="font-mono">${escapeHtml(tot.bed_check_out)}</th>
            <th style="text-align: center;" class="font-mono text-accent">${escapeHtml(tot.bed_occupied)}</th>
            <th>-</th>
          </tr>
        `;
      }

      // Render Top 10 Origins Table
      const oBody = document.getElementById('tourism-origins-body');
      const foreign = report.foreign_origins || [];
      const domestic = report.domestic_origins || [];
      document.getElementById('t-origins-summary').textContent =
        `${foreign.length} Negara | ${domestic.length} Daerah`;

      if (oBody) {
        oBody.innerHTML = '';
        for (let i = 0; i < 10; i++) {
          const fItem = foreign[i];
          const dItem = domestic[i];
          const tr = document.createElement('tr');
          tr.innerHTML = `
            <td style="text-align: center;" class="font-mono text-muted">${i + 1}.</td>
            <td>${fItem ? `<strong>${escapeHtml(fItem.origin_name)}</strong>` : '<span class="text-muted">-</span>'}</td>
            <td style="text-align: right;" class="font-mono">${fItem ? `${escapeHtml(fItem.guest_count)} Orang` : '<span class="text-muted">-</span>'}</td>
            <td>${dItem ? `<strong>${escapeHtml(dItem.origin_name)}</strong>` : '<span class="text-muted">-</span>'}</td>
            <td style="text-align: right;" class="font-mono">${dItem ? `${escapeHtml(dItem.guest_count)} Orang` : '<span class="text-muted">-</span>'}</td>
          `;
          oBody.appendChild(tr);
        }
      }

      // Render full list if > 10 entries in either category
      const allCard = document.getElementById('tourism-all-origins-card');
      const allForeignBody = document.getElementById('tourism-all-foreign-body');
      const allDomesticBody = document.getElementById('tourism-all-domestic-body');
      if (allCard && allForeignBody && allDomesticBody) {
        if (foreign.length > 10 || domestic.length > 10) {
          allCard.classList.remove('hidden');
          allForeignBody.innerHTML = foreign.length
            ? foreign.map(item => `
                <tr>
                  <td style="text-align: center;" class="font-mono text-muted">${escapeHtml(item.rank)}</td>
                  <td>${escapeHtml(item.origin_name)}</td>
                  <td style="text-align: right;" class="font-mono"><strong>${escapeHtml(item.guest_count)}</strong> Orang</td>
                  <td style="text-align: right;" class="font-mono text-muted">${escapeHtml(item.reservation_count)}</td>
                </tr>
              `).join('')
            : '<tr><td colspan="4" class="text-center text-muted">Tidak ada data mancanegara.</td></tr>';

          allDomesticBody.innerHTML = domestic.length
            ? domestic.map(item => `
                <tr>
                  <td style="text-align: center;" class="font-mono text-muted">${escapeHtml(item.rank)}</td>
                  <td>${escapeHtml(item.origin_name)}</td>
                  <td style="text-align: right;" class="font-mono"><strong>${escapeHtml(item.guest_count)}</strong> Orang</td>
                  <td style="text-align: right;" class="font-mono text-muted">${escapeHtml(item.reservation_count)}</td>
                </tr>
              `).join('')
            : '<tr><td colspan="4" class="text-center text-muted">Tidak ada data nusantara.</td></tr>';
        } else {
          allCard.classList.add('hidden');
        }
      }

      // Stay duration category
      const avgEl = document.getElementById('t-avg-stay-value');
      if (avgEl) {
        avgEl.textContent = `${report.avg_stay_nights.toFixed(2).replace('.', ',')} Malam`;
      }
      const box1 = document.getElementById('t-stay-box-1');
      const box2 = document.getElementById('t-stay-box-2');
      const box3 = document.getElementById('t-stay-box-3');
      const markStay = (el, active) => {
        if (!el) return;
        el.classList.toggle('checked', active);
        el.textContent = active ? '✓' : '';
      };
      markStay(box1, report.stay_category === '1-2 MALAM');
      markStay(box2, report.stay_category === '3 MALAM');
      markStay(box3, report.stay_category === '> 3 MALAM');
    } catch (err) {
      console.error('Gagal memuat laporan Form 1 Pariwisata:', err);
      const wBody = document.getElementById('tourism-weekly-body');
      if (wBody) {
        wBody.innerHTML = `<tr><td colspan="10" class="text-center text-warning">Gagal memuat data Form 1 Pariwisata: ${escapeHtml(err)}</td></tr>`;
      }
    }
  }

  // =========================================================================
  // 10. Module 6: Generator & Arsip Sertifikat Training (FEAT-10)
  // =========================================================================
  const DEFAULT_CERT_BODY_TEMPLATE = 'for successfully completing the {durasi} {jenis_program} in the {departemen} at {properti}, from {mulai} to {selesai}.';
  const DEFAULT_CERT_ISSUED_CITY = 'Kuta, Lombok';

  function suggestEnglishDuration(startIso, endIso) {
    const words = [
      'one', 'two', 'three', 'four', 'five', 'six',
      'seven', 'eight', 'nine', 'ten', 'eleven', 'twelve',
    ];
    if (!startIso || !endIso) return 'three-month';
    const s = new Date(`${startIso}T00:00:00`);
    const e = new Date(`${endIso}T00:00:00`);
    if (isNaN(s.getTime()) || isNaN(e.getTime())) return 'three-month';
    const days = Math.round((e - s) / 86400000);
    if (days < 0) return 'one-day';
    if (days <= 5) {
      const inclusiveDays = days + 1;
      return `${words[inclusiveDays - 1] || inclusiveDays}-day`;
    }
    if (days < 28) {
      const weeks = Math.max(1, Math.round(days / 7));
      return `${words[weeks - 1] || weeks}-week`;
    }
    const months = Math.max(1, Math.round(days / 30.4375));
    return months <= 12 ? `${words[months - 1]}-month` : `${months}-month`;
  }

  function showCertStatus(message, type = 'info') {
    const el = document.getElementById('cert-status-live');
    if (!el) return;
    if (!message) {
      el.classList.add('hidden');
      el.textContent = '';
      return;
    }
    el.className = `cert-status-line ${type}`;
    el.textContent = message;
  }

  function getSelectedCertPaperSize() {
    const checked = document.querySelector('input[name="cert_paper_size"]:checked');
    return checked?.value === 'A4' ? 'A4' : 'F4';
  }

  function setSelectedCertPaperSize(paperSize) {
    const targetVal = String(paperSize || 'F4').toUpperCase() === 'A4' ? 'A4' : 'F4';
    const radio = document.querySelector(`input[name="cert_paper_size"][value="${targetVal}"]`);
    if (radio) radio.checked = true;
    updateCertPaperFrameLabel(targetVal);
  }

  function updateCertPaperFrameLabel(paperSize) {
    const isA4 = String(paperSize || 'F4').toUpperCase() === 'A4';
    const frame = document.getElementById('cert-preview-frame');
    const label = document.getElementById('cert-paper-dim-label');
    if (frame) frame.classList.toggle('paper-a4', isA4);
    if (label) {
      label.textContent = isA4
        ? 'Kertas A4 Horizontal (297 x 210 mm)'
        : 'Kertas F4 Horizontal (330 x 215 mm)';
    }
  }

  const CERT_MAX_DIVISIONS = 5;

  function parseDepartmentString(raw) {
    if (!raw || typeof raw !== 'string') return [{ name: '', duration: '' }];
    const parts = raw.split(';').map((s) => s.trim()).filter((s) => s && s !== '-');
    if (parts.length === 0) return [{ name: '', duration: '' }];
    return parts.map((seg) => {
      const m = /^(.*?)\s*\(([^()]+)\)\s*$/.exec(seg);
      if (m && m[1].trim()) {
        return { name: m[1].trim(), duration: m[2].trim() };
      }
      return { name: seg, duration: '' };
    });
  }

  function readDivisionRowsFromDom() {
    const rows = [];
    document.querySelectorAll('#cert-divisions-list .cert-division-row').forEach((rowEl) => {
      const name = rowEl.querySelector('.cert-div-name')?.value || '';
      const duration = rowEl.querySelector('.cert-div-duration')?.value || '';
      rows.push({ name, duration });
    });
    return rows.length ? rows : [{ name: '', duration: '' }];
  }

  function serializeDivisionRows() {
    const formatted = [];
    document.querySelectorAll('#cert-divisions-list .cert-division-row').forEach((rowEl) => {
      const name = (rowEl.querySelector('.cert-div-name')?.value || '').trim();
      const rawDuration = (rowEl.querySelector('.cert-div-duration')?.value || '').trim();
      const cleanDur = rawDuration.replace(/[()]/g, '').trim();
      if (!name) return;
      formatted.push(cleanDur ? `${name} (${cleanDur})` : name);
    });
    return formatted.length ? formatted.join('; ') : null;
  }

  function findDepartmentByNameOrCode(val) {
    const q = String(val || '').trim().toLowerCase();
    if (!q) return null;
    const stripped = q.replace(/\s+department$/i, '').trim();
    return (
      (state.certDepartments || []).find(
        (d) =>
          d.name.toLowerCase() === q ||
          d.name.toLowerCase() === stripped ||
          d.code.toLowerCase() === q
      ) || null
    );
  }

  function getSelectedDepartmentCode() {
    const selected = Array.from(
      document.querySelectorAll('#cert-divisions-list .cert-division-row .cert-div-name')
    ).filter((el) => el.value.trim());
    if (selected.length >= 2) return 'ROT';
    if (selected.length === 1) {
      const dept = findDepartmentByNameOrCode(selected[0].value);
      return dept ? dept.code : null;
    }
    return null;
  }

  function extractCoveredClause(desc) {
    const trimmed = String(desc || '').trim().replace(/[.!?]+$/, '').trim();
    if (!trimmed) return '';
    if (/^the program covered\s+/i.test(trimmed)) {
      return trimmed.replace(/^the program covered\s+/i, '').trim();
    }
    if (/^training in\s+/i.test(trimmed)) {
      return trimmed.replace(/^training in\s+/i, '').trim();
    }
    return trimmed;
  }

  function combineDeptDescriptions(depts) {
    const cleanDepts = (depts || []).filter((d) => String(d.program_description || '').trim());
    if (cleanDepts.length === 0) return '';
    if (cleanDepts.length === 1) {
      const raw = String(cleanDepts[0].program_description).trim();
      if (/^training in\s+/i.test(raw)) {
        const clause = extractCoveredClause(raw);
        return `The program covered ${clause}.`;
      }
      return /[.!?]$/.test(raw) ? raw : `${raw}.`;
    }
    const clauses = cleanDepts.map((d) => extractCoveredClause(d.program_description)).filter(Boolean);
    if (clauses.length === 0) return '';
    if (clauses.length === 1) return `The program covered ${clauses[0]}.`;
    const head = clauses.slice(0, -1).join('; ');
    const last = clauses[clauses.length - 1];
    return `The program covered ${head}; as well as ${last}.`;
  }

  function lowercaseLeadingWordExceptAcronym(phrase) {
    const trimmed = String(phrase || '').trim();
    if (!trimmed) return '';
    const firstBreak = trimmed.search(/[\s(]/);
    const firstWord = firstBreak === -1 ? trimmed : trimmed.slice(0, firstBreak);
    if (/^[A-Z0-9.]{2,}$/.test(firstWord) || /^(Booking\.com|Agoda)$/i.test(firstWord)) {
      return trimmed;
    }
    return `${trimmed[0].toLowerCase()}${trimmed.slice(1)}`;
  }

  const DEFAULT_CERT_NO_HINT = 'Format otomatis: 001/ABR-FO/XI/2026 (kode departemen & bulan Romawi, urutan unik per tahun)';

  function setCertNoLocked(locked) {
    const isLocked = Boolean(locked);
    const noInput = document.getElementById('cert-no');
    const autoBtn = document.getElementById('btn-cert-auto-no');
    const hintEl = document.getElementById('cert-no-hint');
    if (noInput) {
      noInput.readOnly = isLocked;
      noInput.classList.toggle('is-locked', isLocked);
      noInput.setAttribute('aria-readonly', isLocked ? 'true' : 'false');
    }
    if (autoBtn) autoBtn.disabled = isLocked;
    if (hintEl) {
      hintEl.textContent = isLocked
        ? 'Terkunci permanen: nomor sertifikat tidak berubah setelah diterbitkan (termasuk jika departemen diubah menjadi rotasi).'
        : DEFAULT_CERT_NO_HINT;
    }
  }

  function formatCompetencySentence(items) {
    const normalized = (items || [])
      .map((it) => lowercaseLeadingWordExceptAcronym(it).replace(/[.,;]+$/, '').trim())
      .filter(Boolean);
    if (normalized.length === 0) return '';
    if (normalized.length === 1) return `The trainee demonstrated competence in ${normalized[0]}.`;
    if (normalized.length === 2) {
      const hasAnd = normalized.some((item) => /\band\b/i.test(item));
      return hasAnd
        ? `The trainee demonstrated competence in ${normalized[0]}, as well as ${normalized[1]}.`
        : `The trainee demonstrated competence in ${normalized[0]} and ${normalized[1]}.`;
    }
    const needsSemicolon = normalized.some((item) => /[,;]/.test(item) || /\band\b/i.test(item));
    const head = normalized.slice(0, -1);
    const last = normalized[normalized.length - 1];
    return needsSemicolon
      ? `The trainee demonstrated competence in ${head.join('; ')}; and ${last}.`
      : `The trainee demonstrated competence in ${head.join(', ')}, and ${last}.`;
  }

  function parseDurationToMonths(raw) {
    const text = String(raw || '').trim().toLowerCase();
    if (!text) return null;
    const wordNums = {
      one: 1, two: 2, three: 3, four: 4, five: 5, six: 6,
      seven: 7, eight: 8, nine: 9, ten: 10, eleven: 11, twelve: 12,
    };
    const m = /^(\d+(?:[.,]\d+)?|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve)[-\s]+(year|years|yr|yrs|tahun|thn|month|months|mo|mos|bulan|bln|week|weeks|wk|wks|minggu|mgg|day|days|hari)\b/.exec(text);
    if (!m) return null;
    const num = wordNums[m[1]] ?? parseFloat(m[1].replace(',', '.'));
    if (!Number.isFinite(num) || num <= 0) return null;
    const unit = m[2];
    if (unit.startsWith('year') || unit.startsWith('yr') || unit === 'tahun' || unit === 'thn') return num * 12;
    if (unit.startsWith('week') || unit.startsWith('wk') || unit === 'minggu' || unit === 'mgg') return num / 4.33;
    if (unit.startsWith('day') || unit === 'hari') return num / 30.0;
    return num;
  }

  function formatMonthsSummary(val) {
    const rounded = Math.round(val * 10) / 10;
    return Number.isInteger(rounded) ? `${rounded} bulan` : `${rounded.toFixed(1)} bulan`;
  }

  function updateDivisionDurationBalance() {
    const el = document.getElementById('cert-divisions-balance');
    const emptyState = { exceeded: false, under: false, sumMonths: null, periodMonths: null, message: '' };
    const rows = readDivisionRowsFromDom().filter((r) => r.name.trim());
    const withDur = rows.filter((r) => r.duration.trim());
    if (withDur.length === 0) {
      if (el) {
        el.setAttribute('role', 'status');
        el.className = 'cert-field-hint hidden';
        el.textContent = '';
      }
      return emptyState;
    }

    let sumMonths = 0;
    let unparsed = 0;
    withDur.forEach((r) => {
      const m = parseDurationToMonths(r.duration);
      if (m === null) unparsed += 1;
      else sumMonths += m;
    });

    const startVal = document.getElementById('cert-start')?.value;
    const endVal = document.getElementById('cert-end')?.value;
    const durVal = document.getElementById('cert-duration')?.value;
    let periodMonths = parseDurationToMonths(durVal);
    if (periodMonths === null && startVal && endVal) {
      const s = new Date(`${startVal}T00:00:00`);
      const e = new Date(`${endVal}T00:00:00`);
      if (!isNaN(s.getTime()) && !isNaN(e.getTime()) && e >= s) {
        periodMonths = Math.max(0.1, (e - s) / (86400000 * 30.4375));
      }
    }

    const isLockedNo = Boolean(state.certEditingId || document.getElementById('cert-no')?.readOnly);
    if (periodMonths !== null && sumMonths > periodMonths + 0.35) {
      const msg = `Error: total durasi divisi (${formatMonthsSummary(sumMonths)}) melebihi periode utama (${formatMonthsSummary(periodMonths)}). Penyimpanan diblokir sampai durasi disesuaikan.`;
      if (el) {
        el.className = 'cert-field-hint cert-balance-error';
        el.setAttribute('role', 'alert');
        el.textContent = msg;
      }
      return { exceeded: true, under: false, sumMonths, periodMonths, message: msg };
    }

    if (el) el.setAttribute('role', 'status');
    const rotLockNote = isLockedNo ? 'nomor sertifikat tetap terkunci' : 'kode nomor: ABR-ROT';
    if (unparsed > 0 || periodMonths === null) {
      const msg = rows.length >= 2
        ? `Rotasi ${rows.length} divisi (${rotLockNote}). Pastikan total durasi divisi tidak melebihi periode utama.`
        : 'Format durasi divisi tidak dikenali; gunakan contoh seperti "3 months" atau "4 weeks".';
      if (el) {
        el.className = 'cert-field-hint';
        el.textContent = msg;
      }
      return { exceeded: false, under: false, sumMonths, periodMonths, message: msg };
    }

    if (sumMonths < periodMonths - 0.35) {
      const msg = `Peringatan: total durasi divisi (${formatMonthsSummary(sumMonths)}) lebih singkat dari periode utama (${formatMonthsSummary(periodMonths)}). Sertifikat tetap dapat disimpan bila memang demikian.`;
      if (el) {
        el.className = 'cert-field-hint cert-balance-warn';
        el.textContent = msg;
      }
      return { exceeded: false, under: true, sumMonths, periodMonths, message: msg };
    }

    const matchLockNote = isLockedNo ? 'nomor sertifikat terkunci' : 'kode nomor: ABR-ROT';
    const msg = rows.length >= 2
      ? `Total durasi ${rows.length} divisi (${formatMonthsSummary(sumMonths)}) sesuai periode utama (${matchLockNote}).`
      : `Durasi divisi (${formatMonthsSummary(sumMonths)}) sesuai periode utama.`;
    if (el) {
      el.className = 'cert-field-hint text-success';
      el.textContent = msg;
    }
    return { exceeded: false, under: false, sumMonths, periodMonths, message: msg };
  }

  function populateProgramTypeSelect(selectedCode = '') {
    const select = document.getElementById('cert-program-type');
    if (!select) return;
    const list = (state.certProgramTypes || []).filter((p) => p.is_active !== false);
    const opts = list.map(
      (pt) =>
        `<option value="${escapeHtml(pt.code)}">${escapeHtml(pt.code)} &mdash; ${escapeHtml(pt.name)}</option>`
    );
    opts.push('<option value="">Kustom (isi manual di Detail Tambahan)</option>');
    select.innerHTML = opts.join('');
    if (selectedCode && list.some((p) => p.code.toUpperCase() === selectedCode.toUpperCase())) {
      const matched = list.find((p) => p.code.toUpperCase() === selectedCode.toUpperCase());
      select.value = matched.code;
    } else if (list.length > 0 && selectedCode !== '__custom__') {
      select.value = list[0].code;
    } else {
      select.value = '';
    }
  }

  function renderDeptCriteriaPanel() {
    const panel = document.getElementById('cert-dept-criteria-panel');
    const titleEl = document.getElementById('cert-dept-criteria-title');
    const descEl = document.getElementById('cert-dept-criteria-desc');
    const compContainer = document.getElementById('cert-dept-competencies-list');
    if (!panel || !titleEl || !descEl || !compContainer) return;

    const seenCodes = new Set();
    const matchedDepts = [];
    document.querySelectorAll('#cert-divisions-list .cert-division-row .cert-div-name').forEach((el) => {
      const dept = findDepartmentByNameOrCode(el.value);
      if (dept && !seenCodes.has(dept.code)) {
        seenCodes.add(dept.code);
        matchedDepts.push(dept);
      }
    });

    if (matchedDepts.length === 0) {
      panel.classList.add('hidden');
      compContainer.innerHTML = '';
      return;
    }

    panel.classList.remove('hidden');
    const primary = matchedDepts[0];
    const isLockedNo = Boolean(state.certEditingId || document.getElementById('cert-no')?.readOnly);
    titleEl.textContent =
      matchedDepts.length === 1
        ? `${primary.code} - ${primary.name}`
        : `Rotasi ${matchedDepts.map((d) => d.code).join(' + ')} (${isLockedNo ? 'Nomor Sertifikat Terkunci' : 'Kode Nomor: ABR-ROT'})`;
    descEl.textContent = combineDeptDescriptions(matchedDepts);

    const allComps = [];
    matchedDepts.forEach((d) => {
      String(d.competencies || '')
        .split(/\r?\n/)
        .map((s) => s.trim())
        .filter(Boolean)
        .forEach((item) => {
          if (!allComps.includes(item)) allComps.push(item);
        });
    });

    compContainer.innerHTML = allComps
      .map(
        (comp) => `
        <label class="cert-comp-chip">
          <input type="checkbox" class="cert-comp-checkbox" value="${escapeHtml(comp)}">
          <span>${escapeHtml(comp)}</span>
        </label>
      `
      )
      .join('');
  }

  function renderDivisionRows(items = [{ name: '', duration: '' }]) {
    const container = document.getElementById('cert-divisions-list');
    if (!container) return;
    const list = items.length ? items.slice(0, CERT_MAX_DIVISIONS) : [{ name: '', duration: '' }];
    const activeDepts = (state.certDepartments || []).filter((d) => d.is_active !== false);

    container.innerHTML = list
      .map((item, idx) => {
        const curName = String(item.name || '').trim();
        const curLower = curName.toLowerCase();
        const matched = activeDepts.find(
          (d) =>
            d.name.toLowerCase() === curLower ||
            d.name.toLowerCase() === curLower.replace(/\s+department$/i, '').trim() ||
            d.code.toLowerCase() === curLower
        );
        const optionsHtml = [
          `<option value="">-- Pilih Departemen (${idx === 0 ? 'Utama' : `Rotasi ke-${idx + 1}`}) --</option>`,
          ...activeDepts.map((d) => {
            const selected = matched && matched.code === d.code ? ' selected' : '';
            return `<option value="${escapeHtml(d.name)}"${selected}>${escapeHtml(d.code)} &mdash; ${escapeHtml(d.name)}</option>`;
          }),
        ];
        if (curName && !matched) {
          optionsHtml.push(
            `<option value="${escapeHtml(curName)}" selected>${escapeHtml(curName)} (Kustom)</option>`
          );
        }
        return `
      <div class="cert-division-row" data-div-index="${idx}">
        <select class="cert-field-control cert-div-name" aria-label="Nama divisi ke-${idx + 1}">
          ${optionsHtml.join('')}
        </select>
        <input type="text" class="cert-field-control cert-div-duration"
          value="${escapeHtml(item.duration)}"
          aria-label="Durasi divisi ke-${idx + 1}"
          placeholder="mis. 3 months">
        <button type="button" class="btn btn-secondary btn-sm cert-division-remove"
          data-remove-div="${idx}"
          aria-label="${list.length > 1 ? `Hapus divisi ke-${idx + 1}` : 'Kosongkan divisi'}">${list.length > 1 ? 'Hapus' : 'Bersihkan'}</button>
      </div>
    `;
      })
      .join('');

    const addBtn = document.getElementById('btn-cert-add-division');
    if (addBtn) addBtn.disabled = list.length >= CERT_MAX_DIVISIONS;
    renderDeptCriteriaPanel();
    updateDivisionDurationBalance();
  }

  function syncAchievementPresetSelect() {
    const input = document.getElementById('cert-achievement');
    const select = document.getElementById('cert-achievement-preset');
    if (!input || !select) return;
    const val = input.value.trim();
    if (!val) {
      select.value = '';
      return;
    }
    const matched = Array.from(select.options).some((opt) => opt.value && opt.value !== '__custom__' && opt.value === val);
    select.value = matched ? val : '__custom__';
  }

  function formatDivisionsDisplay(raw) {
    if (!raw) return '-';
    const items = parseDepartmentString(raw).filter((it) => it.name.trim());
    if (!items.length) return '-';
    return items.map((it) => (it.duration ? `${it.name} (${it.duration})` : it.name)).join(' & ');
  }

  function collectCertFormPayload(overrideStatus = null, { forAutoCreate = false } = {}) {
    const val = (id) => document.getElementById(id)?.value?.trim() || '';
    const optVal = (id) => {
      const v = val(id);
      return v.length > 0 ? v : null;
    };
    const def = state.certSettings || {};
    const progCode = optVal('cert-program-type');
    const progRow = progCode
      ? (state.certProgramTypes || []).find((p) => p.code.toUpperCase() === progCode.toUpperCase())
      : null;

    return {
      certificate_no: forAutoCreate ? null : optVal('cert-no'),
      recipient_name: val('cert-recipient'),
      duration_text: val('cert-duration'),
      program_name: optVal('cert-program') || progRow?.name || def.program_name || 'hospitality training program',
      start_date: val('cert-start'),
      end_date: val('cert-end'),
      issued_date: val('cert-issued'),
      issued_city: optVal('cert-issued-city') || def.issued_city || DEFAULT_CERT_ISSUED_CITY,
      body_template: optVal('cert-body-template') || def.body_template || DEFAULT_CERT_BODY_TEMPLATE,
      department: serializeDivisionRows(),
      department_code: getSelectedDepartmentCode(),
      program_type_code: progCode,
      institution: optVal('cert-institution'),
      achievement_text: optVal('cert-achievement'),
      notes: optVal('cert-notes'),
      signer_name: optVal('cert-signer') || def.signer_name || 'Agus Sumardi Wijaya',
      signer_title: optVal('cert-signer-title') || def.signer_title || 'General Manager',
      paper_size: getSelectedCertPaperSize(),
      status: overrideStatus || state.certEditingStatus || 'active',
    };
  }

  function validateCertEndDateCustomValidity() {
    const startEl = document.getElementById('cert-start');
    const endEl = document.getElementById('cert-end');
    if (!startEl || !endEl) return;
    if (startEl.value && endEl.value && endEl.value < startEl.value) {
      endEl.setCustomValidity('Tanggal selesai tidak boleh lebih awal dari tanggal mulai.');
    } else {
      endEl.setCustomValidity('');
    }
  }

  function syncCertFieldAria(el, forceFlagInvalid = false) {
    if (!el || !el.matches?.('input, select, textarea')) return;
    validateCertEndDateCustomValidity();
    const isValid = el.validity.valid;
    const shouldShowError = !isValid && (forceFlagInvalid || el.matches(':user-invalid') || el.classList.contains('user-invalid-fallback'));
    if (forceFlagInvalid && !isValid) {
      el.classList.add('user-invalid-fallback');
    } else if (isValid) {
      el.classList.remove('user-invalid-fallback');
    }
    el.setAttribute('aria-invalid', shouldShowError ? 'true' : 'false');
    const errorId = el.dataset.error;
    if (!errorId) return;
    const ids = (el.getAttribute('aria-describedby') || '')
      .split(/\s+/)
      .filter((id) => id && id !== errorId);
    if (shouldShowError) ids.push(errorId);
    if (ids.length) {
      el.setAttribute('aria-describedby', ids.join(' '));
    } else {
      el.removeAttribute('aria-describedby');
    }
  }

  function clearCertFormAriaErrors() {
    document.querySelectorAll('#form-certificate .cert-field-control').forEach((el) => {
      el.setCustomValidity('');
      el.classList.remove('user-invalid-fallback');
      el.setAttribute('aria-invalid', 'false');
      const errorId = el.dataset.error;
      if (errorId) {
        const ids = (el.getAttribute('aria-describedby') || '')
          .split(/\s+/)
          .filter((id) => id && id !== errorId);
        if (ids.length) el.setAttribute('aria-describedby', ids.join(' '));
        else el.removeAttribute('aria-describedby');
      }
    });
  }

  function validateCertificateForm() {
    validateCertEndDateCustomValidity();
    const form = document.getElementById('form-certificate');
    if (!form) return false;
    let firstInvalid = null;
    form.querySelectorAll('input[required]').forEach((el) => {
      const ok = el.checkValidity();
      syncCertFieldAria(el, !ok);
      if (!ok && !firstInvalid) firstInvalid = el;
    });
    if (firstInvalid) {
      firstInvalid.focus();
      showCertStatus('Mohon lengkapi atau perbaiki kolom sertifikat yang ditandai.', 'error');
      return false;
    }
    const balance = updateDivisionDurationBalance();
    if (balance.exceeded) {
      const firstDivDur = Array.from(document.querySelectorAll('#cert-divisions-list .cert-div-duration'))
        .find((input) => input.value.trim());
      firstDivDur?.focus();
      showCertStatus(balance.message, 'error');
      return false;
    }
    return true;
  }

  function scheduleCertPreview(immediate = false) {
    clearTimeout(state.certPreviewTimer);
    if (immediate) {
      refreshCertPreview();
    } else {
      state.certPreviewTimer = setTimeout(refreshCertPreview, 120);
    }
  }

  async function refreshCertPreview(explicitPayload = null) {
    const container = document.getElementById('cert-preview-container');
    if (!container) return;
    const seq = ++state.certPreviewSeq;
    const payload = explicitPayload || collectCertFormPayload();
    updateCertPaperFrameLabel(payload.paper_size);
    try {
      const svg = await invoke('preview_certificate_svg', { payload });
      if (seq !== state.certPreviewSeq) return;
      container.innerHTML = svg;
    } catch (err) {
      if (seq !== state.certPreviewSeq) return;
      container.innerHTML = `<div class="p-4 text-center text-muted">Gagal memuat pratinjau sertifikat: ${escapeHtml(err)}</div>`;
    }
  }

  function getIssuedYearAndMonth() {
    const issuedVal = document.getElementById('cert-issued')?.value || '';
    const m = /^(\d{4})-(\d{2})/.exec(issuedVal);
    if (m) {
      return {
        year: parseInt(m[1], 10),
        month: parseInt(m[2], 10),
      };
    }
    const now = new Date();
    return { year: now.getFullYear(), month: now.getMonth() + 1 };
  }

  async function refreshNextCertNoIfAuto() {
    const noInput = document.getElementById('cert-no');
    if (state.certManualNo || state.certEditingId || noInput?.readOnly) return;
    const { year, month } = getIssuedYearAndMonth();
    const departmentCode = getSelectedDepartmentCode();
    try {
      const nextNo = await invoke('get_next_certificate_no', {
        year,
        month,
        departmentCode,
      });
      if (noInput) noInput.value = nextNo;
    } catch (err) {
      console.warn('Gagal mengambil nomor sertifikat berikutnya:', err);
    }
  }

  function applyCertDefaultsToForm({ keepBatchFields = false } = {}) {
    const def = state.certSettings || {};
    const todayIso = getSystemLocalDateStr();
    const listProg = (state.certProgramTypes || []).filter((p) => p.is_active !== false);
    const defaultProg = listProg[0] || null;
    const defaultMonths = defaultProg?.default_duration_months || 3;
    const startDefault = getMonthsAgoLocalDateStr(defaultMonths);

    state.certEditingId = null;
    state.certEditingStatus = 'active';
    state.certManualNo = false;
    state.certStaffSavedLock = false;
    setCertNoLocked(false);

    const editBanner = document.getElementById('cert-edit-banner');
    if (editBanner) editBanner.classList.add('hidden');
    const heading = document.getElementById('cert-form-heading');
    if (heading) heading.textContent = 'Buat Sertifikat Baru';
    const btnSave = document.getElementById('btn-cert-save');
    if (btnSave) {
      btnSave.textContent = 'Simpan Sertifikat';
      btnSave.disabled = false;
    }
    const btnSaveNext = document.getElementById('btn-cert-save-next');
    if (btnSaveNext) {
      btnSaveNext.classList.remove('hidden');
      btnSaveNext.disabled = false;
    }

    if (keepBatchFields) {
      document.getElementById('cert-recipient').value = '';
      document.getElementById('cert-notes').value = '';
      renderDeptCriteriaPanel();
      updateDivisionDurationBalance();
    } else {
      state.certManualDuration = false;
      state.certManualDates = false;
      populateProgramTypeSelect(defaultProg?.code || '');
      const progSelectEl = document.getElementById('cert-program-type');
      const selectedProg = (state.certProgramTypes || []).find((p) => p.code === progSelectEl?.value);
      document.getElementById('cert-recipient').value = '';
      document.getElementById('cert-start').value = startDefault;
      document.getElementById('cert-end').value = todayIso;
      document.getElementById('cert-issued').value = todayIso;
      document.getElementById('cert-issued-city').value = def.issued_city || DEFAULT_CERT_ISSUED_CITY;
      document.getElementById('cert-duration').value = suggestEnglishDuration(startDefault, todayIso);
      document.getElementById('cert-program').value = selectedProg?.name || def.program_name || 'hospitality training program';
      document.getElementById('cert-body-template').value = '';
      renderDivisionRows([{ name: '', duration: '' }]);
      document.getElementById('cert-achievement').value = '';
      syncAchievementPresetSelect();
      document.getElementById('cert-institution').value = '';
      document.getElementById('cert-signer').value = def.signer_name || 'Agus Sumardi Wijaya';
      document.getElementById('cert-signer-title').value = def.signer_title || 'General Manager';
      document.getElementById('cert-notes').value = '';
      setSelectedCertPaperSize(def.paper_size || 'F4');
    }
    clearCertFormAriaErrors();
  }

  function populateCertFormFromRow(row, asEdit = false, { skipPreview = false } = {}) {
    const def = state.certSettings || {};
    state.certEditingId = asEdit ? row.id : null;
    state.certEditingStatus = row.status || 'active';
    state.certManualNo = true;
    state.certManualDuration = true;
    state.certManualDates = true;
    state.certStaffSavedLock = false;
    setCertNoLocked(Boolean(asEdit));
    const btnSaveReset = document.getElementById('btn-cert-save');
    const btnNextReset = document.getElementById('btn-cert-save-next');
    if (btnSaveReset) btnSaveReset.disabled = false;
    if (btnNextReset) btnNextReset.disabled = false;

    const matchedProg =
      row.program_type_code ||
      (state.certProgramTypes || []).find(
        (p) => p.name.toLowerCase() === String(row.program_name || '').trim().toLowerCase()
      )?.code ||
      '__custom__';
    populateProgramTypeSelect(matchedProg);

    document.getElementById('cert-no').value = row.certificate_no || '';
    document.getElementById('cert-recipient').value = row.recipient_name || '';
    document.getElementById('cert-start').value = row.start_date || '';
    document.getElementById('cert-end').value = row.end_date || '';
    document.getElementById('cert-duration').value = row.duration_text || '';
    document.getElementById('cert-issued').value = row.issued_date || '';
    document.getElementById('cert-issued-city').value = row.issued_city || def.issued_city || DEFAULT_CERT_ISSUED_CITY;
    document.getElementById('cert-program').value = row.program_name || 'hospitality training program';
    document.getElementById('cert-body-template').value =
      row.body_template && row.body_template !== (def.body_template || DEFAULT_CERT_BODY_TEMPLATE)
        ? row.body_template
        : '';
    renderDivisionRows(parseDepartmentString(row.department));
    document.getElementById('cert-achievement').value = row.achievement_text || '';
    syncAchievementPresetSelect();
    document.getElementById('cert-institution').value = row.institution || '';
    document.getElementById('cert-signer').value = row.signer_name || '';
    document.getElementById('cert-signer-title').value = row.signer_title || '';
    document.getElementById('cert-notes').value = row.notes || '';
    setSelectedCertPaperSize(row.paper_size || 'F4');
    clearCertFormAriaErrors();

    const details = document.getElementById('cert-optional-details');
    if (details && (row.institution || row.notes || document.getElementById('cert-body-template')?.value)) {
      details.open = true;
    }

    const editBanner = document.getElementById('cert-edit-banner');
    const editLabel = document.getElementById('cert-edit-label');
    const heading = document.getElementById('cert-form-heading');
    const btnSave = document.getElementById('btn-cert-save');
    const btnSaveNext = document.getElementById('btn-cert-save-next');

    if (asEdit) {
      if (editBanner) editBanner.classList.remove('hidden');
      if (editLabel) editLabel.textContent = `Mode Edit: ${row.certificate_no} (${row.recipient_name})`;
      if (heading) heading.textContent = 'Edit Data Sertifikat';
      if (btnSave) btnSave.textContent = 'Simpan Perubahan';
      if (btnSaveNext) btnSaveNext.classList.add('hidden');
      document.getElementById('cert-recipient')?.focus();
    } else {
      if (editBanner) editBanner.classList.add('hidden');
      if (heading) heading.textContent = 'Buat Sertifikat Baru';
      if (btnSave) btnSave.textContent = 'Simpan Sertifikat';
      if (btnSaveNext) btnSaveNext.classList.remove('hidden');
    }

    if (!skipPreview) {
      refreshCertPreview();
    }
  }

  function getCertDraftStorageKey() {
    return `anda_erp_cert_draft_${state.currentUser?.username || 'default'}`;
  }

  function serializeCertFormSnapshot() {
    const raw = (id) => document.getElementById(id)?.value?.trim() || '';
    return JSON.stringify({
      certificate_no: raw('cert-no'),
      recipient_name: raw('cert-recipient'),
      start_date: raw('cert-start'),
      end_date: raw('cert-end'),
      program_type_code: raw('cert-program-type'),
      duration_text: raw('cert-duration'),
      issued_date: raw('cert-issued'),
      issued_city: raw('cert-issued-city'),
      department: serializeDivisionRows(),
      achievement_text: raw('cert-achievement'),
      paper_size: getSelectedCertPaperSize(),
      program_name: raw('cert-program'),
      body_template: raw('cert-body-template'),
      institution: raw('cert-institution'),
      signer_name: raw('cert-signer'),
      signer_title: raw('cert-signer-title'),
      notes: raw('cert-notes'),
    });
  }

  function isCertFormDirty() {
    return Boolean(
      state.certInitialized &&
        state.certCleanSnapshot &&
        serializeCertFormSnapshot() !== state.certCleanSnapshot
    );
  }

  function syncCertDirtyUI() {
    const badge = document.getElementById('cert-dirty-badge');
    if (badge) badge.classList.toggle('hidden', !isCertFormDirty());
  }

  function markCertFormClean() {
    state.certCleanSnapshot = serializeCertFormSnapshot();
    syncCertDirtyUI();
  }

  function clearCertDraftFromStorage() {
    try {
      localStorage.removeItem(getCertDraftStorageKey());
    } catch (err) {
      console.warn('Gagal menghapus draf sertifikat:', err);
    }
  }

  function saveCertDraftToStorage() {
    if (!state.certInitialized) return;
    if (!isCertFormDirty()) {
      clearCertDraftFromStorage();
      return;
    }
    try {
      const draft = {
        version: 1,
        savedAt: new Date().toISOString(),
        editingId: state.certEditingId,
        editingStatus: state.certEditingStatus,
        editingLabel: document.getElementById('cert-edit-label')?.textContent || '',
        manualNo: state.certManualNo,
        manualDuration: state.certManualDuration,
        fields: {
          ...collectCertFormPayload(),
          body_template: document.getElementById('cert-body-template')?.value?.trim() || '',
        },
      };
      localStorage.setItem(getCertDraftStorageKey(), JSON.stringify(draft));
    } catch (err) {
      console.warn('Gagal menyimpan draf sertifikat:', err);
    }
  }

  function loadCertDraftFromStorage() {
    try {
      const raw = localStorage.getItem(getCertDraftStorageKey());
      if (!raw) return null;
      const parsed = JSON.parse(raw);
      if (parsed?.version === 1 && parsed?.fields && typeof parsed.fields === 'object') {
        return parsed;
      }
    } catch (err) {
      console.warn('Gagal membaca draf sertifikat tersimpan:', err);
    }
    return null;
  }

  function applyCertDraftToForm(draft) {
    const f = draft?.fields || {};
    const isEditDraft = Boolean(draft.editingId && isAdminUser());
    state.certEditingId = isEditDraft ? draft.editingId : null;
    state.certStaffSavedLock = false;
    setCertNoLocked(isEditDraft);
    if (draft.manualNo || isEditDraft) {
      document.getElementById('cert-no').value = f.certificate_no || '';
      state.certManualNo = Boolean(draft.manualNo || isEditDraft);
    }
    const btnPrimary = document.getElementById('btn-cert-save');
    const btnNext = document.getElementById('btn-cert-save-next');
    if (btnPrimary) btnPrimary.disabled = false;
    if (btnNext) btnNext.disabled = false;
    state.certManualDuration = Boolean(draft.manualDuration);
    state.certManualDates = true;

    populateProgramTypeSelect(f.program_type_code || '__custom__');
    document.getElementById('cert-recipient').value = f.recipient_name || '';
    document.getElementById('cert-start').value = f.start_date || '';
    document.getElementById('cert-end').value = f.end_date || '';
    document.getElementById('cert-duration').value = f.duration_text ?? '';
    document.getElementById('cert-issued').value = f.issued_date || '';
    document.getElementById('cert-issued-city').value = f.issued_city ?? (state.certSettings?.issued_city || DEFAULT_CERT_ISSUED_CITY);
    document.getElementById('cert-program').value = f.program_name ?? '';
    const defTpl = state.certSettings?.body_template || DEFAULT_CERT_BODY_TEMPLATE;
    document.getElementById('cert-body-template').value =
      f.body_template && f.body_template !== defTpl ? f.body_template : '';
    renderDivisionRows(parseDepartmentString(f.department));
    document.getElementById('cert-achievement').value = f.achievement_text || '';
    syncAchievementPresetSelect();
    document.getElementById('cert-institution').value = f.institution || '';
    document.getElementById('cert-signer').value = f.signer_name ?? '';
    document.getElementById('cert-signer-title').value = f.signer_title ?? '';
    document.getElementById('cert-notes').value = f.notes || '';
    setSelectedCertPaperSize(f.paper_size || 'F4');

    const details = document.getElementById('cert-optional-details');
    if (details && (f.institution || f.notes || document.getElementById('cert-body-template')?.value)) {
      details.open = true;
    }

    const editBanner = document.getElementById('cert-edit-banner');
    const editLabel = document.getElementById('cert-edit-label');
    const heading = document.getElementById('cert-form-heading');
    const btnSave = document.getElementById('btn-cert-save');
    const btnSaveNext = document.getElementById('btn-cert-save-next');

    if (isEditDraft) {
      state.certEditingStatus = draft.editingStatus || 'active';
      if (editBanner) editBanner.classList.remove('hidden');
      if (editLabel) {
        editLabel.textContent = draft.editingLabel || `Mode Edit: ${f.certificate_no || ''} (${f.recipient_name || ''})`;
      }
      if (heading) heading.textContent = 'Edit Data Sertifikat';
      if (btnSave) btnSave.textContent = 'Simpan Perubahan';
      if (btnSaveNext) btnSaveNext.classList.add('hidden');
      state.certCleanSnapshot = '__restored_edit_draft__';
    } else {
      state.certEditingId = null;
      state.certEditingStatus = 'active';
      if (editBanner) editBanner.classList.add('hidden');
      if (heading) heading.textContent = 'Buat Sertifikat Baru';
      if (btnSave) btnSave.textContent = 'Simpan Sertifikat';
      if (btnSaveNext) btnSaveNext.classList.remove('hidden');
    }

    clearCertFormAriaErrors();
    syncCertDirtyUI();
  }

  function enterArchivePreviewMode(row) {
    state.certPreviewRow = row;
    const banner = document.getElementById('cert-archive-preview-banner');
    const label = document.getElementById('cert-archive-preview-label');
    if (label) {
      label.textContent = `Pratinjau Arsip: ${row.certificate_no} (${row.recipient_name}) - formulir di kiri tetap aman.`;
    }
    if (banner) banner.classList.remove('hidden');
    refreshCertPreview(rowToCertificateRenderPayload(row));
    showCertStatus(
      `Menampilkan pratinjau arsip ${row.certificate_no} (${row.recipient_name}). Formulir yang sedang Anda isi tetap aman.`,
      'info'
    );
  }

  function exitArchivePreviewMode({ skipPreview = false } = {}) {
    state.certPreviewRow = null;
    const banner = document.getElementById('cert-archive-preview-banner');
    if (banner) banner.classList.add('hidden');
    if (!skipPreview) {
      refreshCertPreview();
    }
  }

  function handleCertFormMutated() {
    if (state.certPreviewRow) {
      exitArchivePreviewMode({ skipPreview: true });
    }
    syncCertDirtyUI();
    saveCertDraftToStorage();
  }

  async function submitCertificateForm({ andNext = false } = {}) {
    if (state.certStaffSavedLock) return;
    if (!validateCertificateForm()) return;

    const btnPrimary = document.getElementById('btn-cert-save');
    const btnNext = document.getElementById('btn-cert-save-next');
    if (btnPrimary) btnPrimary.disabled = true;
    if (btnNext) btnNext.disabled = true;

    try {
      let saved;
      if (state.certEditingId) {
        const payload = collectCertFormPayload();
        saved = await invoke('update_certificate', { id: state.certEditingId, payload });
        state.certEditingStatus = saved.status || 'active';
        exitArchivePreviewMode({ skipPreview: true });
        clearCertDraftFromStorage();
        populateCertFormFromRow(saved, true, { skipPreview: true });
        markCertFormClean();
        showCertStatus(`Perubahan sertifikat ${saved.certificate_no} (${saved.recipient_name}) berhasil disimpan.`, 'success');
      } else {
        const payload = collectCertFormPayload(null, { forAutoCreate: !state.certManualNo });
        saved = await invoke('create_certificate', { payload });
        exitArchivePreviewMode({ skipPreview: true });
        clearCertDraftFromStorage();
        if (andNext) {
          applyCertDefaultsToForm({ keepBatchFields: true });
          await refreshNextCertNoIfAuto();
          markCertFormClean();
          showCertStatus(`Sertifikat ${saved.certificate_no} (${saved.recipient_name}) tersimpan. Silakan isi peserta berikutnya.`, 'success');
          document.getElementById('cert-recipient')?.focus();
        } else if (isAdminUser()) {
          populateCertFormFromRow(saved, true, { skipPreview: true });
          markCertFormClean();
          showCertStatus(`Sertifikat ${saved.certificate_no} (${saved.recipient_name}) tersimpan. Anda dapat langsung mengekspor/mencetak atau menyuntingnya.`, 'success');
        } else {
          document.getElementById('cert-no').value = saved.certificate_no;
          document.getElementById('cert-recipient').value = saved.recipient_name;
          state.certManualNo = true;
          state.certStaffSavedLock = true;
          state.certEditingStatus = saved.status || 'active';
          setCertNoLocked(true);
          renderDeptCriteriaPanel();
          updateDivisionDurationBalance();
          markCertFormClean();
          showCertStatus(`Sertifikat ${saved.certificate_no} (${saved.recipient_name}) tersimpan. Silakan ekspor/cetak atau klik Baru / Reset untuk peserta berikutnya.`, 'success');
        }
      }

      await Promise.all([refreshCertificateYears(), loadCertificatesHistory()]);
      await refreshCertPreview();
    } catch (err) {
      showCertStatus(String(err), 'error');
    } finally {
      if (btnPrimary) btnPrimary.disabled = state.certStaffSavedLock;
      if (btnNext) btnNext.disabled = state.certStaffSavedLock;
    }
  }

  async function refreshCertificateYears() {
    const yearSelect = document.getElementById('cert-filter-year');
    if (!yearSelect) return;
    try {
      const years = await invoke('get_certificate_years');
      const prev = state.certYear || 'all';
      yearSelect.innerHTML = '<option value="all">Semua Tahun</option>';
      (years || []).forEach((y) => {
        const opt = document.createElement('option');
        opt.value = y;
        opt.textContent = y;
        if (y === prev) opt.selected = true;
        yearSelect.appendChild(opt);
      });
      if (prev !== 'all' && !(years || []).includes(prev)) {
        state.certYear = 'all';
        yearSelect.value = 'all';
      }
    } catch (err) {
      console.warn('Gagal memuat daftar tahun sertifikat:', err);
    }
  }

  function formatCertUiDate(raw) {
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(raw || '').trim());
    if (!m) return raw || '-';
    return `${m[3]}/${m[2]}/${m[1]}`;
  }

  function renderCertificateHistoryRows() {
    const tbody = document.getElementById('cert-history-body');
    if (!tbody) return;

    const rows = state.certRows || [];
    if (rows.length === 0) {
      const hasFilters = Boolean(state.certSearch) || state.certYear !== 'all' || state.certStatus !== 'all';
      tbody.innerHTML = hasFilters
        ? '<tr><td colspan="8" class="text-center text-muted">Tidak ada sertifikat yang cocok dengan filter pencarian. Ubah kata kunci atau reset filter.</td></tr>'
        : '<tr><td colspan="8" class="text-center text-muted">Belum ada sertifikat tersimpan. Isi formulir di kiri lalu klik Simpan Sertifikat untuk menerbitkan sertifikat pertama.</td></tr>';
      return;
    }

    const adminMode = isAdminUser();
    tbody.innerHTML = rows.map((row) => {
      const isRevoked = row.status === 'revoked';
      const statusPill = isRevoked
        ? '<span class="cert-status-pill is-revoked">Dibatalkan</span>'
        : '<span class="cert-status-pill is-active">Aktif</span>';
      const deptLabel = row.department ? formatDivisionsDisplay(row.department) : '';
      const deptInst = [deptLabel, row.institution].filter(Boolean).join(' / ') || '-';
      const achSubtitle = row.achievement_text
        ? `<div class="cert-ach-preview" tabindex="0" title="${escapeHtml(row.achievement_text)}"><span class="cert-ach-ellipsis">${escapeHtml(row.achievement_text)}</span><span class="cert-ach-tooltip" role="tooltip">${escapeHtml(row.achievement_text)}</span></div>`
        : '';
      const adminButtons = adminMode
        ? `
          <button type="button" class="btn btn-secondary btn-sm cert-row-btn" data-cert-action="edit" data-cert-id="${row.id}">Edit</button>
          <button type="button" class="btn btn-secondary btn-sm cert-row-btn" data-cert-action="toggle-status" data-cert-id="${row.id}">${isRevoked ? 'Aktifkan' : 'Batalkan'}</button>
          <button type="button" class="btn btn-danger btn-sm cert-row-btn" data-cert-action="delete" data-cert-id="${row.id}">Hapus</button>
        `
        : '';

      return `
        <tr>
          <td data-label="Nomor" class="font-mono cert-cell-nowrap"><strong>${escapeHtml(row.certificate_no)}</strong></td>
          <td data-label="Nama Penerima" class="cert-col-recipient"><strong>${escapeHtml(row.recipient_name)}</strong>${achSubtitle}</td>
          <td data-label="Durasi &amp; Periode">${escapeHtml(row.duration_text)}<div class="text-muted cert-cell-nowrap" style="font-size:11px;">${escapeHtml(formatCertUiDate(row.start_date))} s/d ${escapeHtml(formatCertUiDate(row.end_date))}</div></td>
          <td data-label="Tgl Terbit" class="font-mono cert-cell-nowrap">${escapeHtml(formatCertUiDate(row.issued_date))}</td>
          <td data-label="Departemen / Kampus">${escapeHtml(deptInst)}</td>
          <td data-label="Kertas">${escapeHtml(row.paper_size)}</td>
          <td data-label="Status">${statusPill}</td>
          <td data-label="Aksi">
            <div class="cert-row-actions">
              <button type="button" class="btn btn-secondary btn-sm cert-row-btn" data-cert-action="preview" data-cert-id="${row.id}">Pratinjau</button>
              <button type="button" class="btn btn-secondary btn-sm cert-row-btn" data-cert-action="pdf" data-cert-id="${row.id}">PDF</button>
              <button type="button" class="btn btn-secondary btn-sm cert-row-btn" data-cert-action="png" data-cert-id="${row.id}">PNG</button>
              <button type="button" class="btn btn-secondary btn-sm cert-row-btn" data-cert-action="print" data-cert-id="${row.id}">Cetak</button>
              ${adminButtons}
            </div>
          </td>
        </tr>
      `;
    }).join('');
  }

  async function loadCertificatesHistory() {
    const tbody = document.getElementById('cert-history-body');
    const summaryCounts = document.getElementById('cert-summary-counts');
    const metaEl = document.getElementById('cert-history-meta');
    const pageInfo = document.getElementById('cert-page-info');
    const btnPrev = document.getElementById('btn-cert-prev');
    const btnNext = document.getElementById('btn-cert-next');

    if (tbody && !state.certRows.length) {
      tbody.innerHTML = '<tr><td colspan="8" class="text-center text-muted">Memuat riwayat sertifikat dari database...</td></tr>';
    }

    try {
      const res = await invoke('get_certificates', {
        year: state.certYear,
        status: state.certStatus,
        search: state.certSearch,
        page: state.certPage,
        pageSize: state.certPageSize,
      });

      state.certRows = res.items || [];
      state.certPage = res.page || 1;
      state.certTotalPages = res.total_pages || 1;

      renderCertificateHistoryRows();

      if (summaryCounts) {
        summaryCounts.textContent = `${res.total_count} Tersimpan (${res.active_count} Aktif, ${res.revoked_count} Dibatalkan)`;
      }
      if (metaEl) {
        metaEl.textContent = `Menampilkan ${state.certRows.length} dari ${res.total_count} data sertifikat`;
      }
      if (pageInfo) {
        pageInfo.textContent = `Halaman ${res.page} dari ${res.total_pages} (${res.total_count} data)`;
      }
      if (btnPrev) btnPrev.disabled = res.page <= 1;
      if (btnNext) btnNext.disabled = res.page >= res.total_pages;
    } catch (err) {
      if (tbody) {
        tbody.innerHTML = `<tr><td colspan="8" class="text-center text-warning">Gagal memuat riwayat sertifikat: ${escapeHtml(err)}. Tekan Cari untuk mencoba lagi.</td></tr>`;
      }
      showCertStatus(`Gagal memuat riwayat sertifikat: ${err}`, 'error');
    }
  }

  function rowToCertificateRenderPayload(row) {
    return {
      certificate_no: row.certificate_no,
      recipient_name: row.recipient_name,
      duration_text: row.duration_text,
      program_name: row.program_name,
      start_date: row.start_date,
      end_date: row.end_date,
      issued_date: row.issued_date,
      issued_city: row.issued_city,
      body_template: row.body_template,
      department: row.department,
      achievement_text: row.achievement_text,
      signer_name: row.signer_name,
      signer_title: row.signer_title,
      paper_size: row.paper_size,
      status: row.status,
    };
  }

  async function exportCertificateFormat(format, row = null) {
    const payload = row
      ? rowToCertificateRenderPayload(row)
      : collectCertFormPayload();

    const cmd = format === 'png' ? 'export_certificate_png' : 'export_certificate_pdf';
    try {
      const msg = await invoke(cmd, { payload });
      const unsavedDraft = !row && isCertFormDirty();
      const cancelled = !msg || String(msg).toLowerCase().includes('dibatalkan');
      if (unsavedDraft) {
        showCertStatus(
          cancelled
            ? 'Ekspor dibatalkan. Pengingat: sertifikat pada formulir belum disimpan ke database.'
            : `${msg} (Pengingat: sertifikat pada formulir belum disimpan ke database.)`,
          'warning'
        );
      } else if (msg && !cancelled) {
        showCertStatus(msg, 'success');
      } else if (msg) {
        showCertStatus(msg, 'info');
      }
    } catch (err) {
      showCertStatus(`Gagal mengekspor sertifikat: ${err}`, 'error');
    }
  }

  let certPrintFallbackCleanup = null;

  function cleanupCertificatePrintMode() {
    const dynStyle = document.getElementById('cert-dynamic-page-style');
    const isActive =
      document.documentElement.classList.contains('printing-certificate') ||
      document.body.classList.contains('printing-certificate') ||
      Boolean(dynStyle) ||
      Boolean(certPrintFallbackCleanup);
    if (!isActive) return;

    if (certPrintFallbackCleanup) {
      const dispose = certPrintFallbackCleanup;
      certPrintFallbackCleanup = null;
      dispose();
    }

    document.documentElement.classList.remove('printing-certificate');
    document.body.classList.remove('printing-certificate');
    if (dynStyle) dynStyle.remove();
    const stage = document.getElementById('cert-print-stage');
    if (stage) stage.innerHTML = '';
    document.title = APP_DEFAULT_TITLE;
    if (isTauri) {
      invoke('set_window_title', { title: APP_DEFAULT_TITLE }).catch(() => {});
    }
  }

  function resetCertificatesSessionState() {
    clearTimeout(state.certPreviewTimer);
    state.certPreviewSeq += 1;
    state.certInitialized = false;
    state.certEditingId = null;
    state.certEditingStatus = 'active';
    state.certManualNo = false;
    state.certManualDuration = false;
    state.certManualDates = false;
    state.certStaffSavedLock = false;
    state.certSettings = null;
    state.certDepartments = [];
    state.certProgramTypes = [];
    state.certRows = [];
    state.certPage = 1;
    state.certTotalPages = 1;
    state.certYear = 'all';
    state.certStatus = 'all';
    state.certSearch = '';
    state.certCleanSnapshot = '';
    state.certPreviewRow = null;

    cleanupCertificatePrintMode();

    const dirtyBadge = document.getElementById('cert-dirty-badge');
    if (dirtyBadge) dirtyBadge.classList.add('hidden');
    const previewBanner = document.getElementById('cert-archive-preview-banner');
    if (previewBanner) previewBanner.classList.add('hidden');
    const criteriaPanel = document.getElementById('cert-dept-criteria-panel');
    if (criteriaPanel) criteriaPanel.classList.add('hidden');
    const balanceEl = document.getElementById('cert-divisions-balance');
    if (balanceEl) {
      balanceEl.className = 'cert-field-hint hidden';
      balanceEl.textContent = '';
    }

    const settingsDialog = document.getElementById('dialog-cert-settings');
    if (settingsDialog?.open) settingsDialog.close();
    const catalogDialog = document.getElementById('dialog-cert-catalog');
    if (catalogDialog?.open) catalogDialog.close();

    if (document.getElementById('form-certificate')) {
      applyCertDefaultsToForm({ keepBatchFields: false });
      const noInput = document.getElementById('cert-no');
      if (noInput) noInput.value = '';
    }
    const previewContainer = document.getElementById('cert-preview-container');
    if (previewContainer) previewContainer.innerHTML = '';
    const historyBody = document.getElementById('cert-history-body');
    if (historyBody) historyBody.innerHTML = '';
    const summaryCounts = document.getElementById('cert-summary-counts');
    if (summaryCounts) summaryCounts.textContent = '0 Tersimpan';
    const filterSearch = document.getElementById('cert-filter-search');
    if (filterSearch) filterSearch.value = '';
    const filterYear = document.getElementById('cert-filter-year');
    if (filterYear) filterYear.innerHTML = '<option value="all">Semua Tahun</option>';
    const filterStatus = document.getElementById('cert-filter-status');
    if (filterStatus) filterStatus.value = 'all';
    showCertStatus('');
  }

  async function printCertificateDocument(row = null) {
    const payload = row
      ? rowToCertificateRenderPayload(row)
      : collectCertFormPayload();

    try {
      const svg = await invoke('preview_certificate_svg', { payload });
      const stage = document.getElementById('cert-print-stage');
      if (!stage) return;
      stage.innerHTML = svg;

      const existingStyle = document.getElementById('cert-dynamic-page-style');
      if (existingStyle) existingStyle.remove();
      const pageStyle = document.createElement('style');
      pageStyle.id = 'cert-dynamic-page-style';
      const isA4 = String(payload.paper_size || 'F4').toUpperCase() === 'A4';
      pageStyle.textContent = isA4
        ? '@media print { @page { size: 297mm 210mm; margin: 0; } }'
        : '@media print { @page { size: 330mm 215mm; margin: 0; } }';
      document.head.appendChild(pageStyle);

      document.documentElement.classList.add('printing-certificate');
      document.body.classList.add('printing-certificate');
      const cleanName = `Certificate_${getCleanReportFileName(payload.certificate_no)}_${getCleanReportFileName(payload.recipient_name)}`;
      document.title = cleanName;

      if (certPrintFallbackCleanup) {
        certPrintFallbackCleanup();
        certPrintFallbackCleanup = null;
      }
      const mql = window.matchMedia ? window.matchMedia('print') : null;
      const onMediaChange = (e) => {
        if (!e.matches) cleanupCertificatePrintMode();
      };
      const onWindowFocus = () => {
        cleanupCertificatePrintMode();
      };
      let focusTimer = null;
      if (mql?.addEventListener) {
        mql.addEventListener('change', onMediaChange);
      }
      certPrintFallbackCleanup = () => {
        if (focusTimer) {
          clearTimeout(focusTimer);
          focusTimer = null;
        }
        window.removeEventListener('focus', onWindowFocus);
        if (mql?.removeEventListener) {
          mql.removeEventListener('change', onMediaChange);
        }
      };

      if (isTauri) {
        await invoke('print_certificate', { title: cleanName });
      } else {
        window.print();
      }

      if (!row && isCertFormDirty()) {
        showCertStatus(
          'Dialog cetak dibuka. Pengingat: sertifikat pada formulir belum disimpan ke database - klik Simpan Sertifikat setelah selesai.',
          'warning'
        );
      }

      if (certPrintFallbackCleanup) {
        focusTimer = setTimeout(() => {
          if (document.body.classList.contains('printing-certificate')) {
            window.addEventListener('focus', onWindowFocus, { once: true });
          }
        }, 300);
      }
    } catch (err) {
      cleanupCertificatePrintMode();
      showCertStatus(`Gagal mencetak sertifikat: ${err}`, 'error');
    }
  }

  function applyCertSettingsToDialog(cfg) {
    state.certSettings = cfg;
    document.getElementById('cfg-cert-signer').value = cfg.signer_name || 'Agus Sumardi Wijaya';
    document.getElementById('cfg-cert-title').value = cfg.signer_title || 'General Manager';
    document.getElementById('cfg-cert-program').value = cfg.program_name || 'hospitality training program';
    document.getElementById('cfg-cert-paper').value = cfg.paper_size === 'A4' ? 'A4' : 'F4';
    document.getElementById('cfg-cert-city').value = cfg.issued_city || DEFAULT_CERT_ISSUED_CITY;
    document.getElementById('cfg-cert-template').value = cfg.body_template || DEFAULT_CERT_BODY_TEMPLATE;
    document.getElementById('cfg-cert-show-sig').checked = Boolean(cfg.show_signature);

    const logoPreview = document.getElementById('cfg-cert-logo-preview');
    if (logoPreview && cfg.logo_data_uri) logoPreview.src = cfg.logo_data_uri;

    const sigPreview = document.getElementById('cfg-cert-sig-preview');
    const sigEmpty = document.getElementById('cfg-cert-sig-empty');
    if (cfg.signature_data_uri) {
      if (sigPreview) {
        sigPreview.src = cfg.signature_data_uri;
        sigPreview.classList.remove('hidden');
      }
      if (sigEmpty) sigEmpty.classList.add('hidden');
    } else {
      if (sigPreview) {
        sigPreview.removeAttribute('src');
        sigPreview.classList.add('hidden');
      }
      if (sigEmpty) sigEmpty.classList.remove('hidden');
    }
  }

  function shiftDateByMonthsIso(endIso, months) {
    const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(String(endIso || '').trim());
    if (!m) return getMonthsAgoLocalDateStr(months);
    const y = parseInt(m[1], 10);
    const mon = parseInt(m[2], 10) - 1;
    const day = parseInt(m[3], 10);
    const delta = Math.max(1, Number(months) || 3);
    const firstTarget = new Date(y, mon - delta, 1);
    const maxDay = new Date(firstTarget.getFullYear(), firstTarget.getMonth() + 1, 0).getDate();
    const clamped = Math.min(day, maxDay);
    const yyyy = firstTarget.getFullYear();
    const mm = String(firstTarget.getMonth() + 1).padStart(2, '0');
    const dd = String(clamped).padStart(2, '0');
    return `${yyyy}-${mm}-${dd}`;
  }

  async function reloadCertCatalogData() {
    const [departments, programTypes] = await Promise.all([
      invoke('get_departments'),
      invoke('get_program_types'),
    ]);
    state.certDepartments = departments || [];
    state.certProgramTypes = programTypes || [];
  }

  async function loadCertificatesModuleView() {
    if (!state.certInitialized) {
      try {
        const [cfg, departments, programTypes] = await Promise.all([
          invoke('get_certificate_settings'),
          invoke('get_departments'),
          invoke('get_program_types'),
        ]);
        state.certSettings = cfg;
        state.certDepartments = departments || [];
        state.certProgramTypes = programTypes || [];
      } catch (err) {
        console.warn('Gagal memuat pengaturan/katalog sertifikat:', err);
      }
      applyCertDefaultsToForm({ keepBatchFields: false });
      await refreshNextCertNoIfAuto();
      state.certInitialized = true;
      markCertFormClean();

      const restoredDraft = loadCertDraftFromStorage();
      if (restoredDraft) {
        applyCertDraftToForm(restoredDraft);
        if (!state.certManualNo && !state.certEditingId) {
          await refreshNextCertNoIfAuto();
        }
        showCertStatus(
          'Draf sertifikat yang belum disimpan berhasil dipulihkan. Klik Simpan Sertifikat untuk menyimpan ke database, atau Baru / Reset untuk membuangnya.',
          'warning'
        );
      }
    }
    await Promise.all([
      refreshCertificateYears(),
      loadCertificatesHistory(),
      refreshCertPreview(state.certPreviewRow ? rowToCertificateRenderPayload(state.certPreviewRow) : null),
    ]);
  }

  function initCertCatalogDialog() {
    const dialog = document.getElementById('dialog-cert-catalog');
    const btnOpen = document.getElementById('btn-cert-catalog');
    if (!dialog || !btnOpen) return;

    const tabDept = document.getElementById('tab-catalog-dept');
    const tabProg = document.getElementById('tab-catalog-prog');
    const panelDept = document.getElementById('panel-catalog-dept');
    const panelProg = document.getElementById('panel-catalog-prog');
    const statusEl = document.getElementById('cert-catalog-status');
    const confirmBox = document.getElementById('cert-catalog-confirm');
    const confirmTextEl = document.getElementById('cert-catalog-confirm-text');
    let pendingDelete = null;

    const showCatalogStatus = (msg = '', type = 'info') => {
      if (!statusEl) return;
      if (!msg) {
        statusEl.className = 'cert-status-line info hidden';
        statusEl.textContent = '';
        return;
      }
      statusEl.className = `cert-status-line ${type}`;
      statusEl.textContent = msg;
    };

    const clearPendingDelete = () => {
      pendingDelete = null;
      if (confirmBox) confirmBox.classList.add('hidden');
      if (confirmTextEl) confirmTextEl.textContent = '';
    };

    const askInlineDelete = (kind, id, label) => {
      pendingDelete = { kind, id, label };
      if (confirmTextEl) {
        confirmTextEl.textContent = `Hapus permanen "${label}" dari database?`;
      }
      if (confirmBox) confirmBox.classList.remove('hidden');
    };

    const switchTab = (tabName) => {
      const isDept = tabName === 'dept';
      clearPendingDelete();
      tabDept?.classList.toggle('is-active', isDept);
      tabProg?.classList.toggle('is-active', !isDept);
      tabDept?.setAttribute('aria-selected', isDept ? 'true' : 'false');
      tabProg?.setAttribute('aria-selected', isDept ? 'false' : 'true');
      panelDept?.classList.toggle('hidden', !isDept);
      panelProg?.classList.toggle('hidden', isDept);
    };

    const resetDeptForm = () => {
      document.getElementById('cat-dept-id').value = '';
      document.getElementById('cat-dept-code').value = '';
      document.getElementById('cat-dept-name').value = '';
      document.getElementById('cat-dept-desc').value = '';
      document.getElementById('cat-dept-comp').value = '';
    };

    const resetProgForm = () => {
      document.getElementById('cat-prog-id').value = '';
      document.getElementById('cat-prog-code').value = '';
      document.getElementById('cat-prog-name').value = '';
      document.getElementById('cat-prog-months').value = '3';
      document.getElementById('cat-prog-desc').value = '';
    };

    const renderCatalogLists = () => {
      const deptList = document.getElementById('catalog-dept-list');
      if (deptList) {
        deptList.innerHTML = (state.certDepartments || [])
          .map(
            (d) => `
          <div class="cert-catalog-item">
            <div class="cert-catalog-item-main">
              <div class="cert-catalog-item-title">${escapeHtml(d.code)} &mdash; ${escapeHtml(d.name)}</div>
              <div class="cert-catalog-item-sub">${escapeHtml(d.program_description || '-')}</div>
            </div>
            <div class="cert-catalog-item-actions">
              <button type="button" class="btn btn-secondary btn-sm" data-cat-action="edit-dept" data-cat-id="${d.id}">Edit</button>
              <button type="button" class="btn btn-danger btn-sm" data-cat-action="del-dept" data-cat-id="${d.id}">Hapus</button>
            </div>
          </div>
        `
          )
          .join('') || '<div class="text-muted text-center" style="padding:12px;">Belum ada departemen.</div>';
      }

      const progList = document.getElementById('catalog-prog-list');
      if (progList) {
        progList.innerHTML = (state.certProgramTypes || [])
          .map(
            (p) => `
          <div class="cert-catalog-item">
            <div class="cert-catalog-item-main">
              <div class="cert-catalog-item-title">${escapeHtml(p.code)} &mdash; ${escapeHtml(p.name)} (${p.default_duration_months} bln)</div>
              <div class="cert-catalog-item-sub">${escapeHtml(p.description || '-')}</div>
            </div>
            <div class="cert-catalog-item-actions">
              <button type="button" class="btn btn-secondary btn-sm" data-cat-action="edit-prog" data-cat-id="${p.id}">Edit</button>
              <button type="button" class="btn btn-danger btn-sm" data-cat-action="del-prog" data-cat-id="${p.id}">Hapus</button>
            </div>
          </div>
        `
          )
          .join('') || '<div class="text-muted text-center" style="padding:12px;">Belum ada jenis program.</div>';
      }
    };

    const syncMainFormFromCatalog = async () => {
      const wasDirty = isCertFormDirty();
      const currentRows = readDivisionRowsFromDom();
      renderDivisionRows(currentRows);
      const curProgCode = document.getElementById('cert-program-type')?.value || '';
      populateProgramTypeSelect(curProgCode || '__custom__');
      await refreshNextCertNoIfAuto();
      if (!wasDirty) markCertFormClean();
      else saveCertDraftToStorage();
      scheduleCertPreview(true);
    };

    btnOpen.addEventListener('click', async () => {
      try {
        await reloadCertCatalogData();
        renderCatalogLists();
        resetDeptForm();
        resetProgForm();
        clearPendingDelete();
        showCatalogStatus('');
        switchTab('dept');
        dialog.showModal();
      } catch (err) {
        showCertStatus(`Gagal membuka katalog departemen: ${err}`, 'error');
      }
    });

    document.getElementById('btn-close-cert-catalog')?.addEventListener('click', () => {
      clearPendingDelete();
      dialog.close();
    });
    tabDept?.addEventListener('click', () => switchTab('dept'));
    tabProg?.addEventListener('click', () => switchTab('prog'));
    document.getElementById('btn-cat-dept-new')?.addEventListener('click', () => {
      clearPendingDelete();
      resetDeptForm();
      document.getElementById('cat-dept-code')?.focus();
    });
    document.getElementById('btn-cat-prog-new')?.addEventListener('click', () => {
      clearPendingDelete();
      resetProgForm();
      document.getElementById('cat-prog-code')?.focus();
    });
    document.getElementById('btn-cat-confirm-no')?.addEventListener('click', clearPendingDelete);

    document.getElementById('btn-cat-confirm-yes')?.addEventListener('click', async () => {
      if (!pendingDelete) return;
      const { kind, id, label } = pendingDelete;
      clearPendingDelete();
      try {
        if (kind === 'dept') {
          await invoke('delete_department', { id });
          await reloadCertCatalogData();
          renderCatalogLists();
          resetDeptForm();
          await syncMainFormFromCatalog();
          showCatalogStatus(`Departemen "${label}" dihapus.`, 'success');
        } else {
          await invoke('delete_program_type', { id });
          await reloadCertCatalogData();
          renderCatalogLists();
          resetProgForm();
          await syncMainFormFromCatalog();
          showCatalogStatus(`Jenis program "${label}" dihapus.`, 'success');
        }
      } catch (err) {
        showCatalogStatus(String(err), 'error');
      }
    });

    dialog.addEventListener('click', (e) => {
      const btn = e.target.closest('[data-cat-action]');
      if (!btn) return;
      const action = btn.dataset.catAction;
      const id = Number(btn.dataset.catId);

      if (action === 'edit-dept') {
        clearPendingDelete();
        const d = (state.certDepartments || []).find((x) => x.id === id);
        if (!d) return;
        document.getElementById('cat-dept-id').value = String(d.id);
        document.getElementById('cat-dept-code').value = d.code || '';
        document.getElementById('cat-dept-name').value = d.name || '';
        document.getElementById('cat-dept-desc').value = d.program_description || '';
        document.getElementById('cat-dept-comp').value = d.competencies || '';
        document.getElementById('cat-dept-name')?.focus();
      } else if (action === 'del-dept') {
        const d = (state.certDepartments || []).find((x) => x.id === id);
        if (!d) return;
        askInlineDelete('dept', id, `${d.code} - ${d.name}`);
      } else if (action === 'edit-prog') {
        clearPendingDelete();
        const p = (state.certProgramTypes || []).find((x) => x.id === id);
        if (!p) return;
        document.getElementById('cat-prog-id').value = String(p.id);
        document.getElementById('cat-prog-code').value = p.code || '';
        document.getElementById('cat-prog-name').value = p.name || '';
        document.getElementById('cat-prog-months').value = String(p.default_duration_months || 3);
        document.getElementById('cat-prog-desc').value = p.description || '';
        document.getElementById('cat-prog-name')?.focus();
      } else if (action === 'del-prog') {
        const p = (state.certProgramTypes || []).find((x) => x.id === id);
        if (!p) return;
        askInlineDelete('prog', id, `${p.code} - ${p.name}`);
      }
    });

    document.getElementById('form-catalog-dept')?.addEventListener('submit', async (e) => {
      e.preventDefault();
      clearPendingDelete();
      const idRaw = document.getElementById('cat-dept-id').value.trim();
      const codeEl = document.getElementById('cat-dept-code');
      const nameEl = document.getElementById('cat-dept-name');
      const code = codeEl.value.trim();
      const name = nameEl.value.trim();
      if (!code || !name) {
        showCatalogStatus('Kode dan nama departemen wajib diisi.', 'error');
        (!code ? codeEl : nameEl)?.focus();
        return;
      }
      try {
        await invoke('save_department', {
          payload: {
            id: idRaw ? Number(idRaw) : null,
            code,
            name,
            program_description: document.getElementById('cat-dept-desc').value.trim(),
            competencies: document.getElementById('cat-dept-comp').value.trim(),
          },
        });
        await reloadCertCatalogData();
        renderCatalogLists();
        resetDeptForm();
        await syncMainFormFromCatalog();
        showCatalogStatus(`Departemen ${code.toUpperCase()} (${name}) tersimpan.`, 'success');
      } catch (err) {
        showCatalogStatus(String(err), 'error');
      }
    });

    document.getElementById('form-catalog-prog')?.addEventListener('submit', async (e) => {
      e.preventDefault();
      clearPendingDelete();
      const idRaw = document.getElementById('cat-prog-id').value.trim();
      const codeEl = document.getElementById('cat-prog-code');
      const nameEl = document.getElementById('cat-prog-name');
      const code = codeEl.value.trim();
      const name = nameEl.value.trim();
      const months = Number(document.getElementById('cat-prog-months').value) || 3;
      if (!code || !name) {
        showCatalogStatus('Kode dan frasa jenis program wajib diisi.', 'error');
        (!code ? codeEl : nameEl)?.focus();
        return;
      }
      try {
        await invoke('save_program_type', {
          payload: {
            id: idRaw ? Number(idRaw) : null,
            code,
            name,
            default_duration_months: months,
            description: document.getElementById('cat-prog-desc').value.trim(),
          },
        });
        await reloadCertCatalogData();
        renderCatalogLists();
        resetProgForm();
        await syncMainFormFromCatalog();
        showCatalogStatus(`Jenis program ${code.toUpperCase()} tersimpan.`, 'success');
      } catch (err) {
        showCatalogStatus(String(err), 'error');
      }
    });
  }

  function initCertificatesModule() {
    const form = document.getElementById('form-certificate');
    if (!form) return;

    const startInput = document.getElementById('cert-start');
    const endInput = document.getElementById('cert-end');
    const durationInput = document.getElementById('cert-duration');
    const issuedInput = document.getElementById('cert-issued');
    const noInput = document.getElementById('cert-no');

    // Sync initial aria-invalid attributes
    form.querySelectorAll('.cert-field-control').forEach((el) => {
      if (!el.hasAttribute('aria-invalid')) el.setAttribute('aria-invalid', 'false');
    });

    noInput?.addEventListener('input', () => {
      if (noInput.readOnly) return;
      state.certManualNo = Boolean(noInput.value.trim());
    });
    durationInput?.addEventListener('input', () => {
      state.certManualDuration = Boolean(durationInput.value.trim());
      updateDivisionDurationBalance();
    });

    form.addEventListener('input', (e) => {
      if (!state.certEditingId && state.certEditingStatus !== 'active') {
        state.certEditingStatus = 'active';
      }
      if (e.target.hasAttribute?.('aria-invalid')) syncCertFieldAria(e.target, false);
      if (e.target.classList?.contains('cert-div-duration')) {
        updateDivisionDurationBalance();
      }
      handleCertFormMutated();
      scheduleCertPreview(false);
    });

    form.addEventListener('change', async (e) => {
      const t = e.target;
      if (!state.certEditingId && state.certEditingStatus !== 'active') {
        state.certEditingStatus = 'active';
      }

      if (t.id === 'cert-start' || t.id === 'cert-end') {
        state.certManualDates = true;
        validateCertEndDateCustomValidity();
        if (!state.certManualDuration && !state.certEditingId && startInput?.value && endInput?.value) {
          durationInput.value = suggestEnglishDuration(startInput.value, endInput.value);
          syncCertFieldAria(durationInput, false);
        }
      } else if (t.id === 'cert-program-type') {
        const pt = (state.certProgramTypes || []).find((p) => p.code === t.value);
        const programInput = document.getElementById('cert-program');
        if (pt && programInput) {
          programInput.value = pt.name;
          if (!state.certEditingId && !state.certManualDates && endInput?.value) {
            startInput.value = pt.code.toUpperCase() === 'WS'
              ? endInput.value
              : shiftDateByMonthsIso(endInput.value, pt.default_duration_months);
            if (!state.certManualDuration) {
              durationInput.value = suggestEnglishDuration(startInput.value, endInput.value);
            }
          }
        }
      } else if (t.id === 'cert-issued') {
        await refreshNextCertNoIfAuto();
      } else if (t.classList?.contains('cert-div-name')) {
        renderDeptCriteriaPanel();
        await refreshNextCertNoIfAuto();
      } else if (t.id === 'cert-achievement-preset') {
        const achInput = document.getElementById('cert-achievement');
        if (achInput && t.value !== '__custom__') {
          achInput.value = t.value;
        }
      }

      updateDivisionDurationBalance();
      syncCertFieldAria(t, false);
      handleCertFormMutated();
      scheduleCertPreview(true);
    });

    form.addEventListener('blur', (e) => {
      syncCertFieldAria(e.target, false);
    }, true);

    form.addEventListener('submit', async (e) => {
      e.preventDefault();
      await submitCertificateForm({ andNext: false });
    });

    document.getElementById('btn-cert-add-division')?.addEventListener('click', () => {
      const rows = readDivisionRowsFromDom();
      if (rows.length >= CERT_MAX_DIVISIONS) return;
      rows.push({ name: '', duration: '' });
      renderDivisionRows(rows);
      const selects = document.querySelectorAll('#cert-divisions-list .cert-div-name');
      selects[selects.length - 1]?.focus();
      handleCertFormMutated();
      scheduleCertPreview(false);
    });

    document.getElementById('cert-divisions-list')?.addEventListener('click', async (e) => {
      const removeBtn = e.target.closest('[data-remove-div]');
      if (!removeBtn) return;
      const idx = Number(removeBtn.dataset.removeDiv);
      const rows = readDivisionRowsFromDom();
      if (rows.length <= 1) {
        renderDivisionRows([{ name: '', duration: '' }]);
      } else {
        rows.splice(idx, 1);
        renderDivisionRows(rows);
      }
      await refreshNextCertNoIfAuto();
      handleCertFormMutated();
      scheduleCertPreview(true);
    });

    document.getElementById('btn-apply-dept-desc')?.addEventListener('click', () => {
      const desc = document.getElementById('cert-dept-criteria-desc')?.textContent?.trim();
      const achInput = document.getElementById('cert-achievement');
      if (!desc || !achInput) return;
      achInput.value = /[.!?]$/.test(desc) ? desc : `${desc}.`;
      syncAchievementPresetSelect();
      handleCertFormMutated();
      scheduleCertPreview(true);
    });

    document.getElementById('btn-apply-dept-comp')?.addEventListener('click', () => {
      const boxes = Array.from(document.querySelectorAll('#cert-dept-competencies-list .cert-comp-checkbox'));
      if (!boxes.length) return;
      let picked = boxes.filter((b) => b.checked).map((b) => b.value.trim()).filter(Boolean);
      if (!picked.length) picked = boxes.map((b) => b.value.trim()).filter(Boolean);
      const achInput = document.getElementById('cert-achievement');
      if (!achInput || !picked.length) return;
      achInput.value = formatCompetencySentence(picked);
      syncAchievementPresetSelect();
      handleCertFormMutated();
      scheduleCertPreview(true);
    });

    document.getElementById('cert-achievement')?.addEventListener('input', () => {
      syncAchievementPresetSelect();
    });

    document.getElementById('btn-cert-save-next')?.addEventListener('click', async () => {
      await submitCertificateForm({ andNext: true });
    });

    document.getElementById('btn-cert-auto-no')?.addEventListener('click', async () => {
      if (state.certEditingId || noInput?.readOnly) return;
      state.certManualNo = false;
      const { year, month } = getIssuedYearAndMonth();
      const departmentCode = getSelectedDepartmentCode();
      try {
        noInput.value = await invoke('get_next_certificate_no', {
          year,
          month,
          departmentCode,
        });
        syncCertFieldAria(noInput, false);
        handleCertFormMutated();
        scheduleCertPreview(true);
      } catch (err) {
        showCertStatus(String(err), 'error');
      }
    });

    const resetToNew = async () => {
      if (isCertFormDirty()) {
        const discard = await appConfirm(
          'Draf sertifikat saat ini belum disimpan ke database. Buang perubahan dan mulai formulir baru?',
          'Buang Draf Sertifikat',
          'warning',
          'Buang Draf',
          'Batal'
        );
        if (!discard) return;
      }
      clearCertDraftFromStorage();
      exitArchivePreviewMode({ skipPreview: true });
      applyCertDefaultsToForm({ keepBatchFields: false });
      await refreshNextCertNoIfAuto();
      markCertFormClean();
      showCertStatus('');
      scheduleCertPreview(true);
      document.getElementById('cert-recipient')?.focus();
    };
    document.getElementById('btn-cert-reset')?.addEventListener('click', resetToNew);
    document.getElementById('btn-cert-cancel-edit')?.addEventListener('click', resetToNew);

    document.getElementById('btn-cert-back-to-draft')?.addEventListener('click', () => {
      exitArchivePreviewMode();
      showCertStatus('Kembali ke pratinjau formulir aktif.', 'info');
    });

    document.getElementById('btn-cert-export-pdf')?.addEventListener('click', () => exportCertificateFormat('pdf', state.certPreviewRow));
    document.getElementById('btn-cert-export-png')?.addEventListener('click', () => exportCertificateFormat('png', state.certPreviewRow));
    document.getElementById('btn-cert-print')?.addEventListener('click', () => printCertificateDocument(state.certPreviewRow));

    const persistDirtyDraftOnExit = () => {
      if (isCertFormDirty()) saveCertDraftToStorage();
    };
    window.addEventListener('pagehide', persistDirtyDraftOnExit);
    window.addEventListener('beforeunload', persistDirtyDraftOnExit);

    // History Filters & Pagination
    const filterForm = document.getElementById('form-cert-filter');
    filterForm?.addEventListener('submit', (e) => {
      e.preventDefault();
      state.certSearch = document.getElementById('cert-filter-search')?.value?.trim() || '';
      state.certYear = document.getElementById('cert-filter-year')?.value || 'all';
      state.certStatus = document.getElementById('cert-filter-status')?.value || 'all';
      state.certPage = 1;
      loadCertificatesHistory();
    });

    document.getElementById('cert-filter-year')?.addEventListener('change', () => {
      state.certYear = document.getElementById('cert-filter-year').value;
      state.certPage = 1;
      loadCertificatesHistory();
    });
    document.getElementById('cert-filter-status')?.addEventListener('change', () => {
      state.certStatus = document.getElementById('cert-filter-status').value;
      state.certPage = 1;
      loadCertificatesHistory();
    });
    document.getElementById('cert-page-size')?.addEventListener('change', () => {
      state.certPageSize = Number(document.getElementById('cert-page-size').value) || 25;
      state.certPage = 1;
      loadCertificatesHistory();
    });
    document.getElementById('btn-cert-prev')?.addEventListener('click', () => {
      if (state.certPage > 1) {
        state.certPage--;
        loadCertificatesHistory();
      }
    });
    document.getElementById('btn-cert-next')?.addEventListener('click', () => {
      if (state.certPage < state.certTotalPages) {
        state.certPage++;
        loadCertificatesHistory();
      }
    });

    // Row actions delegation
    document.getElementById('cert-history-body')?.addEventListener('click', async (e) => {
      const btn = e.target.closest('[data-cert-action]');
      if (!btn) return;
      const action = btn.dataset.certAction;
      const id = Number(btn.dataset.certId);
      const row = state.certRows.find((r) => r.id === id);
      if (!row) return;

      if (action === 'preview') {
        enterArchivePreviewMode(row);
      } else if (action === 'pdf') {
        await exportCertificateFormat('pdf', row);
      } else if (action === 'png') {
        await exportCertificateFormat('png', row);
      } else if (action === 'print') {
        await printCertificateDocument(row);
      } else if (action === 'edit') {
        if (isCertFormDirty()) {
          const replace = await appConfirm(
            'Formulir saat ini memiliki perubahan sertifikat yang belum disimpan ke database. Muat data lain untuk diedit dan buang draf saat ini?',
            'Konfirmasi Ganti Formulir',
            'warning',
            'Ya, Timpa',
            'Batal'
          );
          if (!replace) return;
        }
        clearCertDraftFromStorage();
        exitArchivePreviewMode({ skipPreview: true });
        populateCertFormFromRow(row, true);
        markCertFormClean();
      } else if (action === 'toggle-status') {
        const nextStatus = row.status === 'revoked' ? 'active' : 'revoked';
        const prompt = nextStatus === 'revoked'
          ? `Batalkan sertifikat ${row.certificate_no} atas nama ${row.recipient_name}? Sertifikat yang dibatalkan akan diberi tanda REVOKED.`
          : `Aktifkan kembali sertifikat ${row.certificate_no} atas nama ${row.recipient_name}?`;
        if (!(await appConfirm(prompt, 'Konfirmasi Status Sertifikat'))) return;
        try {
          const updated = await invoke('set_certificate_status', { id: row.id, status: nextStatus });
          await loadCertificatesHistory();
          if (state.certPreviewRow?.id === row.id) {
            state.certPreviewRow = updated;
            await refreshCertPreview(rowToCertificateRenderPayload(updated));
          } else if (state.certEditingId === row.id) {
            state.certEditingStatus = updated.status;
            await refreshCertPreview();
          }
          showCertStatus(`Status sertifikat ${row.certificate_no} diubah menjadi ${nextStatus === 'revoked' ? 'Dibatalkan' : 'Aktif'}.`, 'success');
        } catch (err) {
          showCertStatus(String(err), 'error');
        }
      } else if (action === 'delete') {
        const ok = await appConfirm(
          `Hapus permanen data sertifikat ${row.certificate_no} (${row.recipient_name}) dari database?`,
          'Konfirmasi Hapus Sertifikat',
          'error',
          'Hapus',
          'Batal'
        );
        if (!ok) return;
        try {
          await invoke('delete_certificate', { id: row.id });
          if (state.certEditingId === row.id) {
            clearCertDraftFromStorage();
            applyCertDefaultsToForm({ keepBatchFields: false });
            await refreshNextCertNoIfAuto();
            markCertFormClean();
          }
          if (state.certPreviewRow?.id === row.id) {
            exitArchivePreviewMode({ skipPreview: true });
          }
          await Promise.all([
            refreshCertificateYears(),
            loadCertificatesHistory(),
            refreshCertPreview(state.certPreviewRow ? rowToCertificateRenderPayload(state.certPreviewRow) : null),
          ]);
          showCertStatus(`Sertifikat ${row.certificate_no} dihapus dari database.`, 'success');
        } catch (err) {
          showCertStatus(String(err), 'error');
        }
      }
    });

    // Admin Settings <dialog>
    const settingsDialog = document.getElementById('dialog-cert-settings');
    const settingsForm = document.getElementById('form-cert-settings');

    document.getElementById('btn-cert-settings')?.addEventListener('click', async () => {
      try {
        const cfg = await invoke('get_certificate_settings');
        applyCertSettingsToDialog(cfg);
        settingsDialog?.showModal();
      } catch (err) {
        showCertStatus(`Gagal membuka pengaturan sertifikat: ${err}`, 'error');
      }
    });

    document.getElementById('btn-close-cert-settings')?.addEventListener('click', () => settingsDialog?.close());
    document.getElementById('btn-cancel-cert-settings')?.addEventListener('click', () => settingsDialog?.close());
    document.getElementById('btn-reset-cert-template')?.addEventListener('click', () => {
      document.getElementById('cfg-cert-template').value = DEFAULT_CERT_BODY_TEMPLATE;
    });

    document.getElementById('btn-cfg-upload-logo')?.addEventListener('click', async () => {
      try {
        const cfg = await invoke('upload_certificate_asset', { kind: 'logo' });
        if (!cfg) return;
        applyCertSettingsToDialog(cfg);
        await refreshCertPreview(state.certPreviewRow ? rowToCertificateRenderPayload(state.certPreviewRow) : null);
        showCertStatus('Logo sertifikat kustom diperbarui.', 'success');
      } catch (err) {
        alert(String(err));
      }
    });

    document.getElementById('btn-cfg-reset-logo')?.addEventListener('click', async () => {
      try {
        const cfg = await invoke('clear_certificate_asset', { kind: 'logo' });
        applyCertSettingsToDialog(cfg);
        await refreshCertPreview(state.certPreviewRow ? rowToCertificateRenderPayload(state.certPreviewRow) : null);
        showCertStatus('Logo sertifikat dikembalikan ke logo bawaan Anda Bungalows.', 'success');
      } catch (err) {
        alert(String(err));
      }
    });

    document.getElementById('btn-cfg-upload-sig')?.addEventListener('click', async () => {
      try {
        const cfg = await invoke('upload_certificate_asset', { kind: 'signature' });
        if (!cfg) return;
        applyCertSettingsToDialog(cfg);
        await refreshCertPreview(state.certPreviewRow ? rowToCertificateRenderPayload(state.certPreviewRow) : null);
        showCertStatus('Gambar tanda tangan/stempel diperbarui.', 'success');
      } catch (err) {
        alert(String(err));
      }
    });

    document.getElementById('btn-cfg-clear-sig')?.addEventListener('click', async () => {
      try {
        const cfg = await invoke('clear_certificate_asset', { kind: 'signature' });
        applyCertSettingsToDialog(cfg);
        await refreshCertPreview(state.certPreviewRow ? rowToCertificateRenderPayload(state.certPreviewRow) : null);
        showCertStatus('Gambar tanda tangan dihapus (mode tanda tangan basah).', 'info');
      } catch (err) {
        alert(String(err));
      }
    });

    settingsForm?.addEventListener('submit', async (e) => {
      e.preventDefault();
      try {
        const wasDirty = isCertFormDirty();
        const cfg = await invoke('update_certificate_settings', {
          signerName: document.getElementById('cfg-cert-signer').value.trim(),
          signerTitle: document.getElementById('cfg-cert-title').value.trim(),
          programName: document.getElementById('cfg-cert-program').value.trim(),
          paperSize: document.getElementById('cfg-cert-paper').value,
          showSignature: document.getElementById('cfg-cert-show-sig').checked,
          issuedCity: document.getElementById('cfg-cert-city').value.trim(),
          bodyTemplate: document.getElementById('cfg-cert-template').value.trim(),
        });
        applyCertSettingsToDialog(cfg);
        if (!state.certEditingId) {
          document.getElementById('cert-signer').value = cfg.signer_name;
          document.getElementById('cert-signer-title').value = cfg.signer_title;
          document.getElementById('cert-issued-city').value = cfg.issued_city || DEFAULT_CERT_ISSUED_CITY;
          if (!document.getElementById('cert-program-type')?.value) {
            document.getElementById('cert-program').value = cfg.program_name;
          }
          setSelectedCertPaperSize(cfg.paper_size);
        }
        if (!wasDirty) {
          markCertFormClean();
        } else {
          saveCertDraftToStorage();
        }
        settingsDialog?.close();
        await refreshCertPreview(state.certPreviewRow ? rowToCertificateRenderPayload(state.certPreviewRow) : null);
        showCertStatus('Pengaturan default sertifikat disimpan.', 'success');
      } catch (err) {
        alert(String(err));
      }
    });

    initCertCatalogDialog();
  }

  // DOM Content Loaded Entrypoint
  document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('form').forEach(form => {
      if (['form-certificate', 'form-cert-filter', 'form-cert-settings', 'form-catalog-dept', 'form-catalog-prog'].includes(form.id)) return;
      form.addEventListener('submit', (e) => e.preventDefault());
    });
    initAuth();
    initNav();
    initSystemClock();
    initRecapModule();
    initLedgerModule();
    initImportsModule();
    initOptimizerModule();
    initTourismModule();
    initCertificatesModule();
    initSettingsModal();
    initModalsDismiss();
  });

})();
