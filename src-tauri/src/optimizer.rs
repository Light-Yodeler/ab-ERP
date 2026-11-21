use chrono::{Datelike, Duration, NaiveDate};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReservationDetails {
    pub total_person: Option<i64>,
    pub booking_number: Option<String>,
    pub nationality: Option<String>,
    pub room_number: Option<String>,
    pub total_rooms: Option<i64>,
    pub check_in_date: Option<String>,
    pub check_out_date: Option<String>,
    pub total_nights: Option<i64>,
    pub rate: Option<f64>,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionItem {
    pub original_row: usize,
    pub date_str: String,
    pub raw_date: Option<f64>,
    pub amount: f64,
    #[serde(default)]
    pub guest_name: Option<String>,
    #[serde(default)]
    pub details: Option<ReservationDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverOptions {
    pub target: f64,
    pub tolerance: f64,
    pub max_trajectories: Option<usize>,
    pub seed: Option<u64>,
    pub exclude_signatures: Option<Vec<u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverResult {
    pub selected_indices: Vec<usize>,
    pub selected_rows: Vec<usize>,
    pub total_amount: f64,
    pub target_amount: f64,
    pub difference: f64,
    pub duration_ms: f64,
    pub signature: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConflict {
    pub room_number: String,
    pub date: String,
    pub rows: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomAnomaly {
    pub row: usize,
    pub room_number: String,
    pub message: String,
}

fn excel_serial_to_naive_date(serial: f64) -> Option<NaiveDate> {
    if !serial.is_finite() || serial < 1.0 {
        return None;
    }
    let days = serial.floor() as i64;
    let adjusted_days = if days > 60 { days - 2 } else { days - 1 };
    let epoch = NaiveDate::from_ymd_opt(1900, 1, 1)?;
    epoch.checked_add_signed(Duration::days(adjusted_days))
}

fn parse_flex_date(s: &str) -> Option<NaiveDate> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(d) = crate::excel_importer::parse_date_string(trimmed) {
        return Some(d);
    }
    if trimmed.len() >= 10 && trimmed.is_char_boundary(10) {
        if let Ok(d) = NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d") {
            return Some(d);
        }
    }
    None
}

fn normalize_room_part(part: &str) -> String {
    let t = part.trim();
    if !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()) {
        let stripped = t.trim_start_matches('0');
        if stripped.is_empty() {
            "0".to_string()
        } else {
            stripped.to_string()
        }
    } else {
        t.to_string()
    }
}

fn parse_room_tokens(raw: &str, total_rooms: Option<i64>) -> (Vec<String>, Option<String>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return (Vec::new(), None);
    }

    if let Ok(f) = trimmed.parse::<f64>() {
        if f.is_finite() && f >= 0.0 && f.fract() == 0.0 {
            return (vec![format!("{}", f as i64)], None);
        }
    }

    let dot_parts: Vec<&str> = trimmed.split('.').collect();
    if dot_parts.len() == 2
        && !dot_parts[0].is_empty()
        && !dot_parts[1].is_empty()
        && dot_parts[0].chars().all(|c| c.is_ascii_digit())
        && dot_parts[1].chars().all(|c| c.is_ascii_digit())
        && dot_parts[1].chars().any(|c| c != '0')
    {
        let lhs = normalize_room_part(dot_parts[0]);
        let rhs = normalize_room_part(dot_parts[1]);
        let msg = format!(
            "Format nomor kamar desimal '{}' terdeteksi; gunakan pemisah koma (mis. '{}, {}').",
            trimmed, lhs, rhs
        );
        if total_rooms.unwrap_or(1) > 1 {
            let mut tokens = vec![lhs.clone()];
            if rhs != lhs {
                tokens.push(rhs);
            }
            return (tokens, Some(msg));
        }
        return (vec![trimmed.to_string()], Some(msg));
    }

    let mut tokens: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for piece in trimmed.split(|c: char| matches!(c, ',' | '&' | '/' | '+' | ';') || c.is_whitespace()) {
        let clean = piece.trim();
        if clean.is_empty() || clean == "-" {
            continue;
        }
        let norm = normalize_room_part(clean);
        if seen.insert(norm.clone()) {
            tokens.push(norm);
        }
    }
    (tokens, None)
}

fn item_stay_nights(item: &TransactionItem) -> Option<(NaiveDate, i64)> {
    let det = item.details.as_ref()?;
    let check_in = det
        .check_in_date
        .as_deref()
        .and_then(parse_flex_date)
        .or_else(|| parse_flex_date(&item.date_str))
        .or_else(|| item.raw_date.and_then(excel_serial_to_naive_date))?;

    let nights = det
        .check_out_date
        .as_deref()
        .and_then(parse_flex_date)
        .filter(|&co| co > check_in)
        .map(|co| (co - check_in).num_days())
        .or_else(|| det.total_nights.filter(|&n| n >= 1))
        .unwrap_or(1)
        .clamp(1, 366);

    Some((check_in, nights))
}

/// Inspects 13-column reservation items for same-room overlapping nights and room anomalies.
pub fn analyze_room_issues(items: &[TransactionItem]) -> (Vec<RoomConflict>, Vec<RoomAnomaly>) {
    let mut occupancy: BTreeMap<(NaiveDate, String), Vec<usize>> = BTreeMap::new();
    let mut anomalies: Vec<RoomAnomaly> = Vec::new();

    for item in items {
        let Some(det) = item.details.as_ref() else {
            continue;
        };

        let raw_room = det.room_number.as_deref().unwrap_or("");
        let (tokens, decimal_warn) = parse_room_tokens(raw_room, det.total_rooms);

        if let Some(msg) = &decimal_warn {
            anomalies.push(RoomAnomaly {
                row: item.original_row,
                room_number: raw_room.trim().to_string(),
                message: msg.clone(),
            });
        } else if !tokens.is_empty() {
            if let Some(tr) = det.total_rooms {
                if tr > 0 && (tokens.len() as i64) != tr {
                    anomalies.push(RoomAnomaly {
                        row: item.original_row,
                        room_number: raw_room.trim().to_string(),
                        message: format!(
                            "Jumlah nomor kamar ({} kamar: '{}') tidak sesuai Total Kamar ({}).",
                            tokens.len(),
                            raw_room.trim(),
                            tr
                        ),
                    });
                }
            }
        }

        let ci_parsed = det
            .check_in_date
            .as_deref()
            .and_then(parse_flex_date)
            .or_else(|| parse_flex_date(&item.date_str))
            .or_else(|| item.raw_date.and_then(excel_serial_to_naive_date));
        let co_parsed = det.check_out_date.as_deref().and_then(parse_flex_date);
        if !tokens.is_empty() && ci_parsed.is_none() {
            let raw_date = det.check_in_date.as_deref().unwrap_or(&item.date_str);
            anomalies.push(RoomAnomaly {
                row: item.original_row,
                room_number: raw_room.trim().to_string(),
                message: format!(
                    "Format tanggal Check In tidak dikenali ('{}'); jadwal kamar tidak dapat diverifikasi.",
                    raw_date
                ),
            });
        }
        if det.total_nights.is_some_and(|n| n <= 0)
            || matches!((ci_parsed, co_parsed), (Some(ci), Some(co)) if co <= ci)
        {
            anomalies.push(RoomAnomaly {
                row: item.original_row,
                room_number: raw_room.trim().to_string(),
                message: "Rentang tanggal Check In / Check Out atau Total Night tidak valid.".to_string(),
            });
        } else if let (Some(ci), Some(co)) = (ci_parsed, co_parsed) {
            if co > ci {
                let cal_nights = (co - ci).num_days();
                if let Some(tn) = det.total_nights {
                    if tn > 0 && tn != cal_nights {
                        anomalies.push(RoomAnomaly {
                            row: item.original_row,
                            room_number: raw_room.trim().to_string(),
                            message: format!(
                                "Total Night ({}) tidak sesuai selisih Check Out - Check In ({} malam); deteksi konflik memakai durasi kalender {} malam.",
                                tn, cal_nights, cal_nights
                            ),
                        });
                    }
                } else if !tokens.is_empty() {
                    anomalies.push(RoomAnomaly {
                        row: item.original_row,
                        room_number: raw_room.trim().to_string(),
                        message: format!(
                            "Kolom Total Night kosong; sistem memakai durasi {} malam dari Check In/Check Out.",
                            cal_nights
                        ),
                    });
                }
            }
        }

        if let Some((ci, nights)) = item_stay_nights(item) {
            for room in &tokens {
                for d in 0..nights {
                    if let Some(night_date) = ci.checked_add_signed(Duration::days(d)) {
                        occupancy
                            .entry((night_date, room.clone()))
                            .or_default()
                            .push(item.original_row);
                    }
                }
            }
        }
    }

    let mut conflicts: Vec<RoomConflict> = Vec::new();
    for ((date, room), mut rows) in occupancy {
        if rows.len() > 1 {
            rows.sort_unstable();
            rows.dedup();
            if rows.len() > 1 {
                conflicts.push(RoomConflict {
                    room_number: room,
                    date: format!("{:02}/{:02}/{:04}", date.day(), date.month(), date.year()),
                    rows,
                });
            }
        }
    }

    (conflicts, anomalies)
}

fn build_conflict_adjacency(items: &[TransactionItem]) -> Vec<Vec<usize>> {
    let n = items.len();
    let mut adj = vec![Vec::new(); n];
    let mut slots: HashMap<(String, NaiveDate), Vec<usize>> = HashMap::new();

    for (idx, item) in items.iter().enumerate() {
        let Some(det) = item.details.as_ref() else {
            continue;
        };
        let raw_room = det.room_number.as_deref().unwrap_or("");
        let (tokens, _) = parse_room_tokens(raw_room, det.total_rooms);
        if tokens.is_empty() {
            continue;
        }
        if let Some((ci, nights)) = item_stay_nights(item) {
            for room in tokens {
                for d in 0..nights {
                    if let Some(night_date) = ci.checked_add_signed(Duration::days(d)) {
                        slots.entry((room.clone(), night_date)).or_default().push(idx);
                    }
                }
            }
        }
    }

    for indices in slots.into_values() {
        if indices.len() < 2 {
            continue;
        }
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                let a = indices[i];
                let b = indices[j];
                adj[a].push(b);
                adj[b].push(a);
            }
        }
    }

    for nbrs in &mut adj {
        nbrs.sort_unstable();
        nbrs.dedup();
    }
    adj
}

