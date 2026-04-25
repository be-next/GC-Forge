package dev.gcforge.harness.regimes;

import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

class AllocationBurstRegimeTest {

    @Test
    void runs_with_default_params() {
        AllocationBurstRegime regime = new AllocationBurstRegime();
        assertTimeoutPreemptively(Duration.ofSeconds(5), () ->
            assertDoesNotThrow(() ->
                regime.run(Map.of(), Duration.ofMillis(300), 42L)
            )
        );
    }

    @Test
    void rejects_unknown_parameter() {
        AllocationBurstRegime regime = new AllocationBurstRegime();
        Map<String, String> params = Map.of("not_a_key", "1");
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void rejects_burst_below_base_rate() {
        AllocationBurstRegime regime = new AllocationBurstRegime();
        Map<String, String> params = Map.of(
            "base_rate_mb_s", "100",
            "burst_rate_mb_s", "50"
        );
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void rejects_burst_duration_above_period() {
        AllocationBurstRegime regime = new AllocationBurstRegime();
        Map<String, String> params = Map.of(
            "burst_duration_s", "10",
            "burst_period_s", "5"
        );
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void rejects_zero_period() {
        AllocationBurstRegime regime = new AllocationBurstRegime();
        Map<String, String> params = Map.of("burst_period_s", "0");
        assertThrows(IllegalArgumentException.class, () ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void accepts_bursts_count_auto_keyword() {
        AllocationBurstRegime regime = new AllocationBurstRegime();
        Map<String, String> params = Map.of("bursts_count", "auto");
        assertDoesNotThrow(() ->
            regime.run(params, Duration.ofMillis(100), 1L));
    }

    @Test
    void rate_schedule_burst_phase() {
        // Burst window covers seconds 0..5 within a 30s period.
        long base = 30L;
        long burst = 200L;
        long burstDur = 5L;
        long burstPer = 30L;

        // Inside the burst window — burst rate.
        assertEquals(burst, AllocationBurstRegime.rateAt(0, burstDur, burstPer, base, burst));
        assertEquals(burst, AllocationBurstRegime.rateAt(2, burstDur, burstPer, base, burst));
        assertEquals(burst, AllocationBurstRegime.rateAt(4, burstDur, burstPer, base, burst));
        // Outside — base rate.
        assertEquals(base, AllocationBurstRegime.rateAt(5, burstDur, burstPer, base, burst));
        assertEquals(base, AllocationBurstRegime.rateAt(15, burstDur, burstPer, base, burst));
        assertEquals(base, AllocationBurstRegime.rateAt(29, burstDur, burstPer, base, burst));
        // Wraps around to the next burst at second 30.
        assertEquals(burst, AllocationBurstRegime.rateAt(30, burstDur, burstPer, base, burst));
        assertEquals(burst, AllocationBurstRegime.rateAt(31, burstDur, burstPer, base, burst));
    }

    @Test
    void rate_schedule_handles_zero_period() {
        assertEquals(30L, AllocationBurstRegime.rateAt(0, 5, 0, 30, 200));
        assertEquals(30L, AllocationBurstRegime.rateAt(100, 5, 0, 30, 200));
    }
}
