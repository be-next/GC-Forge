package dev.gcforge.harness.regimes;

import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

class HumongousPressureRegimeTest {

    @Test
    void runs_with_default_params() {
        HumongousPressureRegime regime = new HumongousPressureRegime();
        assertTimeoutPreemptively(Duration.ofSeconds(5), () ->
            assertDoesNotThrow(() ->
                regime.run(Map.of(), Duration.ofMillis(300), 42L)
            )
        );
    }

    @Test
    void rejects_unknown_parameter() {
        HumongousPressureRegime regime = new HumongousPressureRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("not_a_key", "1"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_ratio() {
        HumongousPressureRegime regime = new HumongousPressureRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("humongous_ratio", "0.0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_ratio_above_one() {
        HumongousPressureRegime regime = new HumongousPressureRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("humongous_ratio", "1.5"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_humongous_size() {
        HumongousPressureRegime regime = new HumongousPressureRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("humongous_size_kb", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void accepts_humongous_size_auto() {
        HumongousPressureRegime regime = new HumongousPressureRegime();
        assertDoesNotThrow(() ->
            regime.run(Map.of("humongous_size_kb", "auto"), Duration.ofMillis(50), 1L));
    }

    @Test
    void accepts_explicit_size_and_ratio() {
        HumongousPressureRegime regime = new HumongousPressureRegime();
        assertDoesNotThrow(() ->
            regime.run(Map.of(
                "humongous_size_kb", "1024",
                "humongous_ratio", "0.7",
                "allocation_rate_mb_s", "40"
            ), Duration.ofMillis(100), 7L));
    }
}
