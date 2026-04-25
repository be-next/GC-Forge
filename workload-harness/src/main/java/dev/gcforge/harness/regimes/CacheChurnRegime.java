package dev.gcforge.harness.regimes;

import dev.gcforge.harness.Regime;

import java.time.Duration;
import java.time.Instant;
import java.util.ArrayDeque;
import java.util.Deque;
import java.util.Map;
import java.util.Set;

/**
 * R5 — cache-churn.
 *
 * <p>Models a long-lived, FIFO-evicted in-memory cache: each entry is
 * promoted to the survivor pool, lives for {@code entry_lifetime_ms}, then
 * is evicted. The tenuring distribution drives a steady promotion rate
 * from young to old.
 */
public final class CacheChurnRegime implements Regime {

    private static final Set<String> KNOWN_KEYS = Set.of(
        "cache_size_mb",
        "eviction_rate_per_s",
        "entry_lifetime_ms",
        "entry_size_kb"
    );

    @Override
    public String id() {
        return "cache-churn";
    }

    @Override
    public void run(Map<String, String> params, Duration duration, long seed) {
        rejectUnknownKeys(params);

        long cacheMb     = parseLong(params, "cache_size_mb",       500L);
        long evictPerSec = parseLong(params, "eviction_rate_per_s", 1000L);
        long lifetimeMs  = parseLong(params, "entry_lifetime_ms",   2000L);
        long entryKb     = parseLong(params, "entry_size_kb",       8L);

        if (cacheMb <= 0 || evictPerSec <= 0 || lifetimeMs <= 0 || entryKb <= 0) {
            throw new IllegalArgumentException(
                "all cache-churn parameters must be > 0; got"
                    + " cache_size_mb=" + cacheMb
                    + " eviction_rate_per_s=" + evictPerSec
                    + " entry_lifetime_ms=" + lifetimeMs
                    + " entry_size_kb=" + entryKb
            );
        }

        long entryBytes = entryKb * 1024L;
        long cacheBytes = cacheMb * 1024L * 1024L;
        long maxEntries = cacheBytes / entryBytes;
        if (maxEntries == 0) {
            maxEntries = 1;
        }

        // FIFO of cache entries with their insertion timestamps.
        Deque<byte[]> entries = new ArrayDeque<>();
        Deque<Long> timestamps = new ArrayDeque<>();

        long pauseNanos = 1_000_000_000L / evictPerSec;
        // We allocate one new entry per tick; the inverse of the eviction
        // rate matches the steady-state insertion rate after the cache
        // fills up.

        long _seed = seed; // retained for future deterministic behaviour
        Instant deadline = Instant.now().plus(duration);

        while (Instant.now().isBefore(deadline)) {
            byte[] chunk = new byte[(int) entryBytes];
            chunk[0] = (byte) entries.size();
            chunk[chunk.length - 1] = (byte) (entries.size() >>> 8);

            long now = System.currentTimeMillis();
            entries.addLast(chunk);
            timestamps.addLast(now);

            // Evict entries whose lifetime has expired, or whose presence
            // pushes us over the cache's nominal size.
            while (!timestamps.isEmpty()
                && (now - timestamps.peekFirst() >= lifetimeMs
                    || entries.size() > maxEntries)) {
                entries.removeFirst();
                timestamps.removeFirst();
            }

            if (pauseNanos > 0) {
                long sleepMs = pauseNanos / 1_000_000L;
                int sleepNanos = (int) (pauseNanos % 1_000_000L);
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

        if (!entries.isEmpty()) {
            byte[] tail = entries.peekLast();
            if (tail != null && tail.length > 0) {
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
