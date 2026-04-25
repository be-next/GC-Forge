# GC-Forge — Spécifications fonctionnelles

> **Référence** : brief du 25 avril 2026, §3, §4.
> **Statut** : V1.0 — émise par Cowork.

## 1. Vision et positionnement

GC-Forge est un outil **déclaratif** et **reproductible** qui produit à la demande des GC logs Java représentatifs de scénarios choisis, en croisant trois axes :

```
JVM (vendor, version)  ×  Algorithme GC  ×  Régime applicatif
```

L'utilisateur décrit un scénario en YAML, GC-Forge exécute une vraie JVM avec un workload Java paramétrique et capture la sortie native du collecteur. La sortie est un **log GC indistinguable d'un log produit en production**, accompagné d'un **manifeste** qui en consigne tous les paramètres pour la reproductibilité bit-à-bit.

GC-Forge n'est **pas** un simulateur : il ne synthétise pas un faux log à partir d'un modèle statistique. Il ne remplace pas non plus un benchmark applicatif : il ne mesure pas la performance d'une appli, il en exhibe le **comportement GC**.

## 2. Cas d'usage détaillés

Les cinq cas d'usage du brief, dimensionnés en exigences concrètes :

### 2.1 Test et validation de GC-Insight (P0)

**Personæ** : Jérôme (développeur Rust de GC-Insight), CI de GC-Insight.

**Besoins** :
- Produire un **corpus de référence** (logs + vérité-terrain) versionnable et rejouable.
- À chaque release de GC-Insight, rejouer le corpus et comparer les analyses produites avec les phénomènes attendus consignés dans le manifeste.
- Détecter les régressions de parsing (logs JVM réels, pas synthétiques).
- Mesurer la précision des heuristiques sur un dataset stable.

**Exigences fonctionnelles** :
- F-VAL-1 : `gc-forge batch --matrix corpus-reference.yaml` régénère un corpus complet de manière déterministe.
- F-VAL-2 : Chaque log généré est accompagné d'un manifeste contenant les `expected_phenomena` (cf. §6).
- F-VAL-3 : Le hash SHA-256 du log est stable entre runs identiques (modulo timestamps wall-clock — voir §4 reproductibilité).
- F-VAL-4 : GC-Forge expose une commande `gc-forge validate <log> <manifest>` qui re-vérifie a posteriori que le log respecte les invariants du régime (filet de sécurité face à la non-reproductibilité totale du timing GC réel).

### 2.2 Démonstrations commerciales (P0)

**Personæ** : commercial / SE de GC-Insight, prospect.

**Besoins** :
- Disposer d'un **catalogue prêt-à-l'emploi** : un log par capacité d'analyse à démontrer.
- Logs courts (90 s à 5 min) mais « parlants » : le phénomène à exhiber doit être visible sans avoir à scroller 50 MB de log.
- Fiche pédagogique livrée avec chaque preset.

**Exigences** :
- F-DEMO-1 : `gc-forge presets list` affiche le catalogue avec description en une ligne.
- F-DEMO-2 : `gc-forge presets show <name>` affiche la fiche pédagogique (régime, JVM, algo, phénomène attendu, ce que GC-Insight doit révéler).
- F-DEMO-3 : `gc-forge run --preset <name>` génère le log en moins de 5 minutes pour tous les presets MVP.

### 2.3 Pédagogie et contenu (P1)

**Personæ** : Jérôme rédigeant un article de blog, formateur.

**Besoins** :
- Logs en sortie + extraits commentés.
- Possibilité de varier les paramètres autour d'un preset (« et si on doublait la heap ? »).
- Captures de la phénoménologie en plusieurs résolutions (log tronqué pour exemple, log complet pour téléchargement).

**Exigences** :
- F-PED-1 : `gc-forge run --preset <name> --override 'gc.heap.max=4g'` permet le tweak ad hoc.
- F-PED-2 : option `--snippet <event_kind>` qui produit un extrait du log centré sur un type d'événement (ex. `--snippet humongous_allocation`).

