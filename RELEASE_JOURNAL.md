# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "b6d61996d501bb42fbadb0231de3ca4aed2ea035", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-01 12:53 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `b6d6199` | 2026-11-01 12:53 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Corrections
- Les journaux des requêtes n'enregistrent plus les valeurs de la query string : avant, `?limit=`, `?offset=` et `?details=` apparaissaient dans les logs des conteneurs ; maintenant la requête est journalisée sans query string. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Image d'exécution passée en `gcr.io/distroless/cc-debian13:nonroot`, builder `rust:1.99-slim-trixie` et `development.Dockerfile` en `rust:1.99-trixie`, avec le test `tests/dockerfile_test.rs` qui échoue si une image n'est pas sur Debian 13. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Corrections des alertes `cargo audit` : zstd 0.14.3, zstd-safe 8.2.0, zstd-sys 2.1.2 et ctutils 0.4.7 dans `Cargo.lock`, RUSTSEC-2026-0387 ignorée dans `.cargo/audit.toml`. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Matrice d'accès `access-matrix.yaml` et test `tests/endpoints/access_matrix.rs` qui vérifie chaque opération de l'OpenAPI, avec la dépendance de test `yaml_serde`. Aucun effet à l'exécution. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Marqueur RGPD des journaux : stack `docker-compose-gdpr-marker.yml`, scripts `gdpr_marker_test.sh` et `gdpr-marker.yaml`, feature `tracing-actix` de `mairie360_api_lib`, et ajout de `gdpr-report` au `.gitignore`. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))
- Checklist RGPD sur les pull requests via le workflow `.github/workflows/gdpr-pr.yml`. (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138))
- Version de mairie360/CICD passée de v4.4.0 à v4.4.1, puis v4.4.2 et v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181))
- Mises à jour Renovate : digest de `rust:1.99-bookworm` pour `development.Dockerfile` (#178, #183, #185), tokio 1.54.1 sans diff visible (#184), et images `database` et `liquibase-migrations` passées de 3.2.0 à 3.3.0 dans les compose et `TEST_DB_VERSION` (#179). Le contenu du schéma 3.3.0 n'apparaît pas dans les diffs. ([#178](https://github.com/mairie360/ELearning_API/pull/178), [#179](https://github.com/mairie360/ELearning_API/pull/179), [#183](https://github.com/mairie360/ELearning_API/pull/183), [#184](https://github.com/mairie360/ELearning_API/pull/184), [#185](https://github.com/mairie360/ELearning_API/pull/185))

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections
- Les journaux des requêtes n'enregistrent plus les valeurs de la query string : avant, `?limit=`, `?offset=` et `?details=` apparaissaient dans les logs des conteneurs ; maintenant la requête est journalisée sans query string. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Image d'exécution passée en `gcr.io/distroless/cc-debian13:nonroot`, builder `rust:1.99-slim-trixie` et `development.Dockerfile` en `rust:1.99-trixie`, avec le test `tests/dockerfile_test.rs` qui échoue si une image n'est pas sur Debian 13. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Corrections des alertes `cargo audit` : zstd 0.14.3, zstd-safe 8.2.0, zstd-sys 2.1.2 et ctutils 0.4.7 dans `Cargo.lock`, RUSTSEC-2026-0387 ignorée dans `.cargo/audit.toml`. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Matrice d'accès `access-matrix.yaml` et test `tests/endpoints/access_matrix.rs` qui vérifie chaque opération de l'OpenAPI, avec la dépendance de test `yaml_serde`. Aucun effet à l'exécution. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Marqueur RGPD des journaux : stack `docker-compose-gdpr-marker.yml`, scripts `gdpr_marker_test.sh` et `gdpr-marker.yaml`, feature `tracing-actix` de `mairie360_api_lib`, et ajout de `gdpr-report` au `.gitignore`. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))
- Checklist RGPD sur les pull requests via le workflow `.github/workflows/gdpr-pr.yml`. (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138))
- Version de mairie360/CICD passée de v4.4.0 à v4.4.1, puis v4.4.2 et v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181))
- Mises à jour Renovate : digest de `rust:1.99-bookworm` pour `development.Dockerfile` (#178, #183, #185), tokio 1.54.1 sans diff visible (#184), et images `database` et `liquibase-migrations` passées de 3.2.0 à 3.3.0 dans les compose et `TEST_DB_VERSION` (#179). Le contenu du schéma 3.3.0 n'apparaît pas dans les diffs. ([#178](https://github.com/mairie360/ELearning_API/pull/178), [#179](https://github.com/mairie360/ELearning_API/pull/179), [#183](https://github.com/mairie360/ELearning_API/pull/183), [#184](https://github.com/mairie360/ELearning_API/pull/184), [#185](https://github.com/mairie360/ELearning_API/pull/185))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
