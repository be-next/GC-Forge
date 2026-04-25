# GC-Forge — Roadmap

> **Référence** : brief du 25 avril 2026, §8.3.
> **Cible MVP** : MVP étendu — 8 à 10 semaines en solo temps partiel (~12 h/semaine ouvrées).

## 1. Principe directeur

Livrer en MVP un outil **utile pour le cas d'usage prioritaire** (test et validation de GC-Insight) avec **tous les axes essentiels** (3 algos, 2 versions JDK, 7 régimes, manifeste riche, mode batch). Reporter en V1 ce qui ajoute de la couverture (autres JVMs, autres algos, mode natif, BYO-JVM) plutôt que ce qui ajoute de la valeur centrale.

> **Note de cadrage** : le brief §9 vise 4-6 semaines pour le MVP. Cette roadmap propose 8-10 semaines pour un **MVP étendu** retenu en cadrage initial (3 algos vs 1-2, 2 versions JDK vs 1, 7 régimes vs ~5, 14 presets, métadonnées riches dès la première version). Une trajectoire courte 4-6 semaines reste atteignable et est documentée en §10 (« sensibilités »). C'est au commanditaire de trancher entre les deux trajectoires en fonction de l'urgence vs. de la couverture initiale.

## 2. Vue d'ensemble des phases

| Phase | Durée | Effort cumulé | Sortie |
|-------|-------|---------------|--------|
| **Phase 0 — Setup** | 1 semaine | 1 sem | Workspace Cargo, harness Maven, CI vide-but-verte, premier `hello-world` JVM-en-Docker. |
| **Phase 1 — Tronc commun** | 2 semaines | 3 sem | `gc-forge run scenario.yaml` produit un log + manifeste pour 1 régime (steady-state). |
| **Phase 2 — Régimes et algos** | 3 semaines | 6 sem | 7 régimes implémentés × 3 algos (G1/ZGC/Parallel), 14 presets, validation par invariants. |
| **Phase 3 — Batch, qualité, doc** | 2 semaines | 8 sem | `gc-forge batch`, `selftest`, `variance-check`, doc utilisateur, release `0.1.0`. |
| **Phase 4 (option)** | 2 semaines | 10 sem | Mode natif (NativeRunner), polish CLI, intégration CI GC-Insight. **MVP = fin de phase 4 si capacité.** |
| **V1** | T+3 mois | — | Shenandoah, Serial, Corretto, OpenJ9 ; mirror ; variance-check ; Homebrew. |
| **V2** | T+6 mois | — | Synthèse hybride, Zing/Prime, sweep ML, mode service. |

## 3. Phase 0 — Setup (semaine 1)

**Objectif** : un environnement de dev fonctionnel et une CI verte sur du code minimal.

