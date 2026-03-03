#!/bin/bash
echo "🌿 Nettoyage et préparation de Calyx Engine..."

# On supprime TOUTE instance précédente (tournante ou arrêtée)
docker rm -f calyx-engine 2>/dev/null

echo "🚀 Lancement de l'instance Android..."

# On retire Vulkan pour l'instant (cause fréquente de crash en mode guest)
docker run -d --privileged \
    --name calyx-engine \
    -v /tmp/calyx-data:/data \
    -p 5555:5555 \
    redroid/redroid:11.0.0-latest \
    androidboot.redroid_gpu_mode=guest \
    androidboot.redroid_fps=60 \
    androidboot.redroid_width=1080 \
    androidboot.redroid_height=1920 \
    androidboot.redroid_dpi=480

echo "✅ Calyx Engine est en ligne !"
echo "En attente de commandes sur le port 5555..."