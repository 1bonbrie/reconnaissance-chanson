use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

pub enum Commande {
    Inserer,
    Reconnaitre,
}

pub fn utiliser_db(
    connexion: &mut Connection,
    commande: Commande,
    id_chanson: Option<&str>,
    empreintes: &[(usize, usize, usize, usize)],
) -> Result<String> {
    match commande {
        Commande::Inserer => {
            if let Some(id) = id_chanson {
                inserer_empreintes(connexion, id, empreintes)?;
                Ok("Empreintes insérées avec succès".to_string())
            } else {
                Err(rusqlite::Error::InvalidParameterName("id_chanson est requis pour Inserer".to_string()))
            }
        }
        Commande::Reconnaitre => {
            let resultat = trouver_correspondances(connexion, empreintes)?;
            Ok(resultat)
        }
    }
}

pub fn initialiser_db(chemin: &str) -> Result<Connection> {
    let connexion = Connection::open(chemin)?;
    connexion.execute(
        "CREATE TABLE IF NOT EXISTS empreintes (
            id INTEGER PRIMARY KEY,
            id_chanson TEXT,
            freq_ancre INTEGER,
            freq_cible INTEGER,
            delta_temps INTEGER,
            temps_ancre INTEGER
        )",
        [],
    )?;
    connexion.execute(
        "CREATE INDEX IF NOT EXISTS idx_empreintes ON empreintes(freq_ancre, freq_cible, delta_temps)",
        [],
    )?;
    Ok(connexion)
}

pub fn inserer_empreintes(
    connexion: &mut Connection,
    id_chanson: &str,
    empreintes: &[(usize, usize, usize, usize)],
) -> Result<()> {
    let transaction = connexion.transaction()?;
    for (freq_ancre, freq_cible, delta_temps, temps_ancre) in empreintes {
        transaction.execute(
            "INSERT INTO empreintes (id_chanson, freq_ancre, freq_cible, delta_temps, temps_ancre) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id_chanson, *freq_ancre as i64, *freq_cible as i64, *delta_temps as i64, *temps_ancre as i64],
        )?;
    }
    transaction.commit()?;
    Ok(())
}


pub fn trouver_correspondances(
    connexion: &Connection,
    empreintes_recherche: &[(usize, usize, usize, usize)],
) -> Result<String> {
    let mut correspondances = HashMap::new();
    const MIN_CORRESPONDANCES: usize = 800;

    for &(freq_ancre, freq_cible, delta_temps, temps_ancre_recherche) in empreintes_recherche {
        let mut requete = connexion.prepare(
            "SELECT id_chanson, temps_ancre FROM empreintes WHERE freq_ancre = ?1 AND freq_cible = ?2 AND delta_temps = ?3"
        )?;
        let lignes = requete.query_map(params![freq_ancre as i64, freq_cible as i64, delta_temps as i64], |ligne| {
            Ok((ligne.get::<_, String>(0)?, ligne.get::<_, i64>(1)?))
        })?;

        for ligne in lignes {
            let (id_chanson, temps_ancre_base) = ligne?;
            let decalage = temps_ancre_base - temps_ancre_recherche as i64;
            *correspondances.entry((id_chanson, decalage)).or_insert(0) += 1;
        }
    }

    if let Some(((nom_chanson, _), &nb_correspondances)) = correspondances.iter().max_by_key(|entry| entry.1) {
        if nb_correspondances >= MIN_CORRESPONDANCES {
            return Ok(nom_chanson.clone());
        }
    }
    Ok("Aucune correspondance trouvée".to_string())
}