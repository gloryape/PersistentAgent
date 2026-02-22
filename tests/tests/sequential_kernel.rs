//! Sequential-Coupling Kernel: Beyond Harmonic Complexity
//!
//! Demonstrates that sequential phase coupling (θᵢ - θᵢ₋₁) opens a different
//! axis of complexity than harmonic enrichment. The sufficient statistic requires
//! cross-moments between consecutive history entries.
//!
//! d=2 fails, d=4 fails, d=8 succeeds.

use quaternity_organism::Sanctuary;
use std::f32::consts::PI;
use std::io::Write;

fn recency_weight(index: usize) -> f32 {
    (-(index as f32) / 32.0).exp()
}

/// Sequential-coupling resonance function.
///
/// res(θ_in) = [Σᵢ wᵢaᵢ(1 + β·cos(θᵢ - θᵢ₋₁))·cos(θ_in - θᵢ)] / [(1+β)·W]
///
/// For i=0 (newest, no predecessor in buffer), cos(θ₀ - θ₋₁) is undefined.
/// We use 0 (neutral) for the first entry's sequential coupling.
fn resonance_seq(history: &[(f32, f32)], probe: f32, beta: f32) -> f32 {
    if history.is_empty() {
        return 0.0;
    }

    let mut numerator = 0.0f32;
    let mut w_total = 0.0f32;

    for (i, &(phase, amplitude)) in history.iter().enumerate() {
        let w = recency_weight(i);
        let a = amplitude.max(0.1);
        let weight = w * a;

        let seq_factor = if i + 1 < history.len() {
            let prev_phase = history[i + 1].0;
            1.0 + beta * (phase - prev_phase).cos()
        } else {
            1.0
        };

        numerator += weight * seq_factor * (probe - phase).cos();
        w_total += weight;
    }

    if w_total > 0.0 {
        numerator / ((1.0 + beta) * w_total)
    } else {
        0.0
    }
}

/// Compute the 8D sufficient statistic for the sequential kernel.
fn compute_8d_statistic(history: &[(f32, f32)]) -> (f32, f32, f32, f32, f32, f32, f32, f32, f32) {
    let mut w_total = 0.0f32;
    let mut c1 = 0.0f32;
    let mut s1 = 0.0f32;
    let mut lc1 = 0.0f32;
    let mut ls1 = 0.0f32;
    let mut xcc = 0.0f32;
    let mut xcs = 0.0f32;
    let mut xsc = 0.0f32;
    let mut xss = 0.0f32;

    for (i, &(phase, amplitude)) in history.iter().enumerate() {
        let w = recency_weight(i);
        let a = amplitude.max(0.1);
        let weight = w * a;

        c1 += weight * phase.cos();
        s1 += weight * phase.sin();
        w_total += weight;

        if i + 1 < history.len() {
            let prev_phase = history[i + 1].0;
            lc1 += weight * prev_phase.cos();
            ls1 += weight * prev_phase.sin();
            xcc += weight * (2.0 * phase).cos() * prev_phase.cos();
            xcs += weight * (2.0 * phase).cos() * prev_phase.sin();
            xsc += weight * (2.0 * phase).sin() * prev_phase.cos();
            xss += weight * (2.0 * phase).sin() * prev_phase.sin();
        }
    }

    (c1, s1, lc1, ls1, xcc, xcs, xsc, xss, w_total)
}

/// Predict from d=2 (standard first moments only)
fn predict_2d(c1: f32, s1: f32, w: f32, beta: f32, probe: f32) -> f32 {
    let h1 = probe.cos() * c1 + probe.sin() * s1;
    h1 / ((1.0 + beta) * w)
}