### 2.4 Reproduction de pathologies utilisateur (P1)

**Personæ** : utilisateur GC-Insight ayant observé un comportement pathologique en prod.

**Besoins** :
- Décrire son contexte (JVM, flags, heap, comportement observé) et obtenir un scénario miroir à exécuter.
- Comparer son log de prod et le log GC-Forge sur les mêmes axes.

**Exigences (V1, hors MVP)** :
- F-REPRO-1 : `gc-forge mirror <prod.log>` propose un scénario YAML minimal cohérent avec les flags détectés et la phénoménologie observée. Heuristique simple en V1 (mapping flags → JVM + algo, classification grossière du régime).

### 2.5 Datasets pour ML (P2)

**Personæ** : Jérôme entraînant un classifieur de régimes ou un détecteur d'anomalies.

**Besoins** :
- Volume : milliers de logs de quelques minutes chacun.
- Étiquetage automatique (label = régime, sous-label = paramètres clés).
- Variabilité contrôlée (seeds différents, paramètres tirés dans une plage).

**Exigences (V2)** :
- F-ML-1 : `gc-forge sweep <template.yaml> --count 1000 --seed-range 1..1000` génère un dataset paramétrique.
- F-ML-2 : Export d'un index (CSV / Parquet) listant tous les fichiers et leurs labels.

## 3. Catalogue des JVMs et algorithmes GC

### 3.1 JVMs supportées

| Vendor | Distribution | Versions MVP | Versions V1 | Versions V2 |
|--------|--------------|--------------|-------------|-------------|
| Eclipse Adoptium | Temurin | 17, 21 | 11 (legacy), 23 | — |
| Amazon | Corretto | — | 17, 21 | 11, 23 |
| Oracle | GraalVM CE | — | 21 (Native Image hors périmètre) | 23 |
| Eclipse | OpenJ9 | — | 17, 21 | — |
| Azul | Zing / Prime | — | — | 21 (sous condition licence) |
| Oracle | HotSpot Oracle JDK | — | — | 21 (sous condition licence — TCK) |

**MVP** : Temurin uniquement, JDK 17 et 21. Justification :
- Temurin est gratuit, redistribuable, documenté, dispose d'images Docker officielles `eclipse-temurin:{17,21}-jdk` à jour.
- 95 % des cas d'usage GC logs en prod aujourd'hui sont sur HotSpot — couvrir d'abord la cible majoritaire.
- 17 et 21 sont les LTS courants ; 17 reste très installé, 21 monte en charge.
- OpenJ9 et Zing ont des **formats de log GC différents** de HotSpot : leur ajout en V1/V2 demandera un travail dédié de parsing/validation côté GC-Insight, donc à fournir de manière coordonnée.

### 3.2 Algorithmes GC

Compatibilité par JVM (HotSpot/Temurin uniquement pour MVP) :

| Algorithme | Flag JVM | JDK 17 | JDK 21 | MVP | Notes |
|------------|----------|--------|--------|-----|-------|
| G1 | `-XX:+UseG1GC` (défaut ≥9) | ✓ | ✓ | **✓** | Cible principale, le mieux documenté côté pathologies. |
| ZGC | `-XX:+UseZGC` | ✓ (non-generational) | ✓ + **`-XX:+ZGenerational`** | **✓** | Generational ZGC GA en JDK 21 — à exposer en MVP. |
| Parallel | `-XX:+UseParallelGC` | ✓ | ✓ | **✓** | Throughput-oriented, pertinent pour batch. |
| Shenandoah | `-XX:+UseShenandoahGC` | ✓ (Temurin) | ✓ | — | V1. Concurrent low-latency comme ZGC, comportement distinct. |
| Serial | `-XX:+UseSerialGC` | ✓ | ✓ | — | V1. Niche (containers minuscules) mais utile pour pédagogie. |
| CMS | `-XX:+UseConcMarkSweepGC` | ✗ (retiré) | ✗ | — | **V2 sur JDK 8** uniquement, pour cas historiques. |

