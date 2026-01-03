use std::f32::consts::PI;
use rustfft::{FftPlanner, num_complex::Complex};


pub fn fenetre_hamming(echantillons: &[f32]) -> Vec<f32> {
    let n = echantillons.len() as f32;

    echantillons
        .iter()
        .enumerate()
        .map(|(i, &x)| {
            let w = 0.54 - 0.46 * (2.0 * PI * i as f32 / (n - 1.0)).cos();
            w * x
        })
        .collect()
}


pub fn transformation_fourier(fenetre: &[f32]) -> Vec<Complex<f32>> {
    const TAILLE: usize = 1024;
    assert_eq!(
        fenetre.len(),
        TAILLE,
        "La fenêtre doit avoir une taille de {}",
        TAILLE
    );

    let mut tampon: Vec<Complex<f32>> = fenetre
        .iter()
        .map(|&valeur| Complex { re: valeur, im: 0.0 })
        .collect();

    let mut planificateur = FftPlanner::<f32>::new();
    let fft = planificateur.plan_fft_forward(TAILLE);
    fft.process(&mut tampon);

    // On ne garde que la moitié utile (0..TAILLE/2)
    tampon.truncate(TAILLE / 2);
    tampon
}

pub fn generer_spectrogramme(mut metadonnees: Vec<f32>) -> Vec<Vec<Complex<f32>>> {
    let taille_fenetre = 1024;
    let taille_saut = 512;

    let reste = metadonnees.len() % taille_fenetre;
    if reste != 0 {
        let a_ajouter = taille_fenetre - reste;
        metadonnees.extend(std::iter::repeat(0.0).take(a_ajouter));
    }

    let mut spectrogramme = Vec::new();
    let mut position = 0;

    while position + taille_fenetre <= metadonnees.len() {
        let segment = &metadonnees[position..position + taille_fenetre];
        let fenetre = fenetre_hamming(segment);
        let fft_fenetre = transformation_fourier(&fenetre);
        spectrogramme.push(fft_fenetre);

        position += taille_saut;
    }

    spectrogramme
}

pub fn trouver_pics(
    spectrogramme: &[Vec<Complex<f32>>],
    nombre_pics: usize,
) -> Vec<(usize, usize, f32)> {
    let mut liste_pics = Vec::new();

    for (indice_temps, fenetre_freq) in spectrogramme.iter().enumerate() {
        if fenetre_freq.len() < 3 {
            continue;
        }

        let amplitudes: Vec<f32> = fenetre_freq.iter().map(|c| c.norm()).collect();
        let len = amplitudes.len();

        let mut pics_locaux: Vec<(usize, f32)> = Vec::new();
        for k in 1..(len - 1) {
            let amp = amplitudes[k];
            if amp > amplitudes[k - 1] && amp > amplitudes[k + 1] {
                pics_locaux.push((k, amp));
            }
        }

        pics_locaux.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for &(indice_frequence, amplitude) in pics_locaux.iter().take(nombre_pics) {
            liste_pics.push((indice_temps, indice_frequence, amplitude));
        }
    }

    liste_pics
}


pub fn generer_empreintes(
    pics: &[(usize, usize, f32)], 
    max_voisins: usize,          
    max_delta_temps: usize,      
) -> Vec<(usize, usize, usize)> {
    let mut empreintes = Vec::new();

    for i in 0..pics.len() {
        let (temps1, freq1, _) = pics[i];

        let mut candidats: Vec<(usize, usize, f32)> = Vec::new(); 

        for j in (i + 1)..pics.len() {
            let (temps2, freq2, amp2) = pics[j];
            let delta_temps = temps2 - temps1;

            if delta_temps == 0 {
                continue;
            }
            if delta_temps > max_delta_temps {
                break;
            }

            candidats.push((j, freq2, amp2));
        }

        candidats.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        for (j, freq2, _amp2) in candidats.into_iter().take(max_voisins) {
            let (temps2, _, _) = pics[j];
            let delta_temps = temps2 - temps1;
            empreintes.push((freq1, freq2, delta_temps));
        }
    }

    empreintes
}
