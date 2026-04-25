package dev.gcforge.harness;

import org.junit.jupiter.api.Test;

import java.time.Duration;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

/**
 * Smoke tests for {@link WorkloadHarness}. Iteration 1 only checks that the
 * allocation loop runs to completion without throwing within a generous
 * timeout. Behavioural invariants land in iteration 4 alongside regime R1.
 */
class WorkloadHarnessTest {

    @Test
    void allocationLoopCompletesQuickly() {
        assertTimeoutPreemptively(Duration.ofSeconds(5), () ->
            assertDoesNotThrow(() ->
                WorkloadHarness.runAllocationLoop(
                    Duration.ofMillis(200),  // very short loop
                    10L,                     // 10 MB/s
                    1L,                      // 1 MB live set cap
                    42L                      // deterministic seed
                )
            )
        );
    }

    @Test
    void allocationLoopRespectsZeroDuration() {
        assertDoesNotThrow(() ->
            WorkloadHarness.runAllocationLoop(Duration.ZERO, 50L, 10L, 0L)
        );
    }
}