/// Compute a 64-bit FNV-1a hash of sorted selected indices
pub fn compute_signature(selected: &[usize]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &idx in selected {
        h ^= idx as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Solve subset sum using multi-start randomized greedy + 2-opt local search in parallel
pub fn solve_subset(items: &[TransactionItem], options: &SolverOptions) -> Option<SolverResult> {
    if items.is_empty() || !options.target.is_finite() || options.target <= 0.0 {
        return None;
    }

    let start_time = Instant::now();
    let target = options.target;
    let min_stop_diff = (if options.tolerance <= 0.0 { 0.5 } else { options.tolerance }).min(0.5);
    let trajectories_count = options.max_trajectories.unwrap_or(64).clamp(16, 256);
    let base_seed = options.seed.unwrap_or_else(|| {
        let mut rng = rand::thread_rng();
        rng.gen()
    });

    let excluded: HashSet<u64> = options
        .exclude_signatures
        .as_ref()
        .map(|list| list.iter().copied().collect())
        .unwrap_or_default();

    let n = items.len();
    let amounts: Vec<f64> = items.iter().map(|item| item.amount).collect();
    let conflict_adj = build_conflict_adjacency(items);
    let has_conflict = |cand: usize, sel: &[bool], ignore1: Option<usize>, ignore2: Option<usize>| -> bool {
        conflict_adj[cand]
            .iter()
            .any(|&nbr| sel[nbr] && Some(nbr) != ignore1 && Some(nbr) != ignore2)
    };

    // Run parallel trajectories
    let candidates: Vec<(Vec<usize>, f64, f64, u64)> = (0..trajectories_count)
        .into_par_iter()
        .filter_map(|t_idx| {
            let thread_seed = base_seed.wrapping_add((t_idx as u64).wrapping_mul(1013904223));
            let mut rng = ChaCha8Rng::seed_from_u64(thread_seed);

            // Phase 1: Randomized Greedy Initialization
            let mut indices: Vec<usize> = (0..n).collect();
            indices.shuffle(&mut rng);

            let mut selected = vec![false; n];
            let mut curr_sum: f64 = 0.0;

            for &idx in &indices {
                let amt = amounts[idx];
                if curr_sum + amt <= target && !has_conflict(idx, &selected, None, None) {
                    selected[idx] = true;
                    curr_sum += amt;
                }
            }

            // Phase 2: Local Search (1-opt add/remove and 2-opt swaps)
            let mut improved = true;
            let mut iterations = 0;
            const MAX_ITER: usize = 150;

            while improved && iterations < MAX_ITER {
                improved = false;
                iterations += 1;
                let curr_diff = (curr_sum - target).abs();

                if curr_diff <= min_stop_diff {
                    break;
                }

                // 1-item add
                for i in 0..n {
                    if !selected[i] && !has_conflict(i, &selected, None, None) {
                        let new_sum = curr_sum + amounts[i];
                        let new_diff = (new_sum - target).abs();
                        if new_diff < curr_diff - 1e-6 {
                            selected[i] = true;
                            curr_sum = new_sum;
                            improved = true;
                            break;
                        }
                    }
                }
                if improved {
                    continue;
                }

                // 1-item remove
                for i in 0..n {
                    if selected[i] {
                        let new_sum = curr_sum - amounts[i];
                        let new_diff = (new_sum - target).abs();
                        if new_diff < curr_diff - 1e-6 {
                            selected[i] = false;
                            curr_sum = new_sum;
                            improved = true;
                            break;
                        }
                    }
                }
                if improved {
                    continue;
                }

                // 1-for-1 swap with first-improvement for rapid convergence
                let mut swap_found = false;
                for i in 0..n {
                    if selected[i] {
                        for j in 0..n {
                            if !selected[j] && !has_conflict(j, &selected, Some(i), None) {
                                let new_sum = curr_sum - amounts[i] + amounts[j];
                                let new_diff = (new_sum - target).abs();
                                if new_diff < curr_diff - 1e-6 {
                                    selected[i] = false;
                                    selected[j] = true;
                                    curr_sum = new_sum;
                                    improved = true;
                                    swap_found = true;
                                    break;
                                }
                            }
                        }
                        if swap_found {
                            break;
                        }
                    }
                }
                if swap_found {
                    continue;
                }

                // 2-for-1 or 1-for-2 randomized swap perturbations if diff is still above tolerance
                if curr_diff > min_stop_diff && n >= 4 {
                    let sel_list: Vec<usize> = (0..n).filter(|&i| selected[i]).collect();
                    let unsel_list: Vec<usize> = (0..n).filter(|&i| !selected[i]).collect();
                    // Try random sample of 2-item swaps
                    let mut sampled_improved = false;
                    for _ in 0..12 {

                        if sel_list.len() >= 2 && !unsel_list.is_empty() {
                            let s1 = sel_list[rng.gen_range(0..sel_list.len())];
                            let s2 = sel_list[rng.gen_range(0..sel_list.len())];
                            if s1 != s2 {
                                let u1 = unsel_list[rng.gen_range(0..unsel_list.len())];
                                if !has_conflict(u1, &selected, Some(s1), Some(s2)) {
                                    let new_sum = curr_sum - amounts[s1] - amounts[s2] + amounts[u1];
                                    let new_diff = (new_sum - target).abs();
                                    if new_diff < curr_diff - 1e-6 {
                                        selected[s1] = false;
                                        selected[s2] = false;
                                        selected[u1] = true;
                                        curr_sum = new_sum;
                                        sampled_improved = true;
                                        break;
                                    }
                                }
                            }
                        }

                        if !sel_list.is_empty() && unsel_list.len() >= 2 {
                            let s1 = sel_list[rng.gen_range(0..sel_list.len())];
                            let u1 = unsel_list[rng.gen_range(0..unsel_list.len())];
                            let u2 = unsel_list[rng.gen_range(0..unsel_list.len())];
                            if u1 != u2
                                && conflict_adj[u1].binary_search(&u2).is_err()
                                && !has_conflict(u1, &selected, Some(s1), None)
                                && !has_conflict(u2, &selected, Some(s1), None)
                            {
                                let new_sum = curr_sum - amounts[s1] + amounts[u1] + amounts[u2];
                                let new_diff = (new_sum - target).abs();
                                if new_diff < curr_diff - 1e-6 {
                                    selected[s1] = false;
                                    selected[u1] = true;
                                    selected[u2] = true;
                                    curr_sum = new_sum;
                                    sampled_improved = true;
                                    break;
                                }
                            }
                        }
                    }
                    if sampled_improved {
                        improved = true;
                    }
                }
            }

            // Phase 3: Gather chronologically sorted indices
            let mut final_selected: Vec<usize> = Vec::with_capacity(n / 2);
            for i in 0..n {
                if selected[i] {
                    final_selected.push(i);
                }
            }

            let diff = (curr_sum - target).abs();
            let sig = compute_signature(&final_selected);

            Some((final_selected, curr_sum, diff, sig))
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Filter out excluded signatures if alternative solutions are available
    let mut eligible: Vec<&(Vec<usize>, f64, f64, u64)> = candidates
        .iter()
        .filter(|(_, _, _, sig)| !excluded.contains(sig))
        .collect();

    // If all generated were excluded, fallback to all candidates
    if eligible.is_empty() {
        eligible = candidates.iter().collect();
    }

    // Sort by difference ascending (primary), then by item count (secondary)
    eligible.sort_by(|a, b| {
        a.2.partial_cmp(&b.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.len().cmp(&b.0.len()))
    });

    let best = eligible[0];
    let selected_indices = best.0.clone();
    let selected_rows: Vec<usize> = selected_indices.iter().map(|&i| items[i].original_row).collect();
    let total_amount = best.1;
    let difference = best.2;
    let signature = best.3;
    let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

    Some(SolverResult {
        selected_indices,
        selected_rows,
        total_amount,
        target_amount: target,
        difference,
        duration_ms,
        signature,
    })
}
