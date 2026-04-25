package dev.gcforge.harness;

import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.NoSuchElementException;

/**
 * GC-Forge workload harness — entry point.
 *
 * <p>Argument layout (positional):
 * <pre>
 *     [regime-kind] [duration-ISO] [seed-hex|long] [param-key=value]…
 * </pre>
 *
 * <p>Examples:
 * <pre>
 *     java -jar harness.jar steady-state-healthy PT30S 0xC0FFEE \
 *         allocation_rate_mb_s=50 live_set_mb=100
 * </pre>
 *
 * <p>When invoked with zero arguments, falls back to a 10 s
 * {@code steady-state-healthy} run with the seed {@code 0xC0FFEE}. This
 * preserves the {@code make demo} behaviour from iteration 1 without forcing
 * the demo target to teach itself the new CLI.
 *
 * <p>Exit codes:
 * <ul>
 *   <li>{@code 0}: regime ran to completion.</li>
 *   <li>{@code 1}: argument parsing or unknown regime kind.</li>
 *   <li>{@code 2}: regime threw at runtime.</li>
 * </ul>
 */
public final class WorkloadHarness {

    private WorkloadHarness() {
        // utility class
    }

    public static void main(String[] args) {
        try {
            Invocation inv = parse(args);
            Regime regime = RegimeRegistry.lookup(inv.kind);
            regime.run(inv.params, inv.duration, inv.seed);
        } catch (IllegalArgumentException | NoSuchElementException e) {
            System.err.println("error: " + e.getMessage());
            System.exit(1);
        } catch (RuntimeException e) {
            System.err.println("regime failed: " + e.getMessage());
            e.printStackTrace(System.err);
            System.exit(2);
        }
    }

    /**
     * Visible for tests. Parses the positional + key=value arg layout.
     */
    static Invocation parse(String[] args) {
        if (args.length == 0) {
            // Demo / smoke-test fallback: 10 s steady-state at default params.
            return new Invocation(
                "steady-state-healthy", Duration.ofSeconds(10), 0x00C0_FFEEL, new LinkedHashMap<>()
            );
        }
        if (args.length < 3) {
            throw new IllegalArgumentException(
                "expected at least 3 positional arguments [kind] [duration] [seed], got " + args.length
            );
        }
        String kind = args[0];
        Duration duration;
        try {
            duration = Duration.parse(args[1]);
        } catch (RuntimeException e) {
            throw new IllegalArgumentException(
                "invalid duration " + args[1] + " (expected ISO-8601 like PT30S): " + e.getMessage()
            );
        }
        long seed = parseSeed(args[2]);

        Map<String, String> params = new LinkedHashMap<>();
        for (int i = 3; i < args.length; i++) {
            String arg = args[i];
            int eq = arg.indexOf('=');
            if (eq <= 0) {
                throw new IllegalArgumentException(
                    "parameter " + arg + " is not in key=value form"
                );
            }
            params.put(arg.substring(0, eq), arg.substring(eq + 1));
        }
        return new Invocation(kind, duration, seed, params);
    }

    private static long parseSeed(String s) {
        String trimmed = s.trim();
        try {
            if (trimmed.startsWith("0x") || trimmed.startsWith("0X")) {
                return Long.parseUnsignedLong(trimmed.substring(2), 16);
            }
            return Long.parseLong(trimmed);
        } catch (NumberFormatException e) {
            throw new IllegalArgumentException("invalid seed " + s + ": " + e.getMessage());
        }
    }

    /** Visible for tests. Parsed CLI invocation. */
    static final class Invocation {
        final String kind;
        final Duration duration;
        final long seed;
        final Map<String, String> params;

        Invocation(String kind, Duration duration, long seed, Map<String, String> params) {
            this.kind = kind;
            this.duration = duration;
            this.seed = seed;
            this.params = params;
        }
    }
}
