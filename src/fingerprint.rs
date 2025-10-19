use std::f32::consts::PI;
use rustfft::{FftPlanner, num_complex::Complex};

pub fn fenetre_hamming(echantillons: &[f32]) -> Vec<f32> {
    let mut echantillons_fenetre = Vec::with_capacity(echantillons.len());
    let n = echantillons.len() as f32;
    for (i, echantillon) in echantillons.iter().enumerate() {
        let multiplicateur = 0.54 - (0.92 * PI * i as f32 / n).cos();
        echantillons_fenetre.push(multiplicateur * echantillon)
    }
    echantillons_fenetre
}

pub fn transformation_fourier(fenetre: &[f32]) -> Vec<Complex<f32>> {
    const TAILLE:usize = 1024;
    let mut tampon: Vec<Complex<f32>> = fenetre
        .iter()
        .map(|&valeur| Complex { re: valeur, im: 0.0 })
        .collect();
    let mut planificateur = FftPlanner::<f32>::new();
    let fft = planificateur.plan_fft_forward(TAILLE);
    fft.process(&mut tampon);

    tampon
}

pub fn generer_spectrogramme(mut metadonnees: Vec<f32>)  -> Vec<Vec<Complex<f32>>> {
    let taille_fenetre = 1024;
    let taille_saut =  512;

    let reste = metadonnees.len() % taille_fenetre;
    if reste != 0 {
        let a_ajouter = taille_fenetre - reste;
        metadonnees.extend(std::iter::repeat(0.0).take(a_ajouter));
    }

    let mut position = 0;
    let mut spectrogramme = Vec::new();

    while position < metadonnees.len() {
        if position + taille_fenetre > metadonnees.len() {
            position = metadonnees.len() - taille_fenetre;
        }
        
        let segment = &metadonnees[position..position + taille_fenetre];
        let fenetre = fenetre_hamming(segment);
        let fft_fenetre = transformation_fourier(&fenetre);

        spectrogramme.push(fft_fenetre);

        if position + taille_fenetre >= metadonnees.len() {
            break;
        }

        position += taille_fenetre - taille_saut;
    }

    spectrogramme
}

pub fn trouver_pics(
    spectrogramme: &[Vec<Complex<f32>>],
    nombre_pics: usize,
) -> Vec<(usize, usize, f32)> {
    let mut liste_pics = Vec::new();

    for (indice_temps, fenetre_freq) in spectrogramme.iter().enumerate() {
        let mut amplitudes: Vec<(usize, f32)> = fenetre_freq
            .iter()
            .enumerate()
            .map(|(indice_frequence, coeff)| (indice_frequence, coeff.norm()))
            .collect();

        amplitudes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for &(indice_frequence, amplitude) in amplitudes.iter().take(nombre_pics) {
            liste_pics.push((indice_temps, indice_frequence, amplitude));
        }
    }

    liste_pics
}

pub fn generer_empreintes(
    pics: &[(usize, usize, f32)], 
) -> Vec<(usize, usize, usize, usize)> {
    let mut empreintes = Vec::new();
    const VALEUR_FAN: usize = 5; 
    const DELTA_TEMPS_MAX: usize = 200; 
    for (i, &(temp1, frequence1, _)) in pics.iter().enumerate() {
        for j in 1..=VALEUR_FAN {
            if let Some(&(temp2, frequence2, _)) = pics.get(i + j) {
                let delta_temps = temp2.saturating_sub(temp1);
                if delta_temps > 0 && delta_temps <= DELTA_TEMPS_MAX {
                    empreintes.push((frequence1, frequence2, delta_temps, temp1));
                }
            }
        }
    }

    empreintes
}
