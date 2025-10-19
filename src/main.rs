mod sample;
mod fingerprint;
mod db;
mod web;

#[tokio::main]
    async fn main() -> std::io::Result<()> {
        web::demarrer_serveur_web().await
    }