**Justification du choix MVP G1 / ZGC / Parallel** :
- G1 = collecteur par défaut, le plus rencontré en prod, le plus riche en pathologies à exhiber.
- ZGC generational = montée en charge, low-latency, sujet brûlant 2025-2026.
- Parallel = throughput, contraste utile avec G1/ZGC sur les régimes batch/compute.

Shenandoah et Serial reportés en V1 : pas un blocage technique, juste de l'effort marginal une fois l'archi stabilisée.

## 4. Catalogue des régimes applicatifs

Un **régime** est la signature comportementale d'un workload du point de vue GC : taux d'allocation, distribution des durées de vie, pattern temporel, taille des objets.

Pour chaque régime, on définit :
- **Paramètres** : ce que l'utilisateur peut moduler.
- **Signature attendue** : ce que doit produire le log (invariants quantifiés — voir §7).
- **Cas d'usage GC-Insight** : ce que la capacité d'analyse doit révéler.

### Synthèse des régimes MVP

| # | Régime | Paramètres clés | Signature GC attendue (G1) |
|---|--------|-----------------|----------------------------|
| R1 | `steady-state-healthy` | `allocation_rate_mb_s`, `live_set_mb` | Young GC réguliers, pause p99 < 50 ms, 0 mixed pathologique, 0 full. |
| R2 | `allocation-burst` | `base_rate_mb_s`, `burst_rate_mb_s`, `burst_duration_s`, `burst_period_s` | Fréquence young qui pulse au rythme des bursts, pas d'évacuation failure si bien dimensionné. |
| R3 | `humongous-pressure` | `humongous_ratio`, `humongous_size_kb`, `region_size_mb` | Régions humongous visibles, mixed GC déclenchés tôt, pression sur old. |
| R4 | `slow-leak` | `leak_rate_mb_s`, `live_set_initial_mb` | Footprint après-GC qui croît linéairement, mixed plus fréquents puis full GC ou OOM en fin de course. |
| R5 | `cache-churn` | `cache_size_mb`, `eviction_rate_per_s`, `entry_lifetime_ms` | Taux de promotion vers old élevé, mixed GC fréquents, taille old fluctuante. |
| R6 | `mixed-gc-pathological` | `old_gen_pressure`, `fragmentation_factor` | Mixed GC inefficaces (peu de reclaim), durée mixed qui s'allonge, IHOP qui descend. |
| R7 | `microservice-stop-and-go` | `active_period_s`, `idle_period_s`, `active_rate_mb_s` | Alternance bursts young / silence, possibles concurrent cycles en idle. |

### 4.1 R1 — `steady-state-healthy` (référence)

**Paramètres** :
- `allocation_rate_mb_s` (défaut : 50) — débit d'allocation jeune génération.
- `live_set_mb` (défaut : 100) — taille stable de l'objet vivant.
- `object_size_distribution` : `small` (16-256 B) / `medium` (256-4 KB) / `mixed` (défaut).
- `lifetime_distribution` : `short` (mort en young), `mixed` (10 % promus).

**Signature GC attendue** :
- ≥ 80 % des collections sont des young.
- 0 full GC, 0 mixed GC pathologique.
- p99 (pause young) < 50 ms à heap = 2 GB, allocation_rate = 50 MB/s.
- Variance inter-run sur le nombre total de pauses ≤ 5 %.

**Cas d'usage GC-Insight** : log de référence « tout va bien » — sert de baseline pour la détection d'anomalies.

### 4.2 R2 — `allocation-burst`

**Paramètres** :
- `base_rate_mb_s` (défaut : 30).
- `burst_rate_mb_s` (défaut : 200).
- `burst_duration_s` (défaut : 5).
- `burst_period_s` (défaut : 30).
- `bursts_count` (défaut : `auto` = `duration / burst_period`).