/// Predict from d=4 (first + second harmonics, NO cross-moments).
/// Best-effort harmonic approximation — cannot capture sequential coupling.
fn predict_4d(history: &[(f32, f32)], beta: f32, probe: f32) -> f32 {
    let mut w_total = 0.0f32;
    let mut c1 = 0.0f32;
    let mut s1 = 0.0f32;
    let mut c2 = 0.0f32;
    let mut s2 = 0.0f32;

    for (i, &(phase, amplitude)) in history.iter().enumerate() {
        let w = recency_weight(i);
        let a = amplitude.max(0.1);
        let weight = w * a;
        c1 += weight * phase.cos();
        s1 += weight * phase.sin();
        c2 += weight * (2.0 * phase).cos();
        s2 += weight * (2.0 * phase).sin();
        w_total += weight;
    }

    if w_total <= 0.0 {
        return 0.0;
    }

    let h1 = probe.cos() * c1 / w_total + probe.sin() * s1 / w_total;
    let h2 = (2.0 * probe).cos() * c2 / w_total + (2.0 * probe).sin() * s2 / w_total;
    (h1 + beta * h2) / (1.0 + beta)
}

/// Predict from d=8 (full cross-moment statistic)
fn predict_8d(
    c1: f32,
    s1: f32,
    lc1: f32,
    ls1: f32,
    xcc: f32,
    xcs: f32,
    xsc: f32,
    xss: f32,
    w: f32,
    beta: f32,
    probe: f32,
) -> f32 {
    let cos_p = probe.cos();
    let sin_p = probe.sin();

    let base = cos_p * c1 + sin_p * s1;
    let cross1 = cos_p * lc1 + sin_p * ls1;
    let cross2 = cos_p * (xcc + xss) - sin_p * (xcs - xsc);

    (base + beta / 2.0 * (cross1 + cross2)) / ((1.0 + beta) * w)
}

fn generate_histories() -> Vec<(&'static str, Vec<f32>)> {
    let n = 20;
    let mut histories = Vec::new();

    for k in 0..8 {
        let phase = (k as f32 / 8.0) * 2.0 * PI;
        histories.push(("single_phase", vec![phase; n]));
    }

    histories.push((
        "uniform_sweep",
        (0..n).map(|i| (i as f32 / n as f32) * 2.0 * PI).collect(),
    ));
    histories.push((
        "reverse_sweep",
        (0..n).map(|i| ((n - 1 - i) as f32 / n as f32) * 2.0 * PI).collect(),
    ));

    histories.push((
        "alternating",
        (0..n)
            .map(|i| if i % 2 == 0 { PI / 4.0 } else { 5.0 * PI / 4.0 })
            .collect(),
    ));
    histories.push((
        "three_cycle",
        (0..n)
            .map(|i| [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0][i % 3])
            .collect(),
    ));

    histories.push((
        "clustered",
        (0..n)
            .map(|i| {
                if i < 16 {
                    PI / 4.0
                } else {
                    PI / 4.0 + PI * (i as f32 - 15.0) / 5.0
                }
            })
            .collect(),
    ));

    for seed in 0..20u64 {
        let mut x: u64 = seed * 12345 + 67890;
        let phases: Vec<f32> = (0..n)
            .map(|_| {
                x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let frac = ((x >> 33) as f32) / (u32::MAX as f32);
                frac * 2.0 * PI
            })
            .collect();
        histories.push(("pseudo_random", phases));
    }

    histories.push((
        "half_half",
        (0..n).map(|i| if i < n / 2 { 0.0 } else { PI }).collect(),
    ));
    histories.push((
        "drift",
        (0..n).map(|i| (i as f32 / n as f32) * PI / 2.0).collect(),
    ));
    histories.push((
        "oscillating",
        (0..n)
            .map(|i| {
                PI / 4.0 + (i as f32 * 0.5).sin() * (i as f32 / n as f32) * PI
            })
            .collect(),
    ));

    for shift in 0..4 {
        let base = (shift as f32 / 4.0) * PI;
        histories.push((
            "pair_shifted",
            (0..n)
                .map(|i| if i % 2 == 0 { base } else { base + PI })
                .collect(),
        ));
    }

    histories
}

