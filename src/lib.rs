use std::fs;
use std::path::{Path, PathBuf};
use windows::Win32::Foundation::{HMODULE, HWND};
use windows::Win32::System::Diagnostics::Debug::{SetErrorMode, SEM_FAILCRITICALERRORS};
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
use windows::core::HSTRING;

mod text;
use text::MsgId;

const LICENSE_FILE_NAME: &str = "LicensePhysical.txt";

/// Configuration lue depuis "Physical.ini"
struct Config {
    is_disk_needed: bool,
    game_name: String,
    /// Lecteur à scanner (None = scanner tous les lecteurs de A: à Z:)
    drive: Option<char>,
}

/// S'exécute automatiquement et immédiatement au chargement de la DLL
#[ctor::ctor]
fn init() {
    // Évite les boîtes système "Insérez un disque" sur les lecteurs A: et B: (ou lecteurs vides)
    unsafe {
        SetErrorMode(SEM_FAILCRITICALERRORS);
    }

    // 1. Charger la configuration depuis "Physical.ini"
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

/// Lit le fichier "Physical.ini"
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

/// Récupère le chemin de "Physical.ini" (même dossier que le module)
fn get_companion_ini_path() -> Option<PathBuf> {
    let mut buffer = vec![0u16; 32768];
    unsafe {
        let len = GetModuleFileNameW(HMODULE(0), &mut buffer);
        if len == 0 {
            return None;
        }
        let dll_path = String::from_utf16_lossy(&buffer[..len as usize]);
        let path = Path::new(&dll_path);
        Some(path.with_file_name("Physical.ini"))
    }
}

/// Cherche "LicensePhysical.txt" sur le lecteur demandé (ou sur tous les lecteurs
/// de A:\ à Z:\ si aucun n'est précisé) et vérifie que license == GameName
fn validate_license_on_drives(target_game_name: &str, drive: Option<char>) -> bool {
    let drives: Vec<char> = match drive {
        Some(letter) => vec![letter],
        None => (b'A'..=b'Z').map(|b| b as char).collect(),
    };

    let mut license_path: Option<PathBuf> = None;

    for drive_letter in drives {
        let path = PathBuf::from(format!("{}:\\{}", drive_letter, LICENSE_FILE_NAME));
        if path.exists() {
            license_path = Some(path);
            break;
        }
    }

    let path = match license_path {
        Some(p) => p,
        None => {
            show_localized_error(MsgId::FileNotFound);
            return false;
        }
    };

    if let Ok(content) = fs::read_to_string(path) {
        let mut found_license = false;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                if key.trim().eq_ignore_ascii_case("license")
                    && val.trim().eq_ignore_ascii_case(target_game_name)
                {
                    found_license = true;
                    break;
                }
            }
        }

        if !found_license {
            show_localized_error(MsgId::LicenseInvalid);
            return false;
        }
        return true;
    }

    show_localized_error(MsgId::ReadError);
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