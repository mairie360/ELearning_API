# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "60cf5c1f3576cf9f159c7eb9317846f02d37bba5", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-05 23:54 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `60cf5c1` | 2026-11-05 23:54 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

- chore(deps): update rust:1.99-trixie docker digest to bc2c4c2 (#205) ([#205](https://github.com/mairie360/ELearning_API/pull/205))
- chore(deps): pin node.js to c86ecea (#203) ([#203](https://github.com/mairie360/ELearning_API/pull/203))
- feat: MAIR-497 GDPR - Usage simulation and AI review of logs and database before prod ([#198](https://github.com/mairie360/ELearning_API/pull/198))
- chore(deps): update rust:1.99-trixie docker digest to 4c4c8ec (#202) ([#202](https://github.com/mairie360/ELearning_API/pull/202))
- chore(deps): update rust crate actix-web to v4.18.1 (#186) ([#186](https://github.com/mairie360/ELearning_API/pull/186))
- chore(deps): pin dependencies (#194) ([#194](https://github.com/mairie360/ELearning_API/pull/194))
- fix: MAIR-680 [Audit 23/10] Toutes les APIs et BFFs : remplacer le secret JWT HS256 partagé par 12 services (A1) ([#200](https://github.com/mairie360/ELearning_API/pull/200))
- ci: MAIR-771 APIs, BFFs, Database : monter CICD en v5.2.2 pour activer les contrôles RGPD (#196) ([#196](https://github.com/mairie360/ELearning_API/pull/196))
- fix: MAIR-741 APIs : passer l'image d'exécution en distroless Debian 13 (CVE-2026-84782) (#192) ([#192](https://github.com/mairie360/ELearning_API/pull/192))
- fix(deps): MAIR-740 APIs : corriger les alertes RustSec de cargo audit (crypto-bigint, ctutils, zstd) (#190) ([#190](https://github.com/mairie360/ELearning_API/pull/190))
- feat: MAIR-288 Définir la matrice d'accès des APIs et la vérifier depuis l'OpenAPI (#142) ([#142](https://github.com/mairie360/ELearning_API/pull/142))
- fix: MAIR-290 Détecter les données personnelles dans les journaux avec un test du marqueur (#134) ([#134](https://github.com/mairie360/ELearning_API/pull/134))
- ci: MAIR-295 Ajouter une checklist RGPD aux PR et des rappels d'échéances automatiques (#138) ([#138](https://github.com/mairie360/ELearning_API/pull/138))
- chore(deps): update rust:1.99-bookworm docker digest to 48b7f43 (#185) ([#185](https://github.com/mairie360/ELearning_API/pull/185))
- chore(deps): update rust crate tokio to v1.54.1 (#184) ([#184](https://github.com/mairie360/ELearning_API/pull/184))
- chore(deps): update rust:1.99-bookworm docker digest to edaa2a8 (#183) ([#183](https://github.com/mairie360/ELearning_API/pull/183))
- ci: MAIR-612 Release prod: align internal versions and deploy (#181) ([#181](https://github.com/mairie360/ELearning_API/pull/181))
- chore(deps): update mairie360 database schema to v3.3.0 (#179) ([#179](https://github.com/mairie360/ELearning_API/pull/179))
- ci: MAIR-612 Release prod: align internal versions and deploy (#177) ([#177](https://github.com/mairie360/ELearning_API/pull/177))
- chore(deps): update rust:1.99-bookworm docker digest to 5c1a173 (#178) ([#178](https://github.com/mairie360/ELearning_API/pull/178))
- ci: MAIR-612 Release prod: align internal versions and deploy (#175) ([#175](https://github.com/mairie360/ELearning_API/pull/175))

## Différence dev → staging (en dev, pas encore en staging)

- chore(deps): update rust:1.99-trixie docker digest to bc2c4c2 (#205) ([#205](https://github.com/mairie360/ELearning_API/pull/205))
- chore(deps): pin node.js to c86ecea (#203) ([#203](https://github.com/mairie360/ELearning_API/pull/203))
- feat: MAIR-497 GDPR - Usage simulation and AI review of logs and database before prod ([#198](https://github.com/mairie360/ELearning_API/pull/198))
- chore(deps): update rust:1.99-trixie docker digest to 4c4c8ec (#202) ([#202](https://github.com/mairie360/ELearning_API/pull/202))
- chore(deps): update rust crate actix-web to v4.18.1 (#186) ([#186](https://github.com/mairie360/ELearning_API/pull/186))
- chore(deps): pin dependencies (#194) ([#194](https://github.com/mairie360/ELearning_API/pull/194))
- fix: MAIR-680 [Audit 23/10] Toutes les APIs et BFFs : remplacer le secret JWT HS256 partagé par 12 services (A1) ([#200](https://github.com/mairie360/ELearning_API/pull/200))
- ci: MAIR-771 APIs, BFFs, Database : monter CICD en v5.2.2 pour activer les contrôles RGPD (#196) ([#196](https://github.com/mairie360/ELearning_API/pull/196))
- fix: MAIR-741 APIs : passer l'image d'exécution en distroless Debian 13 (CVE-2026-84782) (#192) ([#192](https://github.com/mairie360/ELearning_API/pull/192))
- fix(deps): MAIR-740 APIs : corriger les alertes RustSec de cargo audit (crypto-bigint, ctutils, zstd) (#190) ([#190](https://github.com/mairie360/ELearning_API/pull/190))
- feat: MAIR-288 Définir la matrice d'accès des APIs et la vérifier depuis l'OpenAPI (#142) ([#142](https://github.com/mairie360/ELearning_API/pull/142))
- fix: MAIR-290 Détecter les données personnelles dans les journaux avec un test du marqueur (#134) ([#134](https://github.com/mairie360/ELearning_API/pull/134))
- ci: MAIR-295 Ajouter une checklist RGPD aux PR et des rappels d'échéances automatiques (#138) ([#138](https://github.com/mairie360/ELearning_API/pull/138))
- chore(deps): update rust:1.99-bookworm docker digest to 48b7f43 (#185) ([#185](https://github.com/mairie360/ELearning_API/pull/185))
- chore(deps): update rust crate tokio to v1.54.1 (#184) ([#184](https://github.com/mairie360/ELearning_API/pull/184))
- chore(deps): update rust:1.99-bookworm docker digest to edaa2a8 (#183) ([#183](https://github.com/mairie360/ELearning_API/pull/183))
- ci: MAIR-612 Release prod: align internal versions and deploy (#181) ([#181](https://github.com/mairie360/ELearning_API/pull/181))
- chore(deps): update mairie360 database schema to v3.3.0 (#179) ([#179](https://github.com/mairie360/ELearning_API/pull/179))
- ci: MAIR-612 Release prod: align internal versions and deploy (#177) ([#177](https://github.com/mairie360/ELearning_API/pull/177))
- chore(deps): update rust:1.99-bookworm docker digest to 5c1a173 (#178) ([#178](https://github.com/mairie360/ELearning_API/pull/178))
- ci: MAIR-612 Release prod: align internal versions and deploy (#175) ([#175](https://github.com/mairie360/ELearning_API/pull/175))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

> Résumé Claude indisponible pour ce passage : listes brutes des commits.

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-05 23:54 UTC | dev | — | `60cf5c1` | [run](https://github.com/mairie360/ELearning_API/actions/runs/42132075145) |
| 2026-11-05 02:23 UTC | dev | — | `f10ecb6` | [run](https://github.com/mairie360/ELearning_API/actions/runs/41954027223) |
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
