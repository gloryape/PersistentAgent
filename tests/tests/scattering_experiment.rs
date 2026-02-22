//! History-Dependent Scattering Experiment
//!
//! Tests whether voxels with identical present-state observables but different
//! preparation histories produce different interaction physics.
//!
//! This is the falsifiable core of the Topological Causality claim:
//! if the Reflection Axiom is genuinely non-Markovian, scattering depends
//! on worldline homotopy class, not just instantaneous field configuration.

use quaternity_organism::Sanctuary;
use std::f32::consts::PI;

#[test]
fn test_history_dependent_scattering() {
    // ═══════════════════════════════════════════════════════════════════
    // SETUP: Two voxels with identical present state, different histories
    // ═══════════════════════════════════════════════════════════════════

    let mut sanctuary = Sanctuary::new();
    let n_deposits = 20;

    let coherent_coords = (10, 10, 0);
    let incoherent_coords = (20, 20, 0);

    // Build coherent history using inject_beacon (pushes to history)
    let target_phase = PI / 4.0;
    let energy_per = 0.05;
    for i in 0..n_deposits {
        sanctuary.inject_beacon(
            coherent_coords.0,
            coherent_coords.1,
            coherent_coords.2,
            energy_per,
            target_phase,
            i as u64,
        );
    }

    // Build incoherent history: random phases, same energy
    for i in 0..n_deposits {
        let phase = if i < n_deposits - 1 {
            (i as f32 / n_deposits as f32) * 2.0 * PI
        } else {
            target_phase // final deposit matches coherent voxel
        };
        sanctuary.inject_beacon(
            incoherent_coords.0,
            incoherent_coords.1,
            incoherent_coords.2,
            energy_per,
            phase,
            i as u64,
        );
    }

    // ═══════════════════════════════════════════════════════════════════
    // VERIFY: Present-state observables are similar
    // ═══════════════════════════════════════════════════════════════════

    let coh_density = sanctuary.density_at(coherent_coords);
    let inc_density = sanctuary.density_at(incoherent_coords);

    // Both should have similar total amplitude (same number of deposits, same energy)
    let density_diff = (coh_density - inc_density).abs();
    println!("Coherent density:   {:.4}", coh_density);
    println!("Incoherent density: {:.4}", inc_density);
    println!("Density difference: {:.4}", density_diff);

    // Present-state amplitudes should be close (not identical due to phase
    // entrainment effects, but within reasonable tolerance)
    // NOTE: if this assertion fails, the present states aren't comparable
    // and the experiment needs adjustment to equalize amplitudes.
    assert!(
        density_diff < coh_density * 0.3,
        "Present-state amplitudes differ by >30% — can't compare histories. \
         Coherent={:.4}, Incoherent={:.4}",
        coh_density,
        inc_density
    );

    // ═══════════════════════════════════════════════════════════════════
    // MEASURE: Internal resonance (the history-dependent quantity)
    // ═══════════════════════════════════════════════════════════════════

    let coh_voxel = sanctuary
        .get_voxel(coherent_coords)
        .expect("coherent voxel exists");
    let inc_voxel = sanctuary
        .get_voxel(incoherent_coords)
        .expect("incoherent voxel exists");

    // Both probed with the SAME input phase
    let probe_phase = target_phase;
    let coh_resonance = coh_voxel.calculate_resonance(probe_phase);
    let inc_resonance = inc_voxel.calculate_resonance(probe_phase);

    println!("\n--- History-Dependent Resonance ---");
    println!("Coherent resonance:   {:.4}", coh_resonance);
    println!("Incoherent resonance: {:.4}", inc_resonance);
    println!(
        "Resonance difference: {:.4}",
        (coh_resonance - inc_resonance).abs()
    );

    // THE KEY ASSERTION: coherent history should resonate MORE
    // than incoherent history when probed at the same phase.
    // If this fails, history doesn't matter — groupoid collapses to group.
    assert!(
        coh_resonance > inc_resonance,
        "FALSIFIED: Coherent history ({:.4}) does not resonate more than \
         incoherent history ({:.4}). Memory kernel may be reducible to local state.",
        coh_resonance,
        inc_resonance
    );

    // ═══════════════════════════════════════════════════════════════════
    // SCATTERING: Interact with both voxels using identical input
    // ═══════════════════════════════════════════════════════════════════

    // Advance tick past preparation phase
    sanctuary.set_tick((n_deposits + 1) as u64);

    let probe_energy = 1.0;
    let probe_coherence = 0.9;

    let coh_result = sanctuary.interact(
        coherent_coords,
        probe_energy,
        probe_phase,
        probe_coherence as f64,
    );
    let inc_result = sanctuary.interact(
        incoherent_coords,
        probe_energy,
        probe_phase,
        probe_coherence as f64,
    );

    println!("\n--- Scattering Results (identical probe) ---");
    println!(
        "Coherent:   eff={:.4}  res={:.4}  stiff={:.4}  E_eff={:.4}",
        coh_result.efficiency,
        coh_result.resonance,
        coh_result.resistance,
        coh_result.effective_energy
    );
    println!(
        "Incoherent: eff={:.4}  res={:.4}  stiff={:.4}  E_eff={:.4}",
        inc_result.efficiency,
        inc_result.resonance,
        inc_result.resistance,
        inc_result.effective_energy
    );
    println!(
        "Efficiency delta: {:.4}",
        coh_result.efficiency - inc_result.efficiency
    );
    println!(
        "Energy delta:     {:.4}",
        coh_result.effective_energy - inc_result.effective_energy
    );

    // THE SCATTERING ASSERTION: identical present-state observables
    // + identical probe → different results iff history matters.
    let eff_delta = (coh_result.efficiency - inc_result.efficiency).abs();
    assert!(
        eff_delta > 0.01,
        "FALSIFIED: Efficiency difference ({:.6}) is negligible. \
         Scattering does not depend on history. \
         The Reflection Axiom may be reducible to local dynamics.",
        eff_delta
    );

    // If we get here, history-dependent scattering is confirmed.
    println!("\n✓ CONFIRMED: History-dependent scattering detected.");
    println!(
        "  Efficiency delta: {:.4} (>{:.4} threshold)",
        eff_delta, 0.01
    );
    println!("  Coherent preparation produces measurably different interaction");
    println!("  despite similar present-state observables.");

    // ═══════════════════════════════════════════════════════════════════
    // DEEPER TEST: Is this reducible to a single scalar?
    // ═══════════════════════════════════════════════════════════════════
    //
    // If the resonance difference can be fully captured by a single
    // "accumulated coherence" number, then the history buffer is just
    // an expensive way to compute a scalar — no genuine path-dependence.
    //
    // To test this: create a THIRD voxel with the same resonance value
    // as the coherent voxel, but achieved through a DIFFERENT consistent
    // phase (e.g., all deposits at 3π/4 instead of π/4).
    // Then probe both at π/4.
    //
    // If history is truly path-dependent (groupoid), these should differ
    // even though their "coherence scalars" would be identical.
    // If they produce the same result, history reduces to a scalar.

    let alt_coherent_coords = (30, 30, 0);
    let alt_phase = 3.0 * PI / 4.0; // different consistent direction

    for i in 0..n_deposits {
        sanctuary.inject_beacon(
            alt_coherent_coords.0,
            alt_coherent_coords.1,
            alt_coherent_coords.2,
            energy_per,
            alt_phase,
            i as u64,
        );
    }

    let alt_voxel = sanctuary
        .get_voxel(alt_coherent_coords)
        .expect("alt voxel exists");
    let alt_resonance_same_probe = alt_voxel.calculate_resonance(probe_phase);
    let alt_resonance_own_probe = alt_voxel.calculate_resonance(alt_phase);

    println!("\n--- Phase-Direction Test (groupoid vs scalar) ---");
    println!("Coherent @ π/4, probed @ π/4:  res={:.4}", coh_resonance);
    println!(
        "Coherent @ 3π/4, probed @ π/4: res={:.4}",
        alt_resonance_same_probe
    );
    println!(
        "Coherent @ 3π/4, probed @ 3π/4: res={:.4}",
        alt_resonance_own_probe
    );

    // Both have identical "coherence" (all deposits aligned).
    // But probed at π/4, they should differ — because the actual phase
    // direction matters, not just the fact of consistency.
    let direction_delta = (coh_resonance - alt_resonance_same_probe).abs();
    println!("Direction-dependent delta: {:.4}", direction_delta);

    if direction_delta > 0.05 {
        println!("\n✓ STRONG RESULT: Resonance depends on phase direction, not just coherence.");
        println!("  History is genuinely vectorial (path-dependent), not scalar.");
        println!("  The groupoid structure is physically real.");
    } else {
        println!("\n⚠ WEAK RESULT: Direction doesn't matter much.");
        println!("  History may reduce to a scalar coherence measure.");
        println!("  Groupoid structure may be nominal only.");
    }

    // ═══════════════════════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════════════════════

    println!("\n═══════════════════════════════════════════════════");
    println!("  SCATTERING EXPERIMENT SUMMARY");
    println!("═══════════════════════════════════════════════════");
    println!("  Present-state density delta: {:.4}", density_diff);
    println!(
        "  Resonance delta (history):   {:.4}",
        (coh_resonance - inc_resonance).abs()
    );
    println!("  Efficiency delta (scatter):  {:.4}", eff_delta);
    println!("  Direction delta (groupoid):  {:.4}", direction_delta);
    println!("");
    if eff_delta > 0.01 && direction_delta > 0.05 {
        println!("  VERDICT: Non-Markovian physics confirmed.");
        println!("  History AND direction matter. Groupoid is physical.");
    } else if eff_delta > 0.01 {
        println!("  VERDICT: Non-Markovian physics confirmed (weak form).");
        println!("  History matters but may reduce to scalar coherence.");
    } else {
        println!("  VERDICT: Reflection Axiom may be locally reducible.");
        println!("  Further investigation needed.");
    }
    println!("═══════════════════════════════════════════════════");
}

