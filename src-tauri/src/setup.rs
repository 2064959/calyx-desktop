use std::process::Command;
use std::fs;
use tauri::Manager;
use tauri::Emitter;
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]

pub fn deploy_kernel_to_appdata(app_handle: &tauri::AppHandle) -> Result<(), String> {
    // 1. Définir le chemin cible : AppData/Roaming/CalyxEngine/kernel/
    let mut target_path = app_handle.path().app_config_dir()
        .map_err(|e| format!("Impossible de trouver le dossier AppData : {}", e))?;
    
    target_path.push("kernel");
    
    // Créer les dossiers si nécessaire
    if !target_path.exists() {
        fs::create_dir_all(&target_path)
            .map_err(|e| format!("Erreur création dossier : {}", e))?;
    }

    target_path.push("calyx-kernel");

    // 2. Résoudre le chemin du noyau dans les ressources de l'app
    let resource_path = app_handle.path().resolve("resources/calyx-kernel", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "Noyau calyx-kernel introuvable dans les ressources")?;

    // 3. Copier le fichier s'il est différent ou manquant
    // Note: Pour un moteur d'automatisation, on veut s'assurer que l'utilisateur a toujours la version la plus stable.
    if !target_path.exists() {
        println!("🚚 Déploiement initial du noyau Calyx...");
        fs::copy(&resource_path, &target_path)
            .map_err(|e| format!("Échec de la copie du noyau : {}", e))?;
    }

    Ok(())
}


pub async fn full_setup_check(app_handle: tauri::AppHandle) -> Result<(), String> {
    // --- ÉTAPE 0 : Chemins et Préparation ---
    app_handle.emit("setup-progress", "📂 Initialisation des dossiers...").unwrap();
    
    let app_data = dirs::data_dir().ok_or("Impossible d'accéder à AppData")?;
    let kernel_dir = app_data.join("CalyxEngine").join("kernel");
    fs::create_dir_all(&kernel_dir).map_err(|e| e.to_string())?;
    let target_kernel = kernel_dir.join("calyx-kernel");

    // --- ÉTAPE 1 : Déploiement du Noyau & .wslconfig ---
    // On centralise ici pour ne faire qu'un seul arrêt de WSL
    app_handle.emit("setup-progress", "🧠 Préparation du Kernel Calyx...").unwrap();
    
    let resource_kernel = app_handle.path().resolve("resources/calyx-kernel", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "Le fichier calyx-kernel est manquant dans les ressources")?;

    // On force l'arrêt pour libérer les fichiers
    let _ = Command::new("wsl").args(["--shutdown"]).creation_flags(0x08000000).status();
    std::thread::sleep(std::time::Duration::from_millis(800));

    // Copie du noyau et écriture de la config
    fs::copy(resource_kernel, &target_kernel).map_err(|e| format!("Erreur copie noyau: {}", e))?;
    
    let user_home = dirs::home_dir().ok_or("Dossier utilisateur introuvable")?;
    let wsl_config_path = user_home.join(".wslconfig");
    let kernel_path_escaped = target_kernel.to_str().unwrap().replace("\\", "\\\\");
    
    // On passe à 6GB de RAM par défaut pour être à l'aise avec Android + Docker
    let config_content = format!(
        "[wsl2]\nkernel={}\nnestedVirtualization=true\nmemory=6GB\nprocessors=12", 
        kernel_path_escaped
    );
    fs::write(wsl_config_path, config_content).map_err(|e| e.to_string())?;

    // --- ÉTAPE 2 : Vérification Distro & Docker ---
    app_handle.emit("setup-progress", "🐳 Configuration de Docker & Ubuntu...").unwrap();
    
    let wsl_list = Command::new("wsl").args(["--list", "--verbose"]).output().map_err(|e| e.to_string())?;
    if !String::from_utf8_lossy(&wsl_list.stdout).contains("Ubuntu-22.04") {
        app_handle.emit("setup-progress", "📦 Installation d'Ubuntu 22.04...").unwrap();
        Command::new("wsl").args(["--install", "-d", "Ubuntu-22.04", "--no-launch"]).status().ok();
    }

    // Installation Docker si absent
    let docker_check = Command::new("wsl").args(["-d", "Ubuntu-22.04", "which", "docker"]).output().ok();
    if docker_check.is_none() || !docker_check.unwrap().status.success() {
        app_handle.emit("setup-progress", "🛠️ Installation de Docker...").unwrap();
        let install_cmd = "curl -fsSL https://get.docker.com -o get-docker.sh && sudo sh get-docker.sh && sudo usermod -aG docker $USER";
        Command::new("wsl").args(["-d", "Ubuntu-22.04", "bash", "-c", install_cmd]).status().ok();
    }

    // --- ÉTAPE 3 : Système Nerveux (Binderfs) & Modules ---
    // C'est ici qu'on active ton noyau personnalisé !
    app_handle.emit("setup-progress", "💉 Activation des composants Android...").unwrap();
    
    let setup_cmds = [
        "sudo mkdir -p /dev/binderfs",
        "sudo mount -t binder none /dev/binderfs",
        "sudo chmod 666 /dev/binderfs/*"
    ];

    for cmd in setup_cmds {
        Command::new("wsl")
            .args(["-d", "Ubuntu-22.04", "-u", "root", "bash", "-c", cmd])
            .creation_flags(0x08000000)
            .status().ok();
    }

    // Injection des modules réseau (ton code existant est parfait ici)
    let modules = ["iptable_nat", "nf_nat", "xt_nat", "xt_MASQUERADE", "br_netfilter", "nft_compat"];
    for module in modules {
        Command::new("wsl").args(["-d", "Ubuntu-22.04", "-u", "root", "modprobe", module]).creation_flags(0x08000000).status().ok();
    }

    // --- ÉTAPE 4 : Script de Boot ---
    app_handle.emit("setup-progress", "📜 Déploiement du script de boot...").unwrap();
    deploy_boot_script(&app_handle).await?; // On isole pour la clarté

    app_handle.emit("setup-progress", "✅ Calyx Engine est prêt !").unwrap();
    Ok(())
}

async fn deploy_boot_script(app_handle: &tauri::AppHandle) -> Result<(), String> {
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

