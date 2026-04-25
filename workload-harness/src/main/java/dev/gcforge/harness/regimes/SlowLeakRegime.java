package dev.gcforge.harness.regimes;

import dev.gcforge.harness.Regime;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Set;

/**
 * R4 — slow-leak.
 *
 * <p>Maintains a never-evicted reference list that grows at the configured
 * leak rate. The live-set after each GC therefore climbs linearly with time;
 * mixed GCs become more frequent and full GCs eventually fire as the heap
 * tightens. If the run is long enough relative to the heap size, the JVM
 * throws {@link OutOfMemoryError} and exits non-zero.
 */
public final class SlowLeakRegime implements Regime {

    private static final Set<String> KNOWN_KEYS = Set.of(
        "leak_rate_mb_s",
        "live_set_initial_mb"
    );

    /** Chunk size used to bump the leak. 64 KiB so each tick is cheap. */
    private static final int CHUNK_BYTES = 64 * 1024;

    @Override
    public String id() {
        return "slow-leak";
    }

    @Override
    public void run(Map<String, String> params, Duration duration, long seed) {
        rejectUnknownKeys(params);

        double leakRate = parseDouble(params, "leak_rate_mb_s", 0.5);
        long initialMb  = parseLong(params, "live_set_initial_mb", 200L);

        if (leakRate <= 0.0) {
            throw new IllegalArgumentException(
                "leak_rate_mb_s (" + leakRate + ") must be > 0"
            );
        }
        if (initialMb < 0) {
            throw new IllegalArgumentException(
                "live_set_initial_mb (" + initialMb + ") must be ≥ 0"
            );
        }

        long _seed = seed; // reserved for future deterministic chunk fill

        // Pre-allocate the initial live set as a single contiguous list of
        // chunks; this gives the heap a stable baseline before the leak.
        List<byte[]> liveSet = new ArrayList<>();
        long initialBytes = initialMb * 1024L * 1024L;
        long allocatedInitial = 0L;
        while (allocatedInitial < initialBytes) {
            byte[] chunk = new byte[CHUNK_BYTES];
            chunk[0] = (byte) (_seed & 0xFF);
            liveSet.add(chunk);
            allocatedInitial += CHUNK_BYTES;
        }

        long bytesPerSec = (long) (leakRate * 1024.0 * 1024.0);
        long pauseNs = bytesPerSec == 0 ? 1_000_000L
                                        : 1_000_000_000L * CHUNK_BYTES / bytesPerSec;

        Instant deadline = Instant.now().plus(duration);
        while (Instant.now().isBefore(deadline)) {
            byte[] chunk = new byte[CHUNK_BYTES];
            chunk[0] = (byte) (liveSet.size() & 0xFF);
            liveSet.add(chunk);

            if (pauseNs > 0) {
                long sleepMs = pauseNs / 1_000_000L;
                int sleepNanos = (int) (pauseNs % 1_000_000L);
                try {
                    if (sleepMs > 0 || sleepNanos > 0) {
                        Thread.sleep(sleepMs, sleepNanos);
                    }
                } catch (InterruptedException ie) {
                    Thread.currentThread().interrupt();
                    break;
                }
            }
        }

        // Anchor so the JIT cannot decide the leak is dead.
        if (!liveSet.isEmpty()) {
            byte[] tail = liveSet.get(liveSet.size() - 1);
            if (tail.length > 0) {
                tail[0] = (byte) ((_seed) & 0xFF);
            }
        }
    }

    private void rejectUnknownKeys(Map<String, String> params) {
        for (String key : params.keySet()) {
            if (!KNOWN_KEYS.contains(key)) {
                throw new IllegalArgumentException(
                    "unknown parameter " + key + " for regime " + id()
                        + " (expected one of " + KNOWN_KEYS + ")"
                );
            }
        }
    }

    private static double parseDouble(Map<String, String> params, String key, double fallback) {
        String raw = params.get(key);
        if (raw == null) return fallback;
        try {
            return Double.parseDouble(raw);
        } catch (NumberFormatException e) {
            throw new IllegalArgumentException(
                "parameter " + key + "=" + raw + " is not a valid number"
            );
        }
    }

    private static long parseLong(Map<String, String> params, String key, long fallback) {
        String raw = params.get(key);
        if (raw == null) return fallback;
        try {
            return Long.parseLong(raw);
        } catch (NumberFormatException e) {
            throw new IllegalArgumentException(
                "parameter " + key + "=" + raw + " is not a valid integer"
            );
        }
    }
}
