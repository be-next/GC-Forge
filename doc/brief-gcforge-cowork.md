# Brief — GC-Forge

**Demande** : Production des spécifications fonctionnelles et techniques  
**Destinataire** : Cowork  
**Émetteur** : Jérôme  
**Date** : 25 avril 2026

---

## 1. Contexte

Je développe **GC-Insight**, un SaaS d'analyse de GC logs Java (cible initiale : G1 GC sur JDK 17+, architecture AWS serverless, cœur en Rust avec un crate partagé `gc-core`, design piloté par définitions YAML/regex).

Pour faire vivre GC-Insight — le tester, le démontrer, l'entraîner, et illustrer ses capacités d'analyse — j'ai besoin d'un **corpus riche, reproductible et taxonomisé** de GC logs. Or, ces logs sont pénibles à produire à la main : il faut maintenir un parc de JVMs, écrire des workloads représentatifs, capturer proprement les sorties.

D'où **GC-Forge** : le pendant générateur de GC-Insight. Là où Insight observe, Forge fabrique.

## 2. Vision et objectifs

GC-Forge doit permettre, à partir d'une description déclarative simple, de **produire à la demande des GC logs représentatifs de scénarios choisis**, en croisant trois axes :

1. **JVM** (HotSpot/OpenJDK, GraalVM, Eclipse OpenJ9, Azul Zing/Prime, Amazon Corretto, etc.)
2. **Algorithme GC** (G1, ZGC, Shenandoah, Parallel, Serial, et legacy CMS pour les cas historiques)
3. **Régime de fonctionnement applicatif** : transactionnel haute fréquence, compute massif batch-like, gestion de cache (allocations longue durée), workloads mixtes, microservices "stop-and-go", etc.

Chaque dimension doit être combinable, et chaque combinaison doit pouvoir être modulée pour exposer **différents niveaux de pression GC** (de "tout va bien" à "GC pathologique" en passant par les régimes intermédiaires intéressants à analyser).

## 3. Cas d'usage prioritaires

Les specs doivent dimensionner le projet pour couvrir au moins ces cas, par ordre de priorité :

1. **Test et validation de GC-Insight** : produire des logs de référence (avec vérité-terrain connue) pour valider les analyses, détecter les régressions, et benchmarker la précision des heuristiques.
2. **Démonstrations commerciales** : disposer d'un catalogue prêt-à-l'emploi de logs "parlants" pour illustrer chaque capacité de GC-Insight (humongous allocations, evacuation failures, mixed GC pathologiques, fragmentation, etc.).
3. **Pédagogie et contenu** : générer des logs pour articles de blog, tutoriels, formations — chaque log accompagné de sa "fiche d'identité" (configuration, régime, phénomène attendu).
4. **Reproductibilité de bugs / pathologies** : permettre à un utilisateur de GC-Insight de reproduire un comportement GC observé chez lui, en miroir, pour diagnostiquer.
5. **Datasets pour ML / heuristiques** : à terme, alimenter d'éventuels modèles de classification ou détection d'anomalies.

## 4. Périmètre fonctionnel à spécifier

Cowork est attendu sur les éléments suivants :

