# GC-Forge

Declarative generator of **Java GC logs** — the counterpart to [GC-Insight](#).

Where Insight observes, Forge fabricates: from a YAML description of a `(JVM, GC algorithm, application regime)` scenario, GC-Forge produces on demand a faithful native GC log, accompanied by a reproducible identity card.

## Project status

**Specs phase** — implementation pending. See [`doc/specs/`](./doc/specs/) for the full specifications (in French):

- [Functional specifications](./doc/specs/SPEC-FONCTIONNELLE.md)
- [Technical specifications](./doc/specs/SPEC-TECHNIQUE.md)
- [Roadmap](./doc/specs/ROADMAP.md)
- [Initial backlog](./doc/specs/BACKLOG.md)
- [Risks and open points](./doc/specs/RISQUES.md)

## Use cases

1. **Testing and validating GC-Insight** — produce reference logs with ground truth.
2. **Sales demos** — ready-to-use catalog of "speaking" logs.
3. **Education and content** — articles, tutorials, training material.
4. **Pathology reproduction** — mirror a GC behavior observed in production.
5. **ML datasets** — feed classification or anomaly-detection models.

## Scenario at a glance

```yaml
# scenarios/humongous-pressure-g1.yaml
apiVersion: gc-forge/scenario.v1
kind: Scenario
metadata:
  name: humongous-pressure-g1
  description: G1 under humongous allocation pressure (>50% region).
spec:
  jvm: { vendor: temurin, major: 21 }
  gc:
    algorithm: G1
    heap: { min: 2g, max: 2g }
    options: ["-XX:G1HeapRegionSize=4M"]
  regime:
    kind: humongous-pressure
    parameters:
      humongous_ratio: 0.6
      allocation_rate_mb_s: 80
  duration: 90s
  seed: 0xC0FFEE
```

## License

[MIT](./LICENSE).
