# GC-Forge — Backlog initial

> **Référence** : brief du 25 avril 2026, §8.4.
> Backlog structuré en **epics** et **user stories** de premier niveau, priorisé MVP / V1 / V2.

## Conventions

- **Priorités** : `P0` (MVP, bloquant), `P1` (V1, important), `P2` (V2 ou opportuniste).
- **Estimation** : XS (≤ 0,5 j), S (1 j), M (2-3 j), L (4-7 j), XL (> 1 sem).
- **Format US** : `En tant que <persona>, je veux <action>, afin de <bénéfice>.`
- **Critères d'acceptation** : conditions vérifiables en QA/CI.

---

## EPIC 1 — Infrastructure et tronc commun

**Objectif** : poser le squelette technique sur lequel tout le reste s'empile.

### US-1.1 — Workspace Cargo et CI initiale
- **Priorité** : P0
- **Estimation** : S
- **Story** : En tant que dev GC-Forge, je veux un workspace Cargo multi-crates et une CI verte sur un commit minimal, afin de pouvoir itérer sans dette d'environnement.
- **Critères** :
  - `cargo build --workspace` passe sur Linux et macOS.
  - `cargo fmt --check`, `cargo clippy -D warnings`, `cargo deny check` passent en CI.
  - Workspace contient les 6 crates de la SPEC-TECH §2.1 (squelettes vides acceptés).

### US-1.2 — Projet Maven du harness Java
- **Priorité** : P0
- **Estimation** : S
- **Story** : En tant que dev GC-Forge, je veux un projet Maven qui produit un fat-jar `workload-harness.jar`, afin de pouvoir l'embarquer dans les exécutions JVM.
- **Critères** :
  - `mvn package` produit `target/workload-harness-<version>.jar`.
  - Build reproductible : SHA-256 stable entre runs identiques.
  - CI Java verte sur Maven 3.9 + JDK 17.

### US-1.3 — Image Docker runner
- **Priorité** : P0
- **Estimation** : S
- **Story** : En tant que dev GC-Forge, je veux une image Docker `gc-forge-runner` qui contient Temurin + le harness, afin d'avoir un environnement d'exécution figé.
- **Critères** :
  - Variantes `jdk17` et `jdk21`.
  - Multi-arch `linux/amd64` + `linux/arm64`.
  - Publication automatique sur GHCR au tag.

---

## EPIC 2 — Modèle de scénario

**Objectif** : exposer le modèle déclaratif YAML promis par le brief §4.

### US-2.1 — Parser et validation du schéma `gc-forge/scenario.v1`
- **Priorité** : P0
- **Estimation** : M
- **Story** : En tant qu'utilisateur, je veux écrire un scénario YAML conforme au schéma `scenario.v1` et avoir des erreurs explicites en cas de problème.
- **Critères** :
  - Tous les champs typés Rust avec `serde`.
  - JSON Schema généré via `schemars` et publié dans `schemas/scenario-v1.json`.
  - Erreurs avec ligne/colonne et suggestion (ex. typo sur `kind`).

### US-2.2 — Mécanisme `extends` et résolution
- **Priorité** : P0
- **Estimation** : S
- **Story** : En tant qu'utilisateur, je veux qu'un scénario puisse hériter d'un autre via `extends:`, afin de réutiliser des bases sans dupliquer.
- **Critères** :
  - Merge récursif des maps, override des scalaires.
  - Détection de cycles d'`extends`.
  - Path résolu relativement au fichier extender.

### US-2.3 — Override CLI `--override`
- **Priorité** : P0
- **Estimation** : S
- **Story** : En tant qu'utilisateur, je veux passer `--override 'spec.gc.options.heap.max=4g'` pour tweaker un scénario sans le copier.
- **Critères** :
  - Syntaxe JSONPath simple (notation `a.b.c`).
  - Override appliqué après `extends`.
  - Type-checked contre le schéma.

### US-2.4 — Sous-commande `gc-forge lint`
- **Priorité** : P0
- **Estimation** : S
- **Story** : En tant qu'utilisateur, je veux valider un scénario sans l'exécuter.
- **Critères** :
  - Exit 0 si valide, exit 1 sinon.
  - Vérifie la compatibilité (jvm/algo, algo/régime).
  - Sortie JSON avec `--output json`.

---

## EPIC 3 — Régimes applicatifs

