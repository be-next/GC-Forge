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
 * R2 — allocation-burst.
 *
 * <p>Alternates between a base allocation rate and a higher burst rate on a
 * fixed schedule. The harness expects to see the GC log "pulse" at the burst
 * cadence: young collections frequency rises during the burst window and
 * returns to baseline within ~ 2× burst duration.
 */
public final class AllocationBurstRegime implements Regime {

    private static final Set<String> KNOWN_KEYS = Set.of(
        "base_rate_mb_s",
        "burst_rate_mb_s",
        "burst_duration_s",
        "burst_period_s",
        "bursts_count" // accepted, currently informational
    );

    @Override
    public String id() {
        return "allocation-burst";
    }

    @Override
    public void run(Map<String, String> params, Duration duration, long seed) {
        rejectUnknownKeys(params);

        long baseRate   = parseLong(params, "base_rate_mb_s", 30L);
        long burstRate  = parseLong(params, "burst_rate_mb_s", 200L);
        long burstDur   = parseLong(params, "burst_duration_s", 5L);
        long burstPer   = parseLong(params, "burst_period_s", 30L);

        if (burstRate < baseRate) {
            throw new IllegalArgumentException(
                "burst_rate_mb_s (" + burstRate + ") must be >= base_rate_mb_s (" + baseRate + ")"
            );
        }
        if (burstDur <= 0 || burstPer <= 0 || burstDur > burstPer) {
            throw new IllegalArgumentException(
                "burst_duration_s (" + burstDur + ") and burst_period_s (" + burstPer
                    + ") must be > 0 with duration <= period"
            );
        }

        Random rng = new Random(seed);
        ChunkSizer sizer = new ChunkSizer(ChunkSizer.Distribution.MIXED, rng);

        // Live set is implicit at ~ 2× burst budget so the heap can absorb a
        // burst without forcing evacuation failures.
        long liveSetCap = Math.max(64L * 1024 * 1024, 2L * burstRate * 1024 * 1024);
        Deque<byte[]> liveSet = new ArrayDeque<>();
        long liveSetBytes = 0L;

        Instant start = Instant.now();
        Instant deadline = start.plus(duration);
        Instant lastTick = Instant.now();
        long bytesThisSecond = 0L;

        while (Instant.now().isBefore(deadline)) {
            long elapsedS = Duration.between(start, Instant.now()).toSeconds();
            long phasePos = elapsedS % burstPer;
            long currentRateMb = (phasePos < burstDur) ? burstRate : baseRate;
            long byteBudget = currentRateMb * 1024L * 1024L;

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
        }

        // Anchor the live set so the JIT can't elide it.
        if (!liveSet.isEmpty()) {
            byte[] tail = liveSet.peekLast();
            if (tail != null && tail.length > 0) {
                tail[0] = (byte) (rng.nextInt() & 0xFF);
            }
        }
    }

    /**
     * Returns the rate (in MiB/s) the regime would use at {@code elapsedSeconds}
     * into the run. Visible for tests so the schedule can be verified without
     * actually allocating anything.
     */
    public static long rateAt(long elapsedSeconds, long burstDurationS, long burstPeriodS,
                              long baseRateMbS, long burstRateMbS) {
        if (burstPeriodS <= 0) {
            return baseRateMbS;
        }
        long phase = ((elapsedSeconds % burstPeriodS) + burstPeriodS) % burstPeriodS;
        return phase < burstDurationS ? burstRateMbS : baseRateMbS;
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
        if (raw == null || "auto".equalsIgnoreCase(raw)) {
            return fallback;
        }
        try {
            return Long.parseLong(raw);
        } catch (NumberFormatException e) {
            throw new IllegalArgumentException(
                "parameter " + key + "=" + raw + " is not a valid integer"
            );
        }
    }
}
