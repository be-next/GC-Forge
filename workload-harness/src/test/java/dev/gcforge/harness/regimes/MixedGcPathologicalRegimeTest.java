package dev.gcforge.harness.regimes;

import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

class MixedGcPathologicalRegimeTest {

    @Test
    void runs_with_low_pressure() {
        // Default 0.7 of the test JVM's heap could be huge; lower it.
        MixedGcPathologicalRegime regime = new MixedGcPathologicalRegime();
        assertTimeoutPreemptively(Duration.ofSeconds(10), () ->
            assertDoesNotThrow(() ->
                regime.run(Map.of(
                    "old_gen_pressure", "0.05",
                    "fragmentation_factor", "1.5"
                ), Duration.ofMillis(200), 42L)
            )
        );
    }

    @Test
    void rejects_unknown_parameter() {
        MixedGcPathologicalRegime regime = new MixedGcPathologicalRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("not_a_key", "1"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_pressure() {
        MixedGcPathologicalRegime regime = new MixedGcPathologicalRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("old_gen_pressure", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_pressure_above_one() {
        MixedGcPathologicalRegime regime = new MixedGcPathologicalRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("old_gen_pressure", "1.5"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_fragmentation_below_one() {
        MixedGcPathologicalRegime regime = new MixedGcPathologicalRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("fragmentation_factor", "0.5"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_fragmentation_above_three() {
        MixedGcPathologicalRegime regime = new MixedGcPathologicalRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("fragmentation_factor", "3.5"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_survivor_age_too_high() {
        MixedGcPathologicalRegime regime = new MixedGcPathologicalRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("survivor_age_target", "16"), Duration.ofMillis(50), 1L));
    }
}
