# GC-Forge — Risques et points ouverts

> **Référence** : brief du 25 avril 2026, §8.5.

## Conventions

- **Probabilité (P)** : 1 (rare) à 5 (très probable).
- **Impact (I)** : 1 (négligeable) à 5 (bloquant projet).
- **Sévérité** : `P × I`.
- **Mitigation** : action préventive ou de réduction.
- **Déclencheurs** : signaux qui doivent réveiller l'équipe.

---

## 1. Risques techniques

### R-T1 — Variance inter-host trop élevée pour valider les invariants
- **P** : 3  **I** : 4  **Sévérité** : 12
- **Description** : Le timing du GC réel dépend de l'OS, du CPU, de la charge. Si la variance sur les métriques agrégées dépasse 5 % entre runs ou entre hôtes, les invariants ne tiennent plus, et la promesse « rejeu reproductible » s'effondre.
- **Mitigation** :
  - Imposer Docker en MVP avec limites CPU/mémoire explicites.
  - Mesurer la variance dès Phase 2 (chaque régime livré inclut un `variance-check`).
  - Si problème : tolérance plus large (10-15 % au lieu de 5 %) sur métriques sensibles, invariants formulés en **plages** plutôt qu'en **seuils**.
  - Repli : pinning CPU sur les runners CI (`taskset` Linux), désactivation turbo.
- **Déclencheurs** : variance > 10 % sur métriques agrégées, divergence systématique entre Linux et macOS.
- **Owner** : dev GC-Forge (Jérôme).

### R-T2 — Format du log GC qui change entre versions JDK
- **P** : 2  **I** : 3  **Sévérité** : 6
- **Description** : Une mineure JDK peut introduire un nouveau tag ou modifier un format. Le parser de `gc-core` doit suivre.
- **Mitigation** :
  - Tests d'intégration par version JDK (matrice CI).
  - `gc-core` versionné en SemVer.
  - Veille sur les release notes OpenJDK.
- **Déclencheurs** : nouvelle minor JDK qui casse un test de parsing.
- **Owner** : `gc-core` (Jérôme, partagé Forge ↔ Insight).

### R-T3 — Reproductibilité du jar Java imparfaite
- **P** : 3  **I** : 2  **Sévérité** : 6
- **Description** : Maven peut produire des jars dont le SHA-256 varie (timestamps embarqués, ordre de fichiers).
- **Mitigation** :
  - Plugin `reproducible-build-maven-plugin` ou `maven-jar-plugin` configuré pour timestamps fixes.
  - Test en CI : 2 builds successifs produisent le même hash.
- **Déclencheurs** : test de hash qui échoue.
- **Owner** : dev harness.

### R-T4 — OpenJ9 force une refonte du parser
- **P** : 4 (si V1 inclut OpenJ9)  **I** : 3  **Sévérité** : 12
- **Description** : OpenJ9 a un format de log distinct de HotSpot. Étendre `gc-core` peut révéler que l'abstraction d'événements est trop HotSpot-centric.
- **Mitigation** :
  - Reporter OpenJ9 en V1.0 (pas en MVP).
  - Faire un spike de 2-3 j avant de planifier V1 pour évaluer l'effort réel.
  - Concevoir `gc-core::EventKind` en gardant à l'esprit qu'OpenJ9 viendra.
- **Déclencheurs** : spike OpenJ9 qui révèle un effort > 2 sprints.
- **Owner** : `gc-core`.

### R-T5 — ZGC generational immature en JDK 17
- **P** : 5  **I** : 2  **Sévérité** : 10
- **Description** : Le mode generational de ZGC n'est GA qu'en JDK 21. En JDK 17 ZGC est non-generational, ce qui change la signature des logs.
- **Mitigation** :
  - Documenter clairement : `gc.options.generational: true` n'est dispo qu'en JDK 21+.
  - `gc-forge lint` rejette `generational: true` sur JDK 17.
  - Presets `*-zgc-*` ciblent JDK 21 par défaut.
- **Déclencheurs** : aucun, c'est une contrainte connue.
- **Owner** : SPEC + dev.

