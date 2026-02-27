use std::process::Command;
use std::fs;
use tauri::Manager;
use tauri::Emitter;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;


pub async fn full_setup_check(app_handle: tauri::AppHandle) -> Result<(), String> {
    // --- ÉTAPE 1 : WSL2 & Ubuntu ---
    app_handle.emit("setup-progress", "🔍 Vérification de WSL2...").unwrap();
    let _resource_path = app_handle.path().resolve("resources/bzImage", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "Le fichier bzImage est manquant dans les ressources")?;

    let wsl_list = Command::new("wsl")
        .args(["--list", "--verbose"])
        .output()
        .map_err(|e| format!("Erreur WSL list: {}", e))?;
    
    let output_str = String::from_utf8_lossy(&wsl_list.stdout);
    if !output_str.contains("Ubuntu-22.04") {
        // Remplace tes println! par ceci :
        app_handle.emit("setup-progress", "📦 Installation d'Ubuntu 22.04...").unwrap();
        // On utilise l'installateur silencieux
        Command::new("wsl")
            .args(["--install", "-d", "Ubuntu-22.04", "--no-launch"]) 
            .status()
            .map_err(|e| format!("Erreur install Ubuntu: {}", e))?;
    }

    // --- ÉTAPE 2 : Le Kernel Custom (Injection) ---
    app_handle.emit("setup-progress", "🧠 Injection du Kernel Calyx...").unwrap();
    let app_data = dirs::data_dir().ok_or("Impossible d'accéder à AppData")?;
    let kernel_dir = app_data.join("CalyxEngine").join("kernel");
    fs::create_dir_all(&kernel_dir).map_err(|e| e.to_string())?;
    
    let target_bzimage = kernel_dir.join("calyx-kernel");

    // 1. FORCE SHUTDOWN : On éteint WSL pour libérer le verrou sur le fichier calyx-kernel
    app_handle.emit("setup-progress", "⚠️ Arrêt de WSL pour libérer le noyau...").unwrap();
    let _ = Command::new("wsl")
        .args(["--shutdown"])
        .creation_flags(0x08000000)
        .status();

    // Petit délai pour laisser Windows libérer le descripteur de fichier
    std::thread::sleep(std::time::Duration::from_millis(500));

    // 2. COPIE : Maintenant que WSL est éteint, le fichier n'est plus utilisé, on peut le copier sans erreur
    // On récupère le bzImage depuis les ressources du package Tauri
    let resource_path = app_handle.path().resolve("resources/calyx-kernel", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "Le fichier calyx-kernel est manquant dans les ressources de l'app")?;

    // On copie le noyau vers le dossier fixe seulement s'il est différent ou absent
    fs::copy(resource_path, &target_bzimage).map_err(|e| e.to_string())?;

    // Mise à jour du .wslconfig
    let user_home = dirs::home_dir().ok_or("Dossier utilisateur introuvable")?;
    let wsl_config_path = user_home.join(".wslconfig");
    
    let kernel_path_escaped = target_bzimage.to_str().unwrap().replace("\\", "\\\\");
    let config_content = format!(
        "[wsl2]
        kernel={}
        nestedVirtualization=true
        memory=4GB", kernel_path_escaped
    );

    fs::write(wsl_config_path, config_content).map_err(|e| e.to_string())?;

    // --- ÉTAPE 3 : Docker dans Ubuntu ---
    app_handle.emit("setup-progress", "🐳 Vérification de Docker...").unwrap();
    // On vérifie si docker répond dans la distro
    let docker_check = Command::new("wsl")
        .args(["-d", "Ubuntu-22.04", "which", "docker"])
        .output()
        .map_err(|e| e.to_string())?;

    if !docker_check.status.success() {
        app_handle.emit("setup-progress", "🛠️ Installation de Docker (côté Linux)...").unwrap();
        let install_script = "curl -fsSL https://get.docker.com -o get-docker.sh && sudo sh get-docker.sh && sudo usermod -aG docker $USER";
        Command::new("wsl")
            .args(["-d", "Ubuntu-22.04", "bash", "-c", install_script])
            .status()
            .map_err(|e| e.to_string())?;
    }

    // --- ÉTAPE 4 : Déploiement du script de boot dans WSL ---
    app_handle.emit("setup-progress", "📜 Déploiement du script de boot dans WSL...").unwrap();

    // 1. Résoudre le chemin de la ressource sur Windows
    let script_resource_path = app_handle.path()
        .resolve("resources/android_boot_script.sh", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "Le script de boot est manquant dans les ressources")?;

    println!("📍 Chemin résolu par Tauri : {:?}", script_resource_path);

    // 2. Lire le contenu et forcer le format de fin de ligne Linux (LF)
    // Indispensable pour éviter l'erreur "\r: command not found" dans WSL
    let script_content = std::fs::read_to_string(&script_resource_path)
        .map_err(|e| format!("Erreur de lecture du script : {}", e))?
        .replace("\r\n", "\n");

    // 3. Injecter le script dans WSL via l'entrée standard (stdin)
    // On utilise 'cat' pour créer le fichier et 'chmod' pour le rendre exécutable
    let mut child = Command::new("wsl")
        .args(["-d", "Ubuntu-22.04", "bash", "-c", "cat > ~/android_boot_script.sh && chmod +x ~/android_boot_script.sh"])
        .stdin(std::process::Stdio::piped())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .spawn()
        .map_err(|e| format!("Erreur lors de l'injection du script dans WSL : {}", e))?;

    // Envoi du contenu du script vers WSL
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(script_content.as_bytes()).map_err(|e| e.to_string())?;
    }

    child.wait().map_err(|e| format!("Erreur lors de la finalisation du transfert : {}", e))?;

    // --- FINALISATION ---
    // On force un shutdown pour que le nouveau Kernel soit chargé au prochain lancement
    app_handle.emit("setup-progress", "♻️ Redémarrage de WSL pour appliquer le noyau...").unwrap();
    let _ = Command::new("wsl").arg("--shutdown").status();
    app_handle.emit("setup-progress", "✅ Setup terminé ! Environnement prêt !").unwrap();

    println!("💉 Injection des modules réseau dans le noyau 6.6...");
    
    let modules = [
        "iptable_nat", 
        "nf_nat", 
        "xt_nat", 
        "xt_addrtype", 
        "xt_MASQUERADE", 
        "nft_masq", 
        "xt_conntrack", 
        "nf_conntrack", 
        "nf_conntrack_netlink", 
        "iptable_filter", 
        "br_netfilter",
        "nft_compat"
    ];

    for module in modules {
        Command::new("wsl")
            .args(["-d", "Ubuntu-22.04", "-u", "root", "modprobe", module])
            .creation_flags(0x08000000)
            .status()
            .ok(); // On ignore si certains modules sont déjà intégrés ("built-in")
    }

    Ok(())
}