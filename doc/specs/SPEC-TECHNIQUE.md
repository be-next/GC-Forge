# GC-Forge — Spécifications techniques

> **Référence** : brief du 25 avril 2026, §5, §6.
> **Statut** : V1.0 — émise par Cowork.
> **Lecteur cible** : développeur Rust qui doit pouvoir démarrer l'implémentation du MVP sans ambiguïté majeure.

## 1. Vue d'ensemble

GC-Forge est composé de :

```
┌─────────────────────────────────────────────────────────────┐
│  gc-forge (CLI Rust)                                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │ scenario │  │ regimes  │  │ runner   │  │ validate │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
│              ↓ readlock ↓                                   │
│           gc-core (crate partagé avec GC-Insight)           │
└─────────────────────────────────────────────────────────────┘
              │                                   │
              │ lance                             │ parse logs
              ▼                                   ▼
┌──────────────────────────┐          ┌──────────────────────┐
│  JVM (HotSpot/Temurin)   │   ───►   │  log GC natif        │
│  + workload-harness.jar  │          │  manifest.yaml       │
│  (Docker ou natif)       │          └──────────────────────┘
└──────────────────────────┘
```

Trois composants distribuables :
1. Un **binaire CLI** Rust (`gc-forge`).
2. Un **harness Java** (`workload-harness.jar`) — fat-jar paramétré par CLI/JSON.
3. Optionnellement des **images Docker** légères (Temurin + harness embarqué).

## 2. Workspace Rust et cohérence avec GC-Insight

### 2.1 Workspace Cargo

GC-Forge **n'est pas** un sous-module de GC-Insight ; les deux projets coexistent dans deux repos distincts mais consomment un même crate `gc-core` publié.

```
gc-insight/                          # repo séparé, déjà existant
├── crates/
│   ├── gc-core/                     # ◄── source de vérité
│   ├── gc-insight-cli/
│   └── gc-insight-lambda/

gc-forge/                            # ce repo
├── Cargo.toml                       # workspace
├── crates/
│   ├── gc-forge-cli/                # binaire CLI
│   ├── gc-forge-scenario/           # parsing/validation YAML scenario
│   ├── gc-forge-regimes/            # implémentations des 7 régimes
│   ├── gc-forge-runner/             # orchestration JVM (Docker/natif)
│   ├── gc-forge-validate/           # validation post-run
│   └── gc-forge-presets/            # presets embarqués (build.rs)
├── workload-harness/                # projet Java/Maven
│   ├── pom.xml
│   └── src/main/java/...
├── schemas/                         # JSON Schemas générés
├── presets/                         # YAML scenarios
└── doc/specs/                       # ce répertoire
```

### 2.2 Le crate `gc-core` partagé

`gc-core` est versionné en SemVer, publié sur crates.io (ou registre privé GitLab le temps de la phase pré-1.0).

**Ce que `gc-core` contient (cohérence Forge ↔ Insight)** :

```rust
// gc-core/src/model/mod.rs (schéma indicatif, à itérer en implémentation)
pub mod jvm {
    pub enum Vendor { Temurin, Corretto, GraalVM, OpenJ9, Other(String) }
    pub struct JvmInfo { pub vendor: Vendor, pub version: Version, pub flags: Vec<String> }
}

pub mod gc {
    pub enum Algorithm { G1, ZGC { generational: bool }, Parallel, Shenandoah, Serial, CMS }
    pub struct HeapConfig { pub min: ByteSize, pub max: ByteSize, pub new_size: Option<ByteSize> }
    pub enum EventKind { YoungGc, MixedGc, FullGc, ConcurrentMark, EvacuationFailure, HumongousAllocation, ... }
    pub struct GcEvent { pub kind: EventKind, pub timestamp: Duration, pub pause_ms: f64, pub heap_before: u64, pub heap_after: u64, ... }
}

pub mod regime {
    pub enum RegimeKind { SteadyStateHealthy, AllocationBurst, HumongousPressure, SlowLeak, CacheChurn, MixedGcPathological, MicroserviceStopAndGo }
    pub struct PhenomenonId(pub &'static str);
}

pub mod manifest {
    pub struct RunManifest { ... }     // schéma §6 SPEC-FONCTIONNELLE
}
```