### R-T6 — Docker Desktop sur macOS = lourd à installer
- **P** : 3  **I** : 3  **Sévérité** : 9
- **Description** : Beaucoup d'utilisateurs macOS n'ont pas Docker, ou l'ont en version commerciale payante.
- **Mitigation** :
  - Mode natif (Phase 4 du MVP / V1) qui se passe de Docker.
  - Documenter Colima / OrbStack comme alternatives gratuites.
  - Pas de hard requirement Docker dans la doc utilisateur — Docker est un *runner* parmi d'autres.
- **Déclencheurs** : feedback utilisateur récurrent.
- **Owner** : dev runner.

### R-T7 — Cold start JVM amplifie la durée des runs courts
- **P** : 4  **I** : 2  **Sévérité** : 8
- **Description** : Cold start Temurin = 1-2 s. Sur un preset de 90 s, c'est 2 % de bruit injectés au début.
- **Mitigation** :
  - Implémenter `warmup` dans le harness : la durée mesurée commence après le warmup.
  - Logger le warmup mais l'exclure des invariants par défaut.
- **Déclencheurs** : aucun, design dès le départ.
- **Owner** : dev harness.

### R-T8 — Coût d'exécution du corpus complet en CI
- **P** : 3  **I** : 2  **Sévérité** : 6
- **Description** : 14 presets × 3 algos × 2 JDKs × 3 seeds = 252 runs × 90s-5min = quelques heures.
- **Mitigation** :
  - Parallélisation `gc-forge batch --parallel`.
  - Corpus de référence CI = sous-ensemble (ex. 1 seed, JDK 21 only).
  - Corpus complet = nightly ou weekly, pas par PR.
- **Déclencheurs** : durée CI bloquante.
- **Owner** : CI.

### R-T9 — Instabilité du parsing `gc+humongous=trace`
- **P** : 2  **I** : 2  **Sévérité** : 4
- **Description** : Les logs `gc+humongous` contiennent des messages au format moins stable que les logs principaux.
- **Mitigation** :
  - Parser tolérant : log non-parsable = warning, pas erreur.
  - Tests de régression sur logs réels capturés.
- **Déclencheurs** : faux positifs de parser.
- **Owner** : `gc-core`.

---

## 2. Risques projet

### R-P1 — Capacité solo temps partiel insuffisante
- **P** : 3  **I** : 4  **Sévérité** : 12
- **Description** : 12 h ouvrées/semaine théoriques peuvent tomber à 6 h en cas de charge GC-Insight. La cible 8-10 semaines glisse.
- **Mitigation** :
  - Découpe en livrables intermédiaires utilisables (cf. ROADMAP §3-7).
  - Phase 4 considérée comme optionnelle dès le départ.
  - Si glissement > 2 semaines fin Phase 2 : revenir sur le périmètre régimes (ex. enlever R6 pathologique, le reporter en V1).
- **Déclencheurs** : retard cumulé > 1 semaine en fin de phase.
- **Owner** : Jérôme.

### R-P2 — Contention avec GC-Insight sur `gc-core`
- **P** : 3  **I** : 3  **Sévérité** : 9
- **Description** : Forge et Insight partagent `gc-core`. Une évolution unilatérale peut casser l'autre.
- **Mitigation** :
  - SemVer strict.
  - Tests cross-projet (`gc-core-roundtrip`).
  - PR sur `gc-core` rebuilde forcément Forge ET Insight en CI.
- **Déclencheurs** : conflit régulier sur les structures partagées.
- **Owner** : `gc-core` BDFL.

### R-P3 — Dérive de scope sous pression « presets-utiles-en-démo »
- **P** : 4  **I** : 2  **Sévérité** : 8
- **Description** : Tentation d'ajouter sans cesse des presets « parlants » au lieu de finaliser le tronc.
- **Mitigation** :
  - Quota MVP = 14 presets (dont 2 bonus). Au-delà : V1.
  - Backlog clair (cf. BACKLOG §EPIC 7).
- **Déclencheurs** : commits fréquents dans `presets/` sans avancer le tronc.
- **Owner** : Jérôme.

---

## 3. Risques produit / écosystème

