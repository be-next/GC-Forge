package dev.gcforge.harness;

import org.junit.jupiter.api.Test;

import java.time.Duration;
import java.util.Map;
import java.util.NoSuchElementException;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** Tests for {@link WorkloadHarness#parse} and the no-args fallback. */
class WorkloadHarnessTest {

    @Test
    void no_args_falls_back_to_default_steady_state() {
        WorkloadHarness.Invocation inv = WorkloadHarness.parse(new String[]{});
        assertEquals("steady-state-healthy", inv.kind);
        assertEquals(Duration.ofSeconds(10), inv.duration);
        assertEquals(0x00C0_FFEEL, inv.seed);
        assertTrue(inv.params.isEmpty(), "params = " + inv.params);
    }

    @Test
    void parses_full_argv() {
        WorkloadHarness.Invocation inv = WorkloadHarness.parse(new String[]{
            "steady-state-healthy", "PT30S", "0xC0FFEE",
            "allocation_rate_mb_s=50", "live_set_mb=100"
        });
        assertEquals("steady-state-healthy", inv.kind);
        assertEquals(Duration.ofSeconds(30), inv.duration);
        assertEquals(0x00C0_FFEEL, inv.seed);
        assertEquals(Map.of("allocation_rate_mb_s", "50", "live_set_mb", "100"), inv.params);
    }

    @Test
    void seed_accepts_decimal() {
        WorkloadHarness.Invocation inv = WorkloadHarness.parse(new String[]{
            "steady-state-healthy", "PT5S", "12345"
        });
        assertEquals(12_345L, inv.seed);
    }

    @Test
    void rejects_short_argv() {
        assertThrows(IllegalArgumentException.class,
            () -> WorkloadHarness.parse(new String[]{"steady-state-healthy"}));
    }

    @Test
    void rejects_bad_duration() {
        assertThrows(IllegalArgumentException.class,
            () -> WorkloadHarness.parse(new String[]{
                "steady-state-healthy", "30s", "0xC0FFEE"
            }));
    }

    @Test
    void rejects_param_without_equals() {
        assertThrows(IllegalArgumentException.class,
            () -> WorkloadHarness.parse(new String[]{
                "steady-state-healthy", "PT5S", "1", "bare_token"
            }));
    }

    @Test
    void registry_contains_steady_state() {
        assertTrue(RegimeRegistry.contains("steady-state-healthy"));
    }

    @Test
    void registry_throws_on_unknown_kind() {
        assertThrows(NoSuchElementException.class,
            () -> RegimeRegistry.lookup("bogus"));
    }

    @Test
    void no_args_main_runs_to_completion() {
        // We can't easily call main() without exiting; just exercise the
        // fallback path through parse + lookup + run.
        WorkloadHarness.Invocation inv = WorkloadHarness.parse(new String[]{});
        Regime regime = RegimeRegistry.lookup(inv.kind);
        assertTimeoutPreemptively(Duration.ofSeconds(15), () ->
            assertDoesNotThrow(() ->
                regime.run(inv.params, Duration.ofMillis(500), inv.seed)
            )
        );
    }
}
