package dev.gcforge.harness.regimes;

import dev.gcforge.harness.alloc.ChunkSizer;
import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Random;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

class SteadyStateRegimeTest {

    @Test
    void runs_with_default_params() {
        SteadyStateRegime regime = new SteadyStateRegime();
        assertTimeoutPreemptively(Duration.ofSeconds(5), () ->
            assertDoesNotThrow(() ->
                regime.run(Map.of(), Duration.ofMillis(300), 42L)
            )
        );
    }

    @Test
    void rejects_unknown_parameter() {
        SteadyStateRegime regime = new SteadyStateRegime();
        Map<String, String> params = Map.of("bogus_knob", "1");
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void rejects_non_integer_allocation_rate() {
        SteadyStateRegime regime = new SteadyStateRegime();
        Map<String, String> params = Map.of("allocation_rate_mb_s", "fast");
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void rejects_unknown_object_size_distribution() {
        SteadyStateRegime regime = new SteadyStateRegime();
        Map<String, String> params = Map.of("object_size_distribution", "huge");
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void rejects_unknown_lifetime_distribution() {
        SteadyStateRegime regime = new SteadyStateRegime();
        Map<String, String> params = Map.of("lifetime_distribution", "weekend");
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void runs_short_distribution_only() {
        SteadyStateRegime regime = new SteadyStateRegime();
        Map<String, String> params = new LinkedHashMap<>();
        params.put("lifetime_distribution", "short");
        params.put("object_size_distribution", "small");
        assertDoesNotThrow(() ->
            regime.run(params, Duration.ofMillis(200), 7L));
    }

    @Test
    void chunk_sizer_is_seed_deterministic() {
        // Two ChunkSizers with the same seed yield the same sequence.
        ChunkSizer a = new ChunkSizer(ChunkSizer.Distribution.MIXED, new Random(123L));
        ChunkSizer b = new ChunkSizer(ChunkSizer.Distribution.MIXED, new Random(123L));
        for (int i = 0; i < 100; i++) {
            assertEquals(a.sample(), b.sample(), "diverged at " + i);
        }
    }

    @Test
    void chunk_sizer_small_stays_in_range() {
        ChunkSizer sizer = new ChunkSizer(ChunkSizer.Distribution.SMALL, new Random(0L));
        for (int i = 0; i < 200; i++) {
            int s = sizer.sample();
            if (s < 16 || s >= 256) {
                throw new AssertionError("small chunk out of range: " + s);
            }
        }
    }

    @Test
    void chunk_sizer_medium_stays_in_range() {
        ChunkSizer sizer = new ChunkSizer(ChunkSizer.Distribution.MEDIUM, new Random(0L));
        for (int i = 0; i < 200; i++) {
            int s = sizer.sample();
            if (s < 256 || s >= 4096) {
                throw new AssertionError("medium chunk out of range: " + s);
            }
        }
    }
}
