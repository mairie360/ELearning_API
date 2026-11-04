# Journal des releases — ELearning_API

<!-- release-journal-state {"dev": "c6a0d5855228a73283065267b41ee0436d8679ae", "prod": "12c5977817c325cfd8971f2b078d158c35a0157b", "prod_tag": "v1.2.0", "staging": "12c5977817c325cfd8971f2b078d158c35a0157b", "updated": {"dev": "2026-11-04 06:47 UTC", "prod": "2026-10-21 09:11 UTC", "staging": "2026-10-21 03:21 UTC"}} -->
> Tenu à jour par la CI (`mairie360/CICD`, action `release/release-journal`) à chaque passage en dev, staging et prod. Ne pas modifier à la main : le fichier est réécrit au passage suivant.

| Environnement | Version | Commit | Mis à jour |
| --- | --- | --- | --- |
| prod | v1.2.0 | `12c5977` | 2026-10-21 09:11 UTC |
| staging | — | `12c5977` | 2026-10-21 03:21 UTC |
| dev | — | `c6a0d58` | 2026-11-04 06:47 UTC |

## Ajouté sur main depuis la dernière release prod (v1.2.0)

#### Changements de comportement
- Les tokens HS256 sont désormais refusés avec un 401 : l'API ne vérifie plus les sessions qu'avec la clé publique Ed25519 de Core (`JWT_PUBLIC_KEY`) et n'utilise plus `JWT_SECRET`. Un appelant qui signe ses JWT avec un secret partagé doit passer à un jeton EdDSA émis par Core. Tant que `JWT_SECRET` est défini, la lib accepte encore HS256 (transition) : il faut le retirer une fois `JWT_PUBLIC_KEY` en place. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- La stack de dev prend par défaut la clé publique de test de la lib, avec `JWT_ALLOW_WEAK_SECRET=true` (dev uniquement). (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Les journaux de requêtes n'affichent plus la query string ni le `Referer`, seulement l'adresse client, la méthode, le chemin, le statut, la taille et la durée. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Mise à jour de `zstd`, `zstd-safe`, `zstd-sys` et `ctutils`, et ignorance de RUSTSEC-2026-0387 (`crypto-bigint`) dans `.cargo/audit.toml` pour que `cargo audit` passe. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Ajout de `access-matrix.yaml` et d'un test qui vérifie chaque opération de l'OpenAPI, sans changement de réponse de l'API. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Image d'exécution passée sur distroless Debian 13 et builders sur `trixie`, avec un test de garde sur les `FROM`. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Checklist RGPD sur les pull requests (`gdpr-pr.yml`) et test du marqueur RGPD dans les journaux de la stack. (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138) ; MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))
- CICD monté en v4.4.1, v4.4.2 puis v4.4.3 pour les releases prod, puis en v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` transmis aux workflows. (MAIR-612 [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181) ; MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196))
- Images des compose épinglées par digest (redis, postgres, curl, k6, newman, nginx, node, zap, rust) et version de la base `database` / `liquibase-migrations` passée de 3.2.0 à 3.3.0 (le diff ne montre que la version, pas le contenu des migrations). (Renovate, [#179](https://github.com/mairie360/ELearning_API/pull/179), [#183](https://github.com/mairie360/ELearning_API/pull/183), [#185](https://github.com/mairie360/ELearning_API/pull/185), [#194](https://github.com/mairie360/ELearning_API/pull/194), [#202](https://github.com/mairie360/ELearning_API/pull/202))
- Mises à jour de dépendances : `actix-web` 4.18.1 ([#186](https://github.com/mairie360/ELearning_API/pull/186)) et `tokio` 1.54.1 ([#184](https://github.com/mairie360/ELearning_API/pull/184)). Le diff de certaines PR (MAIR-680, MAIR-290, MAIR-288) est tronqué : le détail ci-dessus repose sur les fichiers visibles.

## Différence dev → staging (en dev, pas encore en staging)

#### Changements de comportement
- Les tokens HS256 sont désormais refusés avec un 401 : l'API ne vérifie plus les sessions qu'avec la clé publique Ed25519 de Core (`JWT_PUBLIC_KEY`) et n'utilise plus `JWT_SECRET`. Un appelant qui signe ses JWT avec un secret partagé doit passer à un jeton EdDSA émis par Core. Tant que `JWT_SECRET` est défini, la lib accepte encore HS256 (transition) : il faut le retirer une fois `JWT_PUBLIC_KEY` en place. (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- La stack de dev prend par défaut la clé publique de test de la lib, avec `JWT_ALLOW_WEAK_SECRET=true` (dev uniquement). (MAIR-680, [#200](https://github.com/mairie360/ELearning_API/pull/200))
- Les journaux de requêtes n'affichent plus la query string ni le `Referer`, seulement l'adresse client, la méthode, le chemin, le statut, la taille et la durée. (MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))

#### Technique
- Mise à jour de `zstd`, `zstd-safe`, `zstd-sys` et `ctutils`, et ignorance de RUSTSEC-2026-0387 (`crypto-bigint`) dans `.cargo/audit.toml` pour que `cargo audit` passe. (MAIR-740, [#190](https://github.com/mairie360/ELearning_API/pull/190))
- Ajout de `access-matrix.yaml` et d'un test qui vérifie chaque opération de l'OpenAPI, sans changement de réponse de l'API. (MAIR-288, [#142](https://github.com/mairie360/ELearning_API/pull/142))
- Image d'exécution passée sur distroless Debian 13 et builders sur `trixie`, avec un test de garde sur les `FROM`. (MAIR-741, [#192](https://github.com/mairie360/ELearning_API/pull/192))
- Checklist RGPD sur les pull requests (`gdpr-pr.yml`) et test du marqueur RGPD dans les journaux de la stack. (MAIR-295, [#138](https://github.com/mairie360/ELearning_API/pull/138) ; MAIR-290, [#134](https://github.com/mairie360/ELearning_API/pull/134))
- CICD monté en v4.4.1, v4.4.2 puis v4.4.3 pour les releases prod, puis en v5.2.2 avec `ANTHROPIC_WORKSPACE_ID` transmis aux workflows. (MAIR-612 [#175](https://github.com/mairie360/ELearning_API/pull/175), [#177](https://github.com/mairie360/ELearning_API/pull/177), [#181](https://github.com/mairie360/ELearning_API/pull/181) ; MAIR-771, [#196](https://github.com/mairie360/ELearning_API/pull/196))
- Images des compose épinglées par digest (redis, postgres, curl, k6, newman, nginx, node, zap, rust) et version de la base `database` / `liquibase-migrations` passée de 3.2.0 à 3.3.0 (le diff ne montre que la version, pas le contenu des migrations). (Renovate, [#179](https://github.com/mairie360/ELearning_API/pull/179), [#183](https://github.com/mairie360/ELearning_API/pull/183), [#185](https://github.com/mairie360/ELearning_API/pull/185), [#194](https://github.com/mairie360/ELearning_API/pull/194), [#202](https://github.com/mairie360/ELearning_API/pull/202))
- Mises à jour de dépendances : `actix-web` 4.18.1 ([#186](https://github.com/mairie360/ELearning_API/pull/186)) et `tokio` 1.54.1 ([#184](https://github.com/mairie360/ELearning_API/pull/184)). Le diff de certaines PR (MAIR-680, MAIR-290, MAIR-288) est tronqué : le détail ci-dessus repose sur les fichiers visibles.

## Différence staging → prod (en staging, pas encore en prod)

_Rien._

## Historique des passages

<!-- release-journal-history:start -->
| Date | Environnement | Version | Commit | Run |
| --- | --- | --- | --- | --- |
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
