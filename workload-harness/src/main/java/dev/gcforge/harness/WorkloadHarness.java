package dev.gcforge.harness;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/**
 * GC-Forge workload harness — entry point.
 *
 * <p>Iteration 1 (bootstrap): minimal allocation loop sufficient to produce a
 * non-trivial GC log on Temurin 17/21. Real regimes plug in starting iteration 4
 * via {@code RegimeRegistry} (see SPEC-TECHNIQUE §4.4).
 *
 * <p>The loop allocates byte arrays at a configurable rate for a configurable
 * duration, holding a bounded live-set so that young collections fire and the
 * heap reaches a steady state.
 */
public final class WorkloadHarness {

    private WorkloadHarness() {
        // utility class
    }

    /**
     * Entry point. Recognised arguments (positional, all optional):
     * <ol>
     *   <li>{@code duration} — ISO-8601 (e.g. {@code PT30S}). Default {@code PT10S}.</li>
     *   <li>{@code allocationRateMbPerSec} — long. Default {@code 50}.</li>
     *   <li>{@code liveSetMb} — long. Default {@code 50}.</li>
     *   <li>{@code seed} — long. Default {@code 0}.</li>
     * </ol>
     */
    public static void main(String[] args) {
        Duration duration = args.length > 0 ? Duration.parse(args[0]) : Duration.ofSeconds(10);
        long rateMbPerSec = args.length > 1 ? Long.parseLong(args[1]) : 50L;
        long liveSetMb    = args.length > 2 ? Long.parseLong(args[2]) : 50L;
        long seed         = args.length > 3 ? Long.parseLong(args[3]) : 0L;

        runAllocationLoop(duration, rateMbPerSec, liveSetMb, seed);
    }

    /**
     * Steady-state allocation loop.
     *
     * <p>Allocates {@code chunkBytes}-sized {@code byte[]} chunks at the
     * requested rate, retaining the most recent ones until the live set
     * approximates {@code liveSetMb}. The youngest entries are evicted FIFO
     * once the cap is reached, so the live set stabilises and old gen pressure
     * stays bounded — the canonical "steady-state-healthy" shape of regime R1.
     */
    static void runAllocationLoop(Duration duration, long rateMbPerSec, long liveSetMb, long seed) {
        final int chunkBytes = 64 * 1024;          // 64 KiB chunks
        final long bytesPerSec = rateMbPerSec * 1024L * 1024L;
        final long pauseMicros = bytesPerSec == 0 ? 1_000 : 1_000_000L * chunkBytes / bytesPerSec;
        final int liveSetCap = (int) Math.max(1L, liveSetMb * 1024L * 1024L / chunkBytes);

        Random rng = new Random(seed);
        List<byte[]> liveSet = new ArrayList<>(liveSetCap + 1);
        Instant deadline = Instant.now().plus(duration);

        while (Instant.now().isBefore(deadline)) {
            byte[] chunk = new byte[chunkBytes];
            // Touch a few cells so the JVM cannot elide the allocation.
            chunk[0] = (byte) rng.nextInt();
            chunk[chunk.length - 1] = (byte) rng.nextInt();

            liveSet.add(chunk);
            if (liveSet.size() > liveSetCap) {
                liveSet.remove(0);
            }

            sleepMicros(pauseMicros);
        }
    }

    private static void sleepMicros(long micros) {
        if (micros <= 0) {
            return;
        }
        try {
            long millis = micros / 1_000;
            int nanos   = (int) ((micros % 1_000) * 1_000);
            Thread.sleep(millis, nanos);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
