# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "f10ecb6bd10fbb9ce9b66208da2c80d04a50282c", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-05 02:23 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `f10ecb6` | 2026-11-05 02:23 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- Les jetons de session ne sont plus vérifiés avec un secret partagé : l'API lit `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API) et n'a plus besoin de `JWT_SECRET`. Un jeton HS256 est refusé (401) dès que `JWT_SECRET` n'est pas défini ; les jetons EdDSA signés par Core restent acceptés. Un déploiement doit donc définir `JWT_PUBLIC_KEY` et retirer `JWT_SECRET`. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))

#### Corrections
- Journaux de requêtes : avant, chaque requête journalisée contenait la query string et le `Referer` (des valeurs refusées comme `?limit=` se retrouvaient dans les logs). Maintenant seuls l'adresse client, la méthode, le chemin sans query string, le statut, la taille et la durée sont journalisés. Les erreurs de base de la lib sont décrites par SQLSTATE, contrainte, table et colonne, sans le message Postgres ni le détail. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Test RGPD de bout en bout : simulation d'usage `gdpr-simulation.yaml`, `docker-compose-gdpr-simulation.yml` et `gdpr_simulation_test.sh` (MAIR-497, [#198](https://github.com/mairie360/ELearning_API/pull/198)) ; checklist RGPD sur les PR (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138)).
- Image d'exécution passée en `distroless/cc-debian13`, builder et `development.Dockerfile` en `trixie`, avec le test `dockerfile_test.rs` (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192)).
- Alertes RustSec corrigées (zstd 0.14.3, zstd-safe 8.2.0, zstd-sys 2.1.2, ctutils 0.4.7) et RUSTSEC-2026-0387 ignoré dans `.cargo/audit.toml` (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190)).
- Matrice d'accès `access-matrix.yaml` et test `access_matrix.rs`, sans effet à l'exécution (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142)).
- Lib `mairie360_api_lib` 4.2.0 → 4.3.0 (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200)).
- CICD : passage de v4.4.0 à v4.4.3 (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181)), puis v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` et le test `mair_771_cicd_version.rs` (MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196)), puis v5.3.1 (MAIR-497, [#198](https://github.com/mairie360/ELearning_API/pull/198)).
- Images des stacks de test épinglées par digest (Renovate, [#194](https://github.com/mairie360/ELearning_API/pull/194), [#203](https://github.com/mairie360/ELearning_API/pull/203)), `rust:1.99` mis à jour ([#178](https://github.com/mairie360/ELearning_API/pull/178), [#183](https://github.com/mairie360/ELearning_API/pull/183), [#185](https://github.com/mairie360/ELearning_API/pull/185), [#202](https://github.com/mairie360/ELearning_API/pull/202)), schéma Database 3.2.0 → 3.3.0 ([#179](https://github.com/mairie360/ELearning_API/pull/179)).
- Mises à jour `actix-web` 4.18.1 ([#186](https://github.com/mairie360/ELearning_API/pull/186)) et `tokio` 1.54.1 ([#184](https://github.com/mairie360/ELearning_API/pull/184)) : le diff n'est pas fourni, changement non vérifié.

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les jetons de session ne sont plus vérifiés avec un secret partagé : l'API lit `JWT_PUBLIC_KEY` (clé publique Ed25519 de Core_API) et n'a plus besoin de `JWT_SECRET`. Un jeton HS256 est refusé (401) dès que `JWT_SECRET` n'est pas défini ; les jetons EdDSA signés par Core restent acceptés. Un déploiement doit donc définir `JWT_PUBLIC_KEY` et retirer `JWT_SECRET`. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))

#### Corrections
- Journaux de requêtes : avant, chaque requête journalisée contenait la query string et le `Referer` (des valeurs refusées comme `?limit=` se retrouvaient dans les logs). Maintenant seuls l'adresse client, la méthode, le chemin sans query string, le statut, la taille et la durée sont journalisés. Les erreurs de base de la lib sont décrites par SQLSTATE, contrainte, table et colonne, sans le message Postgres ni le détail. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Test RGPD de bout en bout : simulation d'usage `gdpr-simulation.yaml`, `docker-compose-gdpr-simulation.yml` et `gdpr_simulation_test.sh` (MAIR-497, [#198](https://github.com/mairie360/ELearning_API/pull/198)) ; checklist RGPD sur les PR (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138)).
- Image d'exécution passée en `distroless/cc-debian13`, builder et `development.Dockerfile` en `trixie`, avec le test `dockerfile_test.rs` (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192)).
- Alertes RustSec corrigées (zstd 0.14.3, zstd-safe 8.2.0, zstd-sys 2.1.2, ctutils 0.4.7) et RUSTSEC-2026-0387 ignoré dans `.cargo/audit.toml` (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190)).
- Matrice d'accès `access-matrix.yaml` et test `access_matrix.rs`, sans effet à l'exécution (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142)).
- Lib `mairie360_api_lib` 4.2.0 → 4.3.0 (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200)).
- CICD : passage de v4.4.0 à v4.4.3 (MAIR-612, [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181)), puis v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` et le test `mair_771_cicd_version.rs` (MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196)), puis v5.3.1 (MAIR-497, [#198](https://github.com/mairie360/ELearning_API/pull/198)).
- Images des stacks de test épinglées par digest (Renovate, [#194](https://github.com/mairie360/ELearning_API/pull/194), [#203](https://github.com/mairie360/ELearning_API/pull/203)), `rust:1.99` mis à jour ([#178](https://github.com/mairie360/ELearning_API/pull/178), [#183](https://github.com/mairie360/ELearning_API/pull/183), [#185](https://github.com/mairie360/ELearning_API/pull/185), [#202](https://github.com/mairie360/ELearning_API/pull/202)), schéma Database 3.2.0 → 3.3.0 ([#179](https://github.com/mairie360/ELearning_API/pull/179)).
- Mises à jour `actix-web` 4.18.1 ([#186](https://github.com/mairie360/ELearning_API/pull/186)) et `tokio` 1.54.1 ([#184](https://github.com/mairie360/ELearning_API/pull/184)) : le diff n'est pas fourni, changement non vérifié.

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