**Ce que `gc-core` ne contient pas** :
- Les heuristiques d'analyse de GC-Insight (restent dans `gc-insight-core`).
- Les implémentations des régimes (restent dans `gc-forge-regimes`).
- Le parser de log unifié (peut être dans `gc-core` si jugé partageable, sinon spécifique à Insight).

### 2.3 Stratégie d'évolution de `gc-core`

- Toute évolution de `gc-core` qui casse Forge ou Insight = bump majeur.
- Période de pré-1.0 : `0.x.y` avec bumps mineurs autorisés à casser, **sous condition** de mettre à jour Forge et Insight dans la même release coordonnée.
- À partir de 1.0 : règle SemVer stricte.

**Garde-fou de cohérence** : un test d'intégration `gc-core-roundtrip` qui parse un log produit par Forge avec le parser d'Insight et reconstruit la séquence d'événements.

## 3. Approche réelle vs synthétique — décision

**Décision : approche réelle au MVP, hybride en V2.**

### 3.1 Argumentaire

Pour : approche réelle (exécution vraie JVM + harness Java).
- **Fidélité absolue du format** : un log produit par HotSpot avec `-Xlog:gc*` est par construction conforme à ce que produit la même JVM en prod. Aucun risque de divergence cachée.
- **Robustesse aux évolutions JVM** : si Oracle modifie le format du log GC en JDK 25, on n'a rien à patcher : on relance la nouvelle JVM, le log change comme en prod.
- **Crédibilité produit** : sur un sujet où GC-Insight se vend par sa précision, fournir des fixtures synthétiques serait un péché originel.
- **Simplicité d'ingénierie** : un harness Java de quelques milliers de lignes est plus simple à maintenir qu'un générateur Rust qui doit modéliser 6 algorithmes GC × N versions de JVM.

Pour : approche synthétique (générer le log directement en Rust).
- Plus rapide (pas de cold-start JVM).
- Reproductibilité bit-à-bit triviale.
- Pas de dépendance aux JVMs.

Contre : approche synthétique.
- **Dette de maintenance** : il faudrait un modèle par-vendor-par-version-par-algo, et le maintenir à jour à chaque release JDK.
- **Risque de drift** : un format synthétique va inévitablement diverger du format réel sur des cas que personne n'a anticipés.
- **Aveu de faiblesse** : « nos fixtures sont fausses, mais c'est suffisant ». Inadmissible pour un produit dont la valeur est l'analyse.

### 3.2 Place de la synthèse — V2

L'approche synthétique conserve un rôle en V2 pour des cas où l'exécution réelle est coûteuse ou impossible :
- **Volumes massifs** pour ML : générer 10 000 logs courts pour un dataset.
- **Cas extrêmes** difficiles à provoquer en réel (ex. très grand heap, très long run) — synthèse à partir d'une trace réelle « dilatée ».
- **Bench déterministe de GC-Insight** : un sous-ensemble synthétique avec hash bit-stable pour la CI.

Le module `gc-forge-synth` (V2) générera des logs **calibrés sur** des logs réels du même régime, pas à partir de zéro. Architecture : un autoencoder ou un modèle de Markov par régime, entraîné sur le corpus réel.

### 3.3 Conséquence sur l'architecture MVP

Pas de module synthèse en MVP. La trajectoire vers la synthèse n'impose **aucune contrainte** sur l'architecture MVP — l'API Rust qui produit un log et son manifeste est strictement la même qu'on l'obtienne d'une JVM ou d'un générateur.

## 4. Architecture des composants

### 4.1 `gc-forge-cli` — point d'entrée

CLI bâtie sur `clap` (cohérence GC-Insight). Sous-commandes : `run`, `batch`, `validate`, `lint`, `presets`, `variance-check`, `selftest`.

Pseudo-code du flot `run` :

