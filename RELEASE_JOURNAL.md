# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "ba18081d50321354e5961ed0c2d8d2703141bc5e", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-22 06:44 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `ba18081` | 2026-10-22 06:44 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Technique
- Mise à jour de tokio de 1.54.0 à 1.54.1 (selon la description de la PR, un retour arrière sur les timers ; le diff de Cargo.lock n'est pas fourni, donc non vérifiable ici) ([#184](https://github.com/mairie360/ELearning_API/pull/184)).
- Version du schéma de base passée de 3.2.0 à 3.3.0 : images ghcr.io/mairie360/database et liquibase-migrations dans docker-compose.yml et ses variantes integration, performance et security, plus TEST_DB_VERSION dans .cargo/config.toml. Le contenu des migrations 3.3.0 n'apparaît pas dans ces diffs ([#179](https://github.com/mairie360/ELearning_API/pull/179)).
- Image de développement rust:1.99-bookworm épinglée par digest dans development.Dockerfile (e12566d puis 5c1a173, puis edaa2a8) ([#178](https://github.com/mairie360/ELearning_API/pull/178), [#183](https://github.com/mairie360/ELearning_API/pull/183)).
- Pipeline de release : mairie360/CICD passe de v4.4.0 à v4.4.3 dans cicd.yml, promote-staging.yml et promote-prod.yml, ce qui corrige la création du tag et de la release lors de la promotion en prod (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181)).

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Mise à jour de tokio de 1.54.0 à 1.54.1 (selon la description de la PR, un retour arrière sur les timers ; le diff de Cargo.lock n'est pas fourni, donc non vérifiable ici) ([#184](https://github.com/mairie360/ELearning_API/pull/184)).
- Version du schéma de base passée de 3.2.0 à 3.3.0 : images ghcr.io/mairie360/database et liquibase-migrations dans docker-compose.yml et ses variantes integration, performance et security, plus TEST_DB_VERSION dans .cargo/config.toml. Le contenu des migrations 3.3.0 n'apparaît pas dans ces diffs ([#179](https://github.com/mairie360/ELearning_API/pull/179)).
- Image de développement rust:1.99-bookworm épinglée par digest dans development.Dockerfile (e12566d puis 5c1a173, puis edaa2a8) ([#178](https://github.com/mairie360/ELearning_API/pull/178), [#183](https://github.com/mairie360/ELearning_API/pull/183)).
- Pipeline de release : mairie360/CICD passe de v4.4.0 à v4.4.3 dans cicd.yml, promote-staging.yml et promote-prod.yml, ce qui corrige la création du tag et de la release lors de la promotion en prod (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181)).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
