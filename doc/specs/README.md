# GC-Forge — Spécifications

Production des spécifications fonctionnelles et techniques pour GC-Forge, le générateur de GC logs Java pendant de [GC-Insight](#).

**Statut** : V1.0 — émis le 25 avril 2026 par Cowork à la demande de Jérôme.

## Documents

| # | Document | Objet |
|---|----------|-------|
| 1 | [SPEC-FONCTIONNELLE.md](./SPEC-FONCTIONNELLE.md) | Cas d'usage, modèle de scénario YAML, catalogue des régimes/JVMs/algos, formats d'entrée et de sortie, presets pré-packagés du MVP. |
| 2 | [SPEC-TECHNIQUE.md](./SPEC-TECHNIQUE.md) | Architecture, choix technologiques argumentés, workspace Rust, harness Java, gestion du parc JVM, packaging, stratégie de tests, reproductibilité, CI. |
| 3 | [ROADMAP.md](./ROADMAP.md) | Trajectoire MVP → V1 → V2, jalons, critères de sortie, dépendances. |
| 4 | [BACKLOG.md](./BACKLOG.md) | Epics et user stories de premier niveau, priorisation MVP. |
| 5 | [RISQUES.md](./RISQUES.md) | Registre des risques (techniques, projet, légaux, produit) et points ouverts. |

## Synthèse des décisions structurantes

Les choix tranchés dans les specs (référence : §7 du brief) :

| # | Question | Décision retenue | Document |
|---|----------|------------------|----------|
| 1 | Approche réelle vs synthétique | **Réelle** au MVP (exécution de vraies JVMs avec workloads paramétrables), trajectoire **hybride** en V2 (synthèse pour cas extrêmes). | TECH §3 |
| 2 | Modèle de scénario | **YAML déclaratif** (schéma `gc-forge/scenario v1`), validé par JSON Schema. | FONC §5 |
| 3 | Régimes MVP | **7 régimes** : steady-state-healthy, allocation-burst, humongous-pressure, slow-leak, cache-churn, mixed-gc-pathological, microservice-stop-and-go. | FONC §4 |
| 4 | JVMs/algos MVP | **Eclipse Temurin** (HotSpot) JDK 17 & 21, algos **G1 / ZGC / Parallel**. Corretto, GraalVM, OpenJ9, Shenandoah, Serial reportés en V1. | FONC §3 |
| 5 | Gestion parc JVM | **Docker en MVP** (eclipse-temurin officielles), **mode natif** via téléchargement Adoptium en V1. Pas de BYO-JVM avant V1. | TECH §5 |
| 6 | Open-source vs propriétaire | **Open-source MIT**. Levier marketing pour GC-Insight, moat ailleurs. | TECH §9 |
| 7 | Format métadonnées | **YAML** par défaut (cohérent avec scénario), **JSON** optionnel pour CI/ML. Schéma `gc-forge/run-manifest v1`. | FONC §6 |
| 8 | Critères de qualité | **Rules-as-code** par régime (invariants quantifiés), **suite de validation** rejouée à chaque release, **variance inter-run** ≤ 5 % sur métriques clés. | FONC §7, TECH §7 |

## Cohérence avec GC-Insight

GC-Forge réutilise et étend l'écosystème GC-Insight :

- **Crate partagé `gc-core`** : structures communes (modèle GC, événements normalisés, identifiants) — voir TECH §2.
- **Philosophie déclarative YAML** : mêmes conventions que les définitions de patterns de GC-Insight.
- **Cohérence d'expérience CLI** : mêmes conventions de sous-commandes, formats de sortie, niveaux de verbosité.

## MVP visé

**Cible : MVP étendu, 8-10 semaines solo en temps partiel** (cf. ROADMAP §2).

Périmètre : `gc-forge run <scenario.yaml>` produit un log G1/ZGC/Parallel sur Temurin 17 ou 21, accompagné d'un manifeste, exécuté soit en Docker soit en natif, avec 14 presets pédagogiques inclus et un mode batch matriciel.

**Note sur la dérive vs §9 du brief** : le brief vise 4-6 semaines pour un MVP. Cowork propose 8-10 semaines (avec phase 4 « mode natif » optionnelle) pour un **périmètre élargi** validé en début de mission : 3 algos (vs 1-2), 2 versions JDK (vs 1), 7 régimes (vs ~5), 14 presets, métadonnées riches dès J1. La trajectoire courte 4-6 semaines reste atteignable en coupant : Phase 4 entière, mode batch (Phase 3), et 2-3 régimes « secondaires » (R5 cache-churn, R6 mixed-pathological, R7 microservice). Cette option est explicitée en ROADMAP §10 (« sensibilités »).

## Critères de succès des specs

Référence : §9 du brief. À auto-évaluer après lecture :

- [ ] Un développeur Rust comprend l'archi cible et démarre l'implémentation sans ambiguïté majeure.
- [ ] Tous les points du §7 sont tranchés et argumentés.
- [ ] La cohérence avec GC-Insight est démontrée (réutilisation `gc-core`, conventions communes).
- [ ] Le MVP proposé est livrable en 8-10 semaines solo temps partiel.
- [ ] Le document permet de bâtir un pitch interne ou auprès de tiers.
