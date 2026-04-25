package dev.gcforge.harness.regimes;

import dev.gcforge.harness.Regime;
import dev.gcforge.harness.alloc.ChunkSizer;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.Deque;
import java.util.Map;
import java.util.Random;
import java.util.Set;

/**
 * R7 — microservice-stop-and-go.
 *
 * <p>Alternates between active periods that allocate at
 * {@code active_rate_mb_s} MiB/s and idle periods with no allocation work.
 * On G1, the idle phases tend to host concurrent-mark cycles; on ZGC, the
 * idle/active contrast is gentler because the collector is concurrent
 * across the board.
 */
public final class MicroserviceStopGoRegime implements Regime {

    private static final Set<String> KNOWN_KEYS = Set.of(
        "active_period_s",
        "idle_period_s",
        "active_rate_mb_s",
        "cycles"
    );

    @Override
    public String id() {
        return "microservice-stop-and-go";
    }

    @Override
    public void run(Map<String, String> params, Duration duration, long seed) {
        rejectUnknownKeys(params);

        long activeS    = parseLong(params, "active_period_s",  10L);
        long idleS      = parseLong(params, "idle_period_s",    20L);
        long activeRate = parseLong(params, "active_rate_mb_s", 100L);
        // `cycles` is informational; the count derives from the duration.
        parseCycles(params);

        if (activeS <= 0 || idleS <= 0 || activeRate <= 0) {
            throw new IllegalArgumentException(
                "active_period_s, idle_period_s, and active_rate_mb_s must all be > 0; got "
                    + activeS + ", " + idleS + ", " + activeRate
            );
        }

        Random rng = new Random(seed);
        ChunkSizer sizer = new ChunkSizer(ChunkSizer.Distribution.MIXED, rng);
        long byteBudget = activeRate * 1024L * 1024L;
        long liveSetCap = Math.max(64L * 1024 * 1024, byteBudget); // ~ 1 second of allocs
        Deque<byte[]> liveSet = new ArrayDeque<>();
        long liveSetBytes = 0L;

        long periodS = activeS + idleS;
        Instant start = Instant.now();
        Instant deadline = start.plus(duration);
        Instant phaseStart = start;
        long bytesThisSecond = 0L;
        Instant lastTick = Instant.now();

        while (Instant.now().isBefore(deadline)) {
            long elapsed = Duration.between(start, Instant.now()).toSeconds();
            long phasePos = elapsed % periodS;

            if (phasePos < activeS) {
                // Active: allocate
                int sz = sizer.sample();
                byte[] chunk = new byte[sz];
                chunk[0] = (byte) sz;
                chunk[chunk.length - 1] = (byte) (sz >>> 8);

                liveSet.addLast(chunk);
                liveSetBytes += sz;
                while (liveSetBytes > liveSetCap && !liveSet.isEmpty()) {
                    byte[] evicted = liveSet.removeFirst();
                    liveSetBytes -= evicted.length;
                }

                bytesThisSecond += sz;
                Instant now = Instant.now();
                long elapsedNs = Duration.between(lastTick, now).toNanos();
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
                    lastTick = Instant.now();
                }
            } else {
                // Idle: just sleep. No allocations.
                long remainingPhaseS = periodS - phasePos;
                long sleepMs = Math.min(remainingPhaseS * 1000L, 250L);
                try {
                    Thread.sleep(sleepMs);
                } catch (InterruptedException ie) {
                    Thread.currentThread().interrupt();
                    break;
                }
            }
            // Track phase transitions (informational; no behavioural effect).
            if (phasePos == 0) {
                phaseStart = Instant.now();
            }
        }

        // Anchor.
        if (!liveSet.isEmpty()) {
            byte[] tail = liveSet.peekLast();
            if (tail != null && tail.length > 0) {
                tail[0] = (byte) (rng.nextInt() & 0xFF);
            }
        }
        long _unused = phaseStart.toEpochMilli();
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

    private static void parseCycles(Map<String, String> params) {
        String raw = params.get("cycles");
        if (raw == null || "auto".equalsIgnoreCase(raw)) return;
        try {
            long n = Long.parseLong(raw);
            if (n <= 0) {
                throw new IllegalArgumentException(
                    "cycles (" + n + ") must be > 0 or \"auto\""
                );
            }
        } catch (NumberFormatException e) {
            throw new IllegalArgumentException(
                "cycles=" + raw + " must be a positive integer or \"auto\""
            );
        }
    }
}