**Signature** :
- Fréquence des young GC pulse au rythme des bursts (visible sur tracé).
- À la sortie d'un burst, retour à la fréquence de base en ≤ 2× `burst_duration_s`.
- Pas d'évacuation failure si `burst_rate_mb_s` ≤ `young_capacity / pause_young`.

**Cas d'usage** : démontrer la détection de bursts par GC-Insight, et la corrélation avec les pauses.

### 4.3 R3 — `humongous-pressure`

**Paramètres** :
- `humongous_ratio` (0.0–1.0, défaut : 0.5) — fraction de l'allocation qui est humongous.
- `humongous_size_kb` (défaut : auto = `1.1 × region_size`).
- `region_size_mb` (défaut : auto, dépend de heap).
- `allocation_rate_mb_s` (défaut : 80).

**Signature** :
- ≥ 1 humongous allocation par seconde dans le log.
- Régions humongous visibles dans le log G1 (`humongous regions: N`).
- Mixed GC déclenchés malgré `IHOP` non atteint (humongous force la marche).
- Si `humongous_ratio` > 0.7 : possibilité d'évacuation failure.

**Cas d'usage** : démontrer la capacité de GC-Insight à isoler les humongous comme cause racine.

### 4.4 R4 — `slow-leak`

**Paramètres** :
- `leak_rate_mb_s` (défaut : 0.5) — croissance du live-set.
- `live_set_initial_mb` (défaut : 200).
- `duration` (défaut : `auto` jusqu'à OOM ou 30 min).

**Signature** :
- Live-set après-GC croît avec pente ≈ `leak_rate_mb_s × T`.
- Fréquence des mixed GC croît avec le temps.
- Full GC (ou OOM si fail-fast) en fin de course.
- Pause moyenne croît de manière monotone (corrélation Pearson > 0.7 avec t).

**Cas d'usage** : démontrer la détection de fuite mémoire par GC-Insight (régression linéaire sur la footprint after-GC).

### 4.5 R5 — `cache-churn`

**Paramètres** :
- `cache_size_mb` (défaut : 500).
- `eviction_rate_per_s` (défaut : 1000) — entrées évincées/sec.
- `entry_lifetime_ms` (défaut : 2000) — durée de vie typique d'une entrée.
- `entry_size_kb` (défaut : 8).

**Signature** :
- Promotion rate (young → old) ≥ 30 % du throughput young.
- Old gen oscille entre `cache_size × 0.8` et `cache_size × 1.2`.
- Mixed GC fréquents mais réguliers (pas pathologiques).

**Cas d'usage** : démontrer la signature « cache » vs « leak » — différencier croissance bornée et croissance non-bornée.

### 4.6 R6 — `mixed-gc-pathological`

**Paramètres** :
- `old_gen_pressure` (0.0–1.0, défaut : 0.7) — fraction du heap qu'occupe l'old.
- `fragmentation_factor` (1.0–3.0, défaut : 2.0) — multiplicateur de la fragmentation simulée.
- `survivor_age_target` (défaut : 15) — survivants résistants.

**Signature** :
- Durée des mixed GC croît au fil du temps.
- Reclaim par mixed GC < 5 % du heap.
- IHOP (Initiating Heap Occupancy Percent) effectif descend (G1 ergonomic ajuste).
- Possible évacuation failure → full GC en fin de course.

**Cas d'usage** : cas pathologique premium — démontrer la valeur diagnostique de GC-Insight sur un cas où la cause racine n'est pas évidente.

### 4.7 R7 — `microservice-stop-and-go`

**Paramètres** :
- `active_period_s` (défaut : 10).
- `idle_period_s` (défaut : 20).
- `active_rate_mb_s` (défaut : 100).
- `cycles` (défaut : `auto`).

**Signature** :
- Périodes d'inactivité avec ≤ 1 young GC / 10 s.
- Concurrent mark cycles déclenchés en idle (G1).
- Avec ZGC : plus lisse, moins de signature visible — pédagogique en soi.

**Cas d'usage** : montrer le contraste de comportement entre algos sur un workload « moderne » microservice.

### 4.8 Régime exploratoire (V1) — `compute-batch`

À documenter en V1 : régime « batch compute » avec allocations massives temporaires, intéressant à contraster sur Parallel vs G1.

## 5. Modèle de scénario (schéma YAML)

### 5.1 Schéma `gc-forge/scenario.v1`

```yaml
apiVersion: gc-forge/scenario.v1
kind: Scenario

metadata:
  name: <string, slug, requis>
  version: <semver, défaut "1.0.0">
  description: <string, optionnel>
  tags: [<string>, ...]
  authors: [<string>, ...]

spec:
  # Axe 1 : JVM
  jvm:
    vendor: temurin | corretto | graalvm | openj9   # MVP: temurin
    major: 17 | 21                                  # MVP
    distribution: jdk | jre                         # défaut: jdk
    extra_flags: [<string>, ...]                    # flags non-GC

  # Axe 2 : algorithme GC + heap
  gc:
    algorithm: G1 | ZGC | Parallel                  # MVP
    options:
      generational: true | false                    # ZGC: défaut true
      heap:
        min: <size>                                 # ex: "2g", "512m"
        max: <size>                                 # ex: "2g"
        new_size: <size, optionnel>                 # G1: peu utile, Parallel: clé
      pause_target_ms: <int, optionnel>             # G1: -XX:MaxGCPauseMillis
      region_size_mb: <int, optionnel>              # G1: -XX:G1HeapRegionSize
      ihop_percent: <int, optionnel>                # G1
    extra_flags: [<string>, ...]                    # flags GC custom
    log_format: unified | legacy                    # MVP: unified (-Xlog:gc*)

  # Axe 3 : régime applicatif
  regime:
    kind: <string>                                  # voir §4
    parameters: { ... }                             # voir §4 par régime

  # Pilotage de l'exécution
  duration: <duration>                              # ex: "90s", "5m"
  warmup: <duration>                                # défaut: "10s"
  seed: <hex|int>                                   # requis pour reproductibilité

  # Sortie
  output:
    log_path: <path, optionnel>                     # défaut: <name>-<seed>.log
    manifest_path: <path, optionnel>                # défaut: <name>-<seed>.manifest.yaml
    capture_jfr: <bool>                             # défaut: false (V1+)

  # Validation post-run
  expected:
    phenomena: [<phenomenon-id>, ...]               # déclenche `validate` automatique
    invariants: [<invariant-rule>, ...]             # règles supplémentaires custom
```

### 5.2 Validation

- Le schéma est exprimé en **JSON Schema** (généré depuis Rust via `schemars`) et publié dans `schemas/scenario-v1.json`.
- `gc-forge lint <scenario.yaml>` valide le scénario sans l'exécuter.
- Erreurs de validation explicites : `gc.algorithm: "Z" not in {G1, ZGC, Parallel}; available algos for jvm.major=17: G1, ZGC, Parallel, Shenandoah(V1)`.

### 5.3 Composition et héritage

Pour limiter la duplication entre scénarios proches :

```yaml
extends: presets/g1-baseline.yaml
spec:
  regime:
    kind: humongous-pressure
    parameters:
      humongous_ratio: 0.7
```

L'extension est résolue à la lecture (merge récursif des maps, override des scalaires).

### 5.4 Override CLI

`--override` accepte des paths style JSONPath :

```bash
gc-forge run scenario.yaml \
  --override 'spec.gc.options.heap.max=4g' \
  --override 'spec.regime.parameters.humongous_ratio=0.8'
```

## 6. Formats de sortie

### 6.1 Log brut (priorité absolue)

Le log GC est **strictement** la sortie native du collecteur, capturée sans transformation :
- HotSpot JDK 9+ : Unified Logging (`-Xlog:gc*=info,gc+heap=debug,gc+age=trace:file=<path>:time,level,tags`).
- Le format des tags est fixé par GC-Forge pour cohérence avec GC-Insight (cf. TECH §6).
- Un log produit par GC-Forge doit être **indistinguable** d'un log produit par une vraie appli Java avec les mêmes flags. C'est l'invariant non-négociable.

### 6.2 Manifeste — schéma `gc-forge/run-manifest.v1`

Émis à côté de chaque log. Format YAML par défaut, JSON via `--manifest-format=json`.

```yaml
apiVersion: gc-forge/run-manifest.v1
kind: RunManifest

# Identité du run
run:
  id: <uuid v7>
  started_at: <ISO 8601>
  ended_at: <ISO 8601>
  duration_actual: <duration>
  exit_status: success | failure | oom | timeout
  host:
    os: <linux|macos>
    arch: <x86_64|aarch64>
    cpu_count: <int>
    container: docker:<image-tag> | native

# Lien avec le scénario
scenario:
  source_path: <path>
  source_sha256: <hex>
  resolved: { ... }              # scénario après merge/override, snapshot complet

# Configuration JVM réelle
jvm:
  vendor: temurin
  version: <full version, ex: "21.0.2+13">
  flags: [<string>, ...]         # flags effectivement passés à la JVM

# Reproductibilité
reproducibility:
  seed: <hex>
  workload_jar_sha256: <hex>     # hash du harness Java utilisé
  gc_forge_version: <semver+git_sha>

# Sortie
output:
  log_path: <path>
  log_sha256: <hex>
  log_size_bytes: <int>

# Vérité-terrain
expected_phenomena: [<phenomenon-id>, ...]
expected_invariants: [{ rule: <string>, threshold: <value> }, ...]

# Validation post-run (rempli par `validate`)
validation:
  status: passed | failed | skipped
  results: [{ rule, observed, threshold, passed: bool }, ...]
  validated_at: <ISO 8601>
  validator_version: <semver>
```

### 6.3 Catalogue des `phenomenon-id`

Liste contrôlée, versionnée, alignée avec ce que GC-Insight sait détecter :

| ID | Description |
|----|-------------|
| `young_gc_steady` | Fréquence young stable. |
| `allocation_burst` | Pulsations détectables d'allocation. |
| `humongous_allocation` | Allocations humongous présentes. |
| `evacuation_failure` | Au moins une évacuation failure. |
| `mixed_gc_efficient` | Mixed GC reclamant > 30 % du heap visé. |
| `mixed_gc_pathological` | Mixed GC reclamant < 5 % du heap. |
| `slow_leak` | Live-set après-GC croît linéairement. |
| `full_gc` | Au moins un full GC. |
| `oom` | OutOfMemoryError. |
| `concurrent_cycle_in_idle` | Concurrent cycle déclenché en période d'inactivité. |
| `promotion_pressure` | Taux de promotion young → old > seuil. |

### 6.4 Index de batch

`gc-forge batch` produit en plus un fichier `index.csv` (et `index.parquet` en V1) listant tous les runs avec leurs labels — cible cas d'usage ML.

## 7. Critères de qualité d'un log généré

Un log est **valide** s'il respecte les invariants quantifiés du régime qui l'a produit. Trois mécaniques :

### 7.1 Invariants par régime (rules-as-code)

Chaque régime expose une fonction `validate(log, params) -> Result<ValidationReport>` côté Rust. Exemple pour `R1 steady-state-healthy` :

```rust
// gc-forge-regimes/src/steady_state.rs (schéma indicatif)
fn validate(parsed: &ParsedLog, params: &SteadyStateParams) -> ValidationReport {
    rule!("young_ratio >= 0.8", parsed.young_count as f64 / parsed.total_count as f64);
    rule!("full_count == 0", parsed.full_count);
    rule!("p99_pause_ms < 50", parsed.young_pause_p99_ms());
    rule!("variance_pause_count_pct < 5", parsed.cv_pause_count() * 100.0);
}
```

Les `expected_invariants` du manifeste sont alimentés à partir de ces règles.

### 7.2 Suite de validation rejouée à chaque release

`gc-forge selftest` exécute tous les presets MVP et vérifie que chacun passe ses invariants. Bloque la CI en cas d'échec. Cible : 100 % des presets passent à chaque commit sur `main`.

### 7.3 Variance inter-run

Un même scénario rejoué N fois (même seed, même JVM, même hôte) doit produire des logs aux métriques agrégées **proches**. Deux niveaux de tolérance, par catégorie de métrique :

| Catégorie de métrique | Cible objectif | Tolérance acceptable | Plafond bloquant |
|----------------------|----------------|----------------------|------------------|
| Comptages (n young, n mixed) | CV ≤ 5 % | CV ≤ 8 % | CV > 10 % ⇒ régime non qualifié |
| Sommes (durée totale, octets alloués) | CV ≤ 5 % | CV ≤ 8 % | CV > 10 % ⇒ régime non qualifié |
| Moyennes (heap après-GC, pause moyenne) | CV ≤ 5 % | CV ≤ 10 % | CV > 12 % |
| Centiles extrêmes (p99, max) | CV ≤ 15 % | CV ≤ 20 % | CV > 25 % |

L'asymétrie entre métriques agrégées et centiles extrêmes reflète la non-déterminisme inhérent du timing GC réel — un seul outlier peut faire bouger un p99 de 30 % sans altérer la signature globale du régime.

**Stratégie de fallback** (cf. RISQUES R-T1) : si la cible 5 % n'est pas tenable sur un régime donné après 2 itérations de stabilisation, on accepte la tolérance ≤ 8 %, on documente l'écart, et on relâche les invariants associés. Si même la tolérance n'est pas tenue, le régime est mis en « experimental » et exclu de `selftest`.

`gc-forge variance-check <scenario.yaml> --runs 10` automatise cette mesure pour qualifier un nouveau régime.

### 7.4 Reproductibilité bit-à-bit — limites

La reproductibilité bit-à-bit du **log** n'est pas atteignable sur une JVM réelle (le timing GC dépend du planificateur OS, de la fréquence CPU, de la charge système). On garantit en revanche :
- Reproductibilité du **manifeste résolu** (config + workload + seed).
- Reproductibilité **sémantique** du log : même phénomènes, mêmes ordres de grandeur, invariants respectés.
- Hash SHA-256 stable du **harness Java** et du scénario résolu (mais pas du log).

## 8. Presets pré-packagés du MVP

Le MVP livre **14 presets** (7 régimes × 2 algos clés) plus quelques bonus pédagogiques. Tous ciblent JDK 21 par défaut, avec variante JDK 17 disponible via `--jvm.major=17`.

| ID | Régime | Algo | Heap | Durée | Phénomène phare |
|----|--------|------|------|-------|-----------------|
| `steady-g1-baseline` | R1 | G1 | 2 GB | 90 s | `young_gc_steady` |
| `steady-zgc-baseline` | R1 | ZGC gen | 2 GB | 90 s | `young_gc_steady` |
| `burst-g1-30s` | R2 | G1 | 2 GB | 5 min | `allocation_burst` |
| `burst-parallel-30s` | R2 | Parallel | 2 GB | 5 min | `allocation_burst` |
| `humongous-g1-classic` | R3 | G1 | 2 GB | 2 min | `humongous_allocation`, `mixed_gc_efficient` |
| `humongous-g1-evac-fail` | R3 | G1 | 1 GB | 2 min | `humongous_allocation`, `evacuation_failure` |
| `leak-g1-slow` | R4 | G1 | 1 GB | 10 min | `slow_leak`, `full_gc` |
| `leak-zgc-slow` | R4 | ZGC gen | 1 GB | 10 min | `slow_leak` |
| `cache-g1-churn` | R5 | G1 | 4 GB | 5 min | `promotion_pressure` |
| `cache-parallel-churn` | R5 | Parallel | 4 GB | 5 min | `promotion_pressure` |
| `mixed-pathological-g1` | R6 | G1 | 2 GB | 5 min | `mixed_gc_pathological` |
| `microservice-g1-stop-go` | R7 | G1 | 1 GB | 5 min | `concurrent_cycle_in_idle` |
| `microservice-zgc-stop-go` | R7 | ZGC gen | 1 GB | 5 min | (contraste) |

**Bonus pédagogiques** :
- `compare-young-pause-g1-vs-zgc` : matrice qui produit les deux baselines côte-à-côte.
- `humongous-region-size-sweep` : 4 runs avec `region_size` ∈ {1,2,4,8} MB pour exhiber l'effet du paramètre.

## 9. Modes d'exécution

### 9.1 Mode unitaire

```bash
gc-forge run scenario.yaml [--override KEY=VAL]... [--out-dir DIR]
gc-forge run --preset humongous-g1-classic
```

### 9.2 Mode batch

```bash
gc-forge batch matrix.yaml [--parallel N] [--out-dir DIR]
```

`matrix.yaml` exemple :

```yaml
apiVersion: gc-forge/matrix.v1
kind: Matrix
spec:
  base: presets/g1-baseline.yaml
  axes:
    jvm.major: [17, 21]
    gc.algorithm: [G1, ZGC, Parallel]
    regime.kind: [steady-state-healthy, allocation-burst, humongous-pressure]
  seeds: [1, 2, 3]
  filters:                       # exclure combinaisons illégales
    - { gc.algorithm: ZGC, regime.kind: humongous-pressure }   # ZGC ne fait pas humongous spécial
```

Produit : `out/index.csv` + un fichier log/manifeste par cellule.

### 9.3 Mode validate

```bash
gc-forge validate <log> --manifest <manifest.yaml>
```

Re-vérifie a posteriori les invariants attendus. Code de retour non-nul si échec.

### 9.4 Mode lint

```bash
gc-forge lint scenario.yaml
```

Validation syntaxique + sémantique sans exécution (vérifie compatibilité JVM/algo, cohérence régime/algo, plages de paramètres).

### 9.5 Mode presets

```bash
gc-forge presets list [--regime R3] [--algo G1]
gc-forge presets show humongous-g1-classic
gc-forge presets export <name> > my-scenario.yaml   # extrait pour tweaker
```

### 9.6 Mode variance-check (V1)

```bash
gc-forge variance-check scenario.yaml --runs 10 [--report.html]
```

## 10. Interface CLI — conventions globales

- Sortie machine via `--output json` sur toutes les sous-commandes.
- Codes de retour : 0 succès, 1 erreur utilisateur (yaml invalide, JVM manquante), 2 erreur d'exécution JVM, 3 invariants violés, 4 erreur interne.
- Verbose : `-v`, `-vv`, `-vvv` ; `--quiet` pour silence sauf erreurs.
- Couleurs auto, `NO_COLOR` respecté.
- Conventions cohérentes avec la CLI GC-Insight (mêmes flags transversaux).

## 11. Ce qui n'est explicitement PAS dans le périmètre

- Pas de génération de logs **synthétiques** (réservé à V2 pour cas extrêmes).
- Pas de mesure de performance applicative (latence métier, throughput RPS) — GC-Forge n'est pas un benchmark.
- Pas de simulation multi-JVM ou de workload distribué — un scénario = un process JVM.
- Pas de support des langages JVM autres que Java (Kotlin, Scala) — sans intérêt pour la production de logs GC.
- Pas d'UI graphique — CLI only au moins jusqu'en V1.

---

**Cohérence avec GC-Insight** : tout `phenomenon-id` produit par GC-Forge doit être détectable par GC-Insight, et inversement, les capacités d'analyse de GC-Insight doivent être démontrables par au moins un preset GC-Forge. Cette traçabilité est tracée dans une matrice `phenomenon × preset × insight-capability` à maintenir dans `doc/traceability.md` (V1).
