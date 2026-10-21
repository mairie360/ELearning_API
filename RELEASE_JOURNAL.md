# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "8c0d27fdd7b46451be972ebf26de2e828c87fce1", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-21 08:29 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `8c0d27f` | 2026-10-21 08:29 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Nouveautés
- Promotion manuelle vers staging et prod : `promote-staging.yml` et `promote-prod.yml` (déclenchement `workflow_dispatch`, entrée optionnelle `sha`, un seul run à la fois par cible). (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

#### Changements de comportement
- Un push sur `main` s'arrête désormais à `release-dev` : staging et prod ne se lancent plus automatiquement. La promotion se fait à la main avec `gh workflow run promote-staging.yml -R mairie360/ELearning_API` (ou `promote-prod.yml`), et `-f sha=<sha>` pour un commit donné. Staging refuse un commit sans image `dev-<sha>` ni run `main` vert ; prod refuse un commit sans `staging-<sha>`. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

#### Corrections
- Promotion prod avec `mairie360/CICD` v4.4.0 : la release ne se publiait jamais, car le dry run semantic-release ne lisait pas son miroir. Avec v4.4.1, la version est publiée. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175))
- Promotion prod : avant, `promote-prod` retaggait les images puis échouait (HTTP 403) à créer le tag et la release. Avec v4.4.2, le tag et la release sont créés. (MAIR-612, [#177](https://github.com/mairie360/ELearning_API/pull/177))
- Journal des releases : avant, le step ne recevait pas `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` ni `RELEASE_APP_PK`, faute de mapping dans `secrets:`. Maintenant ces secrets sont transmis dans `cicd.yml`. (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166))

#### Technique
- Journal des releases (CICD v4.3.0 puis v4.3.1) : le workflow réutilisable réécrit `RELEASE_JOURNAL.md` sur `main` avec un résumé rédigé par Claude. (MAIR-571, [#167](https://github.com/mairie360/ELearning_API/pull/167), [#169](https://github.com/mairie360/ELearning_API/pull/169))
- Pins `mairie360/CICD` successifs dans `cicd.yml` et les `promote-*.yml` : v4.2.2, v4.3.0, v4.3.1, v4.4.0, v4.4.1, v4.4.2. ([#163](https://github.com/mairie360/ELearning_API/pull/163), [#173](https://github.com/mairie360/ELearning_API/pull/173), [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177))
- `mairie360_api_lib` passe de 3.4.0 à 4.2.0 (dépendances et dev-dépendances), sans changement de source selon la PR. Les tests et `AppState` se connectent comme le rôle `elearning_api` ; fixtures en superuser ; nouveau test `tests/queries/db_role.rs` ; `TEST_DB_VERSION` passe de 3.0.0 à 3.2.0. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Stacks docker-compose : l'API s'y connecte comme `elearning_api`, et Liquibase reçoit `-Delearning_api_password`. Images `database` et `liquibase-migrations` passées de 3.0.0 à 3.2.0. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162) ; [#160](https://github.com/mairie360/ELearning_API/pull/160))
- `cargo audit` : RUSTSEC-2026-0367, 0368 et 0369 ignorés dans `.cargo/audit.toml` (zip du build de utoipa-swagger-ui). (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Dépendances : tokio 1.53.3 vers 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168), diff non fourni) ; utoipa-swagger-ui 10.0.1 vers 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171), diff non fourni) ; digest de l'image `rust:1.99-bookworm` de `development.Dockerfile` ([#178](https://github.com/mairie360/ELearning_API/pull/178)).
- Documentation : `CLAUDE.md` décrit la promotion et le rôle `elearning_api` ; `renovate.json` regroupe les pins CICD des trois workflows.

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections
- Promotion prod avec `mairie360/CICD` v4.4.0 : la release ne se publiait jamais. Avec v4.4.1 (pin introduit ici), la version est publiée. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175))
- Promotion prod : avant, `promote-prod` retaggait les images puis échouait (HTTP 403) à créer le tag et la release. Avec v4.4.2 (pin introduit ici), le tag et la release sont créés. (MAIR-612, [#177](https://github.com/mairie360/ELearning_API/pull/177))

#### Technique
- Digest de l'image `rust:1.99-bookworm` de `development.Dockerfile` mis à jour : seul l'environnement de développement local change. ([#178](https://github.com/mairie360/ELearning_API/pull/178))

## Différence staging → prod (en staging, pas encore en prod)

#### Nouveautés
- Promotion manuelle vers staging et prod : `promote-staging.yml` et `promote-prod.yml` (déclenchement `workflow_dispatch`, entrée optionnelle `sha`, un seul run à la fois par cible). (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

#### Changements de comportement
- Un push sur `main` s'arrête désormais à `release-dev` : staging et prod ne se lancent plus automatiquement. La promotion se fait à la main avec `gh workflow run promote-staging.yml -R mairie360/ELearning_API` (ou `promote-prod.yml`), et `-f sha=<sha>` pour un commit donné. Staging refuse un commit sans image `dev-<sha>` ni run `main` vert ; prod refuse un commit sans `staging-<sha>`. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

#### Corrections
- Journal des releases : avant, le step ne recevait pas `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` ni `RELEASE_APP_PK`, faute de mapping dans `secrets:`. Maintenant ces secrets sont transmis dans `cicd.yml`. (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166))

#### Technique
- Journal des releases (CICD v4.3.0 puis v4.3.1) : le workflow réutilisable réécrit `RELEASE_JOURNAL.md` sur `main`. (MAIR-571, [#167](https://github.com/mairie360/ELearning_API/pull/167), [#169](https://github.com/mairie360/ELearning_API/pull/169))
- Pins `mairie360/CICD` : v4.2.2 ([#163](https://github.com/mairie360/ELearning_API/pull/163)), v4.3.0, v4.3.1, puis v4.4.0 dans `cicd.yml` et les `promote-*.yml` (#173).
- `mairie360_api_lib` passe de 3.4.0 à 4.2.0, sans changement de source selon la PR. Les tests et `AppState` se connectent comme le rôle `elearning_api` ; `TEST_DB_VERSION` passe à 3.2.0 ; `cargo audit` ignore RUSTSEC-2026-0367, 0368 et 0369. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Stacks docker-compose : l'API s'y connecte comme `elearning_api`. Images `database` et `liquibase-migrations` passées de 3.0.0 à 3.2.0. ([#160](https://github.com/mairie360/ELearning_API/pull/160), [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Dépendances : tokio 1.53.3 vers 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168), diff non fourni) ; utoipa-swagger-ui 10.0.1 vers 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171), diff non fourni).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 08:29 UTC | dev | — | `8c0d27f` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39568593745) |
| 2026-10-21 08:27 UTC | dev | — | `f149732` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39567976318) |
| 2026-10-21 03:47 UTC | dev | — | `ad7a855` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39539811823) |
| 2026-10-21 03:21 UTC | staging | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39537392237) |
| 2026-10-21 01:45 UTC | dev | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39527799559) |
| 2026-10-20 17:07 UTC | staging | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
