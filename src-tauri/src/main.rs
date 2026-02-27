// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::Manager;

// Commande pour démarrer l'instance Android en mode caché
#[tauri::command]
async fn start_silent_engine() -> Result<String, String> {
    // Ici, on simule le lancement du sous-système. 
    // Plus tard, on appellera une image Docker ou WSL spécifique à Calyx.
    let output = Command::new("wsl")
        .arg("-d")
        .arg("Calyx-Engine") // Nom de notre future instance
        .arg("echo")
        .arg("Android Heartbeat Started")
        .output();

    match output {
        Ok(_) => Ok("Calyx Engine is running in the background.".into()),
        Err(_) => Err("Failed to start Calyx Engine. Is WSL enabled?".into()),
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Optionnel : On peut cacher la fenêtre principale au boot si on veut 
            // que Calyx soit 100% discret au démarrage du PC.
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start_silent_engine])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