**Objectif** : produire le comportement GC attendu pour chacun des 7 régimes.

### US-3.1 — Trait `Regime` et registry
- **Priorité** : P0
- **Estimation** : S
- **Story** : En tant que dev GC-Forge, je veux un trait Rust `Regime` et un registry, afin que chaque régime suive le même contrat.
- **Critères** :
  - Trait défini avec méthodes : `id`, `parameters_schema`, `workload_args`, `invariants`, `expected_phenomena`, `validate`.
  - Registry typé `RegimeRegistry::lookup(&str) -> Option<Box<dyn Regime>>`.

### US-3.2 — Régime `steady-state-healthy`
- **Priorité** : P0
- **Estimation** : M
- **Story** : Implémenter R1 côté Rust + côté harness Java.
- **Critères** :
  - Invariants définis (cf. SPEC-FONC §4.1) : young_ratio, full_count, p99_pause_ms.
  - Test d'intégration : preset `steady-g1-baseline` produit un log valide.
  - Variance CV ≤ 5 % sur 5 runs.

### US-3.3 à US-3.8 — Autres régimes
- **Priorité** : P0 (R2, R3, R4, R5, R6, R7)
- **Estimation** : M chacun (sauf R6 et R4 = L)
- **Story** : Idem US-3.2 pour les régimes R2, R3, R4, R5, R6, R7.
- **Critères** : invariants définis, presets passants, variance acceptable.

### US-3.9 — Briques d'allocation Java réutilisables
- **Priorité** : P0
- **Estimation** : M
- **Story** : En tant que dev du harness, je veux des classes utilitaires (`AllocationEngine`, `BurstScheduler`, `LeakReservoir`, `LruCacheChurn`) réutilisables entre régimes.
- **Critères** :
  - Tests unitaires Java sur chaque brique.
  - Reproductibilité à seed donné.

---

## EPIC 4 — Exécution et orchestration

**Objectif** : lancer une JVM, capturer un log, gérer la robustesse.

### US-4.1 — `DockerRunner` MVP
- **Priorité** : P0
- **Estimation** : L
- **Story** : Lancer une JVM dans un container Docker avec les bons flags et capturer le log.
- **Critères** :
  - Images Temurin 17 et 21 supportées.
  - Volume de sortie monté.
  - `--network=none` par défaut.
  - Codes de retour traduits en `RunOutcome`.
  - Timeout configurable, kill propre.

### US-4.2 — Capture du log GC unifié
- **Priorité** : P0
- **Estimation** : S
- **Story** : Construire la string `-Xlog:...` selon l'algo et capturer le fichier de log.
- **Critères** :
  - Flags par algo conformes à SPEC-TECH §6.1.
  - Log écrit sans transformation, hash SHA-256 calculé.

### US-4.3 — Gestion des erreurs JVM
- **Priorité** : P0
- **Estimation** : S
- **Story** : Distinguer OOM, timeout, échec de lancement, erreur runtime.
- **Critères** :
  - `RunOutcome::status` ∈ {Success, Oom, Timeout, JvmError, Internal}.
  - Stderr JVM capturé pour diag.

### US-4.4 — `NativeRunner` (V1)
- **Priorité** : P1
- **Estimation** : L
- **Story** : Téléchargement Adoptium, cache local, exécution native sans Docker.
- **Critères** :
  - Cache `~/.gc-forge/jvms/`.
  - Vérification SHA-256 contre checksums Adoptium.
  - Lock file pour exécutions concurrentes.

### US-4.5 — Sélection auto Docker/Native
- **Priorité** : P1
- **Estimation** : S
- **Story** : Choisir automatiquement le runner disponible.

### US-4.6 — BYO-JVM
- **Priorité** : P1
- **Estimation** : S
- **Story** : Permettre à l'utilisateur de fournir son propre `JAVA_HOME`.

---

## EPIC 5 — Manifeste et reproductibilité

**Objectif** : produire la fiche d'identité de chaque run et garantir la reproductibilité sémantique.

### US-5.1 — Schéma `gc-forge/run-manifest.v1`
- **Priorité** : P0
- **Estimation** : S
- **Story** : Définir la structure du manifeste (cf. SPEC-FONC §6.2).
- **Critères** :
  - Type Rust + JSON Schema généré.
  - Sérialisation YAML par défaut, JSON via flag.