**Livrables** :
- Workspace Cargo avec les crates de §2.1 SPEC-TECH (squelettes).
- `workload-harness/pom.xml` qui build un fat-jar « hello-world » (boucle d'allocation triviale).
- GitHub Actions : `lint`, `test`, `build`. Verts sur main.
- `gc-forge --version` fonctionne.
- `Dockerfile` pour `ghcr.io/<org>/gc-forge-runner:dev-jdk21` (Temurin 21 + harness embarqué).
- README dev avec `make bootstrap`, `make test`, `make demo`.

**Critères de sortie** : un commit qui passe la CI et qui permet de lancer `make demo` et de voir un log GC sortir d'un container Docker, même rudimentaire.

**Risques** : multi-arch Docker sur Apple Silicon — prévoir 1-2 jours de buffer.

## 4. Phase 1 — Tronc commun (semaines 2–3)

**Objectif** : livrer la chaîne complète sur **un seul** régime (`steady-state-healthy`) et un seul algo (G1 sur Temurin 21).

**Périmètre** :
1. Parser de scénario YAML (`gc-forge-scenario`) avec validation par typage Rust + JSON Schema généré.
2. `extends` + `--override` fonctionnels.
3. `DockerRunner` : peut lancer une JVM Docker, passer des flags, capturer un log.
4. Harness Java `SteadyStateRegime` paramétrable (allocation_rate, live_set, lifetime).
5. `gc-forge run scenario.yaml` produit log + manifeste valides.
6. Manifeste contient scénario résolu + jvm.version + sha256 du log + sha256 du jar.
7. Premier preset : `steady-g1-baseline`.
8. `gc-forge lint` (validation sans exécution).

**Critères de sortie** :
- `gc-forge run presets/steady-g1-baseline.yaml` finit en < 2 min, produit un log valide G1 + un manifeste complet.
- `gc-forge lint` détecte un YAML invalide.
- Test d'intégration CI qui exécute le preset en docker-in-docker.

**Anti-goals** : pas de validation post-run (Phase 2), pas de batch (Phase 3), pas d'autre algo.

## 5. Phase 2 — Régimes et algos (semaines 4–6)

**Objectif** : couvrir les 7 régimes × 3 algos et la validation par invariants.

**Itérations recommandées** (1 itération = 1 semaine) :

### Sprint 4 — Algorithmes
- ZGC generational (flags + capture).
- Parallel.
- Tests unitaires par algo : un même `steady-state` doit produire un log conforme sur les 3 algos.
- Presets baseline pour ZGC et Parallel.

### Sprint 5 — Régimes 1/2
Implémentation et presets :
- `allocation-burst` (R2)
- `humongous-pressure` (R3)
- `cache-churn` (R5)

Pour chaque régime : code Java du harness, code Rust des invariants, preset YAML, test d'intégration.

### Sprint 6 — Régimes 2/2 + validation
Implémentation :
- `slow-leak` (R4)
- `mixed-gc-pathological` (R6)
- `microservice-stop-and-go` (R7)
- `gc-forge validate <log> --manifest <m.yaml>` opérationnel pour tous les régimes.
- Parser de log GC partagé via `gc-core` (au moins suffisant pour les invariants — pas un parser exhaustif).

**Critères de sortie phase 2** :
- 14 presets MVP livrés et passants.
- `gc-forge validate` retourne 0 sur les 14 presets quand exécutés à seed nominal.
- Documentation utilisateur partielle (1 paragraphe par régime + paramètres).
- `gc-core-roundtrip` test inter-projets : un log produit par GC-Forge est parsé par le proto-Insight et reproduit la séquence d'événements.

**Anti-goals** : pas de variance-check (Phase 3), pas de mode natif (Phase 4).

## 6. Phase 3 — Batch, qualité, doc, release `0.1.0` (semaines 7–8)

**Objectif** : packaging et qualité de release.

**Livrables** :
1. `gc-forge batch matrix.yaml` avec parallélisation contrôlée.
2. `gc-forge selftest` : exécute tous les presets, vérifie invariants, rapport résumé.
3. `gc-forge variance-check <preset> --runs N` : mesure CV inter-run.
4. `gc-forge presets list/show/export`.
5. Documentation utilisateur complète :
   - `doc/user/getting-started.md`
   - `doc/user/regimes.md` (un paragraphe par régime + paramètres + signature attendue)
   - `doc/user/scenario-reference.md` (référence du schéma)
   - `doc/user/cli-reference.md` (sous-commandes)
6. CI nightly qui exécute `selftest` + `variance-check`.
7. Release `0.1.0` :
   - GitHub Release avec binaires Linux x64/arm64, macOS x64/arm64.
   - Image Docker `ghcr.io/<org>/gc-forge:0.1.0` et variantes runner-jdk{17,21}.
   - Crates publiés sur crates.io.
   - CHANGELOG, README pitch produit.
8. **Pitch écrit** : `doc/pitch.md` 1 page « pourquoi GC-Forge ».

**Critères de sortie** :
- Un nouvel utilisateur peut installer GC-Forge, lancer un preset et lire le manifeste sans support.
- `gc-forge selftest` passe en CI nightly.
- Variance inter-run mesurée et documentée pour les 14 presets.

**À ce stade le brief §9 doit être démontrable** :
- Un dev Rust comprend l'archi → SPEC-TECH suffit.
- Choix structurants explicites → SPEC-FONC + SPEC-TECH OK.
- Cohérence GC-Insight démontrée → `gc-core-roundtrip` test.
- Pitch interne ou tiers → `doc/pitch.md`.

## 7. Phase 4 — Mode natif et polish (semaines 9–10)

**Objectif** : éliminer la dépendance Docker pour les utilisateurs qui ne l'ont pas.

**Livrables** :
1. `NativeRunner` : téléchargement Adoptium, vérif checksums, cache `~/.gc-forge/jvms/`.
2. Sélecteur auto Docker/Native : utilise Docker si dispo, sinon Native, configurable par flag.
3. Tests d'intégration sur les deux runners pour les 14 presets.
4. Polish CLI : messages d'erreur, suggestions, complétion shell (bash/zsh/fish).
5. Tap Homebrew (`brew install <tap>/gc-forge`).
6. Intégration CI GC-Insight : job `regenerate-corpus-reference` opérationnel et bloquant.

**Critères de sortie** : MVP livré.

**Phase 4 = optionnelle** dans le sens où GC-Forge `0.1.0` (fin Phase 3) est utilisable et utile. Phase 4 fait passer en `0.2.0` avec mode natif et finalise l'intégration GC-Insight.

## 8. V1 — Couverture et confort (T+3 mois après MVP)

**Périmètre V1** :

### V1.0 — Élargissement JVMs
- Corretto 17 et 21 (effectivement Temurin avec une étiquette différente — peu d'effort).
- OpenJ9 17 et 21 — **format de log différent**, demande extension du parser `gc-core`. Effort sérieux.
- GraalVM CE 21 (HotSpot variant — peu d'effort).

### V1.1 — Algos manquants
- Shenandoah (effort moyen — proche de ZGC en termes de tooling).
- Serial (trivial).
- Compute-batch régime supplémentaire.

### V1.2 — Reproduction de pathologies
- `gc-forge mirror <prod.log>` heuristique (cas d'usage F-REPRO du brief).
- Onboarding : « collez votre log, je vous propose un scénario miroir ».

### V1.3 — Confort
- Export JFR (`capture_jfr: true` dans scenario).
- Sortie HTML auto-contenue de variance-check (`--report.html`).
- Snippets (option `--snippet humongous`).

## 9. V2 — Stratégique (T+6 mois)

**Pistes** :
- **Synthèse hybride** : générateur Rust calibré sur traces réelles, pour datasets ML.
- **Mode `sweep`** : balayage paramétrique sur N seeds.
- **Zing / Prime / Oracle JDK** sous condition de licence.
- **Mode service** : daemon HTTP qui produit des logs à la demande (SaaS interne).
- **CMS legacy** sur JDK 8 pour cas historiques (effort spécifique).

## 10. Estimation et cadence

**Hypothèse de capacité** : 12 h ouvrées / semaine en solo (compatible « temps partiel » du brief). Avec une marge de 20 %, capacité réaliste = **9-10 h utiles / semaine**.

| Phase | Durée calendaire | Effort utile | Buffer |
|-------|------------------|--------------|--------|
| 0 | 1 sem | 8 h | 2 h |
| 1 | 2 sem | 18 h | 4 h |
| 2 | 3 sem | 27 h | 6 h |
| 3 | 2 sem | 18 h | 4 h |
| 4 | 2 sem | 18 h | 4 h |
| **Total** | **10 sem** | **89 h** | **20 h** |

**Sensibilités** :
- +30 % si OpenJ9 entre dans le MVP (parsing différent).
- +20 % si la variance inter-host pose problème et impose un travail de stabilisation (CPU pinning, etc.).
- −10 à −20 % si on coupe Phase 4 (mode natif reporté en V1.0).
- **−40 % vers une trajectoire « MVP brief original » (4-6 sem)** : 1 algo (G1 only), 1 JDK (21 only), 5 régimes (couper R5/R6/R7), pas de mode batch, pas de Phase 3 polish ni Phase 4. Périmètre : `gc-forge run scenario.yaml` avec validation a posteriori, 5 presets. C'est un démonstrateur, pas encore un outil de validation CI.

**Recommandation** : viser fin Phase 3 (release `0.1.0`) à 8 semaines, garder Phase 4 comme optionnelle si la capacité tient. Si urgence ou capacité réduite, basculer sur la trajectoire courte explicitée ci-dessus.

## 11. Jalons et points de décision

| Jalon | Sem | Décision/Validation à prendre |
|-------|-----|-------------------------------|
| J1 — Bootstrap CI verte | 1 | Aucune. Si non atteint en 1 sem, simplifier la matrice CI. |
| J2 — Premier log produit | 3 | Valider le format de log capturé (cohérence Insight). |
| J3 — 3 algos × steady-state | 4 | Vérifier que les 3 parsers de log sont alignés. |
| J4 — 7 régimes implémentés | 6 | **Go/no-go release** : si la qualité des invariants n'est pas au rendez-vous, replanifier. |
| J5 — Release `0.1.0` | 8 | Annonce publique, mise en visibilité GC-Insight. |
| J6 — Mode natif | 10 | MVP livré ou décision de packaging V1. |

## 12. Dépendances externes critiques

- **Disponibilité de `gc-core`** : si `gc-core` est immature côté GC-Insight au démarrage de Phase 1, ajouter 2 semaines pour le stabiliser ou cloner les structures dans GC-Forge temporairement.
- **Images Docker Eclipse Temurin** : très stable. Pas de risque.
- **Adoptium API** (V1) : stable depuis 2021, peu risquée.
- **Pas de dépendance AWS** — conformément au brief §6.

## 13. Métriques de succès post-MVP

À mesurer 4 semaines après release `0.1.0` :
- ≥ 1 contributeur externe (issue ou PR).
- Corpus de référence GC-Insight régénéré 100 % en CI.
- ≥ 3 articles de blog ou supports utilisant des logs GC-Forge.
- Présence dans l'index search (`site:github.com gc-forge`).
- Bug-rate sur les invariants : 0 régression silencieuse sur les 14 presets.
