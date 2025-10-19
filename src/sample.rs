use minimp3::Decoder;
use std::fs::File;

pub fn filtre_passe_bas(frequence_coupure: f32, echantillons: &[f32]) -> Vec<f32> {
    const FREQUENCE_ECHANTILLONNAGE: u32 = 44100;
    let fc = frequence_coupure / FREQUENCE_ECHANTILLONNAGE as f32;
    let alpha = 2.0 * std::f32::consts::PI * fc / (2.0 * std::f32::consts::PI * fc + 1.0);

    let mut echantillons_filtres = vec![0.0; echantillons.len()];
    if !echantillons.is_empty() {
        echantillons_filtres[0] = echantillons[0];
    }
    for i in 1..echantillons.len() {
        echantillons_filtres[i] = alpha * echantillons[i] + (1.0 - alpha) * echantillons_filtres[i - 1];
    }
    echantillons_filtres
}

pub fn sous_echantillonner(echantillons: Vec<f32>) -> Vec<f32> {
    const FREQUENCE_ECHANTILLONNAGE: u32 = 44100;
    const FACTEUR_SOUS_ECHANTILLONNAGE: usize = 4;
    let frequence_coupure = (FREQUENCE_ECHANTILLONNAGE / FACTEUR_SOUS_ECHANTILLONNAGE as u32) as f32 * 0.45;
    let filtres = filtre_passe_bas(frequence_coupure, &echantillons);

    filtres
        .iter()
        .step_by(FACTEUR_SOUS_ECHANTILLONNAGE)
        .cloned()
        .collect()
}

pub fn obtenir_metadonnees(chemin: &str) -> Result<Vec<f32>, String> {
    const FREQUENCE_ECHANTILLONNAGE: u32 = 44100;
    const DUREE_MINIMALE_SECONDES: f32 = 8.0;

    let mut decodeur = Decoder::new(File::open(chemin).map_err(|_| "Impossible d'ouvrir le fichier audio")?);
    let mut metadonnees = Vec::new();

    while let Ok(trame) = decodeur.next_frame() {
        match trame.channels {
            1 => {
                metadonnees.extend(trame.data.iter().map(|&d| d as f32));
            }
            2 => {
                let taille = trame.data.len();
                let mut i = 0;
                while i + 1 < taille {
                    let gauche = trame.data[i] as f32;
                    let droite = trame.data[i + 1] as f32;
                    let moyenne = (gauche + droite) * 0.5;
                    metadonnees.push(moyenne);
                    i += 2;
                }
            }
            _ => {
                return Err("Le fichier audio n'est pas stéréo ou mono".to_string());
            }
        }
    }

    let duree = metadonnees.len() as f32 / FREQUENCE_ECHANTILLONNAGE as f32;
    if duree < DUREE_MINIMALE_SECONDES {
        return Err(format!(
            "Durée audio trop courte ({:.2} s < {:.2} s)",
            duree, DUREE_MINIMALE_SECONDES
        ));
    }

    Ok(metadonnees)
}






