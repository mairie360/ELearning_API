# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod": "87d9e8f56c7938d9d29e6cadb3b7539d9fdee328", "prod_tag": "v1.1.0", "staging": "e3ffc1c01cdbe5173a535e6e3109eed900eb77f5", "updated": {"dev": "2026-10-21 01:45 UTC", "staging": "2026-10-20 17:07 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.1.0 | `87d9e8f` | — |
| staging | — | `e3ffc1c` | 2026-10-20 17:07 UTC |
| dev | — | `12c5977` | 2026-10-21 01:45 UTC |

## Ajouté sur main depuis la dernière release prod (v1.1.0)

#### Technique

- Un push sur `main` ne libère plus que dev : `cicd.yml` s'arrête à `release-dev`, et deux workflows manuels `promote-staging.yml` et `promote-prod.yml` prennent le relais (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173)).
- `mairie360/CICD` passe de v4.2.1 à v4.4.0 au fil de #163, #167, #169 et #173. `cicd.yml` transmet désormais `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` pour le journal des releases, qui réécrit `RELEASE_JOURNAL.md` sur `main` (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166)).
- Dépendances mises à jour : tokio 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168)), utoipa-swagger-ui 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171)), et selon la PR #173, `rustls-webpki` 0.103.16 et `webpki-roots` / `webpki-root-certs` 1.0.10 pour débloquer `cargo audit` (RUSTSEC-2026-0381/0382/0383).
- Les tests d'API et les stacks docker-compose se connectent comme le rôle `elearning_api` au lieu du superuser ; `mairie360_api_lib` passe en 4.2.0 ; `TEST_DB_VERSION` passe à 3.2.0 ; trois advisories zip sont ignorées dans `.cargo/audit.toml` (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162)).
- L'image `ghcr.io/mairie360/database` et `liquibase-migrations` passent de 3.0.0 à 3.2.0 dans les fichiers compose (chore, [#160](https://github.com/mairie360/ELearning_API/pull/160)).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement

- Un push sur `main` ne déploie plus en staging : `cicd.yml` s'arrête à `release-dev`. Le passage en staging se fait à la main avec `gh workflow run promote-staging.yml -R mairie360/ELearning_API` (dernier commit de `main`) ou avec `-f sha=SHA`. Il exige l'image `dev-<sha>` et un run vert sur `main`, puis lance les stacks d'intégration, ZAP et k6 sur cette image (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173)).

#### Technique

- `cargo audit` est débloqué sur `main` : selon la PR, `rustls-webpki` passe à 0.103.16 et `webpki-roots` / `webpki-root-certs` à 1.0.10 (RUSTSEC-2026-0381/0382/0383) (MAIR-494, [#173](https://github.com/mairie360/ELearning_API/pull/173)).
- `utoipa-swagger-ui` passe de 10.0.1 à 10.0.2 ([#171](https://github.com/mairie360/ELearning_API/pull/171)). Le diff est vide ; la mise à jour vient de la description de la PR.

## Différence staging → prod (en staging, pas encore en prod)

#### Technique

- `mairie360/CICD` passe de v4.2.2 à v4.3.1 (#167, #169). `cicd.yml` transmet `ANTHROPIC_API_KEY`, `RELEASE_APP_ID` et `RELEASE_APP_PK` pour l'étape de journal des releases, qui réécrit `RELEASE_JOURNAL.md` sur `main` (MAIR-571, [#166](https://github.com/mairie360/ELearning_API/pull/166)).
- Les tests d'API et les stacks compose passent sous le rôle Postgres `elearning_api`, avec `mairie360_api_lib` en 4.2.0 : la PR indique qu'aucun changement de code source n'a été nécessaire pour les breaking changes 4.x (MAIR-556, [#162](https://github.com/mairie360/ELearning_API/pull/162)).
- L'image Database et liquibase passe de 3.0.0 à 3.2.0 dans les compose (chore, [#160](https://github.com/mairie360/ELearning_API/pull/160)). Le diff ne concerne que les compose, pas les déploiements staging ou prod.
- `tokio` passe de 1.53.3 à 1.54.0 ([#168](https://github.com/mairie360/ELearning_API/pull/168)).
- Le job npm audit de `front-libs` ne couvre plus que les dépendances de production (CICD v4.2.2, [#163](https://github.com/mairie360/ELearning_API/pull/163)). Ce changement concerne le dépôt CICD et n'a pas d'effet direct sur ce service.

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-10-21 01:45 UTC | dev | — | `12c5977` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39527799559) |
| 2026-10-20 17:07 UTC | staging | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
| 2026-10-19 20:24 UTC | dev | — | `e3ffc1c` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39316666630) |
<!-- release-journal-history:end -->
