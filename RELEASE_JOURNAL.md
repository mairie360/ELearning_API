# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "9498d6f56660e82ee6568cc4377b9214770bae89", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-03 16:14 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `9498d6f` | 2026-11-03 16:14 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- Les jetons de session HS256 sont refusés (401) : l'API ne vérifie plus que les jetons EdDSA signés par Core_API avec la clé publique `JWT_PUBLIC_KEY`. Le secret partagé `JWT_SECRET` disparaît des stacks de test, et le dev utilise la clé de test de la lib avec `JWT_ALLOW_WEAK_SECRET=true`. Tant que `JWT_SECRET` est défini, la lib accepte encore HS256 (transition). Les appelants qui émettaient leurs propres HS256 doivent passer par Core. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Les journaux de requêtes ne contiennent plus la query string ni le `Referer`, seulement adresse client, méthode, chemin, statut, taille et durée. Les erreurs de la lib ne décrivent plus la valeur reçue (SQLSTATE, contrainte, table, colonne). (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Image d'exécution passée à distroless Debian 13 (`cc-debian13:nonroot`) et builders en trixie, pour corriger la CVE-2026-84782 de `libssl3` ; test de régression `tests/dockerfile_test.rs`. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Dépendances : `mairie360_api_lib` 4.2.0 → 4.3.0, zstd, ctutils mis à jour, et RUSTSEC-2026-0387 ignoré dans `.cargo/audit.toml`. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190)) ; tokio 1.54.1 ([#184](https://github.com/mairie360/ELearning_API/pull/184)) ; actix-web 4.18.1 ([#186](https://github.com/mairie360/ELearning_API/pull/186), diff absent, description seule).
- Matrice d'accès `access-matrix.yaml` et tests associés, sans effet à l'exécution. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Checklist RGPD sur les PR (`gdpr-pr.yml`) et stack de marqueur RGPD. (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138))
- Images Docker de test et de base épinglées par digest, et base `rust` de `development.Dockerfile` mise à jour par Renovate. ([#194](https://github.com/mairie360/ELearning_API/pull/194), [#185](https://github.com/mairie360/ELearning_API/pull/185), [#183](https://github.com/mairie360/ELearning_API/pull/183), [#178](https://github.com/mairie360/ELearning_API/pull/178))
- Schéma de base `ghcr.io/mairie360/database` et Liquibase en 3.3.0 ([#179](https://github.com/mairie360/ELearning_API/pull/179)).
- Workflows CI vers mairie360/CICD v4.4.1, v4.4.2, v4.4.3 puis v5.2.2 (contrôles RGPD), avec `ANTHROPIC_WORKSPACE_ID`. ([#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181), [#196](https://github.com/mairie360/ELearning_API/pull/196))

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les jetons de session HS256 sont refusés (401) : l'API ne vérifie plus que les jetons EdDSA signés par Core_API avec la clé publique `JWT_PUBLIC_KEY`. Le secret partagé `JWT_SECRET` disparaît des stacks de test, et le dev utilise la clé de test de la lib avec `JWT_ALLOW_WEAK_SECRET=true`. Tant que `JWT_SECRET` est défini, la lib accepte encore HS256 (transition). Les appelants qui émettaient leurs propres HS256 doivent passer par Core. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Les journaux de requêtes ne contiennent plus la query string ni le `Referer`, seulement adresse client, méthode, chemin, statut, taille et durée. Les erreurs de la lib ne décrivent plus la valeur reçue (SQLSTATE, contrainte, table, colonne). (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Image d'exécution passée à distroless Debian 13 (`cc-debian13:nonroot`) et builders en trixie, pour corriger la CVE-2026-84782 de `libssl3` ; test de régression `tests/dockerfile_test.rs`. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Dépendances : `mairie360_api_lib` 4.2.0 → 4.3.0, zstd, ctutils mis à jour, et RUSTSEC-2026-0387 ignoré dans `.cargo/audit.toml`. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190)) ; tokio 1.54.1 ([#184](https://github.com/mairie360/ELearning_API/pull/184)) ; actix-web 4.18.1 ([#186](https://github.com/mairie360/ELearning_API/pull/186), diff absent, description seule).
- Matrice d'accès `access-matrix.yaml` et tests associés, sans effet à l'exécution. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Checklist RGPD sur les PR (`gdpr-pr.yml`) et stack de marqueur RGPD. (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138))
- Images Docker de test et de base épinglées par digest, et base `rust` de `development.Dockerfile` mise à jour par Renovate. ([#194](https://github.com/mairie360/ELearning_API/pull/194), [#185](https://github.com/mairie360/ELearning_API/pull/185), [#183](https://github.com/mairie360/ELearning_API/pull/183), [#178](https://github.com/mairie360/ELearning_API/pull/178))
- Schéma de base `ghcr.io/mairie360/database` et Liquibase en 3.3.0 ([#179](https://github.com/mairie360/ELearning_API/pull/179)).
- Workflows CI vers mairie360/CICD v4.4.1, v4.4.2, v4.4.3 puis v5.2.2 (contrôles RGPD), avec `ANTHROPIC_WORKSPACE_ID`. ([#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181), [#196](https://github.com/mairie360/ELearning_API/pull/196))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-03 16:14 UTC | dev | — | `9498d6f` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41669638867) |
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
