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

  function logout() {
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

    // Highlight parent dropdown toggle
    const tabToDropdown = {
      recap: 'pajak',
      tourism: 'pajak',
      ledger: 'transaksi',
      imports: 'transaksi',
      optimizer: 'audit',
    };

    const targetGroup = tabToDropdown[tabName] || 'pajak';
    document.querySelectorAll('.nav-dropdown').forEach(dropdown => {
      const toggle = dropdown.querySelector('.nav-dropdown-toggle');
      if (dropdown.dataset.dropdown === targetGroup) {
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

  // DOM Content Loaded Entrypoint
  document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('form').forEach(form => {
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
    initSettingsModal();
    initModalsDismiss();
  });

})();
