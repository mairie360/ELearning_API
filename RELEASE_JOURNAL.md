# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "37b0f3721dc54f68b5ac6a2a010d6237a6a06167", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-04 18:19 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `37b0f37` | 2026-11-04 18:19 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement

- Les jetons HS256 sont désormais refusés (401) : l'API ne lit plus `JWT_SECRET` et n'accepte que les jetons EdDSA signés par Core_API avec la clé publique `JWT_PUBLIC_KEY`. Tout appelant qui signait ses propres jetons HS256 doit être migré, et un déploiement doit retirer `JWT_SECRET` et définir `JWT_PUBLIC_KEY`. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Les journaux de requêtes n'enregistrent plus la query string ni le `Referer` : seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. Les erreurs de base de données sont décrites par SQLSTATE, contrainte, table et colonne, sans le message Postgres ni le `DETAIL`. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique

- Image d'exécution passée en `gcr.io/distroless/cc-debian13:nonroot`, builder et `development.Dockerfile` en `trixie` (même Debian partout). Test `dockerfile_test.rs` ajouté. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Matrice d'accès `access-matrix.yaml` et test qui vérifie chaque opération de l'OpenAPI, sans changement du comportement de l'API. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Contrôles RGPD en CI : simulation `gdpr-simulation.yaml`, compose et script dédiés. Les workflows passent à CICD v5.3.1. (MAIR-497, [#198](https://github.com/mairie360/ELearning_API/pull/198))
- Workflows alignés sur CICD v5.2.2, avec `ANTHROPIC_WORKSPACE_ID` transmis, et test de cohérence de version. (MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196))
- Checklist RGPD sur les PR via `gdpr-pr.yml`. (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138))
- Passage de CICD v4.4.0 à v4.4.3 dans les workflows de release. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181))
- Corrections des alertes RustSec de `cargo audit` : zstd, zstd-safe, zstd-sys et ctutils mis à jour, et `RUSTSEC-2026-0387` ignoré. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Mises à jour Renovate : images Docker épinglées par digest (curl, nginx, node, postgres, redis, zap, k6, newman), digests Rust, tokio 1.54.1, actix-web 4.18.1 (diff absent de la PR, à vérifier), et schéma Database / liquibase en 3.3.0 (le contenu des migrations n'apparaît pas dans ce diff). (MAIR-…, [#179](https://github.com/mairie360/ELearning_API/pull/179), [#184](https://github.com/mairie360/ELearning_API/pull/184), [#186](https://github.com/mairie360/ELearning_API/pull/186), [#194](https://github.com/mairie360/ELearning_API/pull/194), [#202](https://github.com/mairie360/ELearning_API/pull/202))

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement

- Les jetons HS256 sont désormais refusés (401) : l'API ne lit plus `JWT_SECRET` et n'accepte que les jetons EdDSA signés par Core_API avec la clé publique `JWT_PUBLIC_KEY`. Tout appelant qui signait ses propres jetons HS256 doit être migré, et un déploiement doit retirer `JWT_SECRET` et définir `JWT_PUBLIC_KEY`. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Les journaux de requêtes n'enregistrent plus la query string ni le `Referer` : seuls l'adresse du client, la méthode, le chemin sans query, le statut, la taille et la durée sont journalisés. Les erreurs de base de données sont décrites par SQLSTATE, contrainte, table et colonne, sans le message Postgres ni le `DETAIL`. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique

- Image d'exécution passée en `gcr.io/distroless/cc-debian13:nonroot`, builder et `development.Dockerfile` en `trixie` (même Debian partout). Test `dockerfile_test.rs` ajouté. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Matrice d'accès `access-matrix.yaml` et test qui vérifie chaque opération de l'OpenAPI, sans changement du comportement de l'API. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Contrôles RGPD en CI : simulation `gdpr-simulation.yaml`, compose et script dédiés. Les workflows passent à CICD v5.3.1. (MAIR-497, [#198](https://github.com/mairie360/ELearning_API/pull/198))
- Workflows alignés sur CICD v5.2.2, avec `ANTHROPIC_WORKSPACE_ID` transmis, et test de cohérence de version. (MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196))
- Checklist RGPD sur les PR via `gdpr-pr.yml`. (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138))
- Passage de CICD v4.4.0 à v4.4.3 dans les workflows de release. (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181))
- Corrections des alertes RustSec de `cargo audit` : zstd, zstd-safe, zstd-sys et ctutils mis à jour, et `RUSTSEC-2026-0387` ignoré. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Mises à jour Renovate : images Docker épinglées par digest (curl, nginx, node, postgres, redis, zap, k6, newman), digests Rust, tokio 1.54.1, actix-web 4.18.1 (diff absent de la PR, à vérifier), et schéma Database / liquibase en 3.3.0 (le contenu des migrations n'apparaît pas dans ce diff). (MAIR-…, [#179](https://github.com/mairie360/ELearning_API/pull/179), [#184](https://github.com/mairie360/ELearning_API/pull/184), [#186](https://github.com/mairie360/ELearning_API/pull/186), [#194](https://github.com/mairie360/ELearning_API/pull/194), [#202](https://github.com/mairie360/ELearning_API/pull/202))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-04 18:19 UTC | dev | — | `37b0f37` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41882887364) |
| 2026-11-04 06:47 UTC | dev | — | `c6a0d58` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41786987311) |
| 2026-11-03 16:14 UTC | dev | — | `9498d6f` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41669638867) |
| 2026-11-03 11:56 UTC | dev | — | `f780ed2` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41630569815) |
| 2026-11-03 01:35 UTC | dev | — | `82404ec` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41559233738) |
| 2026-11-03 00:00 UTC | dev | — | `bae9d89` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41548469175) |
| 2026-11-01 12:53 UTC | dev | — | `b6d6199` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41307600724) |
| 2026-10-22 23:43 UTC | dev | — | `733fcba` | [run](https://github.com/mairie360/ELearning_API/actions/runs/39872754764) |
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
