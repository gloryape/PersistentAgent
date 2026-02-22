//! Harmonic Hierarchy Test: Demonstrating the Complexity Hierarchy
//!
//! Shows that adding a second harmonic to the memory kernel makes the d=2
//! sufficient statistic FAIL while the d=4 statistic SUCCEEDS.
//! This is the empirical demonstration of Paper #2's central classification:
//!   cos kernel → d=2, multi-harmonic kernel → d=2K

use quaternity_organism::Sanctuary;
use std::f32::consts::PI;
use std::io::Write;

// ═══════════════════════════════════════════════════════════════════
// WEIGHT FUNCTIONS (matching Voxel::calculate_resonance exactly)
// ═══════════════════════════════════════════════════════════════════

/// Recency weight: exp(-index / 32.0), matching MEMORY_DEPTH=64, scale=32
fn recency_weight(index: usize) -> f32 {
    (-(index as f32) / 32.0).exp()
}

// ═══════════════════════════════════════════════════════════════════
// TWO-HARMONIC RESONANCE (the richer kernel)
// ═══════════════════════════════════════════════════════════════════

/// Compute resonance with first + second harmonic.
///
/// resonance = [Σ wᵢaᵢ cos(θ-θᵢ) + γ₂ Σ wᵢaᵢ cos(2(θ-θᵢ))] / [(1+γ₂)W]
///
/// When gamma2=0, this reduces to the standard calculate_resonance.
fn resonance_2h(history: &[(f32, f32)], probe: f32, gamma2: f32) -> f32 {
    if history.is_empty() {
        return 0.0;
    }

    let mut sum_h1 = 0.0f32; // first harmonic weighted sum
    let mut sum_h2 = 0.0f32; // second harmonic weighted sum
    let mut w_total = 0.0f32;

    for (i, &(phase, amplitude)) in history.iter().enumerate() {
        let w = recency_weight(i);
        let a = amplitude.max(0.1);
        let weight = w * a;

        let delta = probe - phase;
        sum_h1 += weight * delta.cos();
        sum_h2 += weight * (2.0 * delta).cos();
        w_total += weight;
    }

    if w_total > 0.0 {
        (sum_h1 + gamma2 * sum_h2) / ((1.0 + gamma2) * w_total)
    } else {
        0.0
    }
}

// ═══════════════════════════════════════════════════════════════════
// SUFFICIENT STATISTICS
// ═══════════════════════════════════════════════════════════════════

/// Compute d=4 sufficient statistic: (C1/W, S1/W, C2/W, S2/W, W)
///
/// C1 = Σ wᵢaᵢ cos(θᵢ),  S1 = Σ wᵢaᵢ sin(θᵢ)   — first harmonic
/// C2 = Σ wᵢaᵢ cos(2θᵢ), S2 = Σ wᵢaᵢ sin(2θᵢ)   — second harmonic
/// W  = Σ wᵢaᵢ
fn compute_4d_statistic(history: &[(f32, f32)]) -> (f32, f32, f32, f32, f32) {
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

    if w_total > 0.0 {
        (c1 / w_total, s1 / w_total, c2 / w_total, s2 / w_total, w_total)
    } else {
        (0.0, 0.0, 0.0, 0.0, 0.0)
    }
}

/// Predict resonance from d=2 statistic (first harmonic only)
fn predict_2d(c1w: f32, s1w: f32, gamma2: f32, probe: f32) -> f32 {
    let h1 = probe.cos() * c1w + probe.sin() * s1w;
    h1 / (1.0 + gamma2)
}

/// Predict resonance from d=4 statistic (both harmonics)
fn predict_4d(c1w: f32, s1w: f32, c2w: f32, s2w: f32, gamma2: f32, probe: f32) -> f32 {
    let h1 = probe.cos() * c1w + probe.sin() * s1w;
    let h2 = (2.0 * probe).cos() * c2w + (2.0 * probe).sin() * s2w;
    (h1 + gamma2 * h2) / (1.0 + gamma2)
}

