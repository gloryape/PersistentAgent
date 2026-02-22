//! Compression Attack: Sufficient Statistic Test
//!
//! Determines whether the Reflection Axiom's memory kernel admits a finite-
//! dimensional sufficient statistic. If (C/W, S/W) — the first circular moment
//! of the recency-weighted history — perfectly predicts scattering at all probe
//! phases, then the groupoid structure embeds into R² and the history buffer is
//! reducible.
//!
//! This is the hardest test the theory faces. An honest result strengthens the
//! paper regardless of outcome.

use quaternity_organism::Sanctuary;
use std::f32::consts::PI;
use std::io::Write;

/// The recency weight function, matching calculate_resonance() exactly.
/// MEMORY_DEPTH = 64, so decay_scale = 64/2 = 32.
fn recency_weight(index: usize) -> f32 {
    (-(index as f32) / 32.0).exp()
}

/// Compute the 2D sufficient statistic (C/W, S/W) from a history buffer.
///
/// Given a sequence of (phase, amplitude) deposits in history order
/// (index 0 = newest, matching VecDeque front), returns:
///   (c_over_w, s_over_w)
///
/// where:
///   W = Σ wᵢ · max(aᵢ, 0.1)
///   C = Σ wᵢ · max(aᵢ, 0.1) · cos(θᵢ)
///   S = Σ wᵢ · max(aᵢ, 0.1) · sin(θᵢ)
///
/// This matches the exact weighting in Voxel::calculate_resonance().
fn compute_2d_statistic(history: &[(f32, f32)]) -> (f32, f32) {
    let mut w_total = 0.0f32;
    let mut c_total = 0.0f32;
    let mut s_total = 0.0f32;

    for (i, &(phase, amplitude)) in history.iter().enumerate() {
        let w = recency_weight(i);
        let a = amplitude.max(0.1);
        let weight = w * a;

        c_total += weight * phase.cos();
        s_total += weight * phase.sin();
        w_total += weight;
    }

    if w_total > 0.0 {
        (c_total / w_total, s_total / w_total)
    } else {
        (0.0, 0.0)
    }
}

/// Predict resonance from the 2D statistic for a given probe phase.
///
/// resonance(θ_in) = cos(θ_in) · (C/W) + sin(θ_in) · (S/W)
fn predict_resonance_2d(c_over_w: f32, s_over_w: f32, probe_phase: f32) -> f32 {
    probe_phase.cos() * c_over_w + probe_phase.sin() * s_over_w
}

/// Compute scalar coherence (1D statistic): ||(C/W, S/W)||
fn compute_1d_statistic(c_over_w: f32, s_over_w: f32) -> f32 {
    (c_over_w * c_over_w + s_over_w * s_over_w).sqrt()
}

