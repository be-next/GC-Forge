package dev.gcforge.harness.regimes;

import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

class MicroserviceStopGoRegimeTest {

    @Test
    void runs_with_short_period_overrides() {
        MicroserviceStopGoRegime regime = new MicroserviceStopGoRegime();
        assertTimeoutPreemptively(Duration.ofSeconds(5), () ->
            assertDoesNotThrow(() ->
                regime.run(Map.of(
                    "active_period_s", "1",
                    "idle_period_s", "1",
                    "active_rate_mb_s", "10"
                ), Duration.ofMillis(300), 42L)
            )
        );
    }

    @Test
    void rejects_unknown_parameter() {
        MicroserviceStopGoRegime regime = new MicroserviceStopGoRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("not_a_key", "1"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_active_period() {
        MicroserviceStopGoRegime regime = new MicroserviceStopGoRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("active_period_s", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_idle_period() {
        MicroserviceStopGoRegime regime = new MicroserviceStopGoRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("idle_period_s", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_active_rate() {
        MicroserviceStopGoRegime regime = new MicroserviceStopGoRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("active_rate_mb_s", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void accepts_cycles_auto_keyword() {
        MicroserviceStopGoRegime regime = new MicroserviceStopGoRegime();
        assertDoesNotThrow(() ->
            regime.run(Map.of(
                "active_period_s", "1",
                "idle_period_s", "1",
                "active_rate_mb_s", "10",
                "cycles", "auto"
            ), Duration.ofMillis(150), 1L));
    }

    @Test
    void accepts_explicit_cycles() {
        MicroserviceStopGoRegime regime = new MicroserviceStopGoRegime();
        assertDoesNotThrow(() ->
            regime.run(Map.of(
                "active_period_s", "1",
                "idle_period_s", "1",
                "active_rate_mb_s", "10",
                "cycles", "5"
            ), Duration.ofMillis(150), 1L));
    }

    @Test
    void rejects_negative_cycles() {
        MicroserviceStopGoRegime regime = new MicroserviceStopGoRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("cycles", "-1"), Duration.ofMillis(50), 1L));
    }
}