### US-5.2 — Émission du manifeste post-run
- **Priorité** : P0
- **Estimation** : S
- **Story** : Construire un manifeste à partir du scénario résolu et du `RunOutcome`.
- **Critères** :
  - Hash log + hash jar + version GC-Forge présents.
  - `expected_phenomena` peuplés depuis le régime.

### US-5.3 — Hash du harness reproductible
- **Priorité** : P0
- **Estimation** : XS
- **Story** : Le SHA-256 de `workload-harness.jar` doit être stable.
- **Critères** :
  - Build Maven avec timestamps figés.
  - Ordre de fichiers dans le jar déterministe.

---

## EPIC 6 — Validation post-run

**Objectif** : vérifier qu'un log généré respecte bien les invariants annoncés.

### US-6.1 — Parser de log GC partagé
- **Priorité** : P0
- **Estimation** : L
- **Story** : Avoir un parser de log GC unifié dans `gc-core` (ou prototype dans `gc-forge-validate` à pousser ensuite vers `gc-core`).
- **Critères** :
  - Couvre G1, ZGC, Parallel sur JDK 17 et 21.
  - Extraction d'événements suffisante pour tous les invariants définis dans les régimes.
  - Pas un parser exhaustif — focus sur ce dont on a besoin.

### US-6.2 — Engine d'invariants
- **Priorité** : P0
- **Estimation** : M
- **Story** : Évaluer un ensemble d'`Invariant` contre un `ParsedLog`.
- **Critères** :
  - Type `Invariant { rule: String, threshold: Threshold, ... }`.
  - `ValidationReport` consigné dans le manifeste.
  - Codes de retour distinguant succès / violation / erreur de parsing.

### US-6.3 — Sous-commande `gc-forge validate`
- **Priorité** : P0
- **Estimation** : XS
- **Story** : Re-vérifier a posteriori un log + manifeste.
- **Critères** : exit 0 si tous les invariants passent, exit 3 sinon.

---

## EPIC 7 — Presets

### US-7.1 — Presets MVP (14)
- **Priorité** : P0
- **Estimation** : M (1 par régime + variantes)
- **Story** : Livrer les 14 presets MVP de la SPEC-FONC §8.
- **Critères** :
  - Embarqués via `include_str!` dans le binaire.
  - Tous passent `selftest`.

### US-7.2 — `gc-forge presets list/show/export`
- **Priorité** : P0
- **Estimation** : S
- **Story** : Naviguer le catalogue depuis la CLI.
- **Critères** :
  - `list` filtrable par régime/algo.
  - `show` affiche la fiche pédagogique.
  - `export` écrit le YAML brut sur disque pour tweaking.

---

## EPIC 8 — Mode batch

### US-8.1 — Schéma matrix.v1
- **Priorité** : P0
- **Estimation** : S
- **Story** : Définir le schéma `gc-forge/matrix.v1` (cf. SPEC-FONC §9.2).

### US-8.2 — Génération du produit cartésien et filtres
- **Priorité** : P0
- **Estimation** : S
- **Story** : Calculer les cellules à exécuter à partir d'axes et de filtres d'exclusion.

### US-8.3 — Exécution parallèle bornée
- **Priorité** : P0
- **Estimation** : M
- **Story** : Lancer N runs en parallèle, agréger résultats, gérer les échecs partiels.
- **Critères** :
  - `--parallel` configurable.
  - Échec d'une cellule ne bloque pas les autres.
  - Rapport final consolidé.

### US-8.4 — Index CSV
- **Priorité** : P0
- **Estimation** : XS
- **Story** : Émettre `out/index.csv` listant runs et labels.

### US-8.5 — Index Parquet (V1)
- **Priorité** : P1
- **Estimation** : S
- **Story** : Variant Parquet pour datasets ML.

---

## EPIC 9 — Qualité et CI

### US-9.1 — `gc-forge selftest`
- **Priorité** : P0
- **Estimation** : S
- **Story** : Exécuter tous les presets, vérifier invariants, rapport résumé.
- **Critères** : exit code et rapport JSON.

### US-9.2 — Job CI nightly
- **Priorité** : P0
- **Estimation** : S
- **Story** : `selftest` exécuté chaque nuit en CI.

