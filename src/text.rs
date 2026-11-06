use windows::Win32::Globalization::GetUserDefaultUILanguage;

#[derive(Clone, Copy)]
pub enum Lang {
    En,
    Fr,
    De,
    Es,
    It,
    Pt,
}

#[derive(Clone, Copy)]
pub enum MsgId {
    /// Le fichier de licence n'existe sur aucun lecteur ({file} est remplacé)
    FileNotFound,
    /// Le fichier existe mais ne contient pas la bonne licence
    LicenseInvalid,
    /// Le fichier existe mais n'a pas pu être lu ({file} est remplacé)
    ReadError,
}

/// Détecte la langue d'affichage de Windows (anglais par défaut)
pub fn detect_language() -> Lang {
    let lang_id = unsafe { GetUserDefaultUILanguage() };
    // Les 10 bits de poids faible = langue principale (sans la région)
    match lang_id & 0x3FF {
        0x09 => Lang::En,
        0x0C => Lang::Fr,
        0x07 => Lang::De,
        0x0A => Lang::Es,
        0x10 => Lang::It,
        0x16 => Lang::Pt,
        _ => Lang::En,
    }
}

/// Retourne (titre, message) pour une langue et un type d'erreur
pub fn get(lang: Lang, id: MsgId) -> (&'static str, &'static str) {
    match lang {
        Lang::En => match id {
            MsgId::FileNotFound => (
                "License not found",
                "The license was not found: the file {file} could not be found on any of your drives.\n\nPlease insert the disc containing the license and restart the game.",
            ),
            MsgId::LicenseInvalid => (
                "License not found",
                "The license was not found.\n\nPlease insert the required disc and restart the game.",
            ),
            MsgId::ReadError => (
                "Read error",
                "Unable to read the contents of the file {file}.",
            ),
        },
        Lang::Fr => match id {
            MsgId::FileNotFound => (
                "Licence introuvable",
                "La licence est introuvable : le fichier {file} n'a été trouvé sur aucun de vos lecteurs.\n\nVeuillez insérer le disque contenant la licence puis relancer le jeu.",
            ),
            MsgId::LicenseInvalid => (
                "Licence introuvable",
                "La licence est introuvable.\n\nVeuillez insérer le disque requis puis relancer le jeu.",
            ),
            MsgId::ReadError => (
                "Erreur de lecture",
                "Impossible de lire le contenu du fichier {file}.",
            ),
        },
        Lang::De => match id {
            MsgId::FileNotFound => (
                "Lizenz nicht gefunden",
                "Die Lizenz wurde nicht gefunden: Die Datei {file} wurde auf keinem Ihrer Laufwerke gefunden.\n\nBitte legen Sie den Datenträger mit der Lizenz ein und starten Sie das Spiel neu.",
            ),
            MsgId::LicenseInvalid => (
                "Lizenz nicht gefunden",
                "Die Lizenz wurde nicht gefunden.\n\nBitte legen Sie den erforderlichen Datenträger ein und starten Sie das Spiel neu.",
            ),
            MsgId::ReadError => (
                "Lesefehler",
                "Der Inhalt der Datei {file} konnte nicht gelesen werden.",
            ),
        },
        Lang::Es => match id {
            MsgId::FileNotFound => (
                "Licencia no encontrada",
                "No se encontró la licencia: el archivo {file} no se encontró en ninguna de sus unidades.\n\nInserte el disco que contiene la licencia y reinicie el juego.",
            ),
            MsgId::LicenseInvalid => (
                "Licencia no encontrada",
                "No se encontró la licencia.\n\nInserte el disco necesario y reinicie el juego.",
            ),
            MsgId::ReadError => (
                "Error de lectura",
                "No se puede leer el contenido del archivo {file}.",
            ),
        },
        Lang::It => match id {
            MsgId::FileNotFound => (
                "Licenza non trovata",
                "Licenza non trovata: il file {file} non è stato trovato su nessuna delle unità.\n\nInserire il disco contenente la licenza e riavviare il gioco.",
            ),
            MsgId::LicenseInvalid => (
                "Licenza non trovata",
                "Licenza non trovata.\n\nInserire il disco richiesto e riavviare il gioco.",
            ),
            MsgId::ReadError => (
                "Errore di lettura",
                "Impossibile leggere il contenuto del file {file}.",
            ),
        },
        Lang::Pt => match id {
            MsgId::FileNotFound => (
                "Licença não encontrada",
                "A licença não foi encontrada: o ficheiro {file} não foi encontrado em nenhuma das suas unidades.\n\nInsira o disco que contém a licença e reinicie o jogo.",
            ),
            MsgId::LicenseInvalid => (
                "Licença não encontrada",
                "A licença não foi encontrada.\n\nInsira o disco necessário e reinicie o jogo.",
            ),
            MsgId::ReadError => (
                "Erro de leitura",
                "Não foi possível ler o conteúdo do ficheiro {file}.",
            ),
        },
    }
}