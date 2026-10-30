# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "733fcbae8c8f4dd24f43511df97ef7bb9b3d07e4", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-22 23:43 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `733fcba` | 2026-10-22 23:43 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- Stacks docker-compose (`docker-compose.yml`, `docker-compose-integration.yml`, `docker-compose-performance.yml`, `docker-compose-security.yml`) et `TEST_DB_VERSION` dans `.cargo/config.toml` : les images `ghcr.io/mairie360/database` et `ghcr.io/mairie360/liquibase-migrations` passent de 3.2.0 à 3.3.0. Le diff ne contient pas les migrations, donc les changements de tables, colonnes ou contraintes ne sont pas visibles ici et restent à vérifier dans la version 3.3.0 du schéma. (#179, [#179](https://github.com/mairie360/ELearning_API/pull/179))

#### Technique
- Image de développement `development.Dockerfile` : le digest de `rust:1.99-bookworm` passe de `e12566d` à `5c1a173` ([#178](https://github.com/mairie360/ELearning_API/pull/178)), puis à `edaa2a8` ([#183](https://github.com/mairie360/ELearning_API/pull/183)), puis à `48b7f43` ([#185](https://github.com/mairie360/ELearning_API/pull/185)). Aucun effet sur l'API en exécution.
- Dépendance `tokio` passée de 1.54.0 à 1.54.1 ([#184](https://github.com/mairie360/ELearning_API/pull/184)). Le diff n'est pas fourni : seule la description de la PR est connue (revert d'un changement sur les timers).
- Pins `mairie360/CICD` passés de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, en trois PR successives (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181)). Les corrections concernent la publication des tags et releases par `promote-prod`, sans effet sur l'API.

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Stacks docker-compose (`docker-compose.yml`, `docker-compose-integration.yml`, `docker-compose-performance.yml`, `docker-compose-security.yml`) et `TEST_DB_VERSION` dans `.cargo/config.toml` : les images `ghcr.io/mairie360/database` et `ghcr.io/mairie360/liquibase-migrations` passent de 3.2.0 à 3.3.0. Le diff ne contient pas les migrations, donc les changements de tables, colonnes ou contraintes ne sont pas visibles ici et restent à vérifier dans la version 3.3.0 du schéma. (#179, [#179](https://github.com/mairie360/ELearning_API/pull/179))

#### Technique
- Image de développement `development.Dockerfile` : le digest de `rust:1.99-bookworm` passe de `e12566d` à `5c1a173` ([#178](https://github.com/mairie360/ELearning_API/pull/178)), puis à `edaa2a8` ([#183](https://github.com/mairie360/ELearning_API/pull/183)), puis à `48b7f43` ([#185](https://github.com/mairie360/ELearning_API/pull/185)). Aucun effet sur l'API en exécution.
- Dépendance `tokio` passée de 1.54.0 à 1.54.1 ([#184](https://github.com/mairie360/ELearning_API/pull/184)). Le diff n'est pas fourni : seule la description de la PR est connue (revert d'un changement sur les timers).
- Pins `mairie360/CICD` passés de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, en trois PR successives (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181)). Les corrections concernent la publication des tags et releases par `promote-prod`, sans effet sur l'API.

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
