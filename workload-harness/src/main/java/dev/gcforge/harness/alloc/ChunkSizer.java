package dev.gcforge.harness.alloc;

import java.util.Random;

/**
 * Samples the size (in bytes) of the next allocation, drawn from one of three
 * documented distributions:
 *
 * <ul>
 *   <li>{@link Distribution#SMALL} — 16..256 bytes uniform.</li>
 *   <li>{@link Distribution#MEDIUM} — 256..4096 bytes uniform.</li>
 *   <li>{@link Distribution#MIXED} — 80% small, 20% medium (default).</li>
 * </ul>
 *
 * <p>Reference: SPEC-FONCTIONNELLE §4.1 (R1 steady-state-healthy parameters).
 */
public final class ChunkSizer {

    public enum Distribution {
        SMALL, MEDIUM, MIXED;

        public static Distribution parse(String s) {
            return switch (s.toLowerCase()) {
                case "small"  -> SMALL;
                case "medium" -> MEDIUM;
                case "mixed"  -> MIXED;
                default       -> throw new IllegalArgumentException(
                    "unknown object_size_distribution " + s + " (expected: small, medium, mixed)"
                );
            };
        }
    }

    private static final int SMALL_MIN  = 16;
    private static final int SMALL_MAX  = 256;
    private static final int MEDIUM_MIN = 256;
    private static final int MEDIUM_MAX = 4096;

    private final Distribution distribution;
    private final Random rng;

    public ChunkSizer(Distribution distribution, Random rng) {
        this.distribution = distribution;
        this.rng = rng;
    }

    /** Returns the next chunk size in bytes. */
    public int sample() {
        return switch (distribution) {
            case SMALL  -> uniform(SMALL_MIN, SMALL_MAX);
            case MEDIUM -> uniform(MEDIUM_MIN, MEDIUM_MAX);
            case MIXED  -> rng.nextDouble() < 0.8
                ? uniform(SMALL_MIN, SMALL_MAX)
                : uniform(MEDIUM_MIN, MEDIUM_MAX);
        };
    }

    private int uniform(int min, int maxExclusive) {
        return min + rng.nextInt(maxExclusive - min);
    }
}
