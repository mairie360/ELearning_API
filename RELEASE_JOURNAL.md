# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "421175174944ffb955ab28c3796ec450bd13a61f", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-21 09:08 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `4211751` | 2026-10-21 09:08 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Technique
- Versions CICD alignées de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` : corrections successives de la promotion prod (publication de la version, du tag et de la release par `promote-prod`). (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181))
- Image de base de `development.Dockerfile` épinglée sur le nouveau digest de `rust:1.99-bookworm` (`5c1a173`). Seul le Dockerfile de développement change, pas le Dockerfile de production. ([#178](https://github.com/mairie360/ELearning_API/pull/178))
- Schéma Postgres `ghcr.io/mairie360/database` et migrations `liquibase-migrations` passés de 3.2.0 à 3.3.0 dans les docker-compose (principal, intégration, performance, sécurité) et dans `TEST_DB_VERSION`. Le contenu des migrations 3.3.0 n'apparaît pas dans ces diffs : à vérifier dans le dépôt de la base. ([#179](https://github.com/mairie360/ELearning_API/pull/179))

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Versions CICD alignées de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` : corrections successives de la promotion prod (publication de la version, du tag et de la release par `promote-prod`). (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181))
- Image de base de `development.Dockerfile` épinglée sur le nouveau digest de `rust:1.99-bookworm` (`5c1a173`). Seul le Dockerfile de développement change, pas le Dockerfile de production. ([#178](https://github.com/mairie360/ELearning_API/pull/178))
- Schéma Postgres `ghcr.io/mairie360/database` et migrations `liquibase-migrations` passés de 3.2.0 à 3.3.0 dans les docker-compose (principal, intégration, performance, sécurité) et dans `TEST_DB_VERSION`. Le contenu des migrations 3.3.0 n'apparaît pas dans ces diffs : à vérifier dans le dépôt de la base. ([#179](https://github.com/mairie360/ELearning_API/pull/179))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
