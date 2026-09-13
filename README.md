# Panier

Suivi du prix des courses, produit par produit et magasin par magasin. On saisit
ce qu'on a payé en sortant du magasin ; l'app répond à la question « où est-ce que
je dois acheter quoi ».

Tout est stocké **sur l'appareil**. Il n'y a pas de serveur, pas de compte, pas de
réseau : aucune ligne de code de cette application n'ouvre une connexion.

Écrit en Rust avec [Dioxus](https://dioxuslabs.com/) ; la même base de code produit
un APK Android, un exécutable desktop (Linux, Windows) et un site statique.

## Ce que ça fait

- **Ajouter** un achat : produit, magasin, prix payé, quantité, unité, date. Les
  noms déjà saisis sont proposés en autocomplétion ; le magasin et la date restent
  d'une ligne à l'autre, parce qu'un ticket de caisse, c'est dix produits dans un
  seul magasin le même jour.
- **Prix unitaire** : tout est ramené au prix par kilo, par litre ou par pièce.
  C'est ce qui rend comparables une plaquette de 250 g et un pain de 500 g. Les
  quantités se saisissent en g ou ml si c'est ce qui est écrit sur l'emballage —
  la conversion est faite à l'enregistrement.
- **Produits** : la liste de tout ce qui a été acheté, avec le dernier prix, la
  tendance, et le magasin le moins cher. Filtrable par magasin.
- **Détail d'un produit** : la courbe du prix unitaire dans le temps, le
  comparatif par magasin, et l'historique complet.
- **Où acheter** : chaque produit rangé sous le magasin qui le vend le moins cher
  en moyenne, avec l'écart par unité.
- **Réglages** : français/anglais, sauvegarde et restauration en JSON, purge.

## Développement

```bash
./scripts/dev_web.sh        # navigateur, avec hot reload
./scripts/dev_desktop.sh    # fenêtre native
./scripts/dev_android.sh    # téléphone ou émulateur branché en USB
```

`dev_web.sh` écoute sur `0.0.0.0`, donc le téléphone peut aussi simplement ouvrir
`http://<ip-de-la-machine>:8080` sans rien installer.

Prérequis : Rust stable et `dioxus-cli` 0.7.10 (`cargo binstall dioxus-cli@0.7.10`).
Pour Android, voir [docs/android-local.md](docs/android-local.md).

## Vérifications

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Les tests couvrent le calcul : prix unitaire, conversions d'unité, classement des
magasins, tendances, filtres, et la parité des deux fichiers de traduction.

## Installer l'APK sur son téléphone

Pousser un tag déclenche la CI, qui construit et attache les binaires à la release
GitHub :

```bash
git tag v0.1.0 && git push origin v0.1.0
```

L'APK apparaît en asset de la release sous le nom `panier-v0.1.0-arm64-v8a.apk`.
Il se télécharge directement depuis le téléphone ; Android demandera d'autoriser
l'installation depuis cette source.

L'APK est signé avec `android/debug.keystore`, qui est versionné **exprès** : sans
clé stable, Android refuse d'installer une nouvelle version par-dessus l'ancienne.
Ce n'est pas un secret — c'est la clé de debug standard, que n'importe qui peut
regénérer. Elle ne convient pas pour une publication sur le Play Store.

Pour construire l'APK en local plutôt qu'en CI, voir
[docs/android-local.md](docs/android-local.md).

## Où sont les données

| Plateforme | Emplacement |
| --- | --- |
| Navigateur | `localStorage`, clé `panier-db-v1` |
| Linux | `~/.local/share/panier/panier-db-v1` |
| Windows | `%LOCALAPPDATA%\panier\panier-db-v1` |
| Android | `/data/data/io.github.r0ndoudou.panier/files/panier/panier-db-v1` |

Ces emplacements ne communiquent pas entre eux : un téléphone et un navigateur ont
chacun leur propre historique. Les réglages permettent d'exporter en JSON depuis
l'un et de restaurer dans l'autre.

Désinstaller l'application efface ses données. **Exporter avant.**

## Structure

```
src/model.rs     tout le domaine : achats, prix unitaires, comparaisons (et leurs tests)
src/storage.rs   persistance locale, une seule clé JSON
src/common.rs    routes et constantes du premier rendu
src/pages/       une page par onglet, plus les widgets partagés
scripts/         lancement et packaging pour les trois plateformes
```

## Licence

Apache-2.0.
