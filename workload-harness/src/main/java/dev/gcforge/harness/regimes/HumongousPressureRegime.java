package dev.gcforge.harness.regimes;

import dev.gcforge.harness.Regime;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.Deque;
import java.util.Map;
import java.util.Random;
import java.util.Set;

/**
 * R3 — humongous-pressure.
 *
 * <p>At each allocation step, with probability {@code humongous_ratio} the
 * harness allocates a {@code humongous_size_kb}-KiB chunk (large enough to
 * be classified as humongous by G1); otherwise a small 1-KiB allocation.
 * The heap therefore experiences a stream of humongous allocations
 * intermixed with normal traffic.
 */
public final class HumongousPressureRegime implements Regime {

    private static final Set<String> KNOWN_KEYS = Set.of(
        "humongous_ratio",
        "humongous_size_kb",
        "region_size_mb",
        "allocation_rate_mb_s"
    );

    /** Default humongous chunk size when caller passes {@code auto}. */
    private static final int AUTO_HUMONGOUS_KB = 2 * 1024; // 2 MiB

    /** Small companion allocation size (KiB). */
    private static final int SMALL_KB = 1;

    @Override
    public String id() {
        return "humongous-pressure";
    }

    @Override
    public void run(Map<String, String> params, Duration duration, long seed) {
        rejectUnknownKeys(params);

        double ratio = parseDouble(params, "humongous_ratio", 0.5);
        int humongousKb = parseHumongousSize(params, "humongous_size_kb", AUTO_HUMONGOUS_KB);
        long rateMbS = parseLong(params, "allocation_rate_mb_s", 80L);
        // region_size_mb is informational at the harness level (the JVM
        // decides) and accepted only for spec compatibility.
        parseLong(params, "region_size_mb", 0L);

        if (ratio <= 0.0 || ratio > 1.0) {
            throw new IllegalArgumentException(
                "humongous_ratio (" + ratio + ") must be in (0, 1]"
            );
        }
        if (humongousKb <= 0) {
            throw new IllegalArgumentException(
                "humongous_size_kb (" + humongousKb + ") must be > 0"
            );
        }

        Random rng = new Random(seed);

        long byteBudget = rateMbS * 1024L * 1024L;
        long humongousBytes = (long) humongousKb * 1024L;
        long smallBytes = (long) SMALL_KB * 1024L;

        // Live set sized to absorb a few humongous chunks without immediately
        // forcing a full collection cascade. At ratio 0.5 and 80 MiB/s, this
        // keeps a few seconds of humongous traffic resident.
        long liveSetCap = Math.max(64L * 1024 * 1024, 4L * humongousBytes);

        Deque<byte[]> liveSet = new ArrayDeque<>();
        long liveSetBytes = 0L;

        Instant deadline = Instant.now().plus(duration);
        Instant lastTick = Instant.now();
        long bytesThisSecond = 0L;

        while (Instant.now().isBefore(deadline)) {
            boolean humongous = rng.nextDouble() < ratio;
            int sz = (int) (humongous ? humongousBytes : smallBytes);
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

    private static int parseHumongousSize(Map<String, String> params, String key, int fallback) {
        String raw = params.get(key);
        if (raw == null || "auto".equalsIgnoreCase(raw)) {
            return fallback;
        }
        try {
            return Integer.parseInt(raw);
        } catch (NumberFormatException e) {
            throw new IllegalArgumentException(
                "parameter " + key + "=" + raw + " must be a positive integer or \"auto\""
            );
        }
    }
}
