# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "e3ffc1c01cdbe5173a535e6e3109eed900eb77f5", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "e3ffc1c01cdbe5173a535e6e3109eed900eb77f5", "updated": {"dev": "2026-10-19 20:24 UTC", "staging": "2026-10-20 17:07 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `e3ffc1c` | 2026-10-20 17:07 UTC |
| dev | — | `e3ffc1c` | 2026-10-19 20:24 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Technique
- Aucun changement fonctionnel pour les appelants de l'API. Les tests et les quatre stacks `docker-compose*.yml` connectent désormais l'API avec le rôle Postgres `elearning_api` au lieu du superuser. Le nouveau test `tests/queries/db_role.rs` vérifie le refus `42501` sur `sessions` et `users.password`. Côté dépendances, `mairie360_api_lib` passe de 3.4.0 à 4.2.0, sans modification du code source, et les images `ghcr.io/mairie360/database` / `liquibase-migrations` passent de 3.0.0 à 3.2.0, y compris `TEST_DB_VERSION`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162), [#160](https://github.com/mairie360/ELearning_API/pull/160))
- 5 changements de CI et de dépendances : workflow `mairie360/CICD` passé de v4.2.1 à v4.3.1 en trois mises à jour, et transmission des secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` pour le journal des releases. Mise à jour de `tokio` en 1.54.0. (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166), [#169](https://github.com/mairie360/ELearning_API/pull/169), [#167](https://github.com/mairie360/ELearning_API/pull/167), [#163](https://github.com/mairie360/ELearning_API/pull/163), [#168](https://github.com/mairie360/ELearning_API/pull/168))

## Différence dev → staging (en dev, pas encore en staging)

_Rien._

## Différence staging → prod (en staging, pas encore en prod)

#### Technique
- Aucun changement fonctionnel pour les appelants de l'API. Les tests et les quatre stacks `docker-compose*.yml` connectent désormais l'API avec le rôle Postgres `elearning_api` au lieu du superuser. Le nouveau test `tests/queries/db_role.rs` vérifie le refus `42501` sur `sessions` et `users.password`. Côté dépendances, `mairie360_api_lib` passe de 3.4.0 à 4.2.0, sans modification du code source, et les images `ghcr.io/mairie360/database` / `liquibase-migrations` passent de 3.0.0 à 3.2.0, y compris `TEST_DB_VERSION`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162), [#160](https://github.com/mairie360/ELearning_API/pull/160))
- 5 changements de CI et de dépendances : workflow `mairie360/CICD` passé de v4.2.1 à v4.3.1 en trois mises à jour, et transmission des secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` pour le journal des releases. Mise à jour de `tokio` en 1.54.0. (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166), [#169](https://github.com/mairie360/ELearning_API/pull/169), [#167](https://github.com/mairie360/ELearning_API/pull/167), [#163](https://github.com/mairie360/ELearning_API/pull/163), [#168](https://github.com/mairie360/ELearning_API/pull/168))

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-20 17:07 UTC | staging | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
