# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-10-21 01:45 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `12c5977` | 2026-10-21 01:45 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Changements de comportement

- Les stacks docker-compose (`docker-compose.yml`, `docker-compose-integration.yml`, `docker-compose-performance.yml`, `docker-compose-security.yml`) connectent désormais l'API avec le rôle Postgres `elearning_api` au lieu du superuser `postgres`. Une requête hors des grants de ce rôle échoue en `permission denied`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))

#### Technique

- Les images `ghcr.io/mairie360/database` et `ghcr.io/mairie360/liquibase-migrations` passent de 3.0.0 à 3.2.0 dans les quatre stacks compose. ([#160](https://github.com/mairie360/ELearning_API/pull/160))
- Dépendances : `tokio` 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168)), `utoipa-swagger-ui` 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171)), `mairie360_api_lib` 4.2.0 sans changement de source ([#162](https://github.com/mairie360/ELearning_API/pull/162)), `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 pour débloquer `cargo audit` ([#173](https://github.com/mairie360/ELearning_API/pull/173)).
- Tests : les tests de requêtes et les `AppState` passent par le rôle `elearning_api`, les fixtures restent en superuser ; nouveau test `tests/queries/db_role.rs` ; `TEST_DB_VERSION` passe à 3.2.0 ; `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369 (zip, build-dependency de `utoipa-swagger-ui`). (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- CI : `cicd.yml` passe sur `mairie360/CICD` v4.4.0 et s'arrête à `release-dev` sur un push vers `main` ; staging et prod passent par les workflows manuels `promote-staging.yml` et `promote-prod.yml`. Renovate regroupe les trois fichiers. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))
- CI : les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis au workflow réutilisable. (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166))
- CI : pins `mairie360/CICD` passés de v4.2.1 à v4.2.2 ([#163](https://github.com/mairie360/ELearning_API/pull/163)), de v4.2.2 à v4.3.0 ([#167](https://github.com/mairie360/ELearning_API/pull/167)), puis à v4.3.1 ([#169](https://github.com/mairie360/ELearning_API/pull/169)).

## Différence dev → staging (en dev, pas encore en staging)

_Rien._

## Différence staging → prod (en staging, pas encore en prod)

#### Changements de comportement

- Les stacks docker-compose (`docker-compose.yml`, `docker-compose-integration.yml`, `docker-compose-performance.yml`, `docker-compose-security.yml`) connectent désormais l'API avec le rôle Postgres `elearning_api` au lieu du superuser `postgres`. Une requête hors des grants de ce rôle échoue en `permission denied`. (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))

#### Technique

- Les images `ghcr.io/mairie360/database` et `ghcr.io/mairie360/liquibase-migrations` passent de 3.0.0 à 3.2.0 dans les quatre stacks compose. ([#160](https://github.com/mairie360/ELearning_API/pull/160))
- Dépendances : `tokio` 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168)), `utoipa-swagger-ui` 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171)), `mairie360_api_lib` 4.2.0 sans changement de source ([#162](https://github.com/mairie360/ELearning_API/pull/162)), `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 pour débloquer `cargo audit` ([#173](https://github.com/mairie360/ELearning_API/pull/173)).
- Tests : les tests de requêtes et les `AppState` passent par le rôle `elearning_api`, les fixtures restent en superuser ; nouveau test `tests/queries/db_role.rs` ; `TEST_DB_VERSION` passe à 3.2.0 ; `.cargo/audit.toml` ignore RUSTSEC-2026-0367, 0368 et 0369 (zip, build-dependency de `utoipa-swagger-ui`). (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162))
- CI : `cicd.yml` passe sur `mairie360/CICD` v4.4.0 et s'arrête à `release-dev` sur un push vers `main` ; staging et prod passent par les workflows manuels `promote-staging.yml` et `promote-prod.yml`. Renovate regroupe les trois fichiers. (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173))
- CI : les secrets `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` sont transmis au workflow réutilisable. (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166))
- CI : pins `mairie360/CICD` passés de v4.2.1 à v4.2.2 ([#163](https://github.com/mairie360/ELearning_API/pull/163)), de v4.2.2 à v4.3.0 ([#167](https://github.com/mairie360/ELearning_API/pull/167)), puis à v4.3.1 ([#169](https://github.com/mairie360/ELearning_API/pull/169)).

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 03:21 UTC | staging | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39537392237) |
| 2026-10-21 01:45 UTC | dev | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39527799559) |
| 2026-10-20 17:07 UTC | staging | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
