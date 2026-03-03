// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use std::os::windows::process::CommandExt;
use tauri::Manager;

// On déclare le module setup qui contient ta logique d'installation
mod setup;

// Commande pour démarrer l'instance Android en mode caché
#[tauri::command]
async fn start_silent_engine(app_handle: tauri::AppHandle, package_name: String) -> Result<String, String> {
    // 1. Résoudre le chemin du script dans les ressources Windows
    let script_path = app_handle.path().resolve("resources/android_boot_script.sh", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "Script de boot introuvable dans les ressources")?;

    // Conversion du chemin Windows (C:\...) en chemin WSL (/mnt/c/...)
    let wsl_script_path = format!(
        "/mnt/{}", 
        script_path.to_string_lossy()
            .replace("\\", "/")
            .replace(":", "")
            .to_lowercase()
    );

    println!("🐳 Démarrage de Docker...");
    // 2. Démarrer Docker (en tant que root pour éviter la demande de mot de passe sudo)
    let docker_status = Command::new("wsl")
        .args(["-d", "Ubuntu-22.04", "-u", "root", "service", "docker", "start"])
        .creation_flags(0x08000000) // Cache la fenêtre console
        .status()
        .map_err(|e| format!("Erreur système Docker : {}", e))?;

    if !docker_status.success() {
        return Err("Échec du démarrage du service Docker dans WSL".into());
    }

    println!("🚀 Lancement du script Android depuis : {}", wsl_script_path);
    // 3. Lancer le script via bash
    let _child = Command::new("wsl")
        .args(["-d", "Ubuntu-22.04", "bash", "-c", "~/android_boot_script.sh"])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Erreur script : {}", e))?;

    Ok("Docker est prêt et l'instance Android démarre.".into())
}

#[tauri::command]
async fn launch_app_window(package_name: String) -> Result<(), String> {
    println!("📱 Tentative de connexion à l'instance...");

    // 1. Forcer la connexion ADB (Utilise 127.0.0.1 plutôt que localhost pour plus de stabilité)
    let _ = Command::new("adb")
        .args(["connect", "127.0.0.1:5555"])
        .creation_flags(0x08000000)
        .status();

    // 2. Attendre que l'appareil soit prêt (Loop de 5 secondes max)
    let mut ready = false;
    for _ in 0..10 {
        let output = Command::new("adb")
            .args(["-s", "127.0.0.1:5555", "get-state"])
            .creation_flags(0x08000000)
            .output();

        if let Ok(out) = output {
            let state = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if state == "device" {
                ready = true;
                break;
            }
        }
        println!("⏳ Instance en cours de boot, attente...");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    if !ready {
        return Err("L'instance Android ne répond pas après 10 secondes.".into());
    }

    // 3. Lancer scrcpy
    println!("✅ Appareil prêt. Lancement de la fenêtre...");
    Command::new("scrcpy")
        .args([
            "--serial", "127.0.0.1:5555",
            "--window-title", &package_name,
            "--no-audio",
            "--always-on-top",
            
        ])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("Erreur scrcpy : {}", e))?;

    Command::new("wsl")
        .args(["-d", "Ubuntu-22.04", "-u", "root", "service", "docker", "logs", "calyx-engine", "--tail", "50"])
        .creation_flags(0x00000000) // Ne Cache pas la fenêtre console
        .output()
        .map_err(|e| format!("Erreur logs Docker : {}", e))?;
        

    Ok(())
}

#[tauri::command]
async fn get_engine_status() -> String {
    let output = Command::new("wsl")
        .args(["-d", "Ubuntu-22.04", "docker", "inspect", "-f", "{{.State.Status}}", "calyx-engine"])
        .creation_flags(0x08000000)
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            s // Retournera "running", "exited", etc.
        },
        _ => "stopped".to_string(),
    }
}

// N'oublie pas d'ajouter get_engine_status dans generate_handler!

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            
            // On lance la vérification de l'environnement au démarrage
            tauri::async_runtime::spawn(async move {
                println!("🛠️ Vérification des dépendances (WSL, Kernel, Docker)...");
                
                match setup::full_setup_check(handle.clone()).await {
                    Ok(_) => {
                        println!("✅ Environnement prêt !");
                        // Optionnel : On peut lancer le moteur automatiquement ici
                        let _ = start_silent_engine(handle, "Calyx UI".to_string()).await;
                    },
                    Err(e) => {
                        eprintln!("❌ Échec du setup critique : {}", e);
                        // Ici, tu pourrais envoyer un message à ton interface React
                        // pour dire à l'utilisateur de redémarrer son PC
                    }
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start_silent_engine, launch_app_window, get_engine_status])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}