# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "f1497322306f370bfa45b174270e7e11362d83c8", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-21 08:27 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `f149732` | 2026-10-21 08:27 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Changements de comportement
- Un push sur `main` ne lance plus les jobs staging et prod : `cicd.yml` (CICD v4.4.0) s'arrête à `release-dev`. Staging et prod se promeuvent à la main via `promote-staging.yml` et `promote-prod.yml` (`gh workflow run ... -R mairie360/ELearning_API [-f sha=SHA]`, approbation des environnements `Staging` et `Prod`). (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))
- Les stacks docker-compose (`docker-compose.yml`, `-integration`, `-performance`, `-security`) connectent l'API avec le rôle Postgres `elearning_api` (mot de passe `elearning-api-password`) au lieu du superuser `postgres`. Une requête hors des grants de `api_grants.sql` échoue avec `permission denied`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))

#### Technique
- Dépendances : `tokio` 1.53.3 → 1.54.0 (#168), `utoipa-swagger-ui` 10.0.1 → 10.0.2 (#171), `mairie360_api_lib` 3.4.0 → 4.2.0 (#162), image Docker de dev `rust:1.99-bookworm` épinglée à un nouveau digest (#178), `ghcr.io/mairie360/database` et `liquibase-migrations` 3.0.0 → 3.2.0 dans les composes (#160).
- CI et tests : CICD passe de v4.2.1 à v4.4.1 (#163, #167, #169, #175), les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis à `cicd.yml` pour le journal des releases (MAIR-571, #166). Les tests passent sous `elearning_api` avec un test de régression `tests/queries/db_role.rs`, `TEST_DB_VERSION` passe à 3.2.0, et `cargo audit` ignore RUSTSEC-2026-0367/0368/0369 (#162).

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- `development.Dockerfile` : l'image `rust:1.99-bookworm` est épinglée au digest `5c1a173` (#178).
- `cicd.yml`, `promote-staging.yml` et `promote-prod.yml` passent tous les trois à CICD v4.4.1 (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175)), qui corrige la release prod.

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement
- Un push sur `main` ne lance plus les jobs staging et prod : `cicd.yml` s'arrête à `release-dev`. Staging et prod se promeuvent à la main via `promote-staging.yml` et `promote-prod.yml` (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173)).
- Les stacks docker-compose connectent l'API avec le rôle Postgres `elearning_api` au lieu du superuser `postgres`. Une requête hors des grants de `api_grants.sql` échoue avec `permission denied`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))

#### Technique
- Dépendances et images : `mairie360_api_lib` 3.4.0 → 4.2.0, `tokio` 1.53.3 → 1.54.0 (#168), `utoipa-swagger-ui` 10.0.1 → 10.0.2 (#171), `ghcr.io/mairie360/database` et `liquibase-migrations` 3.0.0 → 3.2.0 (#160).
- CI : CICD passe de v4.2.2 à v4.4.0 (#167, #169, #173), les secrets du journal des releases sont transmis à `cicd.yml` (#166), et les tests passent sous `elearning_api`.

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 08:27 UTC | dev | — | `f149732` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39567976318) |
| 2026-10-21 03:47 UTC | dev | — | `ad7a855` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39539811823) |
| 2026-10-21 03:21 UTC | staging | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39537392237) |
| 2026-10-21 01:45 UTC | dev | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39527799559) |
| 2026-10-20 17:07 UTC | staging | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
