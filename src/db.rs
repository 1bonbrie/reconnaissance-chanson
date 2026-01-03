use rusqlite::{params, Connection, Result};
use std::collections::HashMap;

pub enum Commande {
    Inserer,
    Reconnaitre,
}

pub fn hash_empreinte(freq_ancre: usize, freq_cible: usize, delta_temps: usize) -> i64 {
    let fa = (freq_ancre as u64) & 0xFFFF;
    let fc = (freq_cible as u64) & 0xFFFF;
    let dt = (delta_temps as u64) & 0xFFFF;

    let hash = (fa << 32) | (fc << 16) | dt;
    hash as i64
}

pub fn utiliser_db(
    connexion: &mut Connection,
    commande: Commande,
    id_chanson: Option<&str>,
    empreintes: &[(usize, usize, usize)],
) -> Result<String> {
    match commande {
        Commande::Inserer => {
            if let Some(id) = id_chanson {
                inserer_empreintes(connexion, id, empreintes)?;
                Ok("Empreintes insérées avec succès".to_string())
            } else {
                Err(rusqlite::Error::InvalidParameterName(
                    "id_chanson est requis pour Inserer".to_string(),
                ))
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
            hash INTEGER
        )",
        [],
    )?;
    connexion.execute(
        "CREATE INDEX IF NOT EXISTS idx_empreintes_hash
         ON empreintes(hash)",
        [],
    )?;
    Ok(connexion)
}

pub fn inserer_empreintes(
    connexion: &mut Connection,
    id_chanson: &str,
    empreintes: &[(usize, usize, usize)],
) -> Result<()> {
    let transaction = connexion.transaction()?;

    {
        let mut stmt = transaction.prepare(
            "INSERT INTO empreintes (id_chanson, hash)
             VALUES (?1, ?2)",
        )?;

        for (freq_ancre, freq_cible, delta_temps) in empreintes {
            let hash = hash_empreinte(*freq_ancre, *freq_cible, *delta_temps);

            stmt.execute(params![
                id_chanson,
                hash
            ])?;
        }
    }

    transaction.commit()?;
    Ok(())
}

pub fn trouver_correspondances(
    connexion: &Connection,
    empreintes_recherche: &[(usize, usize, usize)],
) -> Result<String> {
    let mut correspondances: HashMap<String, usize> = HashMap::new();
    const MIN_CORRESPONDANCES: usize = 5;

    let mut requete = connexion.prepare(
        "SELECT id_chanson FROM empreintes WHERE hash = ?1",
    )?;

    for &(freq_ancre, freq_cible, delta_temps) in empreintes_recherche {
        let h = hash_empreinte(freq_ancre, freq_cible, delta_temps);

        let lignes = requete.query_map(params![h], |ligne| {
            ligne.get::<_, String>(0)
        })?;

        for ligne in lignes {
            let id_chanson = ligne?;
            *correspondances.entry(id_chanson).or_insert(0) += 1;
        }
    }

    if let Some((nom_chanson, &nb_correspondances)) =
        correspondances.iter().max_by_key(|entry| entry.1)
    {
        if nb_correspondances >= MIN_CORRESPONDANCES {
            return Ok(format!("{} ({} correspondances)", nom_chanson, nb_correspondances));
        }
    }

    Ok("Aucune correspondance trouvée".to_string())
}
