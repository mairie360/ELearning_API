# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "f780ed2483c3439d7dccac9fed8eef5d86afe7e7", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-03 11:56 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `f780ed2` | 2026-11-03 11:56 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- L'API ne vérifie plus les sessions avec `JWT_SECRET` : elle attend `JWT_PUBLIC_KEY`, la clé publique Ed25519 de Core_API (PEM, plusieurs blocs pendant une rotation), et n'accepte que les jetons EdDSA signés par Core. Avant, tout jeton HS256 signé avec le secret partagé par 5 APIs et 7 BFFs était accepté. Maintenant, sans `JWT_SECRET`, chaque jeton HS256 reçoit une 401. Tant que `JWT_SECRET` reste défini, la lib accepte encore les HS256 signés avec lui : il faut le retirer en déployant `JWT_PUBLIC_KEY`. Un appelant qui signe ses propres jetons HS256 (scripts, outils) est cassé. Le stack de dev utilise la clé de test de la lib avec `JWT_ALLOW_WEAK_SECRET=true`. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))

#### Corrections
- Journaux des requêtes : avant, la ligne de requête avec sa query string et l'en-tête `Referer` étaient journalisés, donc les valeurs refusées de `?limit=`, `?offset=` et `?details=` se retrouvaient dans les logs. Maintenant, seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Images Docker de la CI et des stacks de test épinglées par digest (Renovate) : redis, postgres, curl, k6, nginx, node, newman et zap-stable, sans effet fonctionnel attendu. (MAIR-???, [#194](https://github.com/mairie360/ELearning_API/pull/194))
- Image d'exécution passée à `distroless/cc-debian13` avec un builder et l'image de dev en `trixie` pour la même version de Debian ; ajout du test `tests/dockerfile_test.rs`. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Corrections des alertes `cargo audit` : mise à jour de zstd, zstd-safe, zstd-sys et ctutils, et ignorance motivée de RUSTSEC-2026-0387. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Matrice d'accès `access-matrix.yaml` et test qui vérifie que chaque opération de l'OpenAPI y figure et répond selon les rôles ; ajout de la dépendance `yaml_serde` en dev. Aucun changement de réponse. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Stack de test du marqueur RGPD (`gdpr-marker.yaml`, `docker-compose-gdpr-marker.yml`, `gdpr_marker_test.sh`) et ajout de `tracing-actix` à la lib. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))
- Workflow `gdpr-pr.yml` qui vérifie la checklist RGPD de chaque PR (check « GDPR checklist »). (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138))
- Image de base de schéma `mairie360/database` et `liquibase-migrations` passées de 3.2.0 à 3.3.0 dans les stacks compose et `TEST_DB_VERSION` ; le contenu de la 3.3.0 n'est pas visible dans ce diff. (#179, [#179](https://github.com/mairie360/ELearning_API/pull/179))
- Mises à jour Renovate de la chaîne Rust : digests de `rust:1.99-bookworm` (puis `development.Dockerfile` repassé en trixie) et tokio 1.54.0 → 1.54.1 (diff vide). ([#183](https://github.com/mairie360/ELearning_API/pull/183), [#184](https://github.com/mairie360/ELearning_API/pull/184), [#185](https://github.com/mairie360/ELearning_API/pull/185), [#178](https://github.com/mairie360/ELearning_API/pull/178))
- Montées de mairie360/CICD en v4.4.1, v4.4.2 puis v4.4.3 (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181)), puis v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` transmis aux workflows, et test `tests/mair_771_cicd_version.rs`. (MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196))

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- L'API ne vérifie plus les sessions avec `JWT_SECRET` : elle attend `JWT_PUBLIC_KEY`, la clé publique Ed25519 de Core_API (PEM, plusieurs blocs pendant une rotation), et n'accepte que les jetons EdDSA signés par Core. Avant, tout jeton HS256 signé avec le secret partagé par 5 APIs et 7 BFFs était accepté. Maintenant, sans `JWT_SECRET`, chaque jeton HS256 reçoit une 401. Tant que `JWT_SECRET` reste défini, la lib accepte encore les HS256 signés avec lui : il faut le retirer en déployant `JWT_PUBLIC_KEY`. Un appelant qui signe ses propres jetons HS256 (scripts, outils) est cassé. Le stack de dev utilise la clé de test de la lib avec `JWT_ALLOW_WEAK_SECRET=true`. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))

#### Corrections
- Journaux des requêtes : avant, la ligne de requête avec sa query string et l'en-tête `Referer` étaient journalisés, donc les valeurs refusées de `?limit=`, `?offset=` et `?details=` se retrouvaient dans les logs. Maintenant, seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Images Docker de la CI et des stacks de test épinglées par digest (Renovate) : redis, postgres, curl, k6, nginx, node, newman et zap-stable, sans effet fonctionnel attendu. ([#194](https://github.com/mairie360/ELearning_API/pull/194))
- Image d'exécution passée à `distroless/cc-debian13` avec un builder et l'image de dev en `trixie` pour la même version de Debian ; ajout du test `tests/dockerfile_test.rs`. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Corrections des alertes `cargo audit` : mise à jour de zstd, zstd-safe, zstd-sys et ctutils, et ignorance motivée de RUSTSEC-2026-0387. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Matrice d'accès `access-matrix.yaml` et test qui vérifie que chaque opération de l'OpenAPI y figure et répond selon les rôles ; ajout de la dépendance `yaml_serde` en dev. Aucun changement de réponse. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Stack de test du marqueur RGPD (`gdpr-marker.yaml`, `docker-compose-gdpr-marker.yml`, `gdpr_marker_test.sh`) et ajout de `tracing-actix` à la lib. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))
- Workflow `gdpr-pr.yml` qui vérifie la checklist RGPD de chaque PR (check « GDPR checklist »). (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138))
- Image de base de schéma `mairie360/database` et `liquibase-migrations` passées de 3.2.0 à 3.3.0 dans les stacks compose et `TEST_DB_VERSION` ; le contenu de la 3.3.0 n'est pas visible dans ce diff. ([#179](https://github.com/mairie360/ELearning_API/pull/179))
- Mises à jour Renovate de la chaîne Rust : digests de `rust:1.99-bookworm` (puis `development.Dockerfile` repassé en trixie) et tokio 1.54.0 → 1.54.1 (diff vide). ([#183](https://github.com/mairie360/ELearning_API/pull/183), [#184](https://github.com/mairie360/ELearning_API/pull/184), [#185](https://github.com/mairie360/ELearning_API/pull/185), [#178](https://github.com/mairie360/ELearning_API/pull/178))
- Montées de mairie360/CICD en v4.4.1, v4.4.2 puis v4.4.3 (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181)), puis v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` transmis aux workflows, et test `tests/mair_771_cicd_version.rs`. (MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-03 11:56 UTC | dev | — | `f780ed2` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41630569815) |
| 2026-11-03 01:35 UTC | dev | — | `82404ec` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41559233738) |
| 2026-11-03 00:00 UTC | dev | — | `bae9d89` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41548469175) |
| 2026-11-01 12:53 UTC | dev | — | `b6d6199` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41307600724) |
| 2026-10-22 23:43 UTC | dev | — | `733fcba` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39872754764) |
| 2026-10-22 06:44 UTC | dev | — | `ba18081` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39734444851) |
| 2026-10-22 06:37 UTC | dev | — | `6e8ea43` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39734296233) |
| 2026-10-21 09:11 UTC | prod | v1.2.0 | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39573690984) |
| 2026-10-21 09:08 UTC | dev | — | `4211751` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39573528137) |
| 2026-10-21 09:06 UTC | dev | — | `534a52e` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39573426480) |
| 2026-10-21 08:29 UTC | dev | — | `8c0d27f` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39568593745) |
| 2026-10-21 08:27 UTC | dev | — | `f149732` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39567976318) |
| 2026-10-21 03:47 UTC | dev | — | `ad7a855` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39539811823) |
| 2026-10-21 03:21 UTC | staging | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39537392237) |
| 2026-10-21 01:45 UTC | dev | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39527799559) |
| 2026-10-20 17:07 UTC | staging | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