### R-X1 — Pas de traction utilisateur en open-source
- **P** : 3  **I** : 2  **Sévérité** : 6
- **Description** : GC-Forge ne décolle pas en open-source. Pas de contributeurs, pas de buzz.
- **Mitigation** :
  - L'utilité interne pour valider GC-Insight justifie déjà l'investissement, traction externe = bonus.
  - Communiquer dès release `0.1.0` : article blog, post LinkedIn, share sur communautés JVM.
  - Conférences performance JVM (FOSDEM, Devoxx, JFokus).
- **Déclencheurs** : 6 mois post-release sans contributeur externe ni mention publique.
- **Owner** : Jérôme.

### R-X2 — Concurrent qui forke et se positionne
- **P** : 1  **I** : 3  **Sévérité** : 3
- **Description** : Un éditeur concurrent fork GC-Forge et l'utilise pour marketer son propre produit.
- **Mitigation** :
  - MIT = c'est légalement OK, on ne peut pas l'empêcher.
  - Garder l'avance sur la cohérence Forge ↔ Insight.
  - Reconnaissance de la marque GC-Forge dans la communauté.
- **Déclencheurs** : fork actif détecté.
- **Owner** : Jérôme.

### R-X3 — Mauvaise perception « générer des fixtures = preuve de faiblesse »
- **P** : 2  **I** : 2  **Sévérité** : 4
- **Description** : Argument adverse possible : « si vous devez générer vos logs de test, c'est que vous n'avez pas de vrais clients ».
- **Mitigation** :
  - Communiquer clairement : GC-Forge sert à **valider** et **démontrer**, pas à remplacer les vrais logs clients.
  - Les vrais logs clients restent confidentiels — c'est précisément pour ça qu'on a besoin d'un corpus public.
  - Souligner que les éditeurs sérieux (HotSpot, ZGC) génèrent leurs propres fixtures pour leurs tests.
- **Déclencheurs** : objection soulevée par un prospect.
- **Owner** : pitch / sales.

---

## 4. Risques juridiques / licences

