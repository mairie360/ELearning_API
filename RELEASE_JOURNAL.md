# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "534a52ef6376419ed968860b08ad341aa647560f", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-21 09:06 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `534a52e` | 2026-10-21 09:06 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Changements de comportement
- Les stacks docker-compose (dev, intégration, performance, sécurité) connectent désormais l'API avec le rôle Postgres `elearning_api` (mot de passe `elearning-api-password`) au lieu du superutilisateur `postgres`. Une requête que les grants de `api_grants.sql` ne couvrent pas échoue en `permission denied` (SQLSTATE 42501). (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Un push sur `main` ne lance plus que `release-dev`. Staging et prod se déclenchent à la main avec `gh workflow run promote-staging.yml` puis `promote-prod.yml` (`-f sha=` optionnel), avec l'approbation de l'environnement. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

#### Corrections
- Release prod (CICD v4.4.0) : avant, `promote-prod` ne libérait jamais la version (le dry-run du semantic-release ne lisait pas le miroir). Maintenant la version est libérée. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175))
- Release prod (CICD v4.4.1) : avant, `promote-prod` retaggait les images puis échouait à créer le tag et la release (HTTP 403). Maintenant corrigé en CICD v4.4.2. (MAIR-612, [#177](https://github.com/mairie360/ELearning_API/pull/177))

#### Technique
- Images `ghcr.io/mairie360/database` et `liquibase-migrations` passées de 3.0.0 à 3.2.0 (#160), puis à 3.3.0 (#179), dans les compose et `TEST_DB_VERSION`. Le contenu de ces schémas n'est pas dans les diffs, à vérifier dans le dépôt Database.
- Tests MAIR-556 : `mairie360_api_lib` passé de 3.4.0 à 4.2.0 sans changement de source, nouveau test `tests/queries/db_role.rs`, et ignore de RUSTSEC-2026-0367/0368/0369 dans `.cargo/audit.toml`. Le diff de #162 est tronqué.
- CI et dépendances : CICD passé de v4.2.1 à v4.4.2 au fil des PR (#163, #167, #169, #173, #175, #177), journal de release MAIR-571 (`RELEASE_JOURNAL.md` réécrit sur `main` par Claude, secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID`, `RELEASE_APP_PK` transmis via #166), tokio 1.54.0 (#168), utoipa-swagger-ui 10.0.2 (#171), digest `rust:1.99-bookworm` de `development.Dockerfile` (#178).

## Différence dev → staging (en dev, pas encore en staging)

#### Corrections
- Release prod (CICD v4.4.0) : avant, `promote-prod` ne libérait jamais la version. Maintenant la version est libérée. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175))
- Release prod (CICD v4.4.1) : avant, `promote-prod` retaggait les images puis échouait à créer le tag et la release (HTTP 403). Maintenant corrigé en CICD v4.4.2. (MAIR-612, [#177](https://github.com/mairie360/ELearning_API/pull/177))

#### Technique
- Images `ghcr.io/mairie360/database` et `liquibase-migrations` passées de 3.2.0 à 3.3.0 dans les compose et `TEST_DB_VERSION`. Le contenu du schéma n'est pas dans le diff. ([#179](https://github.com/mairie360/ELearning_API/pull/179))
- Image `rust:1.99-bookworm` épinglée à un nouveau digest dans `development.Dockerfile`. ([#178](https://github.com/mairie360/ELearning_API/pull/178))

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement
- L'API se connecte avec le rôle Postgres `elearning_api` dans les stacks docker-compose au lieu du superutilisateur, donc une requête hors grants échoue en `permission denied`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- Promotion manuelle (d'après CLAUDE.md, dans le dépôt mairie360/CICD) : la prod refuse un commit sans `staging-<sha>` et tague la version calculée par semantic-release ; le staging refuse un commit sans `dev-<sha>` et sans run vert sur `main`, puis lance les stacks intégration, ZAP et k6. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))

#### Technique
- Pipeline : `promote-staging.yml` et `promote-prod.yml` ajoutés, `renovate.json` groupe les trois fichiers, et `cicd.yml` transmet `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` pour le journal de release (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166), [#173](https://github.com/mairie360/ELearning_API/pull/173)).
- CICD passé de v4.2.1 à v4.4.0 (#163, #167, #169, #173), avec un journal de release MAIR-571 (`RELEASE_JOURNAL.md` réécrit sur `main`).
- Dépendances : tokio 1.53.3 vers 1.54.0 (#168), utoipa-swagger-ui 10.0.1 vers 10.0.2 (#171). Image `ghcr.io/mairie360/database` passée de 3.0.0 à 3.2.0 (#160), contenu du schéma non visible dans le diff.
- Tests et audit de MAIR-556 : `mairie360_api_lib` passé à 4.2.0 sans changement de source, et ignore de RUSTSEC-2026-0367/0368/0369 (le diff de #162 est tronqué).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 09:06 UTC | dev | — | `534a52e` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39573426480) |
| 2026-10-21 08:29 UTC | dev | — | `8c0d27f` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39568593745) |
| 2026-10-21 08:27 UTC | dev | — | `f149732` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39567976318) |
| 2026-10-21 03:47 UTC | dev | — | `ad7a855` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39539811823) |
| 2026-10-21 03:21 UTC | staging | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39537392237) |
| 2026-10-21 01:45 UTC | dev | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39527799559) |
| 2026-10-20 17:07 UTC | staging | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
