# Physical Never Dies: disc license check

A Windows DLL written in Rust that checks for a disc (or drive) containing a valid license before letting a game start. If the license is missing or invalid, an error message is shown in the Windows display language and the game closes.

Two installation methods are provided:

- **Hook** (recommended): a `winmm.dll` (or `version.dll`) placed next to the game automatically loads `PhysicalNeverDies.dll`. No extra tools needed.
- **Import**: manually add `PhysicalNeverDies.dll` to the game's executable using CFF Explorer.

## Table of contents

- [Output folder contents](#output-folder-contents)
- [How it works](#how-it-works)
- [Installation with the Hook](#installation-with-the-hook-recommended)
- [Installation with CFF Explorer](#installation-with-cff-explorer-import)
- [Configuration](#configuration)
- [Building](#building)
- [Message languages](#message-languages)
- [Error messages](#error-messages)
- [Known limitation](#known-limitation)
- [Troubleshooting](#troubleshooting)

## Output folder contents

```
Output/
├── Hook/
│   ├── x64/
│   │   ├── winmm.dll            (can be renamed to version.dll)
│   │   ├── PhysicalNeverDies.dll
│   │   ├── PhysicalNeverDies.ini
│   │   └── LicensePhysical.txt
│   └── x86/
│       ├── winmm.dll            (can be renamed to version.dll)
│       ├── PhysicalNeverDies.dll
│       ├── PhysicalNeverDies.ini
│       └── LicensePhysical.txt
└── Import/
    ├── x64/
    │   ├── PhysicalNeverDies.dll
    │   ├── PhysicalNeverDies.ini
    │   └── LicensePhysical.txt
    └── x86/
        ├── PhysicalNeverDies.dll
        ├── PhysicalNeverDies.ini
        └── LicensePhysical.txt
```

Each folder is self-contained: all required files are included.

| File | Purpose | Where to put it |
|---|---|---|
| `winmm.dll` (Hook only) | Automatically loads `PhysicalNeverDies.dll` when the game starts. | Game folder, next to the executable. |
| `PhysicalNeverDies.dll` | Performs the license check. | Game folder, next to the executable. |
| `PhysicalNeverDies.ini` | Configuration (check enabled, game name, drive). | Game folder, next to the executable. |
| `LicensePhysical.txt` | The license to verify. | **Root of the disc** or USB drive to use. |

| Folder | Usage |
|---|---|
| `Hook` | Automatic installation: `winmm.dll` loads `PhysicalNeverDies.dll`. |
| `Import` | For **CFF Explorer** users who want to add `PhysicalNeverDies.dll` to the executable's import table. |
| `x64` / `x86` | Pick the folder that matches the game's architecture (64-bit or 32-bit). |

> The folder must match the architecture of the **game**, not of Windows. A 32-bit game running on 64-bit Windows uses `x86`.

## How it works

1. When the DLL is loaded, the `init()` function runs automatically (thanks to `ctor`).
2. It disables the system "Insert a disk" dialogs (`SetErrorMode`) so that empty or missing drives (A:, B:, an empty optical drive) do not block the check.
3. It reads `PhysicalNeverDies.ini` to find out whether the check is enabled, what the game name is (`GameName`) and, optionally, which drive to scan (`Drive`).
4. If the check is enabled, it looks for `LicensePhysical.txt` at the root of the specified drive, or on all detected drives (fixed, removable and optical drives; network drives are ignored) if no drive is specified. The check succeeds as soon as one drive contains a valid license.
5. It verifies that the file contains `license = <GameName>`.
6. On failure, an error dialog is displayed and the process is terminated (`exit(1)`).

## Installation with the Hook (recommended)

1. Find out the game's architecture (64-bit or 32-bit). In Task Manager, 32-bit processes are followed by "(32 bit)".
2. Open `Output/Hook/x64/` or `Output/Hook/x86/` depending on the game's architecture.
3. Copy the following files into the game folder, next to the executable:
   - `winmm.dll`
   - `PhysicalNeverDies.dll`
   - `PhysicalNeverDies.ini`
4. Put `LicensePhysical.txt` at the root of the disc or USB drive you want to use.
5. Adjust `PhysicalNeverDies.ini` if needed (see [Configuration](#configuration)).
6. Launch the game: `winmm.dll` loads `PhysicalNeverDies.dll` automatically and the check runs.

### `winmm.dll` or `version.dll`?

The hook can be renamed depending on which system DLL the game loads:

- Try `winmm.dll` first.
- If the game does not load it (no check, no message), rename it to `version.dll`.
- Use **only one** of the two names at a time.

## Installation with CFF Explorer (Import)

1. Open `Output/Import/x64/` or `Output/Import/x86/` depending on the game's architecture.
2. Copy the following files into the game folder, next to the executable:
   - `PhysicalNeverDies.dll`
   - `PhysicalNeverDies.ini`
3. Make a backup copy of the game's executable.
4. Open the executable with **CFF Explorer**:
   - go to **Import Directory**,
   - right-click, then **Add**, and select `PhysicalNeverDies.dll`,
   - tick the `check_disk` function,
   - save the executable.
5. Put `LicensePhysical.txt` at the root of the disc or USB drive you want to use.
6. Adjust `PhysicalNeverDies.ini` if needed (see [Configuration](#configuration)).
7. Launch the game.

The exported `check_disk` function does nothing. It only exists so the DLL can be added to the import table. The check itself runs when the DLL is loaded.

> Do not use the Hook and the Import method at the same time: one method is enough.

## Configuration

### `PhysicalNeverDies.ini`

Provided in every folder. Place it next to the game's executable (see [Known limitation](#known-limitation)).

Scan all drives:

```ini
IsDiskNeeded = true
GameName = REPO
#Drive = E:\
```

Scan only `E:\`:

```ini
IsDiskNeeded = true
GameName = REPO
Drive = E:\
```

| Key | Value | Description |
|---|---|---|
| `IsDiskNeeded` | `true` / `false` | `false` disables the check entirely. Default: `true`. |
| `GameName` | text | Game name expected in the license. Default: `REPO`. |
| `Drive` | `E:\`, `E:`, `E` | Optional. If set, only this drive is scanned. If commented out, missing or invalid, all detected drives are scanned. |

Empty lines and lines starting with `#` or `;` are ignored. Keys are case-insensitive.

About `Drive`:

- The formats `E:\`, `E:`, `E:/` and `E` are accepted, in upper or lower case.
- An invalid value (for example `Drive = test` or `Drive = E:\Folder`) is ignored and all drives are scanned.

### `LicensePhysical.txt`

Provided in every folder. Place it at the **root of a drive** (for example `D:\LicensePhysical.txt`, on the disc or USB drive).

```
license = REPO
```

The value must match the `GameName` in `PhysicalNeverDies.ini`. The comparison is case-insensitive (`repo` and `REPO` are both accepted). For a strict comparison, replace `eq_ignore_ascii_case(target_game_name)` with `== target_game_name` in `src/lib.rs`.

If you change `GameName` in `PhysicalNeverDies.ini`, also change `license` in `LicensePhysical.txt` so that they stay identical.

## Building

Requirements: [Rust](https://rustup.rs/) with the MSVC toolchain on Windows.

### Project structure

```
physical/
├── Cargo.toml
└── src/
    ├── lib.rs     # check logic
    └── text.rs    # translated error messages
```

### Cargo.toml

```toml
[package]
name = "physical"
version = "1.0.0"
edition = "2021"

[lib]
name = "PhysicalNeverDies"
crate-type = ["cdylib"]

[dependencies]
ctor = "0.2"
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_System_LibraryLoader",
    "Win32_System_Diagnostics",
    "Win32_System_Diagnostics_Debug",
    "Win32_Storage_FileSystem",
    "Win32_UI_WindowsAndMessaging",
    "Win32_Globalization",
] }
```

The `windows` features are required: without them the compiler reports errors such as `cannot find Diagnostics in System`. Both the parent (`Win32_System_Diagnostics`) and child (`Win32_System_Diagnostics_Debug`) features must be enabled.

### Commands

```bash
# 64-bit
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

# 32-bit
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

The generated DLL is located at `target/<target>/release/PhysicalNeverDies.dll`. Then copy it into the matching `x64` or `x86` folders of `Output/Hook/` and `Output/Import/`.

## Message languages

The language is detected with `GetUserDefaultUILanguage` (the Windows display language). English is used by default.

| Language | Code |
|---|---|
| English | `0x09` |
| French | `0x0C` |
| German | `0x07` |
| Spanish | `0x0A` |
| Italian | `0x10` |
| Portuguese | `0x16` |

### Adding a language

1. Add a variant to the `Lang` enum in `src/text.rs`.
2. Add its code in `detect_language()` (for example `0x19` for Russian, `0x11` for Japanese).
3. Add a `match` block for that language in `get()`. The compiler flags any missing case.

## Error messages

| Case | Identifier | When |
|---|---|---|
| File not found | `FileNotFound` | `LicensePhysical.txt` does not exist on the specified drive, or on any drive if `Drive` is not set. |
| Invalid license | `LicenseInvalid` | The file exists but does not contain `license = <GameName>`. |
| Read error | `ReadError` | The file exists but cannot be read. |

## Known limitation

`GetModuleFileNameW(HMODULE(0), ...)` returns the path of the game's **executable**, not of the DLL. `PhysicalNeverDies.ini` is therefore looked up in the `.exe` folder. If the DLL is placed elsewhere, use `GetModuleHandleExW` with the `GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS` flag on the address of `check_disk`.

## Troubleshooting

| Problem | Suggestion |
|---|---|
| No check with the Hook | Rename `winmm.dll` to `version.dll` (only one of the two at a time) and make sure `PhysicalNeverDies.dll` is in the same folder. |
| The game does not start at all | Make sure the DLLs have the same architecture (x64 / x86) as the game. |
| "License not found" although the disc is inserted | Make sure `LicensePhysical.txt` is at the **root** of the drive, and that `Drive` in `PhysicalNeverDies.ini` points to the right drive (or is commented out). |
| "License not found" although the file exists | Make sure `license` matches the `GameName` in `PhysicalNeverDies.ini`. |
| The check does not run | Make sure `IsDiskNeeded` is not set to `false` and that `PhysicalNeverDies.ini` is next to the executable. |
| Message in English although Windows uses another language | The language is not supported yet in `text.rs`, see [Adding a language](#adding-a-language). |
| Antivirus blocks `winmm.dll` | Proxy DLLs are sometimes flagged by mistake. Add an exclusion for the game folder. |
| `cannot find Diagnostics in System` error when building | Add `Win32_System_Diagnostics` and `Win32_System_Diagnostics_Debug` to the `windows` features in `Cargo.toml`. |