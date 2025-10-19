import React, { useState } from "react";

function App() {
  const [fichier, setFichier] = useState(null);
  const [resultat, setResultat] = useState("");
  const [mode, setMode] = useState("inserer");
  const [loading, setLoading] = useState(false);

  const handleFileChange = (e) => {
    setFichier(e.target.files[0]);
    setResultat("");
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!fichier) return;
    setLoading(true);
    setResultat("");
    const formData = new FormData();
    formData.append("fichier", fichier);

    const url =
      mode === "inserer"
        ? "http://localhost:8080/inserer"
        : "http://localhost:8080/reconnaitre";
    try {
      const response = await fetch(url, {
        method: "POST",
        body: formData,
      });
      const text = await response.text();
      setResultat(text);
    } catch (err) {
      setResultat("Erreur de connexion au serveur.");
    }
    setLoading(false);
  };

  return (
    <div
      style={{
        maxWidth: 500,
        margin: "40px auto",
        padding: 30,
        background: "#f9f9f9",
        borderRadius: 12,
        boxShadow: "0 2px 16px #0001",
        fontFamily: "Segoe UI, Arial, sans-serif",
      }}
    >
      <h1 style={{ textAlign: "center", color: "#1976d2" }}>
        Reconnaissance de chansons
      </h1>
      <div style={{ display: "flex", justifyContent: "center", gap: 10, marginBottom: 25 }}>
        <button
          onClick={() => setMode("inserer")}
          disabled={mode === "inserer"}
          style={{
            padding: "8px 18px",
            borderRadius: 6,
            border: "none",
            background: mode === "inserer" ? "#1976d2" : "#e3e3e3",
            color: mode === "inserer" ? "#fff" : "#333",
            cursor: mode === "inserer" ? "default" : "pointer",
            fontWeight: "bold",
            fontSize: 16,
          }}
        >
          Insérer une chanson
        </button>
        <button
          onClick={() => setMode("reconnaitre")}
          disabled={mode === "reconnaitre"}
          style={{
            padding: "8px 18px",
            borderRadius: 6,
            border: "none",
            background: mode === "reconnaitre" ? "#1976d2" : "#e3e3e3",
            color: mode === "reconnaitre" ? "#fff" : "#333",
            cursor: mode === "reconnaitre" ? "default" : "pointer",
            fontWeight: "bold",
            fontSize: 16,
          }}
        >
          Reconnaître une chanson
        </button>
      </div>
      <form onSubmit={handleSubmit} style={{ textAlign: "center" }}>
        <input
          type="file"
          accept=".mp3"
          onChange={handleFileChange}
          style={{
            marginBottom: 15,
            padding: "6px 0",
            fontSize: 15,
          }}
        />
        <br />
        <button
          type="submit"
          style={{
            padding: "10px 28px",
            borderRadius: 6,
            border: "none",
            background: "#1976d2",
            color: "#fff",
            fontWeight: "bold",
            fontSize: 17,
            cursor: fichier && !loading ? "pointer" : "not-allowed",
            opacity: fichier && !loading ? 1 : 0.6,
          }}
          disabled={!fichier || loading}
        >
          {loading
            ? "Traitement..."
            : mode === "inserer"
            ? "Insérer"
            : "Reconnaître"}
        </button>
      </form>
      {resultat && (
        <div
          style={{
            marginTop: 30,
            background: "#e3f2fd",
            padding: 18,
            borderRadius: 8,
            color: "#1976d2",
            fontSize: 17,
            wordBreak: "break-word",
            boxShadow: "0 1px 4px #0001",
          }}
        >
          <strong>Résultat :</strong>
          <pre style={{ margin: 0, fontSize: 16, color: "#333" }}>{resultat}</pre>
        </div>
      )}
    </div>
  );
}

export default App;