```rust
fn cmd_run(args: RunArgs) -> Result<()> {
    let scenario = scenario::load(&args.path)?
        .merged_with(&args.overrides)?
        .resolved();
    scenario::lint(&scenario)?;

    let regime = regimes::resolve(&scenario.spec.regime)?;
    let runner = runner::pick(&args)?;            // Docker | Native
    let workload_args = regime.workload_args(&scenario);

    let outcome = runner.execute(RunSpec {
        jvm: scenario.spec.jvm.clone(),
        flags: scenario.spec.gc.flags(),
        log_target: args.out_dir.join("gc.log"),
        workload_args,
        duration: scenario.spec.duration,
    })?;

    let manifest = manifest::build(&scenario, &outcome)?;
    let validation = validate::run(&manifest, &outcome.log_path)?;
    manifest::write(&args.out_dir, manifest.with(validation))?;
    Ok(())
}
```

### 4.2 `gc-forge-scenario` — parsing et résolution

- Parsing : `serde_yaml` → `Scenario` typé.
- Validation : JSON Schema généré via `schemars` (publié dans `schemas/`), validation runtime via `jsonschema` ou directement par typage Rust + invariants.
- `extends:` résolu récursivement avec détection de cycle.
- `--override` parsé en JSONPath simple, appliqué après merge `extends`.

### 4.3 `gc-forge-regimes` — bibliothèque des régimes

Chaque régime est un module Rust qui implémente :

```rust
pub trait Regime {
    fn id() -> &'static str;
    fn parameters_schema() -> JsonSchema;
    fn workload_args(&self, scenario: &Scenario) -> Vec<String>;     // → CLI args du harness Java
    fn invariants(&self, scenario: &Scenario) -> Vec<Invariant>;
    fn expected_phenomena(&self, scenario: &Scenario) -> Vec<PhenomenonId>;
    fn validate(&self, parsed: &ParsedLog, scenario: &Scenario) -> ValidationReport;
}
```

Le régime ne **génère pas** le log : il prépare les arguments à passer au harness Java et fournit les règles de validation post-run.

### 4.4 `workload-harness` — harness Java paramétrable

Un seul fat-jar qui couvre les 7 régimes. Architecture :

```java
public class WorkloadHarness {
    public static void main(String[] args) {
        Args parsed = Args.parse(args);
        Regime regime = RegimeRegistry.lookup(parsed.regime());
        regime.run(parsed.params(), parsed.duration(), parsed.seed());
    }
}

public interface Regime {
    void run(Map<String,Object> params, Duration duration, long seed);
}

public class SteadyStateRegime implements Regime { ... }
public class AllocationBurstRegime implements Regime { ... }
public class HumongousPressureRegime implements Regime { ... }
// etc.
```

**Choix techniques harness** :
- Java 17 (compile target) — garantit compatibilité Temurin 17 et 21.
- Pas de framework lourd : pas de Spring, pas de Quarkus. JDK + une dépendance optionnelle JMH si on veut des micro-bench déterministes.
- Allocation contrôlée par primitives `byte[]`, `Object[]`, `String` selon le régime.
- Build : Maven ou Gradle ; **Maven** retenu pour simplicité (`mvn package -DskipTests` produit le fat-jar).
- Le hash SHA-256 du jar est consigné dans le manifeste (reproductibilité).

**Pourquoi un seul harness paramétrable plutôt que N projets** :
- Maintenance : un seul cycle build/release.
- Réutilisation : primitives d'allocation partagées (`AllocationEngine`, `LeakReservoir`, `BurstScheduler`).
- Testabilité : tests unitaires Java sur les briques.
- Inconvénient : un peu plus de complexité dans le harness lui-même — accepté.

### 4.5 `gc-forge-runner` — orchestration JVM

Deux backends qui implémentent le même trait :

```rust
pub trait Runner {
    fn name(&self) -> &'static str;
    fn check_available(&self) -> Result<()>;
    fn ensure_jvm(&self, jvm: &JvmSpec) -> Result<JvmHandle>;
    fn execute(&self, spec: RunSpec) -> Result<RunOutcome>;
}

pub struct DockerRunner { ... }      // MVP par défaut
pub struct NativeRunner { ... }      // V1
```

#### 4.5.1 `DockerRunner` (MVP)