### R-L1 — Licence Oracle JDK
- **P** : 5 (si on l'inclut)  **I** : 4  **Sévérité** : 20
- **Description** : Oracle JDK n'est pas redistribuable librement. Pas d'image Docker possible.
- **Mitigation** :
  - **Ne pas l'inclure dans les images Docker GC-Forge.**
  - Mode BYO-JVM (V1+) pour les utilisateurs qui ont leur licence Oracle.
- **Déclencheurs** : aucun, contrainte connue.
- **Owner** : SPEC.

### R-L2 — Licence Azul Zing/Prime
- **P** : 5  **I** : 3  **Sévérité** : 15
- **Description** : Zing/Prime sont propriétaires, redistribution interdite, format de log non documenté publiquement.
- **Mitigation** :
  - Reporter en V2 sous condition d'accord avec Azul.
  - BYO-JVM permet à un client Azul d'utiliser sa propre licence.
- **Owner** : à instruire si V2.

### R-L3 — TCK et compatibilité Java
- **P** : 1  **I** : 2  **Sévérité** : 2
- **Description** : Notre harness Java ne touche pas le TCK (on consomme une JVM, on ne l'implémente pas).
- **Mitigation** : néant.
- **Owner** : néant.

### R-L4 — Dépendances Rust avec licences incompatibles
- **P** : 1  **I** : 3  **Sévérité** : 3
- **Description** : Une crate transitive en GPL/AGPL pollue la licence MIT.
- **Mitigation** :
  - `cargo deny check licenses` en CI avec allowlist permissive (MIT, Apache-2.0, BSD-2/3, ISC, Unicode-DFS-2016, Zlib).
- **Déclencheurs** : `cargo deny` rouge.
- **Owner** : CI.

---

## 5. Risques sécurité

### R-S1 — Exécution non sandboxée du harness
- **P** : 1  **I** : 4  **Sévérité** : 4
- **Description** : Le harness est livré par GC-Forge, mais s'exécute sur la machine hôte. Une vulnérabilité dans une dépendance Java pourrait être exploitée.
- **Mitigation** :
  - Pas de réseau (`--network=none`).
  - Volumes en lecture seule sauf sortie.
  - Dependency scanning (`mvn dependency-check`).
  - Pas d'input utilisateur arbitraire dans le harness — uniquement des paramètres typés.
- **Owner** : dev harness + CI.

### R-S2 — Supply chain (image Docker compromise)
- **P** : 1  **I** : 4  **Sévérité** : 4
- **Description** : Image Docker `eclipse-temurin` ou nos propres images compromises.
- **Mitigation** :
  - Signature Cosign / Sigstore sur images GHCR.
  - SBOM publié à chaque release.
- **Owner** : CI / release.

---

## 6. Points ouverts à trancher après les specs

Liste des décisions volontairement non figées dans les specs, à instruire en cours de développement :

### O-1 — `gc-core` open-source ou propriétaire ?
- **Statut** : recommandation faite (open-source) en SPEC-TECH §9.3, à confirmer avec le projet GC-Insight.
- **Échéance** : avant Phase 1 (semaine 2).
- **Owner** : Jérôme + GC-Insight.

### O-2 — Tap Homebrew tiers ou `homebrew-core` ?
- **Statut** : ouvert. Tap tiers en V1, candidature `homebrew-core` quand l'adoption le justifie (>1k stars typiquement).
- **Échéance** : V1.0.

### O-3 — Format des `expected_invariants` custom dans le scenario
- **Statut** : prévu dans le schéma (cf. SPEC-FONC §5.1) mais syntaxe d'expression non figée. Options :
  - DSL maison (« young_pause_p99 < 50ms »).
  - JSONLogic.
  - Embed Lua/Rhai.
- **Recommandation** : DSL minimal en V1, embed Lua si demande forte en V2.
- **Échéance** : V1.

### O-4 — Stratégie de release du `workload-harness.jar`
- **Statut** : embarqué dans les images Docker, fournit en asset GitHub Release. Faut-il le publier sur Maven Central ?
- **Recommandation** : non en MVP (le harness est un détail d'implémentation), oui en V2 si on expose une API d'embedding pour des utilisateurs qui veulent piloter le harness depuis leur propre code.
- **Échéance** : V2.

### O-5 — Telemetry opt-in
- **Statut** : non-MVP, à instruire en V2 selon adoption.
- **Échéance** : V2.

### O-6 — Cible Windows
- **Statut** : non MVP, non V1. Faisable techniquement (Rust supporte, harness Java multiplateforme), question de demande.
- **Échéance** : à réévaluer 6 mois post-release.

### O-7 — Signature des images Docker
- **Statut** : prévu, outil non choisi (Cosign vs Notary v2).
- **Recommandation** : Cosign / Sigstore (standard CNCF).
- **Échéance** : Phase 3 (release `0.1.0`).

### O-8 — Gouvernance long terme
- **Statut** : BDFL en démarrage. Passage à gouvernance ouverte (steering committee) si > 10 contributeurs réguliers.
- **Échéance** : 12 mois post-release.

---

## 7. Plan de revue des risques

- **Cadence** : revue à chaque jalon de la roadmap (J1 à J6).
- **Rétro** : à chaque release, mise à jour de ce document.
- **Escalade** : si un risque sévérité ≥ 12 vire au rouge (déclencheur activé), arrêt et replanification, pas de poursuite à l'aveugle.

## 8. Synthèse heat-map

```
              Impact →
            1    2    3    4    5
        ┌─────┬─────┬─────┬─────┬─────┐
   P  5 │     │     │ R-L2│ R-L1│     │
   r    ├─────┼─────┼─────┼─────┼─────┤
   o  4 │     │ R-T7│     │ R-T4│     │      sévérité 16-25 = critique
   b    ├─────┼─────┼─────┼─────┼─────┤      sévérité 8-15  = élevée
   a  3 │     │ R-T8│ R-P2│ R-T1│     │      sévérité 4-7   = modérée
   b    │     │ R-T3│ R-T6│ R-P1│     │      sévérité 1-3   = faible
   i    │     │ R-X1│     │     │     │
   l    │     │     │     │     │     │
   i  2 │     │ R-T9│ R-T2│     │     │
   t    │     │ R-X3│     │     │     │
   é    ├─────┼─────┼─────┼─────┼─────┤
      1 │     │ R-L3│ R-X2│ R-S1│     │
        │     │     │ R-L4│ R-S2│     │
        └─────┴─────┴─────┴─────┴─────┘
```

**Risques à surveillance prioritaire** : R-T1 (variance), R-P1 (capacité), R-T4 (OpenJ9), R-L1 (Oracle JDK).
