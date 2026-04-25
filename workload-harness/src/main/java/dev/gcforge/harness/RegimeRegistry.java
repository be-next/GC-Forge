package dev.gcforge.harness;

import dev.gcforge.harness.regimes.AllocationBurstRegime;
import dev.gcforge.harness.regimes.CacheChurnRegime;
import dev.gcforge.harness.regimes.HumongousPressureRegime;
import dev.gcforge.harness.regimes.SlowLeakRegime;
import dev.gcforge.harness.regimes.SteadyStateRegime;

import java.util.LinkedHashMap;
import java.util.Map;
import java.util.NoSuchElementException;
import java.util.function.Supplier;

/**
 * Static registry mapping a regime id (such as {@code "steady-state-healthy"})
 * to a fresh instance.
 *
 * <p>The registry is intentionally tiny: regimes are stateless w.r.t. the
 * registry and instantiated per-run. This keeps {@link WorkloadHarness#main}
 * simple and side-effect free.
 */
public final class RegimeRegistry {

    private static final Map<String, Supplier<Regime>> FACTORIES = new LinkedHashMap<>();

    static {
        register("steady-state-healthy", SteadyStateRegime::new);
        register("allocation-burst", AllocationBurstRegime::new);
        register("humongous-pressure", HumongousPressureRegime::new);
        register("cache-churn", CacheChurnRegime::new);
        register("slow-leak", SlowLeakRegime::new);
        // Future regimes (mixed-gc-pathological, microservice-stop-and-go)
        // register here alongside their iteration's class additions.
    }

    private RegimeRegistry() {
        // utility class
    }

    /** Registers a regime implementation factory under {@code kind}. */
    public static void register(String kind, Supplier<Regime> factory) {
        FACTORIES.put(kind, factory);
    }

    /** Looks up a regime by id, or throws {@link NoSuchElementException}. */
    public static Regime lookup(String kind) {
        Supplier<Regime> factory = FACTORIES.get(kind);
        if (factory == null) {
            throw new NoSuchElementException(
                "unknown regime kind " + kind + "; known: " + FACTORIES.keySet()
            );
        }
        return factory.get();
    }

    /** True when the kind is registered. */
    public static boolean contains(String kind) {
        return FACTORIES.containsKey(kind);
    }
}