### US-9.3 — `gc-forge variance-check`
- **Priorité** : P0
- **Estimation** : M
- **Story** : Mesurer CV inter-run sur N runs.
- **Critères** :
  - Métriques agrégées calculées : count, sum, p50, p99.
  - CV calculé et rapporté.
  - Rapport HTML (V1) — option `--report.html`.

### US-9.4 — Test cross-projet `gc-core-roundtrip`
- **Priorité** : P0
- **Estimation** : M
- **Story** : Un log produit par GC-Forge est parsé par GC-Insight et la séquence d'événements est reproduite.

---

## EPIC 10 — Documentation et release

### US-10.1 — Getting started
- **Priorité** : P0
- **Estimation** : S
- **Story** : `doc/user/getting-started.md` qui fait passer un nouveau venu de zéro à un log produit en 10 min.

### US-10.2 — Référence des régimes
- **Priorité** : P0
- **Estimation** : M
- **Story** : `doc/user/regimes.md` détaillant chaque régime, ses paramètres, sa signature.

### US-10.3 — Référence du schéma scenario
- **Priorité** : P0
- **Estimation** : S

### US-10.4 — Référence CLI
- **Priorité** : P0
- **Estimation** : S

### US-10.5 — Pitch produit
- **Priorité** : P0
- **Estimation** : S
- **Story** : `doc/pitch.md` 1 page « pourquoi GC-Forge ».

### US-10.6 — Release `0.1.0`
- **Priorité** : P0
- **Estimation** : M
- **Story** : Build matrix, GitHub Release, crates.io, GHCR.

### US-10.7 — Tap Homebrew
- **Priorité** : P1
- **Estimation** : S

---

## EPIC 11 — Reproduction et confort (V1)

### US-11.1 — `gc-forge mirror <prod.log>` heuristique
- **Priorité** : P1
- **Estimation** : L
- **Story** : Cas d'usage F-REPRO du brief — proposer un scénario miroir à partir d'un log de prod.

### US-11.2 — Snippets
- **Priorité** : P1
- **Estimation** : S

### US-11.3 — Export JFR
- **Priorité** : P1
- **Estimation** : M

---

## EPIC 12 — Élargissement JVMs et algos (V1)

### US-12.1 — Corretto et GraalVM CE
- **Priorité** : P1
- **Estimation** : S (Corretto), M (GraalVM)

### US-12.2 — OpenJ9
- **Priorité** : P1
- **Estimation** : XL
- **Story** : OpenJ9 a un format de log GC distinct — extension du parser et tests dédiés.

### US-12.3 — Shenandoah
- **Priorité** : P1
- **Estimation** : M

### US-12.4 — Serial GC
- **Priorité** : P1
- **Estimation** : S

### US-12.5 — Régime `compute-batch`
- **Priorité** : P1
- **Estimation** : M

---

## EPIC 13 — V2 stratégique

### US-13.1 — Synthèse hybride pour ML
- **Priorité** : P2
- **Estimation** : XL

### US-13.2 — `gc-forge sweep` pour datasets paramétriques
- **Priorité** : P2
- **Estimation** : L

### US-13.3 — Mode service HTTP
- **Priorité** : P2
- **Estimation** : XL

### US-13.4 — Zing / Prime / Oracle JDK
- **Priorité** : P2
- **Estimation** : XL (juridique compris)

### US-13.5 — CMS legacy sur JDK 8
- **Priorité** : P2
- **Estimation** : L

---

## Synthèse priorisation

| Tranche | Epics | US | Estimation cumulée (jours-équivalent) |
|---------|-------|-----|----------------------------------------|
| **MVP (P0)** | 1–10 | ~35 US | ~50–60 j ouvrés (cohérent avec 8-10 sem × 5 j × ~12 h soit ~50 j-équivalent) |
| **V1 (P1)** | 11–12 + reliquat | ~10 US | ~25 j |
| **V2 (P2)** | 13 | 5 US | ~40 j |

## Premier sprint suggéré (sem 1)

US prioritaires à ouvrir en premier :
1. US-1.1 — workspace Cargo
2. US-1.2 — projet Maven harness
3. US-1.3 — image Docker runner (peut commencer en parallèle)
4. US-2.1 — parser de scénario (squelette typé)
5. US-3.1 — trait `Regime`

Objectif fin sem 1 : un commit qui fait tourner `gc-forge run` sur un scénario quasi-vide et produit un fichier log/manifeste — même rudimentaire.