- Image de base : `eclipse-temurin:21-jdk` ou `:17-jdk`.
- Variante avec harness embarqué : `ghcr.io/<org>/gc-forge-runner:21` (publiée par la CI GC-Forge).
- Lancement : `docker run --rm -v <out>:/out -v <jar>:/work/harness.jar:ro <image> java <flags> -jar /work/harness.jar <args>`.
- CPU/mémoire : par défaut, hériter des limites du host. Option `--docker-cpus`, `--docker-memory` exposée pour reproductibilité.
- Réseau désactivé par défaut (`--network=none`) — le harness n'a pas besoin de réseau.

#### 4.5.2 `NativeRunner` (V1)

- Téléchargement à la demande des distributions Adoptium dans `~/.gc-forge/jvms/temurin-{17,21}/`.
- Vérification SHA-256 contre les checksums officiels Adoptium.
- Cache local + lock file pour exécutions concurrentes.
- Lancement direct du `java` extrait, avec mêmes flags qu'en Docker.

#### 4.5.3 Choix Docker prioritaire en MVP

- **Reproductibilité** : un tag d'image + un harness jar = un environnement figé. Native demande de gérer un cache, des téléchargements, des distributions.
- **Multi-OS** : Docker fonctionne identiquement sur Linux et macOS (avec rosetta sur Apple Silicon pour ARM64 — anyway Temurin a des images multi-arch).
- **Pas de pollution du système** : l'utilisateur n'a pas à installer 5 JDKs.
- **Coût** : Docker Desktop sur macOS = lourd. C'est l'argument pour livrer le natif en V1 et offrir le choix.

### 4.6 `gc-forge-validate` — validation post-run

- Parse le log avec un parser **partagé** avec GC-Insight (idéalement dans `gc-core`).
- Applique les `Invariant`s fournis par le régime + les `expected_invariants` custom du scénario.
- Produit un `ValidationReport` consigné dans le manifeste.

### 4.7 `gc-forge-presets` — presets embarqués

- Les YAML des presets sont embarqués dans le binaire via `include_str!` (généré par `build.rs`).
- `gc-forge presets export <name>` les écrit sur disque.
- Tests : chaque preset doit `lint` proprement et son `selftest` passe.

## 5. Gestion du parc JVM — décision

**Décision : Docker en MVP, natif via téléchargement Adoptium en V1, BYO-JVM ouvert via flag avancé en V1+.**

### 5.1 Comparaison des stratégies

