#!/bin/bash
# android_boot_script.sh

echo "🌿 Préparation du démarrage de Calyx Engine..."

# 1. Vérifier si une ancienne instance tourne déjà et la nettoyer
if [ "$(docker ps -q -f name=calyx-engine)" ]; then
    echo "Une instance tourne déjà. Arrêt en cours..."
    docker stop calyx-engine
fi

echo "🚀 Lancement de l'instance Android minimale..."

# 2. Lancement du conteneur en tâche de fond (-d)
# On expose le port 5555 pour qu'ADB (depuis Windows) puisse s'y connecter
docker run -d --privileged \
    --name calyx-engine \
    --privileged \
    -v /tmp/calyx-data:/data \
    -p 5555:5555 \
    redroid/redroid:11.0.0-latest \
    androidboot.redroid_gpu_mode=guest \
    androidboot.redroid_fps=60 \
    androidboot.redroid_width=1080 \
    androidboot.redroid_height=1920 \
    androidboot.redroid_dpi=480 \
    androidboot.use_vulkan=1 # Prépare le terrain pour l'accélération matérielle de ta RTX 3060

echo "✅ Calyx Engine est en ligne !"
echo "En attente de commandes sur le port 5555..."