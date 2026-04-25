package dev.gcforge.harness.regimes;

import dev.gcforge.harness.Regime;
import dev.gcforge.harness.alloc.ChunkSizer;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Deque;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;

/**
 * R1 — steady-state-healthy.
 *
 * <p>Allocates byte arrays at a configurable mean rate, retaining a bounded
 * live set so that young collections fire and the heap reaches a steady
 * state. The live set is kept FIFO. With {@code lifetime_distribution=mixed},
 * a small fraction of allocations are promoted to a long-lived survivor pool,
 * driving promotion to old gen as the spec asks for.
 */
public final class SteadyStateRegime implements Regime {

    private static final Set<String> KNOWN_KEYS = Set.of(
        "allocation_rate_mb_s",
        "live_set_mb",
        "object_size_distribution",
        "lifetime_distribution"
    );

    @Override
    public String id() {
        return "steady-state-healthy";
    }

    @Override
    public void run(Map<String, String> params, Duration duration, long seed) {
        rejectUnknownKeys(params);

        long allocRateMbS = parseLong(params, "allocation_rate_mb_s", 50L);
        long liveSetMb    = parseLong(params, "live_set_mb",          100L);
        ChunkSizer.Distribution sizeDist = ChunkSizer.Distribution.parse(
            params.getOrDefault("object_size_distribution", "mixed")
        );
        LifetimeDistribution lifetime = LifetimeDistribution.parse(
            params.getOrDefault("lifetime_distribution", "mixed")
        );

        Random rng = new Random(seed);
        ChunkSizer sizer = new ChunkSizer(sizeDist, rng);

        long byteBudget = allocRateMbS * 1024L * 1024L;     // bytes per second target
        long liveSetCap = liveSetMb * 1024L * 1024L;         // bytes
        // The survivor pool holds long-lived chunks (only meaningful for `mixed`).
        long survivorBudget = lifetime == LifetimeDistribution.MIXED
            ? Math.max(1, liveSetCap / 10)
            : 0L;

        Deque<byte[]> youngLive = new ArrayDeque<>();
        long youngLiveBytes = 0L;
        List<byte[]> survivors = new ArrayList<>();
        long survivorsBytes = 0L;

        Instant deadline = Instant.now().plus(duration);
        long lastTick = System.nanoTime();
        long bytesThisSecond = 0L;

        while (Instant.now().isBefore(deadline)) {
            int sz = sizer.sample();
            byte[] chunk = new byte[sz];
            // Touch a couple of cells so the JVM cannot elide the allocation.
            chunk[0] = (byte) sz;
            chunk[chunk.length - 1] = (byte) (sz >>> 8);

            // Promote a fraction of allocations to the survivor pool.
            if (lifetime == LifetimeDistribution.MIXED && rng.nextDouble() < 0.10
                && survivorsBytes + sz <= survivorBudget) {
                survivors.add(chunk);
                survivorsBytes += sz;
            } else {
                youngLive.addLast(chunk);
                youngLiveBytes += sz;
                while (youngLiveBytes > liveSetCap && !youngLive.isEmpty()) {
                    byte[] evicted = youngLive.removeFirst();
                    youngLiveBytes -= evicted.length;
                }
            }

            bytesThisSecond += sz;
            long now = System.nanoTime();
            long elapsedNs = now - lastTick;
            if (elapsedNs >= 1_000_000_000L) {
                bytesThisSecond = 0L;
                lastTick = now;
            } else if (bytesThisSecond >= byteBudget) {
                long sleepNs = 1_000_000_000L - elapsedNs;
                long sleepMs = sleepNs / 1_000_000L;
                int sleepNanos = (int) (sleepNs % 1_000_000L);
                try {
                    if (sleepMs > 0 || sleepNanos > 0) {
                        Thread.sleep(sleepMs, sleepNanos);
                    }
                } catch (InterruptedException ie) {
                    Thread.currentThread().interrupt();
                    break;
                }
                bytesThisSecond = 0L;
                lastTick = System.nanoTime();
            }
        }

        // Touch the survivor pool one more time so the JIT can't decide it
        // was dead.
        if (!survivors.isEmpty()) {
            int idx = rng.nextInt(survivors.size());
            byte[] b = survivors.get(idx);
            if (b.length > 0) {
                Arrays.fill(b, 0, 1, (byte) 1);
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

    enum LifetimeDistribution {
        SHORT, MIXED;

        public static LifetimeDistribution parse(String s) {
            return switch (s.toLowerCase()) {
                case "short" -> SHORT;
                case "mixed" -> MIXED;
                default -> throw new IllegalArgumentException(
                    "unknown lifetime_distribution " + s + " (expected: short, mixed)"
                );
            };
        }
    }
}
