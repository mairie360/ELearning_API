# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "421175174944ffb955ab28c3796ec450bd13a61f", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-21 09:08 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `4211751` | 2026-10-21 09:08 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Nouveautés
- Les promotions staging et prod sont désormais manuelles : `gh workflow run promote-staging.yml` et `promote-prod.yml` (entrée optionnelle `sha`), avec verrou par cible et approbation des environnements `Staging` / `Prod`. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))
- Le workflow de release réécrit `RELEASE_JOURNAL.md` sur `main` avec un résumé rédigé par Claude. Les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis au workflow `cicd.yml`. (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166), [#169](https://github.com/mairie360/ELearning_API/pull/169), [#167](https://github.com/mairie360/ELearning_API/pull/167))

#### Changements de comportement
- Un push sur `main` ne lance plus que `release-dev` : staging et prod ne se déploient plus automatiquement. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))
- Les stacks `docker-compose*.yml` connectent l'API avec le rôle Postgres `elearning_api` (mot de passe `elearning-api-password`) au lieu du superuser `postgres`. Une requête hors des grants du rôle échoue avec `permission denied`. Liquibase reçoit `-Delearning_api_password`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))

#### Technique
- Dépendance `mairie360_api_lib` passée de 3.4.0 à 4.2.0 (`Cargo.toml`), sans changement de code source. Les tests passent par `elearning_api` (`tests/queries/db_role.rs`). (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369 (zip 8 utilisé seulement par le build de utoipa-swagger-ui). (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Mises à jour de dépendances sans diff visible : utoipa-swagger-ui 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171)) et tokio 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168)).
- Les pins du workflow CICD avancent de v4.2.1 à v4.4.3 au fil des PR, dont des corrections de la chaîne de release (`promote-prod`). (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181))
- Images `ghcr.io/mairie360/database` et `liquibase-migrations` passées de 3.0.0 à 3.3.0 (compose et `TEST_DB_VERSION`). Le contenu du schéma n'est pas visible dans ce diff. (Renovate, [#160](https://github.com/mairie360/ELearning_API/pull/160), [#179](https://github.com/mairie360/ELearning_API/pull/179))
- Digest de l'image `rust:1.99-bookworm` mis à jour dans `development.Dockerfile`. (Renovate, [#178](https://github.com/mairie360/ELearning_API/pull/178))

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Pins du workflow CICD passés de v4.4.0 à v4.4.3 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` : v4.4.1 corrige la dry-run semantic-release de `promote-prod`, v4.4.2 corrige la création du tag et de la release (HTTP 403), v4.4.3 pousse le tag avec le token de l'app release. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181))
- Images `ghcr.io/mairie360/database` et `liquibase-migrations` passées de 3.2.0 à 3.3.0 dans les quatre fichiers compose. Le contenu du schéma n'est pas visible dans ce diff. (Renovate, [#179](https://github.com/mairie360/ELearning_API/pull/179))
- Digest de l'image `rust:1.99-bookworm` mis à jour dans `development.Dockerfile`. (Renovate, [#178](https://github.com/mairie360/ELearning_API/pull/178))

## Différence staging → prod (en staging, pas encore en prod)

#### Nouveautés
- Les promotions staging et prod sont désormais manuelles : `gh workflow run promote-staging.yml` et `promote-prod.yml` (entrée optionnelle `sha`), avec verrou par cible et approbation des environnements `Staging` / `Prod`. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))
- Le workflow de release réécrit `RELEASE_JOURNAL.md` sur `main` avec un résumé rédigé par Claude. Les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis au workflow `cicd.yml`. (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166), [#169](https://github.com/mairie360/ELearning_API/pull/169), [#167](https://github.com/mairie360/ELearning_API/pull/167))

#### Changements de comportement
- Un push sur `main` ne lance plus que `release-dev` : staging et prod ne se déploient plus automatiquement. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))
- Les stacks `docker-compose*.yml` connectent l'API avec le rôle Postgres `elearning_api` (mot de passe `elearning-api-password`) au lieu du superuser `postgres`. Une requête hors des grants du rôle échoue avec `permission denied`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))

#### Technique
- Dépendance `mairie360_api_lib` passée de 3.4.0 à 4.2.0, sans changement de code source ; les tests passent par `elearning_api`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Mises à jour sans diff visible : utoipa-swagger-ui 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171)) et tokio 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168)).
- Pins CICD passés de v4.2.1 à v4.4.0, puis v4.3.0 et v4.3.1 pour le journal des releases. (Renovate, [#163](https://github.com/mairie360/ELearning_API/pull/163), [#167](https://github.com/mairie360/ELearning_API/pull/167), [#169](https://github.com/mairie360/ELearning_API/pull/169), [#173](https://github.com/mairie360/ELearning_API/pull/173))
- Images `ghcr.io/mairie360/database` et `liquibase-migrations` passées de 3.0.0 à 3.2.0 (compose et `TEST_DB_VERSION`, qui passe de 3.0.0 à 3.2.0). (Renovate, [#160](https://github.com/mairie360/ELearning_API/pull/160), [#162](https://github.com/mairie360/ELearning_API/pull/162))

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