| Stratégie | Reproductibilité | Coût d'install | Multi-OS | Vitesse cold start | Verdict |
|-----------|------------------|----------------|----------|--------------------|---------| 
| Docker images officielles | ★★★★★ | ★★★ (Docker Desktop sur Mac) | ★★★★★ | ★★★ (1-2 s) | **MVP** |
| Téléchargement Adoptium auto (style SDKMAN) | ★★★★ | ★★★★ | ★★★★★ | ★★★★★ (instant après cache) | **V1** |
| BYO-JVM (l'utilisateur fournit `JAVA_HOME`) | ★★ (dépend du host) | ★★★★★ | ★★★★ | ★★★★★ | V1, opt-in |
| SDKMAN comme dépendance externe | ★★★ | ★★ (install séparé) | ★★★★ | ★★★★ | rejeté |

### 5.2 Pourquoi Docker prioritaire

Le principal cas d'usage du MVP est **CI de GC-Insight + démos commerciales sur poste dev**. Dans les deux cas, Docker est déjà présent ou trivial à installer. La reproductibilité prime sur la vitesse cold-start.

### 5.3 Pourquoi natif prioritaire en V1

Pour les utilisateurs qui veulent `gc-forge run scenario.yaml` instantanément sans Docker (équipes ops, formateurs), un mode natif avec cache local est précieux. C'est un effort d'une ou deux semaines en V1.

### 5.4 BYO-JVM (V1+, opt-in)

```bash
gc-forge run scenario.yaml --jvm-mode=byo --java-home=/opt/zulu21
```

Utilisé pour : tester sur Zing/Prime sans publier d'image officielle, ou Oracle JDK que GC-Forge ne peut pas redistribuer pour des raisons de licence.

## 6. Capture du log GC

### 6.1 Flags de log standard

GC-Forge impose les mêmes flags `-Xlog` pour tous les runs (cohérence avec le parser GC-Insight) :

```
-Xlog:gc*=info,gc+heap=debug,gc+age=trace,gc+phases=debug,gc+humongous=trace:file=<path>:time,level,tags,pid,tid:filecount=0
```

Justifications :
- `gc*=info` : flux principal (collections, pauses).
- `gc+heap=debug` : tailles avant/après par génération.
- `gc+age=trace` : tenuring distribution (utile pour cache-churn, mixed-pathological).
- `gc+phases=debug` : décomposition des phases (root scan, copy, etc.).
- `gc+humongous=trace` : trace des allocations humongous (G1).
- `time,level,tags` : décorateurs canoniques.
- `filecount=0` : pas de rotation, on veut un fichier unique.

### 6.2 Suppression du timestamp wall-clock pour reproductibilité (V1)

Pour tendre vers une reproductibilité bit-à-bit du **manifeste** (le log lui-même reste non-bit-stable, cf. §4.7 SPEC-FONC), on peut au choix :
- Activer `uptime` au lieu de `time` dans les décorateurs (timestamps en secondes depuis le démarrage JVM, reproductibles à la dérive de l'horloge près).
- Post-traiter le log pour rebaser les timestamps relatifs au début (option `--rebase-timestamps`).

Décision : par défaut, **`time` (ISO 8601)** comme un log de prod. Option `--rebase-timestamps` exposée pour la CI.

### 6.3 Cas particuliers algorithmes

| Algo | Spécificités à capturer |
|------|-------------------------|
| G1 | `humongous`, `ergonomics`, `marking`, `phases` |
| ZGC | `nmethod`, `phases`, `heap`, `start`, `marking`, `relocation` |
| Parallel | `phases`, `heap` |

Les flags exacts par algo seront consignés dans `gc-forge-runner/src/log_flags.rs` et générés à partir de l'algo du scénario.

## 7. Stratégie de tests et qualité

### 7.1 Pyramide des tests

| Niveau | Cible | Outil |
|--------|-------|-------|
| Unitaire Rust | Parsing scenario, règles d'invariant, résolution `extends` | `cargo test` |
| Unitaire Java | Briques d'allocation du harness | JUnit 5 |
| Intégration courte (90 s par test) | 1 preset par régime, validation des invariants | `cargo test -- --ignored` (CI nightly) |
| Intégration longue | Tous les presets, variance inter-run | CI weekly |
| Cross-projet | Log Forge → parsé par Insight → invariants vérifiés | `gc-core-roundtrip` |
| Performance | Cold start CLI < 200 ms, MVP scenario complet < 5 min | `criterion` |

### 7.2 `gc-forge selftest`

Commande unique qui rejoue tous les presets MVP, vérifie leurs invariants, mesure variance et émet un rapport. Cible CI nightly.

### 7.3 Determinism harness

Le harness Java doit être **déterministe à seed fixé** :
- `java.util.Random` initialisé par seed unique.
- Pas de `Math.random()` (état global).
- Pas de `System.currentTimeMillis()` dans les décisions de logique métier (uniquement pour mesurer la durée totale).
- `Thread.sleep` autorisé mais non-critique pour le déterminisme du log GC.

Test : `gc-forge variance-check <preset>` doit montrer CV ≤ 5 % sur métriques agrégées.

### 7.4 Couverture

Cible MVP : ≥ 75 % sur les crates Rust. Pas d'objectif strict sur le harness Java (les invariants Rust valident le résultat).

## 8. Reproductibilité et déterminisme

### 8.1 Niveaux de reproductibilité

| Cible | Atteignable | Stratégie |
|-------|-------------|-----------|
| Manifeste résolu | Oui, bit-à-bit | Snapshot du scénario après merge/override + hash du harness + version GC-Forge |
| Workload harness | Oui, bit-à-bit | Build reproductible Maven (mvn 3.9+, Java 17, deps figées dans `pom.xml`) ; SHA-256 du jar consigné |
| Comportement GC | Sémantiquement reproductible | Seed fixé, JVM identifiée par tag d'image, OS/arch consignés dans le manifeste |
| Log GC bit-à-bit | **Non atteignable** sur JVM réelle | Documenté comme limite ; validation par invariants au lieu de hash |

### 8.2 Sources de non-déterminisme listées

- Timing du planificateur OS (impact sur déclenchement concurrent cycles).
- Fréquence CPU dynamique (P/E cores sur Apple Silicon, turbo Intel).
- Charge système concurrente.
- Adresses mémoire (ASLR) — peuvent influer sur certaines optimisations JIT.
- Wall-clock dans le log (mitigé par `--rebase-timestamps`).

### 8.3 Garanties offertes

- **Même scénario, même seed, même image Docker** ⇒ phénomènes attendus présents avec probabilité ≥ 99 % (mesurée par variance-check).
- **Même scénario, hôtes différents** ⇒ invariants tenus à ± 5 % sur métriques agrégées, sauf p99 (± 15 %).

### 8.4 CI : régénération du corpus

`make corpus-reference` (cible Makefile) déclenche `gc-forge batch` sur la matrice de référence et publie l'archive `corpus-reference-<git_sha>.tar.zst` dans les artifacts GitHub. La CI de GC-Insight la consomme par version.

## 9. Open-source vs propriétaire — décision

**Décision : open-source sous MIT dès le MVP.**

### 9.1 Argumentaire

Pour : open-source.
- **Effet de levier marketing pour GC-Insight** : si GC-Forge devient un standard de fait pour produire des fixtures GC dans la communauté JVM, chaque utilisateur croise GC-Insight comme produit auteur.
- **Adoption par les équipes performance** : DBAs, ops, équipes perf qui veulent reproduire un comportement chez eux. Ces utilisateurs sont aussi nos prospects pour Insight.
- **Contributions externes** : chaque utilisateur qui ajoute un régime ou une JVM apporte de la valeur sans coût.
- **Crédibilité technique** : sur un sujet expert (GC tuning), un outil open-source vérifiable inspire confiance.
- **Pas de secret algorithmique** : un harness paramétrique + un orchestrateur, c'est de l'ingénierie reproductible. Le moat n'est pas là.
- **Le moat de GC-Insight est ailleurs** : SaaS, heuristiques d'analyse, expérience produit, dataset, support.

Contre : open-source.
- Risque de fork hostile par un concurrent. **Évalué bas** : très peu d'éditeurs concurrents directs sur l'analyse de GC logs ; ceux qui existent ont déjà leurs propres outils internes.
- Coût de gouvernance (issues, PRs, releases, sécurité). **Acceptable** : MIT = peu de contraintes, gouvernance minimale possible (BDFL).

### 9.2 Modalités

- **Licence** : MIT (permissive, compatible avec usage commercial, simplicité maximale — pas de clause brevet explicite, ce qui correspond au profil de risque IP de GC-Forge : un harness paramétrique sans innovation algorithmique brevetable). Déjà en place dans `LICENSE`.
- **Repo** : public sur GitHub (`<org>/gc-forge`).
- **CLA** : Developer Certificate of Origin (DCO), pas de CLA lourd au démarrage.
- **Gouvernance** : BDFL (Jérôme) ; passage en gouvernance ouverte si adoption significative (> 10 contributeurs réguliers).
- **Code of Conduct** : Contributor Covenant standard.
- **Releases** : SemVer. Tags `vX.Y.Z`. CHANGELOG tenu à jour.

### 9.3 Implications sur GC-Insight

Aucune dépendance technique inverse : GC-Insight ne dépend pas de GC-Forge en runtime (seulement de `gc-core` qui est lui-même indépendant). GC-Forge peut donc évoluer en open-source sans contraindre Insight.

`gc-core` doit décider : open-source (recommandé, comme une « lingua franca » du domaine) ou propriétaire (acceptable mais oblige à mainternir un fork dans GC-Forge). **Recommandation** : `gc-core` open-source MIT, **sans** les heuristiques d'analyse de GC-Insight (qui restent dans `gc-insight-core` propriétaire).

## 10. Packaging et distribution

### 10.1 Cibles MVP

| Plateforme | Format | Construction | Distribution |
|------------|--------|--------------|--------------|
| Linux x86_64 | binaire statique (`musl`) | `cargo build --release --target=x86_64-unknown-linux-musl` | GitHub Releases + `cargo install` |
| macOS x86_64 | binaire | `cargo build --release` | GitHub Releases + Homebrew tap (V1) |
| macOS aarch64 (Apple Silicon) | binaire | `cargo build --release --target=aarch64-apple-darwin` | GitHub Releases + Homebrew tap (V1) |
| Linux aarch64 | binaire | cross via `cross` | GitHub Releases |
| Windows | — | — | V2 |

### 10.2 Images Docker (MVP)

- `ghcr.io/<org>/gc-forge:<version>` — image runtime contenant `gc-forge` + `workload-harness.jar`.
- `ghcr.io/<org>/gc-forge-runner:<version>-jdk{17,21}` — variantes embarquant Temurin pour l'exécution self-contained.

### 10.3 Cargo et publication

- Crates publiés sur crates.io : `gc-core`, `gc-forge-scenario`, `gc-forge-regimes`, `gc-forge-runner`, `gc-forge-validate`. Le binaire `gc-forge-cli` est aussi publié pour `cargo install gc-forge-cli`.
- `workload-harness.jar` publié comme asset GitHub Release (et via Maven Central en V1 si ça a un sens).

### 10.4 Installation utilisateur — UX visée

```bash
# via cargo
cargo install gc-forge-cli

# via Homebrew (V1)
brew install <tap>/gc-forge

# via Docker
docker run --rm -v $PWD:/work ghcr.io/<org>/gc-forge run /work/scenario.yaml
```

## 11. Intégration CI

### 11.1 CI GC-Forge (côté ce repo)

- GitHub Actions, jobs :
  - `lint` : `cargo fmt --check`, `cargo clippy -D warnings`, `cargo deny check`, `mvn validate` côté harness.
  - `test` : `cargo test --workspace` (unitaire), `mvn test` côté harness.
  - `selftest` : `gc-forge selftest` sur tous les presets — déclenché en nightly (10-15 min).
  - `release` : sur tag, build matrix, publication Releases + crates.io + GHCR.

### 11.2 CI GC-Insight (côté repo voisin)

Un job `regenerate-corpus-reference` qui :
1. Pulle l'image `ghcr.io/<org>/gc-forge-runner:<version>-jdk21`.
2. Exécute `gc-forge batch corpus-reference.yaml --out-dir corpus/`.
3. Compare le résultat avec un baseline (sur invariants, pas sur hash).
4. Échoue si un invariant est violé ou si un nouveau phénomène inattendu apparaît.

### 11.3 Versions et compatibilité

- Une release GC-Insight déclare un range de versions GC-Forge supporté (ex. `>=0.3, <0.4`).
- Une matrice de compatibilité est publiée dans le README de GC-Insight.

## 12. Sécurité, sandboxing, exécution

- Le harness Java est **non privilégié**, sans accès réseau (`--network=none` en Docker).
- Volumes Docker en lecture seule sauf répertoire de sortie.
- Pas d'exécution de code utilisateur arbitraire dans le harness — seulement des paramètres typés.
- Resource limits exposés : `--cpus`, `--memory` Docker, `ulimit` natif.

## 13. Observabilité de GC-Forge lui-même

- Logs CLI structurés (`tracing` + `tracing-subscriber`), niveau ajustable.
- Sortie machine : `--output json` retourne le manifeste + résultat de validation en JSON sur stdout.
- Telemetry : aucune par défaut. Option opt-in `--telemetry` en V2 si on veut mesurer adoption (anonymisé, désactivable).

## 14. Risques techniques saillants (renvoi)

Les risques techniques structurants sont consignés dans [RISQUES.md](./RISQUES.md). Les plus saillants à garder en tête à l'implémentation :
- Variance inter-host trop élevée pour valider les invariants (mitigation : Docker + pinning CPU).
- Coût de génération du corpus en CI (mitigation : caching, parallélisation, runs courts).
- Évolution du format de log GC entre JDKs (mitigation : tests par version, parser `gc-core` versionné).
- Licence des distributions JVM tierces (mitigation : Temurin only en MVP).

---

**Lecture recommandée pour démarrer l'implémentation** : §2 (workspace), §4 (composants), §5 (runner), §6 (capture log), puis §8 (reproductibilité). Le harness Java peut démarrer en parallèle de la CLI Rust dès que le contrat d'arguments (§4.4) est figé.
