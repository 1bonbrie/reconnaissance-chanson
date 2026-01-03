use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_multipart::Multipart;
use futures_util::StreamExt as _;
use std::fs::File;
use std::io::Write;
use actix_cors::Cors;

use crate::sample::{obtenir_metadonnees, sous_echantillonner};
use crate::fingerprint::{generer_spectrogramme, trouver_pics, generer_empreintes};
use crate::db;

pub async fn inserer_chanson(mut payload: Multipart) -> impl Responder {
    let mut nom_fichier_original = String::new(); 
    let chemin_fichier = "upload_inserer.mp3";

    while let Some(item) = payload.next().await {
        let mut champ = match item {
            Ok(c) => c,
            Err(e) => return HttpResponse::BadRequest().body(format!("Erreur multipart: {}", e)),
        };
        if nom_fichier_original.is_empty() {
            if let Some(filename) = champ.content_disposition().get_filename() {
                nom_fichier_original = filename.to_string();
            }
        }
        let mut fichier = match File::create(chemin_fichier) {
            Ok(f) => f,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Erreur création fichier: {}", e)),
        };
        while let Some(chunk) = champ.next().await {
            let data = match chunk {
                Ok(d) => d,
                Err(e) => return HttpResponse::BadRequest().body(format!("Erreur chunk: {}", e)),
            };
            if let Err(e) = fichier.write_all(&data) {
                return HttpResponse::InternalServerError().body(format!("Erreur écriture: {}", e));
            }
        }
    }

    let mut connexion = db::initialiser_db("empreintes.db").unwrap();
    match obtenir_metadonnees(chemin_fichier) {
        Ok(metadonnees) => {
            let metadonnees11khz = sous_echantillonner(metadonnees);
            let spectrogramme = generer_spectrogramme(metadonnees11khz);
            let pics = trouver_pics(&spectrogramme, 5);
            let empreintes = generer_empreintes(&pics, 5, 200);

            let mut nom_pour_db = if nom_fichier_original.is_empty() {
                "inconnu".to_string()
            } else {
                nom_fichier_original.clone()
            };
            
            if nom_pour_db.ends_with(".mp3") {
                nom_pour_db.truncate(nom_pour_db.len() - 4);
            }
            let res = db::utiliser_db(
                &mut connexion,
                db::Commande::Inserer,
                Some(&nom_pour_db),
                &empreintes,
            );

            HttpResponse::Ok().body(res.unwrap_or_else(|e| format!("Erreur : {}", e)))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Erreur extraction : {}", e)),
    }
}

pub async fn reconnaitre_chanson(mut payload: Multipart) -> impl Responder {
    let chemin_fichier = "upload_reconnaitre.mp3";
    while let Some(item) = payload.next().await {
        let mut champ = match item {
            Ok(c) => c,
            Err(e) => return HttpResponse::BadRequest().body(format!("Erreur multipart: {}", e)),
        };
        let mut fichier = match File::create(chemin_fichier) {
            Ok(f) => f,
            Err(e) => return HttpResponse::InternalServerError().body(format!("Erreur création fichier: {}", e)),
        };
        while let Some(chunk) = champ.next().await {
            let data = match chunk {
                Ok(d) => d,
                Err(e) => return HttpResponse::BadRequest().body(format!("Erreur chunk: {}", e)),
            };
            if let Err(e) = fichier.write_all(&data) {
                return HttpResponse::InternalServerError().body(format!("Erreur écriture: {}", e));
            }
        }
    }
    let mut connexion = db::initialiser_db("empreintes.db").unwrap();
    match obtenir_metadonnees(chemin_fichier) {
        Ok(metadonnees) => {
            let metadonnees11khz = sous_echantillonner(metadonnees);
            let spectrogramme = generer_spectrogramme(metadonnees11khz);
            let pics = trouver_pics(&spectrogramme, 5);
            let empreintes = generer_empreintes(&pics, 5, 200);
            let res = db::utiliser_db(&mut connexion, db::Commande::Reconnaitre, None, &empreintes);
            HttpResponse::Ok().body(res.unwrap_or_else(|e| format!("Erreur : {}", e)))
        }
        Err(e) => HttpResponse::InternalServerError().body(format!("Erreur extraction : {}", e)),
    }
}

pub async fn demarrer_serveur_web() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .route("/inserer", web::post().to(inserer_chanson))
            .route("/reconnaitre", web::post().to(reconnaitre_chanson))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}