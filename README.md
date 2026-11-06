# Physical : vérification de licence par disque

DLL Windows écrite en Rust qui vérifie la présence d'un disque (ou d'un lecteur) contenant une licence valide avant de laisser un jeu se lancer. Si la licence est absente ou invalide, un message d'erreur est affiché dans la langue de Windows puis le jeu se ferme.

Deux méthodes d'installation sont proposées :

- **Hook** (recommandée) : un `winmm.dll` (ou `version.dll`) à copier à côté du jeu charge automatiquement `Physical.dll`. Aucun outil nécessaire.
- **Import** : ajout manuel de `Physical.dll` dans l'exécutable du jeu avec CFF Explorer.

## Sommaire

- [Contenu du dossier Output](#contenu-du-dossier-output)
- [Fonctionnement](#fonctionnement)
- [Installation avec le Hook](#installation-avec-le-hook-recommandée)
- [Installation avec CFF Explorer](#installation-avec-cff-explorer-import)
- [Configuration](#configuration)
- [Compilation](#compilation)
- [Langues des messages](#langues-des-messages)
- [Messages d'erreur](#messages-derreur)
- [Limitation connue](#limitation-connue)
- [Dépannage](#dépannage)

## Contenu du dossier Output

```
Output/
├── Hook/
│   ├── x64/
│   │   ├── winmm.dll            (renommable en version.dll)
│   │   ├── Physical.dll
│   │   ├── Physical.ini
│   │   └── LicensePhysical.txt
│   └── x86/
│       ├── winmm.dll            (renommable en version.dll)
│       ├── Physical.dll
│       ├── Physical.ini
│       └── LicensePhysical.txt
└── Import/
    ├── x64/
    │   ├── Physical.dll
    │   ├── Physical.ini
    │   └── LicensePhysical.txt
    └── x86/
        ├── Physical.dll
        ├── Physical.ini
        └── LicensePhysical.txt
```

Chaque dossier est autonome : tous les fichiers nécessaires sont fournis.

| Fichier | Rôle | Où le placer |
|---|---|---|
| `winmm.dll` (Hook uniquement) | Charge automatiquement `Physical.dll` au lancement du jeu. | Dossier du jeu, à côté de l'exécutable. |
| `Physical.dll` | Effectue la vérification de la licence. | Dossier du jeu, à côté de l'exécutable. |
| `Physical.ini` | Configuration (vérification activée, nom du jeu, lecteur). | Dossier du jeu, à côté de l'exécutable. |
| `LicensePhysical.txt` | Licence à vérifier. | **Racine du disque** ou de la clé USB à utiliser. |

| Dossier | Usage |
|---|---|
| `Hook` | Installation automatique : le `winmm.dll` charge `Physical.dll`. |
| `Import` | Pour les utilisateurs de **CFF Explorer** qui veulent ajouter `Physical.dll` à la table d'import de l'exécutable. |
| `x64` / `x86` | Choisis le dossier qui correspond à l'architecture du jeu (64 bits ou 32 bits). |

> Le dossier doit correspondre à l'architecture du **jeu**, pas à celle de Windows. Un jeu 32 bits sur un Windows 64 bits utilise `x86`.

## Fonctionnement

1. Au chargement de la DLL, la fonction `init()` s'exécute automatiquement (grâce à `ctor`).
2. Elle désactive les boîtes système « Insérez un disque » (`SetErrorMode`) pour que les lecteurs vides ou absents (A:, B:, lecteur optique vide) ne bloquent pas la vérification.
3. Elle lit `Physical.ini` pour savoir si la vérification est activée, quel est le nom du jeu (`GameName`) et, éventuellement, quel lecteur scanner (`Drive`).
4. Si la vérification est activée, elle cherche `LicensePhysical.txt` à la racine du lecteur indiqué, ou de `A:\` à `Z:\` si aucun lecteur n'est précisé.
5. Elle vérifie que ce fichier contient `license = <GameName>`.
6. En cas d'échec, une boîte de dialogue d'erreur s'affiche et le processus est arrêté (`exit(1)`).

## Installation avec le Hook (recommandée)

1. Repère l'architecture du jeu (64 ou 32 bits). Dans le Gestionnaire des tâches, les processus 32 bits sont suivis de « (32 bits) ».
2. Ouvre `Output/Hook/x64/` ou `Output/Hook/x86/` selon l'architecture du jeu.
3. Copie dans le dossier du jeu, à côté de l'exécutable :
   - `winmm.dll`
   - `Physical.dll`
   - `Physical.ini`
4. Place `LicensePhysical.txt` à la racine du disque ou de la clé USB à utiliser.
5. Adapte `Physical.ini` si besoin (voir [Configuration](#configuration)).
6. Lance le jeu : `winmm.dll` charge `Physical.dll` automatiquement et la vérification s'exécute.

### `winmm.dll` ou `version.dll` ?

Le hook peut être renommé selon la DLL système que le jeu charge :

- Essaie d'abord avec `winmm.dll`.
- Si le jeu ne le charge pas (aucune vérification, aucun message), renomme-le en `version.dll`.
- N'utilise **qu'un seul** des deux noms à la fois.

## Installation avec CFF Explorer (Import)

1. Ouvre `Output/Import/x64/` ou `Output/Import/x86/` selon l'architecture du jeu.
2. Copie dans le dossier du jeu, à côté de l'exécutable :
   - `Physical.dll`
   - `Physical.ini`
3. Fais une copie de sauvegarde de l'exécutable du jeu.
4. Ouvre l'exécutable avec **CFF Explorer** :
   - va dans **Import Directory**,
   - clique droit, puis **Add**, et sélectionne `Physical.dll`,
   - coche la fonction `check_disk`,
   - sauvegarde l'exécutable.
5. Place `LicensePhysical.txt` à la racine du disque ou de la clé USB à utiliser.
6. Adapte `Physical.ini` si besoin (voir [Configuration](#configuration)).
7. Lance le jeu.

La fonction exportée `check_disk` ne fait rien. Elle sert uniquement à pouvoir ajouter la DLL à la table d'import. Le contrôle lui-même s'exécute au chargement de la DLL.

> N'utilise pas le Hook et l'Import en même temps : une seule méthode suffit.

## Configuration

### `Physical.ini`

Fourni dans chaque dossier. À placer à côté de l'exécutable du jeu (voir [Limitation connue](#limitation-connue)).

Scanner tous les lecteurs :

```ini
IsDiskNeeded = true
GameName = REPO
#Drive = E:\
```

Scanner uniquement `E:\` :

```ini
IsDiskNeeded = true
GameName = REPO
Drive = E:\
```

| Clé | Valeur | Description |
|---|---|---|
| `IsDiskNeeded` | `true` / `false` | `false` désactive complètement la vérification. Par défaut : `true`. |
| `GameName` | texte | Nom du jeu attendu dans la licence. Par défaut : `REPO`. |
| `Drive` | `E:\`, `E:`, `E` | Optionnel. Si défini, seul ce lecteur est scanné. Si commenté, absent ou invalide, tous les lecteurs (`A:` à `Z:`) sont scannés. |

Les lignes vides et celles qui commencent par `#` ou `;` sont ignorées. Les clés ne sont pas sensibles à la casse.

À propos de `Drive` :

- Les formats `E:\`, `E:`, `E:/` et `E` sont acceptés, en majuscule ou minuscule.
- Une valeur invalide (par exemple `Drive = test` ou `Drive = E:\Dossier`) est ignorée et tous les lecteurs sont scannés.

### `LicensePhysical.txt`

Fourni dans chaque dossier. À placer à la **racine d'un lecteur** (par exemple `D:\LicensePhysical.txt`, sur le disque ou la clé USB).

```
license = REPO
```

La valeur doit correspondre au `GameName` de `Physical.ini`. La comparaison ignore la casse (`repo` et `REPO` sont acceptés). Pour une comparaison stricte, remplace `eq_ignore_ascii_case(target_game_name)` par `== target_game_name` dans `src/lib.rs`.

Si tu changes `GameName` dans `Physical.ini`, modifie aussi `license` dans `LicensePhysical.txt` pour qu'ils restent identiques.

## Compilation

Prérequis : [Rust](https://rustup.rs/) avec la toolchain MSVC sous Windows.

### Structure du projet

```
physical/
├── Cargo.toml
└── src/
    ├── lib.rs     # logique de vérification
    └── text.rs    # messages d'erreur traduits
```

### Cargo.toml

```toml
[package]
name = "physical"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
ctor = "0.2"
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_System_LibraryLoader",
    "Win32_System_Diagnostics",
    "Win32_System_Diagnostics_Debug",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Globalization",
] }
```

Les features de `windows` sont obligatoires : sans elles, le compilateur renvoie des erreurs du type `cannot find Diagnostics in System`. Les features parent (`Win32_System_Diagnostics`) et enfant (`Win32_System_Diagnostics_Debug`) doivent être activées toutes les deux.

### Commandes

```bash
# 64 bits
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

# 32 bits
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

La DLL générée se trouve dans `target/<cible>/release/physical.dll`. Copie-la ensuite dans les dossiers `x64` ou `x86` correspondants de `Output/Hook/` et `Output/Import/` (renomme-la en `Physical.dll` si besoin).

## Langues des messages

La langue est détectée avec `GetUserDefaultUILanguage` (langue d'affichage de Windows). L'anglais est utilisé par défaut.

| Langue | Code |
|---|---|
| Anglais | `0x09` |
| Français | `0x0C` |
| Allemand | `0x07` |
| Espagnol | `0x0A` |
| Italien | `0x10` |
| Portugais | `0x16` |

### Ajouter une langue

1. Ajoute une variante dans l'enum `Lang` de `src/text.rs`.
2. Ajoute son code dans `detect_language()` (par exemple `0x19` pour le russe, `0x11` pour le japonais).
3. Ajoute un bloc `match` pour cette langue dans `get()`. Le compilateur signale les cas oubliés.

## Messages d'erreur

| Cas | Identifiant | Quand |
|---|---|---|
| Fichier introuvable | `FileNotFound` | `LicensePhysical.txt` n'existe ni sur le lecteur indiqué, ni sur aucun lecteur si `Drive` n'est pas défini. |
| Licence invalide | `LicenseInvalid` | Le fichier existe mais ne contient pas `license = <GameName>`. |
| Erreur de lecture | `ReadError` | Le fichier existe mais ne peut pas être lu. |

## Limitation connue

`GetModuleFileNameW(HMODULE(0), ...)` renvoie le chemin de l'**exécutable** du jeu et non celui de la DLL. `Physical.ini` est donc cherché dans le dossier du `.exe`. Si la DLL est placée ailleurs, il faut utiliser `GetModuleHandleExW` avec le flag `GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS` sur l'adresse de `check_disk`.

## Dépannage

| Problème | Piste |
|---|---|
| Aucune vérification avec le Hook | Renomme `winmm.dll` en `version.dll` (un seul des deux à la fois) et vérifie que `Physical.dll` est dans le même dossier. |
| Le jeu ne démarre pas du tout | Vérifie que les DLL ont la même architecture (x64 / x86) que le jeu. |
| « Licence introuvable » alors que le disque est inséré | Vérifie que `LicensePhysical.txt` est bien **à la racine** du lecteur, et que `Drive` dans `Physical.ini` pointe vers le bon lecteur (ou est commenté). |
| « Licence introuvable » alors que le fichier existe | Vérifie que `license` correspond bien au `GameName` de `Physical.ini`. |
| La vérification ne se fait pas | Vérifie que `IsDiskNeeded` n'est pas à `false` et que `Physical.ini` est à côté de l'exécutable. |
| Message en anglais alors que Windows est dans une autre langue | La langue n'est pas encore gérée dans `text.rs`, voir [Ajouter une langue](#ajouter-une-langue). |
| Antivirus qui bloque `winmm.dll` | Les DLL proxy sont parfois détectées à tort. Ajoute une exclusion pour le dossier du jeu. |
| Erreur `cannot find Diagnostics in System` à la compilation | Ajoute `Win32_System_Diagnostics` et `Win32_System_Diagnostics_Debug` aux features de `windows` dans `Cargo.toml`. |