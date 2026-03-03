use std::process::Command;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tauri::Emitter;
use std::os::windows::process::CommandExt;

// Constante pour éviter les fenêtres CMD intempestives sur Windows
const NO_WINDOW: u32 = 0x08000000;

pub async fn full_setup_check(app_handle: tauri::AppHandle) -> Result<(), String> {
    app_handle.emit("setup-progress", "🚀 Démarrage de la maintenance Calyx...").unwrap();

    // --- ÉTAPE 1 : Préparation des dossiers Windows ---
    let app_data = dirs::data_dir().ok_or("Impossible d'accéder à AppData")?;
    let kernel_dir = app_data.join("CalyxEngine").join("kernel");
    fs::create_dir_all(&kernel_dir).map_err(|e| format!("Erreur dossier AppData: {}", e))?;
    
    let target_kernel = kernel_dir.join("calyx-kernel");
    let resource_kernel = app_handle.path().resolve("resources/calyx-kernel", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "Noyau calyx-kernel introuvable dans les ressources")?;

    // --- ÉTAPE 2 : Synchronisation du Noyau & .wslconfig ---
    app_handle.emit("setup-progress", "🧠 Synchronisation du Kernel personnalisé...").unwrap();

    // On éteint WSL UNE SEULE FOIS ici pour libérer tous les verrous
    let _ = Command::new("wsl").args(["--shutdown"]).creation_flags(NO_WINDOW).status();
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Copie du noyau
    fs::copy(&resource_kernel, &target_kernel).map_err(|e| format!("Échec copie noyau: {}", e))?;

    // Écriture du .wslconfig (8GB RAM pour ton Ryzen 5 3600)
    let user_home = dirs::home_dir().ok_or("Dossier utilisateur introuvable")?;
    let wsl_config_path = user_home.join(".wslconfig");
    let kernel_path_escaped = target_kernel.to_str().unwrap().replace("\\", "\\\\");
    
    let config_content = format!(
        "[wsl2]\nkernel={}\nnestedVirtualization=true\nmemory=8GB\nprocessors=12", 
        kernel_path_escaped
    );
    fs::write(wsl_config_path, config_content).map_err(|e| format!("Erreur .wslconfig: {}", e))?;

    // --- ÉTAPE 3 : Initialisation Linux (Distro & Docker) ---
    app_handle.emit("setup-progress", "🐳 Initialisation de l'environnement Linux...").unwrap();

    let wsl_list = Command::new("wsl").args(["--list", "--verbose"]).output().map_err(|e| e.to_string())?;
    if !String::from_utf8_lossy(&wsl_list.stdout).contains("Ubuntu-22.04") {
        app_handle.emit("setup-progress", "📦 Installation d'Ubuntu 22.04...").unwrap();
        Command::new("wsl").args(["--install", "-d", "Ubuntu-22.04", "--no-launch"]).creation_flags(NO_WINDOW).status().ok();
    }

    let docker_check = Command::new("wsl").args(["-d", "Ubuntu-22.04", "which", "docker"]).output().ok();
    if docker_check.is_none() || !docker_check.unwrap().status.success() {
        app_handle.emit("setup-progress", "🛠️ Installation de Docker...").unwrap();
        let install_cmd = "curl -fsSL https://get.docker.com -o get-docker.sh && sudo sh get-docker.sh && sudo usermod -aG docker $USER";
        Command::new("wsl").args(["-d", "Ubuntu-22.04", "bash", "-c", install_cmd]).creation_flags(NO_WINDOW).status().ok();
    }

    // --- ÉTAPE 4 : Montage du Système Nerveux (Binderfs) ---
    app_handle.emit("setup-progress", "💉 Activation de Binderfs...").unwrap();
    
    let binder_cmds = [
        "mkdir -p /dev/binderfs",
        "mount -t binder none /dev/binderfs",
        "chmod 666 /dev/binderfs/*" // Donne les droits à Docker
    ];

    for cmd in binder_cmds {
        Command::new("wsl")
            .args(["-d", "Ubuntu-22.04", "-u", "root", "bash", "-c", cmd])
            .creation_flags(NO_WINDOW)
            .status().ok();
    }

    // --- ÉTAPE 5 : Injection des modules réseau (Ton code exact) ---
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
            .creation_flags(NO_WINDOW)
            .status()
            .ok(); // On ignore si certains modules sont déjà intégrés ("built-in")
    }

    // --- ÉTAPE 6 : Injection du Script de Boot ---
    app_handle.emit("setup-progress", "📜 Déploiement du script de boot...").unwrap();
    deploy_boot_script(&app_handle).await?;

    app_handle.emit("setup-progress", "✅ Setup terminé ! Prêt pour l'automatisation.").unwrap();
    Ok(())
}

async fn deploy_boot_script(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let script_res = app_handle.path().resolve("resources/android_boot_script.sh", tauri::path::BaseDirectory::Resource)
        .map_err(|_| "Script de boot manquant")?;

    let content = std::fs::read_to_string(&script_res)
        .map_err(|e| e.to_string())?
        .replace("\r\n", "\n"); // Conversion vitale pour Linux

    let mut child = Command::new("wsl")
        .args(["-d", "Ubuntu-22.04", "bash", "-c", "cat > ~/android_boot_script.sh && chmod +x ~/android_boot_script.sh"])
        .stdin(std::process::Stdio::piped())
        .creation_flags(NO_WINDOW)
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
    }
    child.wait().map_err(|e| e.to_string())?;
    Ok(())
}