<<<<<<< HEAD
# Tauri + React + Typescript

This template should help get you started developing with Tauri, React and Typescript in Vite.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
=======
# Calyx 🌿

> **The invisible bridge for Android on your Desktop.**

**Calyx** est un hôte d'applications Android (APK) conçu pour Windows et macOS. Contrairement aux émulateurs traditionnels (comme BlueStacks), Calyx élimine l'interface utilisateur lourde et intrusive pour offrir une expérience de "fenêtrage natif".

L'objectif est simple : **L'utilisateur ne doit jamais voir qu'un émulateur tourne en arrière-plan.**

---

### ✨ Pourquoi Calyx ?

Les solutions actuelles sont souvent :

* ❌ **Lourdes :** Elles consomment énormément de RAM avec des interfaces inutiles.
* ❌ **Inesthétiques :** Elles forcent l'utilisateur dans un "bureau Android" secondaire.
* ❌ **Fragmentées :** L'intégration avec le système d'exploitation hôte est médiocre.

**Calyx change la donne :**

* ✅ **Zero-UI Engine :** Le moteur Android est totalement invisible (Headless).
* ✅ **Native-Like Windows :** Chaque APK s'ouvre dans sa propre fenêtre isolée, avec son icône dans la barre des tâches.
* ✅ **Performance-First :** Utilisation des hyperviseurs natifs (WSL2/Hypervisor.framework) pour une latence minimale.

---

### 🛠️ Architecture Technique

Calyx repose sur une pile technologique moderne pour garantir rapidité et discrétion :

| Composant | Technologie | Rôle |
| --- | --- | --- |
| **Interface** | **Tauri (Rust)** | Gestionnaire d'APK ultra-léger et fluide. |
| **Runtime** | **WSL2 / Virtio** | Couche de virtualisation invisible et performante. |
| **Display Bridge** | **Modified scrcpy** | Extraction de fenêtre individuelle sans bordures d'émulateur. |
| **Translation** | **Libhoudini/NDK** | Support des applications ARM sur processeurs x86. |

---

### 🚀 Fonctionnalités prévues

* [ ] **Drag & Drop Installer :** Glissez un APK, il est prêt à l'emploi.
* [ ] **App Sidedownloading :** Gestionnaire de téléchargement et d'hébergement local.
* [ ] **Multi-Window Support :** Lancez plusieurs apps Android côte à côte avec vos apps Windows/Mac.
* [ ] **System Integration :** Partage du presse-papier, notifications natives et accès aux fichiers.

---

### 🏗️ Comment ça marche ? (Concept)

1. **Le "Calyx Core"** démarre une instance Android minimale sans interface graphique au lancement du système.
2. Lorsque vous lancez une application via **l'interface Calyx**, le moteur envoie une commande de lancement via ADB.
3. Le **Display Bridge** capture uniquement le flux vidéo de l'application cible et l'injecte dans une fenêtre native gérée par Tauri.

---

### 🤝 Contribuer

Le projet est en phase de conception initiale. Toutes les idées sur l'optimisation de la couche de compatibilité sont les bienvenues !

---

**Est-ce que cette présentation te convient pour ton dépôt GitHub ?** Si oui, on peut passer à la **Phase 1 du développement** : Je peux t'aider à configurer la structure de ton projet (les dossiers, le fichier de configuration de base) pour que tu puisses commencer à coder les fondations de **Calyx**.
>>>>>>> origin/main
