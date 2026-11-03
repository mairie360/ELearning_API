# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "bae9d89e38a6012d82e7bfad67c79e981695db23", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-03 00:00 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `bae9d89` | 2026-11-03 00:00 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- Les journaux des requêtes HTTP n'enregistrent plus la query string (donc les valeurs de `?limit=`, `?offset=`, `?details=`) ni l'en-tête `Referer`. Seuls l'adresse du client, la méthode, le chemin, le statut, la taille et la durée sont journalisés. Avant, la ligne de requête complète était journalisée. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Image d'exécution passée de distroless Debian 12 à Debian 13 (builder et `development.Dockerfile` en trixie) pour corriger CVE-2026-84782, avec le test `tests/dockerfile_test.rs`. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Alertes RustSec de `cargo audit` corrigées (zstd, ctutils) et RUSTSEC-2026-0387 ignorée avec justification. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Matrice d'accès `access-matrix.yaml` et test qui vérifie qu'elle couvre chaque opération de l'OpenAPI, sans changement de route. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Checklist RGPD sur les PR, marqueur RGPD dans les journaux et test `tests/mair_771_cicd_version.rs`. (MAIR-295 [#138](https://github.com/mairie360/ELearning_API/pull/138), MAIR-290 [#134](https://github.com/mairie360/ELearning_API/pull/134), MAIR-771 [#196](https://github.com/mairie360/ELearning_API/pull/196))
- Workflows CI et promotion : mairie360/CICD passé en v4.4.1, v4.4.2, v4.4.3 puis v5.2.2. (MAIR-612 [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181), MAIR-771 [#196](https://github.com/mairie360/ELearning_API/pull/196))
- Mises à jour Renovate : schéma `database` et `liquibase-migrations` en 3.3.0 (dans les compose et `TEST_DB_VERSION`), digests `rust:1.99` et tokio 1.54.1. (#179, #183, #184, #185)

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les journaux des requêtes HTTP n'enregistrent plus la query string (donc les valeurs de `?limit=`, `?offset=`, `?details=`) ni l'en-tête `Referer`. Seuls l'adresse du client, la méthode, le chemin, le statut, la taille et la durée sont journalisés. Avant, la ligne de requête complète était journalisée. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Image d'exécution passée de distroless Debian 12 à Debian 13 (builder et `development.Dockerfile` en trixie) pour corriger CVE-2026-84782, avec le test `tests/dockerfile_test.rs`. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Alertes RustSec de `cargo audit` corrigées (zstd, ctutils) et RUSTSEC-2026-0387 ignorée avec justification. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Matrice d'accès `access-matrix.yaml` et test qui vérifie qu'elle couvre chaque opération de l'OpenAPI, sans changement de route. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Checklist RGPD sur les PR, marqueur RGPD dans les journaux et test `tests/mair_771_cicd_version.rs`. (MAIR-295 [#138](https://github.com/mairie360/ELearning_API/pull/138), MAIR-290 [#134](https://github.com/mairie360/ELearning_API/pull/134), MAIR-771 [#196](https://github.com/mairie360/ELearning_API/pull/196))
- Workflows CI et promotion : mairie360/CICD passé en v4.4.1, v4.4.2, v4.4.3 puis v5.2.2. (MAIR-612 [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181), MAIR-771 [#196](https://github.com/mairie360/ELearning_API/pull/196))
- Mises à jour Renovate : schéma `database` et `liquibase-migrations` en 3.3.0 (dans les compose et `TEST_DB_VERSION`), digests `rust:1.99` et tokio 1.54.1. (#179, #183, #184, #185)

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