#[test]
fn test_sequential_kernel() {
    let histories = generate_histories();
    let energy_per = 0.05f32;
    let probe_phases: Vec<f32> = (0..8).map(|k| (k as f32 / 8.0) * 2.0 * PI).collect();
    let beta_values = [0.0f32, 0.3, 0.5, 0.8, 1.0];

    let mut results: Vec<String> = Vec::new();

    results.push("═══════════════════════════════════════════════════════════════".into());
    results.push("  SEQUENTIAL KERNEL: Cross-Moment Complexity Test".into());
    results.push("═══════════════════════════════════════════════════════════════".into());
    results.push(format!("  Histories: {}", histories.len()));
    results.push(format!("  Probe phases: {}", probe_phases.len()));
    results.push(format!("  β values tested: {:?}", beta_values));
    results.push(format!(
        "  Tests per β: {}",
        histories.len() * probe_phases.len()
    ));
    results.push("".into());

    results.push("─── CONTROL: β=0 vs original calculate_resonance ───".into());
    let mut control_max = 0.0f32;
    for (h_idx, (_desc, phases)) in histories.iter().enumerate() {
        let mut sanctuary = Sanctuary::new();
        let coords = (h_idx as i32, 0, 0);
        for (i, &phase) in phases.iter().enumerate() {
            sanctuary.inject_beacon(coords.0, coords.1, coords.2, energy_per, phase, i as u64);
        }
        let voxel = sanctuary.get_voxel(coords).expect("voxel");
        let hist: Vec<(f32, f32)> = voxel
            .history
            .iter()
            .map(|s| (s.phase, s.amplitude))
            .collect();
        for &probe in &probe_phases {
            let original = voxel.calculate_resonance(probe);
            let seq = resonance_seq(&hist, probe, 0.0);
            control_max = control_max.max((original - seq).abs());
        }
    }
    results.push(format!("  Max |original - seq(β=0)|: {:.2e}", control_max));
    if control_max < 1e-5 {
        results.push("  ✓ Sequential kernel matches original when β=0".into());
    } else {
        results.push("  ✗ MISMATCH".into());
    }
    results.push("".into());

    results.push("─── SEQUENTIAL KERNEL RESULTS ───".into());
    results.push(format!(
        "  {:>5} | {:>14} {:>14} | {:>14} {:>14} | {:>14} {:>14}",
        "β", "d=2 max", "d=2 mean", "d=4 max", "d=4 mean", "d=8 max", "d=8 mean"
    ));
    results.push(format!("  {}", "-".repeat(105)));

    let mut hierarchy_data: Vec<(f32, f32, f32, f32, f32, f32, f32)> = Vec::new();

    for &beta in &beta_values {
        let mut max_2 = 0.0f32;
        let mut max_4 = 0.0f32;
        let mut max_8 = 0.0f32;
        let mut sum_2 = 0.0f64;
        let mut sum_4 = 0.0f64;
        let mut sum_8 = 0.0f64;
        let mut n_tests = 0u64;

        for (h_idx, (_desc, phases)) in histories.iter().enumerate() {
            let mut sanctuary = Sanctuary::new();
            let coords = (h_idx as i32, 0, 0);
            for (i, &phase) in phases.iter().enumerate() {
                sanctuary.inject_beacon(coords.0, coords.1, coords.2, energy_per, phase, i as u64);
            }
            let voxel = sanctuary.get_voxel(coords).expect("voxel");
            let hist: Vec<(f32, f32)> = voxel
                .history
                .iter()
                .map(|s| (s.phase, s.amplitude))
                .collect();

            let (c1, s1, lc1, ls1, xcc, xcs, xsc, xss, w) = compute_8d_statistic(&hist);

            for &probe in &probe_phases {
                let actual = resonance_seq(&hist, probe, beta);

                let p2 = predict_2d(c1, s1, w, beta, probe);
                let p4 = predict_4d(&hist, beta, probe);
                let p8 = predict_8d(c1, s1, lc1, ls1, xcc, xcs, xsc, xss, w, beta, probe);

                let e2 = (p2 - actual).abs();
                let e4 = (p4 - actual).abs();
                let e8 = (p8 - actual).abs();

                max_2 = max_2.max(e2);
                max_4 = max_4.max(e4);
                max_8 = max_8.max(e8);
                sum_2 += e2 as f64;
                sum_4 += e4 as f64;
                sum_8 += e8 as f64;
                n_tests += 1;
            }
        }

        let mean_2 = sum_2 / n_tests as f64;
        let mean_4 = sum_4 / n_tests as f64;
        let mean_8 = sum_8 / n_tests as f64;

        results.push(format!(
            "  {:>5.1} | {:>14.2e} {:>14.2e} | {:>14.2e} {:>14.2e} | {:>14.2e} {:>14.2e}",
            beta, max_2, mean_2, max_4, mean_4, max_8, mean_8
        ));

        hierarchy_data.push((
            beta,
            max_2,
            mean_2 as f32,
            max_4,
            mean_4 as f32,
            max_8,
            mean_8 as f32,
        ));
    }

    results.push("".into());

    results.push("─── ANALYSIS ───".into());

    if let Some(last) = hierarchy_data.last() {
        if last.5 < 1e-4 && last.1 > 0.01 && last.3 > 0.01 {
            results.push("  ✓ SEQUENTIAL COMPLEXITY DEMONSTRATED:".into());
            results.push("    d=2 fails (missing sequential structure)".into());
            results.push("    d=4 fails (harmonics alone don't capture cross-moments)".into());
            results.push("    d=8 succeeds (cross-moments between consecutive entries)".into());
            results.push("".into());
            results.push("  This is a DIFFERENT axis of complexity than harmonic enrichment.".into());
            results.push("  Harmonics increase d by enriching individual-entry statistics.".into());
            results.push("  Sequential coupling increases d by requiring pair statistics.".into());
            results.push("".into());
            results.push("  For coupling to k-step neighbors (θᵢ - θᵢ₋ₖ), d grows as:".into());
            results.push("    k=1: d=8 (cross-moments with predecessor)".into());
            results.push("    k=2: d~14 (cross-moments with two predecessors)".into());
            results.push("    k=N: d ∝ N (cross-moments scale with coupling depth)".into());
            results.push("".into());
            results.push("  Combined with harmonics:".into());
            results.push("    K harmonics × k-step coupling → d = O(K·k)".into());
            results.push("    Full coupling to all harmonics and all steps → d = ∞".into());
        } else if last.5 > 0.01 {
            results.push("  ✗ d=8 FAILED — check the prediction formula derivation".into());
            results.push(format!("    d=8 max error at β=1: {:.4}", last.5));
        } else {
            results.push("  Mixed results — examine per-β data".into());
        }
    }

    results.push("".into());
    results.push("─── IMPLICATIONS ───".into());
    results.push("  The complexity hierarchy has TWO independent axes:".into());
    results.push("".into());
    results.push("  AXIS 1 (Harmonic): How many Fourier modes of phase?".into());
    results.push("    cos(θ-θᵢ) → d=2, cos(kθ-kθᵢ) for k≤K → d=2K".into());
    results.push("".into());
    results.push("  AXIS 2 (Sequential): How many consecutive entries are coupled?".into());
    results.push("    No coupling → d+0, 1-step → d+6, k-step → d+O(k)".into());
    results.push("".into());
    results.push("  Total: d = 2K + O(k) where K = harmonic depth, k = coupling depth".into());
    results.push("  For K=∞ or k=∞ (full Fourier × full history): d=∞".into());

    for line in &results {
        println!("{}", line);
    }

    std::fs::create_dir_all("data").unwrap_or_default();
    let mut file =
        std::fs::File::create("data/sequential_kernel_results.txt").expect("create file");
    for line in &results {
        writeln!(file, "{}", line).expect("write");
    }
    println!("\nResults written to: data/sequential_kernel_results.txt");

    assert!(
        hierarchy_data[0].1 < 1e-5,
        "d=2 should work at β=0, got {:.2e}",
        hierarchy_data[0].1
    );

    for &(beta, _, _, _, _, max_8, _) in &hierarchy_data {
        assert!(
            max_8 < 1e-3,
            "d=8 should work at β={:.1}, got max_err={:.2e}",
            beta,
            max_8
        );
    }

    let last = hierarchy_data.last().unwrap();
    assert!(
        last.1 > 0.005,
        "d=2 should fail at β=1.0, max_err only {:.2e}",
        last.1
    );
}