- **Catalogue des JVMs supportées au lancement** (et stratégie d'extension), avec versions cibles.
- **Catalogue des algorithmes GC supportés**, et compatibilité par JVM.
- **Catalogue des "régimes" applicatifs** : pour chacun, définir la signature comportementale attendue (taux d'allocation, durée de vie des objets, taille des objets, pattern temporel, ratio young/old, etc.).
- **Modèle de scénario** : quelle est l'unité de description d'un job GC-Forge ? (proposition à challenger : un fichier YAML déclaratif décrivant {JVM, algo, heap config, régime, durée, paramètres de variabilité, seed}).
- **Paramètres modulables** par scénario : taille de heap, ratios générationnels, seuils, pression mémoire visée, etc.
- **Formats de sortie** : log brut natif de chaque JVM (priorité absolue — format identique à ce que produirait une vraie appli en prod), + métadonnées structurées (fiche d'identité du scénario en JSON/YAML), + éventuellement export vers d'autres formats (JFR, etc.).
- **Catalogue de scénarios pré-packagés** ("presets") : a minima un set initial couvrant les cas les plus pédagogiques (steady-state sain, fuite mémoire lente, allocation burst, humongous, fragmentation old gen, etc.).
- **Mode batch** : produire en une exécution un corpus complet (matrice JVMs × algos × régimes).
- **Reproductibilité** : seed obligatoire, même config = même log à l'identique.

## 5. Périmètre technique à spécifier

Décisions structurantes que Cowork doit instruire et trancher (avec recommandation argumentée) :

- **Approche fondamentale** : (a) exécution réelle de JVMs avec workloads Java générés vs (b) synthèse pure de logs à partir d'un modèle statistique, vs (c) approche hybride. Mon intuition penche fortement pour (a) au moins pour le MVP — la fidélité est non-négociable — mais à challenger.
- **Architecture proposée**, en cohérence avec l'écosystème GC-Insight :
  - Workspace Rust avec extension du crate `gc-core` partagé
  - Composant Java pour les workloads (un harness paramétrable plutôt que N projets distincts)
  - Orchestration : CLI Rust qui pilote le lancement des JVMs et capture les logs
- **Gestion du parc de JVMs** : téléchargement à la demande ? conteneurs Docker pré-construits ? SDKMAN-like ?
- **Packaging et distribution** : binaire CLI cross-platform (Linux/macOS prioritaires), images Docker, éventuellement un mode service.
- **Reproductibilité bit-à-bit** : stratégie pour gérer les sources de non-déterminisme (timing GC réel, seeds JVM, charge système).
- **Versioning des scénarios** : un scénario doit produire un log identique d'une version à l'autre, ou explicitement signaler les changements.
- **Intégration CI** : permettre de regénérer le corpus de référence de GC-Insight depuis la CI.

## 6. Contraintes et exigences transverses

- **Cohérence avec GC-Insight** : même langage cœur (Rust), philosophie déclarative YAML, possibilité de partager des structures via `gc-core`.
- **Multi-OS** : Linux pour CI/serveur, macOS pour le dev local (mon poste).
- **Open core ou propriétaire ?** : à instruire — GC-Forge pourrait être open-source comme outil de génération de fixtures (effet de levier marketing pour GC-Insight), ou rester un asset interne. Recommandation attendue.
- **Coût d'exécution maîtrisé** : générer un corpus complet ne doit pas nécessiter un cluster.
- **Pas de dépendance forte à AWS** : contrairement à GC-Insight, GC-Forge doit pouvoir tourner en local et en CI standard.

## 7. Questions clés à trancher dans les specs

Cowork doit explicitement statuer (avec argumentaire) sur :

1. Approche réelle vs synthétique — choix MVP et trajectoire.
2. Modèle de description d'un scénario (schéma YAML proposé).
3. Liste cible des régimes pour le MVP (proposer 5 à 8 régimes prioritaires, justifier).
4. Liste cible des JVMs/algos pour le MVP (recommander un sous-ensemble réaliste vs cible long-terme).
5. Stratégie de gestion du parc JVM (Docker vs natif vs hybride).
6. Modèle open-source vs propriétaire.
7. Format des métadonnées accompagnant chaque log (schéma).
8. Critères de qualité d'un log généré (comment valider qu'un scénario "transactionnel haute fréquence" produit bien ce qu'on attend ?).

## 8. Livrables attendus

1. **Document de spécifications fonctionnelles** (cas d'usage détaillés, modèle de scénario, catalogue cible des régimes/JVMs/algos, format des entrées et sorties, presets pré-packagés du MVP).
2. **Document de spécifications techniques** (architecture, choix technologiques argumentés, structure du workspace, intégration avec GC-Insight, stratégie de packaging, stratégie de tests).
3. **Roadmap MVP → V1** : que livre-t-on en premier ? Quelle progression ?
4. **Backlog initial** structuré (epics et user stories de premier niveau).
5. **Liste des risques et points ouverts** restant à trancher après les specs.

## 9. Critères de succès des specs

Les specs livrées par Cowork seront jugées réussies si :

- Un développeur Rust peut, à leur lecture seule, comprendre l'architecture cible et démarrer l'implémentation du MVP sans ambiguïté majeure.
- Les choix structurants sont explicites et argumentés (pas de "à voir plus tard" sur les points listés au §7).
- La cohérence avec GC-Insight est démontrée et chiffrée (réutilisation de `gc-core`, conventions communes).
- Le MVP proposé est livrable en effort raisonnable (cible : 4 à 6 semaines de dev solo en temps partiel — à confirmer par Cowork).
- Le document permet de bâtir un pitch en interne ou auprès de tiers (positionnement clair de la valeur).

---

**Note pour Cowork** : ne pas hésiter à challenger les hypothèses de ce brief — notamment sur l'approche réelle vs synthétique, le périmètre du MVP, et l'opportunité open-source. Une spec qui contredit le brief avec un meilleur argumentaire est préférable à une spec qui exécute à la lettre.
