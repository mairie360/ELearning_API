# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "ad7a855a10c9fdeccf73564a622c0a64306d63fd", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-21 03:47 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `ad7a855` | 2026-10-21 03:47 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Technique
- Les stacks Docker Compose (`docker-compose.yml`, intégration, performance, sécurité) passent à `ghcr.io/mairie360/database` et `liquibase-migrations` 3.2.0, et l'API s'y connecte avec le rôle `elearning_api` au lieu du superuser. Les tests d'API passent aussi par ce rôle, avec un test de non-régression sur ses droits. Changement de fichiers de test et de compose, sans modification du code applicatif livré. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162), [#160](https://github.com/mairie360/ELearning_API/pull/160))
- `mairie360_api_lib` passe de 3.4.0 à 4.2.0 (saut de version majeure, la PR indique qu'aucun changement de source n'était nécessaire). `TEST_DB_VERSION` passe de 3.0.0 à 3.2.0. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369, qui concernent zip utilisé uniquement dans le build script de `utoipa-swagger-ui`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Dépendances mises à jour : tokio 1.53.3 vers 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168)) et utoipa-swagger-ui 10.0.1 vers 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171)).
- Workflows CI : pin CICD passé de v4.2.2 à v4.4.1, transmission des secrets de release (ANTHROPIC_API_KEY, RELEASE_APP_ID, RELEASE_APP_PK) à `cicd.yml`, nouveaux workflows manuels `promote-staging.yml` et `promote-prod.yml`, et `renovate.json` qui couvre les trois fichiers. (MAIR-494, MAIR-571, MAIR-612, [#173](https://github.com/mairie360/ELearning_API/pull/173), [#166](https://github.com/mairie360/ELearning_API/pull/166), [#175](https://github.com/mairie360/ELearning_API/pull/175))
- Documentation de `CLAUDE.md` mise à jour sur la promotion et les tests. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

## Différence dev → staging (en dev, pas encore en staging)

#### Technique
- Pin mairie360/CICD passé de v4.4.0 à v4.4.1 dans `cicd.yml`, `promote-staging.yml` et `promote-prod.yml`, pour corriger la release de `promote-prod` (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175)).

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement
- Un push sur `main` ne déclenche plus que `release-dev` : staging et prod ne se déploient plus automatiquement et se promeuvent à la main via `gh workflow run promote-staging.yml` puis `promote-prod.yml` (avec `-f sha=` pour un commit donné). Ce changement touche le processus de release, pas le comportement de l'API. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

#### Technique
- Dépendances : tokio 1.53.3 vers 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168)) et utoipa-swagger-ui 10.0.1 vers 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171)).
- Tests et stacks Compose : `mairie360_api_lib` passe de 3.4.0 à 4.2.0 (majeure, sans changement de source selon la PR), l'API et les tests se connectent avec le rôle `elearning_api`, et les images Database et Liquibase passent de 3.0.0 à 3.2.0. Aucun changement du code applicatif livré. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162), [#160](https://github.com/mairie360/ELearning_API/pull/160))
- Ajout des ignores RUSTSEC-2026-0367, 0368 et 0369 dans `.cargo/audit.toml`, pour le zip de build de `utoipa-swagger-ui`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Pin CICD passé à v4.4.0 puis les workflows `promote-staging.yml` et `promote-prod.yml` ajoutés. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 03:47 UTC | dev | — | `ad7a855` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39539811823) |
| 2026-10-21 03:21 UTC | staging | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39537392237) |
| 2026-10-21 01:45 UTC | dev | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39527799559) |
| 2026-10-20 17:07 UTC | staging | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