/// Generate diverse deterministic histories (no rand dependency).
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
        "alternating_opposed",
        (0..n)
            .map(|i| if i % 2 == 0 { PI / 4.0 } else { 5.0 * PI / 4.0 })
            .collect(),
    ));

    let three_phases = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];
    histories.push((
        "three_phase_cycle",
        (0..n).map(|i| three_phases[i % 3]).collect(),
    ));

    histories.push((
        "clustered_with_outliers",
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

    for seed in 0..20 {
        let mut x: u64 = seed * 12345 + 67890;
        let phases: Vec<f32> = (0..n)
            .map(|_| {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let frac = ((x >> 33) as f32) / (u32::MAX as f32);
                frac * 2.0 * PI
            })
            .collect();
        histories.push(("pseudo_random", phases));
    }

    histories.push((
        "half_and_half",
        (0..n).map(|i| if i < n / 2 { 0.0 } else { PI }).collect(),
    ));

    histories.push((
        "gradual_drift",
        (0..n).map(|i| (i as f32 / n as f32) * PI / 2.0).collect(),
    ));

    histories.push((
        "oscillating",
        (0..n)
            .map(|i| {
                let osc = (i as f32 * 0.5).sin() * (i as f32 / n as f32) * PI;
                PI / 4.0 + osc
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
fn test_compression_attack() {
    let histories = generate_histories();
    let n_deposits = 20;
    let energy_per = 0.05;
    let probe_phases: Vec<f32> = (0..8).map(|k| (k as f32 / 8.0) * 2.0 * PI).collect();
    let probe_energy = 1.0;
    let probe_coherence = 0.9;

    let mut results: Vec<String> = Vec::new();
    results.push(
        "═══════════════════════════════════════════════════════════════".to_string(),
    );
    results.push("  COMPRESSION ATTACK: Sufficient Statistic Test".to_string());
    results.push(
        "═══════════════════════════════════════════════════════════════".to_string(),
    );
    results.push(format!("  Histories tested: {}", histories.len()));
    results.push(format!("  Deposits per history: {}", n_deposits));
    results.push(format!("  Probe phases: {}", probe_phases.len()));
    results.push(format!(
        "  Total prediction tests: {}",
        histories.len() * probe_phases.len()
    ));
    results.push("".to_string());

    let mut max_error_2d: f32 = 0.0;
    let mut max_error_1d: f32 = 0.0;
    let mut total_error_2d: f64 = 0.0;
    let mut total_error_1d: f64 = 0.0;
    let mut n_tests: u64 = 0;

    let mut per_history: Vec<String> = Vec::new();

    let mut max_eff_error_2d: f32 = 0.0;
    let mut total_eff_error_2d: f64 = 0.0;
    let mut n_eff_tests: u64 = 0;

    for (hist_idx, (desc, phases)) in histories.iter().enumerate() {
        let mut sanctuary = Sanctuary::new();
        let coords = (hist_idx as i32, 0, 0);

        for (i, &phase) in phases.iter().enumerate() {
            sanctuary.inject_beacon(coords.0, coords.1, coords.2, energy_per, phase, i as u64);
        }

        let voxel = sanctuary.get_voxel(coords).expect("voxel must exist");

        let history_for_stat: Vec<(f32, f32)> = voxel
            .history
            .iter()
            .map(|s| (s.phase, s.amplitude))
            .collect();

        let (c_over_w, s_over_w) = compute_2d_statistic(&history_for_stat);
        let scalar_coherence = compute_1d_statistic(c_over_w, s_over_w);

        let mut hist_max_err_2d: f32 = 0.0;
        let mut hist_max_err_1d: f32 = 0.0;

        for &probe in &probe_phases {
            let predicted_2d = predict_resonance_2d(c_over_w, s_over_w, probe);
            let actual = voxel.calculate_resonance(probe);

            let error_2d = (predicted_2d - actual).abs();
            max_error_2d = max_error_2d.max(error_2d);
            total_error_2d += error_2d as f64;
            hist_max_err_2d = hist_max_err_2d.max(error_2d);

            let error_1d = (scalar_coherence - actual).abs();
            max_error_1d = max_error_1d.max(error_1d);
            total_error_1d += error_1d as f64;
            hist_max_err_1d = hist_max_err_1d.max(error_1d);

            n_tests += 1;
        }

        let amp_main = sanctuary.density_at(coords);

        for &probe in &probe_phases {
            let mut s2 = Sanctuary::new();
            let c2 = (1000 + hist_idx as i32, 0, 0);
            for (i, &phase) in phases.iter().enumerate() {
                s2.inject_beacon(c2.0, c2.1, c2.2, energy_per, phase, i as u64);
            }
            s2.set_tick(n_deposits as u64 + 1);

            let result = s2.interact(c2, probe_energy, probe, probe_coherence);

            let pred_resonance = predict_resonance_2d(c_over_w, s_over_w, probe);
            let pred_resistance = 0.5 * amp_main * amp_main;
            let pred_base = (probe_energy - pred_resistance).max(0.0);
            let pred_eff_energy = pred_base * (1.0 + 1.5 * pred_resonance);
            let pred_efficiency = if probe_energy > 0.0 {
                pred_eff_energy / probe_energy
            } else {
                0.0
            };

            let eff_error = (pred_efficiency - result.efficiency).abs();
            max_eff_error_2d = max_eff_error_2d.max(eff_error);
            total_eff_error_2d += eff_error as f64;
            n_eff_tests += 1;
        }

        per_history.push(format!(
            "  [{:3}] {:25} | C/W={:+.4} S/W={:+.4} |c|={:.4} | max_err_2d={:.2e} max_err_1d={:.4}",
            hist_idx, desc, c_over_w, s_over_w, scalar_coherence, hist_max_err_2d, hist_max_err_1d
        ));
    }

    let mean_error_2d = total_error_2d / n_tests as f64;
    let mean_error_1d = total_error_1d / n_tests as f64;
    let mean_eff_error_2d = total_eff_error_2d / n_eff_tests as f64;

    results.push("─── RESONANCE PREDICTION (d=2 vs d=1) ───".to_string());
    results.push(format!("  d=2 (C/W, S/W):"));
    results.push(format!(
        "    Max |predicted - actual| resonance: {:.2e}",
        max_error_2d
    ));
    results.push(format!(
        "    Mean |predicted - actual|:          {:.2e}",
        mean_error_2d
    ));
    results.push("".to_string());
    results.push(format!(
        "  d=1 (scalar coherence = ||(C/W, S/W)||):"
    ));
    results.push(format!(
        "    Max |predicted - actual| resonance: {:.4}",
        max_error_1d
    ));
    results.push(format!(
        "    Mean |predicted - actual|:          {:.4}",
        mean_error_1d
    ));
    results.push("".to_string());

    results.push("─── EFFICIENCY PREDICTION (d=2 through full interact()) ───".to_string());
    results.push(format!(
        "  Max |predicted - actual| efficiency: {:.2e}",
        max_eff_error_2d
    ));
    results.push(format!(
        "  Mean |predicted - actual|:           {:.2e}",
        mean_eff_error_2d
    ));
    results.push("".to_string());

    results.push("─── VERDICT ───".to_string());
    if max_error_2d < 1e-5 {
        results.push(
            "  d=2 SUFFICIENT: The pair (C/W, S/W) perfectly predicts resonance.".to_string(),
        );
        results.push("  The history buffer is REDUCIBLE to a 2D complex moment.".to_string());
        results.push("  The groupoid classification is correct but embeds into R².".to_string());
        results.push("".to_string());
        results.push("  IMPLICATION FOR PAPER #2:".to_string());
        results.push(
            "  The current bilinear cosine kernel admits a finite sufficient".to_string(),
        );
        results.push(
            "  statistic. History dependence is real (the 2D moment encodes it)".to_string(),
        );
        results.push(
            "  but the infinite history buffer is unnecessary — the resonance".to_string(),
        );
        results.push("  function depends only on the first circular moment.".to_string());
        results.push("".to_string());
        results.push("  For genuine irreducibility (d = infinity), the kernel would need".to_string());
        results.push("  higher-order phase interactions: cos(k(θ_in - θᵢ)) for k > 1,".to_string());
        results.push("  sequential phase differences (Δθᵢ = θᵢ - θᵢ₋₁), or nonlinear".to_string());
        results.push(
            "  history functionals. These are discussed in the paper as open".to_string(),
        );
        results.push("  directions for kernel enrichment.".to_string());
    } else if max_error_2d < 0.01 {
        results.push(
            "  d=2 APPROXIMATELY SUFFICIENT: Small residual errors detected.".to_string(),
        );
        results.push(
            "  Likely due to floating point or amplitude accumulation effects.".to_string(),
        );
        results.push("  The groupoid embeds approximately but not exactly into R².".to_string());
    } else {
        results.push(
            "  d=2 INSUFFICIENT: The 2D statistic fails to predict resonance.".to_string(),
        );
        results.push(
            "  Higher-dimensional structure exists in the history buffer.".to_string(),
        );
        results.push("  This would support genuine irreducibility of the path groupoid.".to_string());
    }
    results.push("".to_string());

    if max_error_1d > 0.1 {
        results.push(
            "  d=1 INSUFFICIENT: Scalar coherence cannot predict directional resonance.".to_string(),
        );
        results.push(
            "  This confirms Result 2 (directional dependence) from the scattering".to_string(),
        );
        results.push("  experiment: the invariant is vectorial, not scalar.".to_string());
    } else {
        results.push(
            "  d=1 SUFFICIENT (unexpected): Check test logic — this should fail.".to_string(),
        );
    }
    results.push("".to_string());

    results.push("─── PER-HISTORY DETAIL ───".to_string());
    for line in &per_history {
        results.push(line.clone());
    }
    results.push("".to_string());

    results.push("─── MATHEMATICAL EXPLANATION ───".to_string());
    results.push("  The resonance formula is:".to_string());
    results.push("    resonance(θ) = Σᵢ wᵢaᵢcos(θ - θᵢ) / Σᵢ wᵢaᵢ".to_string());
    results.push("".to_string());
    results.push("  Using cos(θ - θᵢ) = cos(θ)cos(θᵢ) + sin(θ)sin(θᵢ):".to_string());
    results.push("    resonance(θ) = cos(θ)·(C/W) + sin(θ)·(S/W)".to_string());
    results.push("".to_string());
    results.push("  where C = Σᵢ wᵢaᵢcos(θᵢ), S = Σᵢ wᵢaᵢsin(θᵢ), W = Σᵢ wᵢaᵢ".to_string());
    results.push("".to_string());
    results.push("  This is a LINEAR function of cos(θ) and sin(θ), parameterized".to_string());
    results.push("  by (C/W, S/W). Therefore the entire history buffer is captured".to_string());
    results.push("  by two real numbers. QED.".to_string());
    results.push("".to_string());
    results.push("  For d=2 to FAIL, the resonance function would need to depend on".to_string());
    results.push("  higher harmonics: cos(2θ), cos(3θ), etc. Each harmonic adds 2".to_string());
    results.push("  dimensions to the sufficient statistic.".to_string());

    for line in &results {
        println!("{}", line);
    }

    std::fs::create_dir_all("data").unwrap_or_default();
    let output_path = "data/compression_attack_results.txt";
    let mut file = std::fs::File::create(output_path).expect("Failed to create output file");
    for line in &results {
        writeln!(file, "{}", line).expect("Failed to write to output file");
    }
    println!("\nResults written to: {}", output_path);

    if max_error_2d > 0.01 {
        println!("\n⚠ UNEXPECTED: d=2 prediction error is large ({:.4})", max_error_2d);
        println!("  This suggests the kernel has higher-order structure.");
        println!("  Check whether amplitude accumulation creates nonlinear effects.");
    }

    assert!(
        max_error_1d > 0.05,
        "d=1 should be insufficient: scalar coherence cannot encode direction. \
         Max error was only {:.6} — check test logic.",
        max_error_1d
    );
}
