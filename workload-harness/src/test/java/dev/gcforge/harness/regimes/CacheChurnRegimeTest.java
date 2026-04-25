package dev.gcforge.harness.regimes;

import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

class CacheChurnRegimeTest {

    @Test
    void runs_with_default_params() {
        CacheChurnRegime regime = new CacheChurnRegime();
        assertTimeoutPreemptively(Duration.ofSeconds(5), () ->
            assertDoesNotThrow(() ->
                regime.run(Map.of(), Duration.ofMillis(200), 42L)
            )
        );
    }

    @Test
    void rejects_unknown_parameter() {
        CacheChurnRegime regime = new CacheChurnRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("not_a_key", "1"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_cache_size() {
        CacheChurnRegime regime = new CacheChurnRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("cache_size_mb", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_eviction_rate() {
        CacheChurnRegime regime = new CacheChurnRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("eviction_rate_per_s", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_lifetime() {
        CacheChurnRegime regime = new CacheChurnRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("entry_lifetime_ms", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void rejects_zero_entry_size() {
        CacheChurnRegime regime = new CacheChurnRegime();
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(Map.of("entry_size_kb", "0"), Duration.ofMillis(50), 1L));
    }

    @Test
    void accepts_explicit_params() {
        CacheChurnRegime regime = new CacheChurnRegime();
        assertDoesNotThrow(() ->
            regime.run(Map.of(
                "cache_size_mb", "8",
                "eviction_rate_per_s", "100",
                "entry_lifetime_ms", "200",
                "entry_size_kb", "4"
            ), Duration.ofMillis(150), 7L));
    }
}