/// Additional test: Does temporal ordering matter?
///
/// Two voxels receive the same set of deposits but in different order.
/// If only the final histogram matters (scalar), they should be identical.
/// If ordering matters (path), they should differ.
#[test]
fn test_temporal_ordering() {
    let mut sanctuary = Sanctuary::new();

    let forward_coords = (50, 50, 0);
    let reverse_coords = (60, 60, 0);

    // Forward: phase sweeps 0 → π
    // Reverse: phase sweeps π → 0
    // Same set of phases, different temporal order.
    let n = 16;
    for i in 0..n {
        let forward_phase = (i as f32 / n as f32) * PI;
        let reverse_phase = ((n - 1 - i) as f32 / n as f32) * PI;

        sanctuary.inject_beacon(
            forward_coords.0,
            forward_coords.1,
            forward_coords.2,
            0.05,
            forward_phase,
            i as u64,
        );
        sanctuary.inject_beacon(
            reverse_coords.0,
            reverse_coords.1,
            reverse_coords.2,
            0.05,
            reverse_phase,
            i as u64,
        );
    }

    // Probe at the midpoint phase
    let probe_phase = PI / 2.0;

    let fwd_voxel = sanctuary.get_voxel(forward_coords).expect("fwd exists");
    let rev_voxel = sanctuary.get_voxel(reverse_coords).expect("rev exists");

    let fwd_res = fwd_voxel.calculate_resonance(probe_phase);
    let rev_res = rev_voxel.calculate_resonance(probe_phase);

    println!("\n--- Temporal Ordering Test ---");
    println!(
        "Forward sweep (0→π) resonance @ π/2: {:.4}",
        fwd_res
    );
    println!(
        "Reverse sweep (π→0) resonance @ π/2: {:.4}",
        rev_res
    );
    println!("Ordering delta: {:.4}", (fwd_res - rev_res).abs());

    // The recency-weighted kernel should produce different resonance
    // because recent deposits weight more heavily.
    // Forward: recent deposits near π → low resonance at π/2
    // Reverse: recent deposits near 0 → low resonance at π/2 (but different profile)
    //
    // Actually: forward ends near π, reverse ends near 0.
    // Probing at π/2, forward's recent history is closer to probe.
    // So forward should resonate slightly more.

    let delta = (fwd_res - rev_res).abs();
    if delta > 0.01 {
        println!("✓ Temporal ordering matters. The kernel is genuinely non-Markovian.");
        println!("  Same phase histogram, different order → different physics.");
    } else {
        println!("⚠ Temporal ordering doesn't matter. Kernel may be order-independent.");
    }
}
