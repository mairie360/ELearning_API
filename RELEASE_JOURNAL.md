# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "b0ea282dad20131eb1c2ac3c2811c18aa0badb36", "prod": "7f790d5b0bf655737514d390926fd09292020b58", "prod_tag": "v1.3.0", "staging": "7f790d5b0bf655737514d390926fd09292020b58", "updated": {"dev": "2026-11-18 14:35 UTC", "prod": "2026-11-12 23:53 UTC", "staging": "2026-11-12 18:51 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.3.0 | `7f790d5` | 2026-11-12 23:53 UTC |
| staging | — | `7f790d5` | 2026-11-12 18:51 UTC |
| dev | — | `b0ea282` | 2026-11-18 14:35 UTC |

## Ajouté sur main depuis la dernière release prod (v1.3.0)

- chore: MAIR-1083 Set the Codecov target to 60 % on the code repositories ([#238](https://github.com/mairie360/ELearning_API/pull/238))
- test: MAIR-534 Add the missing business negative cases of the APIs ([#236](https://github.com/mairie360/ELearning_API/pull/236))
- test: MAIR-533 Remove or reactivate the dead tests of the APIs ([#234](https://github.com/mairie360/ELearning_API/pull/234))
- feat: MAIR-998 Test stacks follow the moving upstream tag of the stage (#230) ([#230](https://github.com/mairie360/ELearning_API/pull/230))
- chore(deps): pin actions/checkout action to 3d3c42e (#232) ([#232](https://github.com/mairie360/ELearning_API/pull/232))
- ci: MAIR-1008 Nightly rehearsal of the promotion gates (#228) ([#228](https://github.com/mairie360/ELearning_API/pull/228))
- test: MAIR-1002 Speed up the APIs' CI tests (Rust cache, one database per test, optimized dependencies) ([#231](https://github.com/mairie360/ELearning_API/pull/231))
- chore(deps): update rust crate mairie360_api_lib to v4.4.0 (#210) ([#210](https://github.com/mairie360/ELearning_API/pull/210))
- fix: MAIR-247 Translate the APIs' OpenAPI descriptions and messages into English (#222) ([#222](https://github.com/mairie360/ELearning_API/pull/222))
- chore(deps): MAIR-243 Bump quinn-udp to the patched 0.5.16 (RUSTSEC-2026-0538) (#224) ([#224](https://github.com/mairie360/ELearning_API/pull/224))
- docs: MAIR-245 Rewrite the APIs' README and API.md (#212) ([#212](https://github.com/mairie360/ELearning_API/pull/212))
- fix: MAIR-243 Align the test Database version and validate the enrolment body (#217) ([#217](https://github.com/mairie360/ELearning_API/pull/217))
- chore(deps): update mairie360 database schema to v3.4.0 (#219) ([#219](https://github.com/mairie360/ELearning_API/pull/219))

## Différence dev → staging (en dev, pas encore en staging)

- chore: MAIR-1083 Set the Codecov target to 60 % on the code repositories ([#238](https://github.com/mairie360/ELearning_API/pull/238))
- test: MAIR-534 Add the missing business negative cases of the APIs ([#236](https://github.com/mairie360/ELearning_API/pull/236))
- test: MAIR-533 Remove or reactivate the dead tests of the APIs ([#234](https://github.com/mairie360/ELearning_API/pull/234))
- feat: MAIR-998 Test stacks follow the moving upstream tag of the stage (#230) ([#230](https://github.com/mairie360/ELearning_API/pull/230))
- chore(deps): pin actions/checkout action to 3d3c42e (#232) ([#232](https://github.com/mairie360/ELearning_API/pull/232))
- ci: MAIR-1008 Nightly rehearsal of the promotion gates (#228) ([#228](https://github.com/mairie360/ELearning_API/pull/228))
- test: MAIR-1002 Speed up the APIs' CI tests (Rust cache, one database per test, optimized dependencies) ([#231](https://github.com/mairie360/ELearning_API/pull/231))
- chore(deps): update rust crate mairie360_api_lib to v4.4.0 (#210) ([#210](https://github.com/mairie360/ELearning_API/pull/210))
- fix: MAIR-247 Translate the APIs' OpenAPI descriptions and messages into English (#222) ([#222](https://github.com/mairie360/ELearning_API/pull/222))
- chore(deps): MAIR-243 Bump quinn-udp to the patched 0.5.16 (RUSTSEC-2026-0538) (#224) ([#224](https://github.com/mairie360/ELearning_API/pull/224))
- docs: MAIR-245 Rewrite the APIs' README and API.md (#212) ([#212](https://github.com/mairie360/ELearning_API/pull/212))
- fix: MAIR-243 Align the test Database version and validate the enrolment body (#217) ([#217](https://github.com/mairie360/ELearning_API/pull/217))
- chore(deps): update mairie360 database schema to v3.4.0 (#219) ([#219](https://github.com/mairie360/ELearning_API/pull/219))

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

> Résumé Claude indisponible pour ce passage : listes brutes des commits.

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
| 2026-11-18 14:35 UTC | dev | — | `b0ea282` | [run](https://github.com/mairie360/ELearning_API/actions/runs/44323167807) |
| 2026-11-17 17:10 UTC | dev | — | `4376f90` | [run](https://github.com/mairie360/ELearning_API/actions/runs/44131711029) |
| 2026-11-17 17:10 UTC | dev | — | `6356d27` | [run](https://github.com/mairie360/ELearning_API/actions/runs/44131696130) |
| 2026-11-17 15:05 UTC | dev | — | `cec6a57` | [run](https://github.com/mairie360/ELearning_API/actions/runs/44107658605) |
| 2026-11-17 11:49 UTC | dev | — | `1511e02` | [run](https://github.com/mairie360/ELearning_API/actions/runs/44075420913) |
| 2026-11-17 00:15 UTC | dev | — | `ea592f9` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43981574109) |
| 2026-11-16 23:07 UTC | dev | — | `db5a940` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43974696052) |
| 2026-11-16 17:24 UTC | dev | — | `54a3128` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43916567791) |
| 2026-11-16 16:52 UTC | dev | — | `87daa38` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43909728844) |
| 2026-11-16 16:29 UTC | dev | — | `15d53c6` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43905412758) |
| 2026-11-13 06:34 UTC | dev | — | `953c8f2` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43377187160) |
| 2026-11-12 23:53 UTC | prod | v1.3.0 | `7f790d5` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43330565399) |
| 2026-11-12 18:51 UTC | staging | — | `7f790d5` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43282726286) |
| 2026-11-12 12:57 UTC | dev | — | `7f790d5` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43220719687) |
| 2026-11-12 11:51 UTC | dev | — | `030df46` | [run](https://github.com/mairie360/ELearning_API/actions/runs/43211645590) |
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
