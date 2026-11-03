# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "82404ec9ace4d188b68ceb0cb3e362ddac600707", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-03 01:35 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `82404ec` | 2026-11-03 01:35 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- Les API n'acceptent plus les jetons de session HS256 signés avec l'ancien secret partagé : ils reçoivent un `401`. Les sessions doivent être des jetons EdDSA signés par Core_API et vérifiés avec `JWT_PUBLIC_KEY`. Tout appelant qui utilise encore des jetons HS256 est refusé. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Déploiement : `JWT_SECRET` ne doit plus être défini et `JWT_PUBLIC_KEY` (clé publique PEM de Core_API) devient requise. Tant que `JWT_SECRET` est défini, les jetons HS256 restent acceptés. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Les journaux de requêtes n'affichent plus la query string ni le `Referer` : seuls l'adresse client, la méthode, le chemin sans query, le statut, la taille et la durée sont tracés. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Déploiement et CI : pins de `mairie360/CICD` passés de v4.4.0 à v5.2.2 pour les workflows, avec `ANTHROPIC_WORKSPACE_ID` et un test de cohérence des versions (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181), MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196)) ; checklist RGPD sur les PR (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138)) ; image d'exécution passée en distroless Debian 13 (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192)) ; matrice d'accès `access-matrix.yaml` vérifiée par les tests, sans changement de comportement (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142)).
- Dépendances : correctifs RustSec de `zstd` et `ctutils`, une exception `RUSTSEC-2026-0387` dans `cargo audit` (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190)) ; mises à jour Renovate des images `rust` (digest), de `tokio` 1.54.1 et des images `database` et `liquibase-migrations` en 3.3.0 (#178, #179, #183, #184, #185).

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les API n'acceptent plus les jetons de session HS256 signés avec l'ancien secret partagé : ils reçoivent un `401`. Les sessions doivent être des jetons EdDSA signés par Core_API et vérifiés avec `JWT_PUBLIC_KEY`. Tout appelant qui utilise encore des jetons HS256 est refusé. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Déploiement : `JWT_SECRET` ne doit plus être défini et `JWT_PUBLIC_KEY` (clé publique PEM de Core_API) devient requise. Tant que `JWT_SECRET` est défini, les jetons HS256 restent acceptés. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Les journaux de requêtes n'affichent plus la query string ni le `Referer` : seuls l'adresse client, la méthode, le chemin sans query, le statut, la taille et la durée sont tracés. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Déploiement et CI : pins de `mairie360/CICD` passés de v4.4.0 à v5.2.2 pour les workflows, avec `ANTHROPIC_WORKSPACE_ID` et un test de cohérence des versions (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181), MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196)) ; checklist RGPD sur les PR (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138)) ; image d'exécution passée en distroless Debian 13 (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192)) ; matrice d'accès `access-matrix.yaml` vérifiée par les tests, sans changement de comportement (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142)).
- Dépendances : correctifs RustSec de `zstd` et `ctutils`, une exception `RUSTSEC-2026-0387` dans `cargo audit` (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190)) ; mises à jour Renovate des images `rust` (digest), de `tokio` 1.54.1 et des images `database` et `liquibase-migrations` en 3.3.0 (#178, #179, #183, #184, #185).

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
