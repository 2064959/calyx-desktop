import { useEffect, useState } from "react";
import "./App.css";
import { invoke } from "@tauri-apps/api/core";

function App() {
  const [status, setStatus] = useState("initializing engine...");

  useEffect(() => {
    // Appelle la fonction Rust au démarrage
    invoke<string>("start_silent_engine")
      .then((res) => setStatus(res))
      .catch((err) => setStatus(`Error: ${err}`));
  }, []);

  return (
    <div className="container">
      <h1>Calyx 🌿</h1>
      <div className={`status-badge ${status.includes("Ready") ? "ready" : "error"}`}>
        {status}
      </div>
      <p>Drag an APK here to host it on your desktop.</p>
    </div>
  );
}

export default App;