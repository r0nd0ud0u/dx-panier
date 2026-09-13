# Construire l'APK en local

La CI construit déjà l'APK à chaque tag (voir le README). Ce document sert
uniquement à installer la chaîne Android sur sa propre machine, pour itérer sur le
téléphone sans passer par GitHub.

## Ce qu'il faut

| Composant | Version | Pourquoi |
| --- | --- | --- |
| Android SDK platform | `android-34` | cible de compilation |
| Android SDK build-tools | `34.0.0` | `aapt2`, `zipalign`, `apksigner` |
| Android NDK | `r27` | compilation Rust vers `aarch64-linux-android` |
| Cible Rust | `aarch64-linux-android` | `rustup target add aarch64-linux-android` |
| ImageMagick | — | redimensionne l'icône dans `patch_android_icon.sh` |

Le NDK r27 est celui qu'utilise la CI. Un NDK nettement plus récent change
régulièrement l'emplacement de `libclang_rt.builtins`, ce que `dx` ne suit pas
toujours ; en cas d'erreur d'édition de liens, c'est la première chose à aligner.

## Installation

Sans Android Studio, avec les seuls outils en ligne de commande :

```bash
# 1. cmdline-tools
mkdir -p ~/Android/sdk/cmdline-tools
# télécharger commandlinetools-linux-*.zip depuis developer.android.com/studio
unzip commandlinetools-linux-*.zip -d ~/Android/sdk/cmdline-tools
mv ~/Android/sdk/cmdline-tools/cmdline-tools ~/Android/sdk/cmdline-tools/latest

# 2. variables d'environnement (à mettre dans ~/.bashrc)
export ANDROID_HOME="$HOME/Android/sdk"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"

# 3. composants du SDK
sdkmanager --licenses
sdkmanager "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;27.2.12479018"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"

# 4. cible Rust
rustup target add aarch64-linux-android
```

Sur Fedora, ImageMagick s'installe avec `sudo dnf install ImageMagick`.

## Itérer sur un téléphone branché

Activer les options développeur et le débogage USB sur le téléphone, puis :

```bash
adb devices                 # le téléphone doit apparaître comme "device", pas "unauthorized"
./scripts/dev_android.sh    # compile, installe et lance, avec hot reload
```

Sur un émulateur, `./scripts/dev_emulator.sh <nom-avd>` le démarre d'abord
(`emulator -list-avds` pour la liste). Un émulateur x86_64 a besoin de
`rustup target add x86_64-linux-android`.

## Produire l'APK de release

```bash
./scripts/bundle_mobile.sh aarch64-linux-android
./scripts/patch_android_icon.sh
adb install -r bundle-android/*.apk
```

`patch_android_icon.sh` est nécessaire parce que dioxus-cli 0.7.10 code en dur
l'icône de lanceur d'Android Studio et n'expose aucun réglage pour la changer : le
script remplace les ressources d'icône dans l'APK déjà construit, puis le re-signe.
L'en-tête du script détaille chaque étape.

## Pannes courantes

**`INSTALL_FAILED_UPDATE_INCOMPATIBLE` / « package conflicts with an existing
package »** — la nouvelle version est signée avec une autre clé que celle déjà
installée. Vérifier que `Dioxus.toml` pointe toujours sur
`android/debug.keystore`, ou désinstaller l'ancienne version — ce qui **efface
l'historique**, donc exporter d'abord depuis les réglages.

**`Only 64-bit Android targets are supported`** — la cible `armv7-linux-androideabi`
est demandée. Dioxus 0.7 ne la gère pas ; utiliser `aarch64-linux-android`.

**Erreur `glib-2.0` via pkg-config** — `dx bundle --platform android` compile aussi
un binaire pour la machine hôte, qui a besoin des dépendances desktop. Sur Fedora :
`sudo dnf install webkit2gtk4.1-devel gtk3-devel libxdo-devel`.

**L'application se lance mais perd ses données à chaque redémarrage** — le chemin
Android codé en dur dans `src/storage.rs` ne correspond plus à `[bundle].identifier`
dans `Dioxus.toml`. Les deux doivent rester identiques.
