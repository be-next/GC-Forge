# GC-Forge

Declarative generator of **Java GC logs** — the counterpart to [GC-Insight](#).

Where Insight observes, Forge fabricates: from a YAML description of a `(JVM, GC algorithm, application regime)` scenario, GC-Forge produces on demand a faithful native GC log, accompanied by a reproducible identity card.

## Project status

**Pre-release of 0.1.0.** The seven MVP regimes are implemented, fourteen
presets ship in-tree, and every CLI subcommand
(`lint`, `run`, `validate`, `batch`, `presets`, `selftest`,
`variance-check`) is operational. The release pipeline lands in the
final iteration before tagging `v0.1.0`.

User documentation lives under [`doc/user/`](./doc/user/README.md).
Internal specifications (in French) live under
[`doc/specs/`](./doc/specs/):

- [Functional specifications](./doc/specs/SPEC-FONCTIONNELLE.md)
- [Technical specifications](./doc/specs/SPEC-TECHNIQUE.md)
- [Roadmap](./doc/specs/ROADMAP.md)
- [Initial backlog](./doc/specs/BACKLOG.md)
- [Risks and open points](./doc/specs/RISQUES.md)

## Quickstart

```sh
# Build the CLI and the runner image once.
make build
make docker-image

# Lint, then run a shipped preset.
gc-forge lint presets/steady-g1-baseline.yaml
gc-forge run  presets/steady-g1-baseline.yaml \
    --image gc-forge-runner:dev-jdk21 \
    --embedded-harness /opt/gc-forge/harness.jar

# Re-check the produced log against its manifest.
gc-forge validate out/steady-g1-baseline-c0ffee.log \
    --manifest  out/steady-g1-baseline-c0ffee.manifest.yaml
```

A walk-through with troubleshooting tips is in
[`doc/user/getting-started.md`](./doc/user/getting-started.md);
the full CLI surface is in
[`doc/user/cli-reference.md`](./doc/user/cli-reference.md).

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
