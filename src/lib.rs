use std::fs;
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};
use windows::Win32::System::Diagnostics::Debug::{SetErrorMode, SEM_FAILCRITICALERRORS};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
use windows::core::{HSTRING, PCWSTR};

mod text;
use text::MsgId;

const LICENSE_FILE_NAME: &str = "LicensePhysical.txt";
const CONFIG_FILE_NAME: &str = "PhysicalNeverDies.ini";

// Types de lecteurs retournés par GetDriveTypeW
const DRIVE_REMOVABLE: u32 = 2; // clé USB, disquette
const DRIVE_FIXED: u32 = 3; // disque dur / SSD
const DRIVE_CDROM: u32 = 5; // CD / DVD / Blu-ray

/// Configuration lue depuis "PhysicalNeverDies.ini"
struct Config {
    is_disk_needed: bool,
    game_name: String,
    /// Lecteur à scanner (None = scanner tous les lecteurs détectés)
    drive: Option<char>,
}

/// Résultat de la vérification sur un lecteur
enum DriveCheck {
    Valid,
    Invalid,
    ReadError,
    Missing,
}

/// S'exécute automatiquement et immédiatement au chargement de la DLL
#[ctor::ctor]
fn init() {
    // Évite les boîtes système "Insérez un disque" sur les lecteurs vides ou absents
    unsafe {
        SetErrorMode(SEM_FAILCRITICALERRORS);
    }

    // 1. Charger la configuration depuis "PhysicalNeverDies.ini"
    let config = load_companion_config();

    // Si le check disque est désactivé, on laisse le jeu se lancer
    if !config.is_disk_needed {
        return;
    }

    // 2. Sinon, on cherche "LicensePhysical.txt" et on vérifie "license"
    if !validate_license_on_drives(&config.game_name, config.drive) {
        std::process::exit(1);
    }
}

/// Lit le fichier "PhysicalNeverDies.ini"
fn load_companion_config() -> Config {
    let mut config = Config {
        is_disk_needed: true,
        game_name: String::from("REPO"),
        drive: None,
    };

    if let Some(ini_path) = get_companion_ini_path() {
        if let Ok(content) = fs::read_to_string(&ini_path) {
            for line in content.lines() {
                let line = line.trim();
                // Les lignes commentées (ex: "#Drive = E:\") sont ignorées
                if line.starts_with('#') || line.starts_with(';') || line.is_empty() {
                    continue;
                }
                if let Some((key, val)) = line.split_once('=') {
                    let key = key.trim().to_lowercase();
                    let val = val.trim();
                    match key.as_str() {
                        "isdiskneeded" => config.is_disk_needed = val.parse().unwrap_or(true),
                        "gamename" => config.game_name = val.to_string(),
                        "drive" => config.drive = parse_drive_letter(val),
                        _ => {}
                    }
                }
            }
        }
    }

    config
}

/// Extrait la lettre d'un lecteur depuis une valeur comme "E:\", "E:", "E:/" ou "E".
/// Retourne None si la valeur est vide ou invalide (tous les lecteurs seront alors scannés).
fn parse_drive_letter(val: &str) -> Option<char> {
    let v = val.trim().trim_matches('"');
    let first = v.chars().next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    // first est un caractère ASCII (1 octet), on peut donc couper à l'index 1
    match &v[1..] {
        "" | ":" | ":\\" | ":/" => Some(first.to_ascii_uppercase()),
        _ => None,
    }
}

/// Récupère le chemin de "PhysicalNeverDies.ini" (même dossier que l'exécutable)
fn get_companion_ini_path() -> Option<PathBuf> {
    let mut buffer = vec![0u16; 32768];
    unsafe {
        let len = GetModuleFileNameW(HMODULE(0), &mut buffer);
        if len == 0 {
            return None;
        }
        let dll_path = String::from_utf16_lossy(&buffer[..len as usize]);
        let path = Path::new(&dll_path);
        Some(path.with_file_name(CONFIG_FILE_NAME))
    }
}

/// Type du lecteur (lettre) selon Windows
fn drive_type(letter: char) -> u32 {
    let root: Vec<u16> = format!("{}:\\", letter)
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) }
}

/// Liste des lecteurs à tester.
/// - Si un lecteur est imposé dans le .ini, on teste uniquement celui-ci.
/// - Sinon, on prend les lecteurs existants de type fixe, amovible ou optique
///   (les lecteurs réseau sont ignorés pour éviter les blocages).
fn candidate_drives(forced: Option<char>) -> Vec<char> {
    if let Some(letter) = forced {
        return vec![letter];
    }

    let mask = unsafe { GetLogicalDrives() };
    (0..26u32)
        .filter(|i| mask & (1 << i) != 0)
        .map(|i| (b'A' + i as u8) as char)
        .filter(|&letter| {
            let t = drive_type(letter);
            t == DRIVE_REMOVABLE || t == DRIVE_FIXED || t == DRIVE_CDROM
        })
        .collect()
}

/// Vérifie si le contenu du fichier contient "license = <GameName>"
fn license_matches(content: &str, target_game_name: &str) -> bool {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            if key.trim().eq_ignore_ascii_case("license")
                && val.trim().eq_ignore_ascii_case(target_game_name)
            {
                return true;
            }
        }
    }
    false
}

/// Vérifie la licence sur un lecteur donné
fn check_drive(letter: char, target_game_name: &str) -> DriveCheck {
    let path = PathBuf::from(format!("{}:\\{}", letter, LICENSE_FILE_NAME));
    if !path.exists() {
        return DriveCheck::Missing;
    }
    match fs::read_to_string(&path) {
        Ok(content) => {
            if license_matches(&content, target_game_name) {
                DriveCheck::Valid
            } else {
                DriveCheck::Invalid
            }
        }
        Err(_) => DriveCheck::ReadError,
    }
}

/// Cherche "LicensePhysical.txt" sur les lecteurs candidats et vérifie que license == GameName.
/// Réussit dès qu'un lecteur contient une licence valide.
fn validate_license_on_drives(target_game_name: &str, drive: Option<char>) -> bool {
    let mut found_invalid = false;
    let mut found_read_error = false;

    for letter in candidate_drives(drive) {
        match check_drive(letter, target_game_name) {
            DriveCheck::Valid => return true,
            DriveCheck::Invalid => found_invalid = true,
            DriveCheck::ReadError => found_read_error = true,
            DriveCheck::Missing => {}
        }
    }

    let error = if found_invalid {
        MsgId::LicenseInvalid
    } else if found_read_error {
        MsgId::ReadError
    } else {
        MsgId::FileNotFound
    };
    show_localized_error(error);
    false
}

/// Affiche une erreur dans la langue du système
fn show_localized_error(id: MsgId) {
    let lang = text::detect_language();
    let (title, body) = text::get(lang, id);
    show_error(title, &body.replace("{file}", LICENSE_FILE_NAME));
}

fn show_error(title: &str, message: &str) {
    let t = HSTRING::from(title);
    let m = HSTRING::from(message);
    unsafe {
        let _ = MessageBoxW(HWND(0), &m, &t, MB_OK | MB_ICONERROR);
    }
}

/// Fonction exportée de secours pour CFF Explorer
#[no_mangle]
pub extern "C" fn check_disk() {}