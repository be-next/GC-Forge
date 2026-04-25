package dev.gcforge.harness.regimes;

import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

class SlowLeakRegimeTest {

    @Test
    void runs_with_default_params() {
        SlowLeakRegime regime = new SlowLeakRegime();
        assertTimeoutPreemptively(Duration.ofSeconds(10), () ->
            assertDoesNotThrow(() ->
                regime.run(Map.of("live_set_initial_mb", "1"), Duration.ofMillis(200), 1L)
            )
        );
    }

    @Test
    void rejects_unknown_parameter() {
        SlowLeakRegime regime = new SlowLeakRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("not_a_key", "1"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_leak_rate() {
        SlowLeakRegime regime = new SlowLeakRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("leak_rate_mb_s", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_negative_leak_rate() {
        SlowLeakRegime regime = new SlowLeakRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("leak_rate_mb_s", "-1"), Duration.ofMillis(50), 1L));
    }

    @Test
    void accepts_explicit_params() {
        SlowLeakRegime regime = new SlowLeakRegime();
        assertDoesNotThrow(() ->
            regime.run(Map.of(
                "leak_rate_mb_s", "1.0",
                "live_set_initial_mb", "1"
            ), Duration.ofMillis(150), 7L));
    }

    @Test
    void rejects_non_numeric_rate() {
        SlowLeakRegime regime = new SlowLeakRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("leak_rate_mb_s", "fast"), Duration.ofMillis(50), 1L));
    }
}
