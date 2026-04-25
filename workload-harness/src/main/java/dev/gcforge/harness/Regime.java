package dev.gcforge.harness;

import java.time.Duration;
import java.util.Map;

/**
 * A regime is a parameterised workload of the JVM, exhibited from the GC's
 * point of view. Implementations run for {@code duration} using the supplied
 * pseudo-random {@code seed} and the regime-specific {@code params}.
 *
 * <p>The runner expects the implementation to be deterministic at a fixed seed:
 * timing is allowed to drift across runs, but the sequence of allocation
 * decisions must be reproducible.
 */
public interface Regime {

    /** Stable identifier registered with {@link RegimeRegistry}. */
    String id();

    /** Runs the regime to completion, allocating against the live heap. */
    void run(Map<String, String> params, Duration duration, long seed);
}
