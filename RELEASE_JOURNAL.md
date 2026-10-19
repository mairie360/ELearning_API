# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "e3ffc1c01cdbe5173a535e6e3109eed900eb77f5", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "updated": {"dev": "2026-10-19 20:24 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `87d9e8f` | — |
| dev | — | `e3ffc1c` | 2026-10-19 20:24 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Technique
- Les tests et les stacks `docker-compose*.yml` locales exécutent désormais le code de l'API avec le rôle Postgres `elearning_api` au lieu du superuser, comme en staging/prod. Une requête hors des droits accordés échoue donc dès les tests, ce que vérifie le nouveau test `tests/queries/db_role.rs`. Ce lot monte aussi `mairie360_api_lib` de 3.4.0 à 4.2.0 sans changement de code, et les images `ghcr.io/mairie360/database` et `liquibase-migrations` de 3.0.0 à 3.2.0. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162), [#160](https://github.com/mairie360/ELearning_API/pull/160))
- 5 changements de CI et de dépendances : workflow `mairie360/CICD` passé de v4.2.1 à v4.3.1 en trois mises à jour, transmission des secrets du journal des releases (`ANTHROPIC_API_KEY`, `RELEASE_APP_ID`, `RELEASE_APP_PK`) et `tokio` monté en 1.54.0. (MAIR-571, [#163](https://github.com/mairie360/ELearning_API/pull/163), [#167](https://github.com/mairie360/ELearning_API/pull/167), [#169](https://github.com/mairie360/ELearning_API/pull/169), [#166](https://github.com/mairie360/ELearning_API/pull/166), [#168](https://github.com/mairie360/ELearning_API/pull/168))

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Les tests et les stacks `docker-compose*.yml` locales exécutent désormais le code de l'API avec le rôle Postgres `elearning_api` au lieu du superuser, comme en staging/prod. Une requête hors des droits accordés échoue donc dès les tests, ce que vérifie le nouveau test `tests/queries/db_role.rs`. Ce lot monte aussi `mairie360_api_lib` de 3.4.0 à 4.2.0 sans changement de code, et les images `ghcr.io/mairie360/database` et `liquibase-migrations` de 3.0.0 à 3.2.0. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162), [#160](https://github.com/mairie360/ELearning_API/pull/160))
- 5 changements de CI et de dépendances : workflow `mairie360/CICD` passé de v4.2.1 à v4.3.1 en trois mises à jour, transmission des secrets du journal des releases (`ANTHROPIC_API_KEY`, `RELEASE_APP_ID`, `RELEASE_APP_PK`) et `tokio` monté en 1.54.0. (MAIR-571, [#163](https://github.com/mairie360/ELearning_API/pull/163), [#167](https://github.com/mairie360/ELearning_API/pull/167), [#169](https://github.com/mairie360/ELearning_API/pull/169), [#166](https://github.com/mairie360/ELearning_API/pull/166), [#168](https://github.com/mairie360/ELearning_API/pull/168))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
