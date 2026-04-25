package dev.gcforge.harness.regimes;

import dev.gcforge.harness.Regime;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;

/**
 * R6 — mixed-gc-pathological.
 *
 * <p>Pre-fills old gen with long-lived chunks in interleaved size classes
 * (small, medium, large) so the resulting fragmentation lowers the
 * effective IHOP. Then sustains a steady allocation rate that keeps the
 * heap pressed against IHOP, forcing G1 into a mixed-GC cycle whose
 * reclaim ratio degrades over time.
 */
public final class MixedGcPathologicalRegime implements Regime {

    private static final Set<String> KNOWN_KEYS = Set.of(
        "old_gen_pressure",
        "fragmentation_factor",
        "survivor_age_target"
    );

    private static final int SMALL_BYTES  = 4 * 1024;
    private static final int MEDIUM_BYTES = 64 * 1024;
    private static final int LARGE_BYTES  = 512 * 1024;

    @Override
    public String id() {
        return "mixed-gc-pathological";
    }

    @Override
    public void run(Map<String, String> params, Duration duration, long seed) {
        rejectUnknownKeys(params);

        double oldGenPressure = parseDouble(params, "old_gen_pressure", 0.7);
        double fragmentation  = parseDouble(params, "fragmentation_factor", 2.0);
        long survivorAge      = parseLong(params, "survivor_age_target", 15L);

        if (!(oldGenPressure > 0.0 && oldGenPressure <= 1.0)) {
            throw new IllegalArgumentException(
                "old_gen_pressure (" + oldGenPressure + ") must be in (0, 1]"
            );
        }
        if (!(fragmentation >= 1.0 && fragmentation <= 3.0)) {
            throw new IllegalArgumentException(
                "fragmentation_factor (" + fragmentation + ") must be in [1.0, 3.0]"
            );
        }
        if (survivorAge < 1 || survivorAge > 15) {
            throw new IllegalArgumentException(
                "survivor_age_target (" + survivorAge + ") must be in [1, 15]"
            );
        }

        Random rng = new Random(seed);

        long maxHeap = Runtime.getRuntime().maxMemory();
        long oldTarget = (long) (maxHeap * oldGenPressure);

        // Build the long-lived pool in interleaved size classes; the
        // fragmentation_factor bumps the share of the larger class so the
        // free-list is rougher.
        List<byte[]> longLived = new ArrayList<>();
        long allocated = 0L;
        while (allocated < oldTarget) {
            double r = rng.nextDouble();
            int sz;
            if (r < 0.6 / fragmentation) {
                sz = SMALL_BYTES;
            } else if (r < 0.6 / fragmentation + 0.3) {
                sz = MEDIUM_BYTES;
            } else {
                sz = LARGE_BYTES;
            }
            byte[] chunk = new byte[sz];
            chunk[0] = (byte) (sz & 0xFF);
            longLived.add(chunk);
            allocated += sz;
        }

        // Steady-state churn: small allocations against the saturated heap.
        Instant deadline = Instant.now().plus(duration);
        List<byte[]> churn = new ArrayList<>(64);
        long pauseNs = 1_000_000L; // 1 ms per allocation
        while (Instant.now().isBefore(deadline)) {
            byte[] chunk = new byte[SMALL_BYTES];
            chunk[0] = (byte) (rng.nextInt() & 0xFF);
            churn.add(chunk);
            if (churn.size() > 64) {
                churn.remove(0);
            }

            try {
                Thread.sleep(pauseNs / 1_000_000L, (int) (pauseNs % 1_000_000L));
            } catch (InterruptedException ie) {
                Thread.currentThread().interrupt();
                break;
            }
        }

        // Anchor.
        if (!longLived.isEmpty()) {
            byte[] tail = longLived.get(longLived.size() - 1);
            if (tail.length > 0) {
                tail[0] = (byte) ((survivorAge) & 0xFF);
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
