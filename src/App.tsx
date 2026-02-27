import { useEffect, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

function App() {
  const [status, setStatus] = useState("Vérification du système...");
  const [engineState, setEngineState] = useState("off"); // off, booting, running
  const [installing, setInstalling] = useState(false);

  useEffect(() => {
    // 1. Écouter les étapes de l'installation (WSL, Docker, Kernel)
    const setupUnlisten = listen<string>("setup-progress", (event) => {
        setStatus(event.payload);
    });

    // 2. Initialisation complète
    const init = async () => {
        try {
            // Le setup.rs s'exécute déjà au boot via Rust, on attend un peu 
            // ou on vérifie manuellement le statut ici.
            const checkStatus = async () => {
                const s = await invoke("get_engine_status");
                if (s === "running") {
                    setEngineState("running");
                    setStatus("Calyx Engine est prêt 🌿");
                } else {
                    // Si pas lancé, on essaie de le démarrer
                    setEngineState("booting");
                    await invoke("start_silent_engine");
                }
            };

            // On check toutes les 3 secondes jusqu'à ce que ce soit prêt
            const interval = setInterval(async () => {
                const s = await invoke("get_engine_status") as string;
                if (s === "running") {
                    setEngineState("running");
                    setStatus("Moteur en ligne");
                    clearInterval(interval);
                }
            }, 3000);

        } catch (err) {
            setStatus(`Erreur critique : ${err}`);
        }
    };

    init();

    // 3. Drag & Drop APK
    const dragUnlisten = listen<{ paths: string[] }>("tauri://drag-drop", (event) => {
        const filePath = event.payload.paths[0];
        if (filePath.endsWith(".apk")) {
            handleInstall(filePath);
        }
    });

    return () => {
        setupUnlisten.then(f => f());
        dragUnlisten.then(f => f());
    };
  }, []);

  const handleInstall = async (path: string) => {
    setInstalling(true);
    setStatus(`Installation de l'APK...`);
    try {
      await invoke("install_apk", { path });
      setStatus("Application installée avec succès !");
    } catch (err) {
      setStatus(`Erreur d'installation : ${err}`);
    }
    setInstalling(false);
  };

  return (
    <div className="container">
      <header>
        <h1>Calyx 🌿</h1>
        <div className={`indicator ${engineState}`}></div>
      </header>

      <div className={`status-badge ${installing || engineState === "booting" ? "loading" : "ready"}`}>
        {status}
      </div>

      <div className="drop-zone">
        <div className="icon">📦</div>
        <p>Glissez votre APK ici pour l'injecter dans l'instance</p>
      </div>

      {engineState === "running" && (
          <button className="btn-primary" onClick={() => invoke("launch_app_window", { packageName: "Calyx UI" })}>
            Ouvrir l'écran Android
          </button>
      )}
    </div>
  );
}

export default App;