// ═══════════════════════════════════════════════════════════════════
// HISTORY GENERATION (deterministic, same diversity as compression_attack)
// ═══════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════
// MAIN TEST
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_harmonic_hierarchy() {
    let histories = generate_histories();
    let energy_per = 0.05f32;
    let probe_phases: Vec<f32> = (0..8).map(|k| (k as f32 / 8.0) * 2.0 * PI).collect();
    let gamma2_values = [0.0f32, 0.3, 0.5, 0.8, 1.0];

    let mut results: Vec<String> = Vec::new();

    results.push("═══════════════════════════════════════════════════════════════".into());
    results.push("  HARMONIC HIERARCHY: Complexity Hierarchy Demonstration".into());
    results.push("═══════════════════════════════════════════════════════════════".into());
    results.push(format!("  Histories: {}", histories.len()));
    results.push(format!("  Probe phases: {}", probe_phases.len()));
    results.push(format!("  γ₂ values tested: {:?}", gamma2_values));
    results.push(format!(
        "  Tests per γ₂: {}",
        histories.len() * probe_phases.len()
    ));
    results.push("".into());

    results.push("─── CONTROL: γ₂=0 vs original calculate_resonance ───".into());
    let mut control_max_err = 0.0f32;
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
            let two_h = resonance_2h(&hist, probe, 0.0);
            let err = (original - two_h).abs();
            control_max_err = control_max_err.max(err);
        }
    }
    results.push(format!(
        "  Max |original - resonance_2h(γ₂=0)|: {:.2e}",
        control_max_err
    ));
    if control_max_err < 1e-5 {
        results.push("  ✓ Two-harmonic function matches original when γ₂=0".into());
    } else {
        results.push("  ✗ MISMATCH — check implementation".into());
    }
    results.push("".into());

    results.push("─── HIERARCHY RESULTS ───".into());
    results.push(format!(
        "  {:>5} | {:>14} {:>14} | {:>14} {:>14} | {}",
        "γ₂", "d=2 max err", "d=2 mean err", "d=4 max err", "d=4 mean err", "Verdict"
    ));
    results.push(format!("  {}", "-".repeat(95)));

    let mut hierarchy_data: Vec<(f32, f32, f32, f32, f32)> = Vec::new();

    for &gamma2 in &gamma2_values {
        let mut max_err_2d = 0.0f32;
        let mut max_err_4d = 0.0f32;
        let mut sum_err_2d = 0.0f64;
        let mut sum_err_4d = 0.0f64;
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

            let (c1w, s1w, c2w, s2w, _w) = compute_4d_statistic(&hist);

            for &probe in &probe_phases {
                let actual = resonance_2h(&hist, probe, gamma2);
                let pred_2d = predict_2d(c1w, s1w, gamma2, probe);
                let pred_4d = predict_4d(c1w, s1w, c2w, s2w, gamma2, probe);

                let err_2d = (pred_2d - actual).abs();
                let err_4d = (pred_4d - actual).abs();

                max_err_2d = max_err_2d.max(err_2d);
                max_err_4d = max_err_4d.max(err_4d);
                sum_err_2d += err_2d as f64;
                sum_err_4d += err_4d as f64;
                n_tests += 1;
            }
        }

        let mean_err_2d = sum_err_2d / n_tests as f64;
        let mean_err_4d = sum_err_4d / n_tests as f64;

        let verdict = if max_err_4d < 1e-5 && max_err_2d > 0.01 {
            "d=2 FAILS, d=4 WORKS"
        } else if max_err_4d < 1e-5 && max_err_2d < 1e-5 {
            "Both work (no 2nd harmonic content)"
        } else if max_err_4d > 0.01 {
            "UNEXPECTED: d=4 fails — check impl"
        } else {
            "Marginal"
        };

        results.push(format!(
            "  {:>5.1} | {:>14.2e} {:>14.2e} | {:>14.2e} {:>14.2e} | {}",
            gamma2, max_err_2d, mean_err_2d, max_err_4d, mean_err_4d, verdict
        ));

        hierarchy_data.push((
            gamma2,
            max_err_2d,
            mean_err_2d as f32,
            max_err_4d,
            mean_err_4d as f32,
        ));
    }

    results.push("".into());

    results.push("─── ANALYSIS ───".into());

    let gamma0 = hierarchy_data.iter().find(|d| d.0 == 0.0);
    let gamma1 = hierarchy_data.iter().find(|d| d.0 == 1.0);

    if let (Some(g0), Some(g1)) = (gamma0, gamma1) {
        if g0.1 < 1e-5 && g0.3 < 1e-5 {
            results.push("  ✓ γ₂=0 control: d=2 and d=4 both exact (first harmonic only)".into());
        }
        if g1.1 > 0.01 && g1.3 < 1e-5 {
            results.push("  ✓ γ₂=1 hierarchy: d=2 FAILS, d=4 exact".into());
            results.push("".into());
            results.push("  HIERARCHY DEMONSTRATED:".into());
            results.push("    Adding the second harmonic cos(2(θ-θᵢ)) to the kernel".into());
            results.push("    doubles the required sufficient statistic dimension.".into());
            results.push(format!("    d=2 max error at γ₂=1: {:.4}", g1.1));
            results.push(format!("    d=4 max error at γ₂=1: {:.2e}", g1.3));
            results.push("".into());
            results.push("  This empirically confirms the complexity hierarchy:".into());
            results.push("    K=1 harmonic  → d=2".into());
            results.push("    K=2 harmonics → d=4".into());
            results.push("    K harmonics   → d=2K (by extension)".into());
            results.push("    K=∞           → d=∞ (irreducible)".into());
        }
    }

    results.push("".into());
    results.push("─── MATHEMATICAL EXPLANATION ───".into());
    results.push("  The two-harmonic resonance function is:".into());
    results.push("    res(θ) = [Σᵢ wᵢaᵢcos(θ-θᵢ) + γ₂·Σᵢ wᵢaᵢcos(2(θ-θᵢ))] / [(1+γ₂)W]".into());
    results.push("".into());
    results.push("  Expanding both cosines:".into());
    results.push("    cos(θ-θᵢ)   = cos(θ)cos(θᵢ) + sin(θ)sin(θᵢ)".into());
    results.push("    cos(2(θ-θᵢ)) = cos(2θ)cos(2θᵢ) + sin(2θ)sin(2θᵢ)".into());
    results.push("".into());
    results.push("  So: res(θ) = [cos(θ)·C₁ + sin(θ)·S₁ + γ₂(cos(2θ)·C₂ + sin(2θ)·S₂)] / [(1+γ₂)W]".into());
    results.push("".into());
    results.push("  The d=2 statistic (C₁/W, S₁/W) captures only the first two terms.".into());
    results.push("  The d=4 statistic (C₁/W, S₁/W, C₂/W, S₂/W) captures all four.".into());
    results.push("".into());
    results.push("  Each harmonic k adds TWO dimensions to the sufficient statistic:".into());
    results.push("    (Cₖ/W, Sₖ/W) = (Σ wᵢaᵢcos(kθᵢ)/W, Σ wᵢaᵢsin(kθᵢ)/W)".into());
    results.push("".into());
    results.push("  For K harmonics: d = 2K. For K → ∞: d → ∞.".into());
    results.push("  This is the Fourier decomposition of the phase history distribution.".into());
    results.push("  A kernel sensitive to all Fourier modes requires the complete".into());
    results.push("  distribution — no finite summary suffices.".into());

    results.push("".into());
    results.push("─── IMPLICATIONS FOR PAPER #2 ───".into());
    results.push("  This result transforms the complexity hierarchy from a theoretical".into());
    results.push("  prediction into an empirical demonstration. The paper can now state:".into());
    results.push("".into());
    results.push("  'We demonstrate the first step of the complexity hierarchy".into());
    results.push("   computationally. A kernel with K circular harmonics requires a".into());
    results.push("   sufficient statistic of dimension d=2K. We verify this for K=1".into());
    results.push("   (d=2, Section 6) and K=2 (d=4, this section). The extension to".into());
    results.push("   arbitrary K is immediate by the Fourier decomposition argument.".into());
    results.push("   A kernel coupling to all harmonics — equivalent to the full phase".into());
    results.push("   distribution rather than its moments — requires d=∞ and cannot be".into());
    results.push("   reduced to any finite state augmentation.'".into());

    for line in &results {
        println!("{}", line);
    }

    std::fs::create_dir_all("data").unwrap_or_default();
    let mut file =
        std::fs::File::create("data/harmonic_hierarchy_results.txt").expect("Failed to create output file");
    for line in &results {
        writeln!(file, "{}", line).expect("Failed to write");
    }
    println!("\nResults written to: data/harmonic_hierarchy_results.txt");

    assert!(
        hierarchy_data[0].1 < 1e-5,
        "Control failed: d=2 should work when gamma2=0, got max_err={:.2e}",
        hierarchy_data[0].1
    );

    for &(gamma2, _, _, max_4d, _) in &hierarchy_data {
        assert!(
            max_4d < 1e-4,
            "d=4 should always work, but got max_err={:.2e} at gamma2={:.1}",
            max_4d,
            gamma2
        );
    }

    let last = hierarchy_data.last().unwrap();
    assert!(
        last.1 > 0.01,
        "d=2 should fail at gamma2=1.0, but max_err was only {:.2e}",
        last.1
    );
}
