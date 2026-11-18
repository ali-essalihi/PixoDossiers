# 🤖 Pixo et les Dossiers (Rust + Tauri v2)

Application éducative hors-ligne pour Windows 10 et Windows 11 (64-bit) permettant aux élèves débutants d'apprendre à créer, organiser, renommer et supprimer des fichiers et des dossiers directement dans l'Explorateur Windows, en équipe et chacun son tour.

---

## ✨ Fonctionnalités

- **100 % hors-ligne** : aucune connexion Internet requise en classe, polices locales (`Fredoka` et `Nunito`) et icônes SVG Twemoji embarquées.
- **Travail en groupe (2 à 6 élèves)** : passage tour par tour avec avatar personnalisé, sauvegarde automatique dans `%APPDATA%\PixoDossiers\state.json`, gestion des groupes via **`Ctrl + Maj + G`**.
- **5 Niveaux progressifs sur le vrai système de fichiers Windows** :
  1. **Niveau 1 — Tri 🗂️** : Trier le dossier `Téléchargements` vers `Images` (4 `.png`), `Vidéos` (2 `.mp4`), `Musique` (2 `.mp3`) et mettre les 2 fichiers brouillons (`.tmp`, `.bak`) à la `Corbeille`.
  2. **Niveau 2 — Nourriture 🍎** : Dans `Images`, créer les dossiers `fruits`, `legumes` et `boissons` et y ranger les 14 images `.jpg`.
  3. **Niveau 3 — Ordinateur 🖥️** : Créer le dossier `ordinateur` sur le `Bureau` et y déplacer les 4 images (`unite-centrale.png`, `clavier.png`, `souris.png`, `ecran.png`) depuis `Images`.
  4. **Niveau 4 — Présentation 📝** : Créer l'arborescence `Documents/<NomDuGroupe>/<Prénom>` et y enregistrer un fichier `.txt` contenant `Bonjour, je suis <Prénom>`.
  5. **Niveau 5 — Dessin Paint 🎨** : Réaliser un dessin dans Paint (`mspaint`) et l'enregistrer au format PNG sous `Documents/<NomDuGroupe>/<Prénom>/mon-dessin.png`.

---

## 📦 Téléchargement et Installation (Enseignants)

Rendez-vous dans l'onglet **[Releases](../../releases)** de ce dépôt GitHub :
- **Installateur Windows (`*.msi` ou `*-setup.exe`)** : installe l'application sur le PC.
- **Version Portable (`PixoDossiers_Windows_x64_Portable.zip`)** : **aucune installation ni droits administrateur requis** — décompressez simplement le fichier ZIP et double-cliquez sur `PixoDossiers.exe`.
- **`SHA256SUMS.txt`** : empreintes SHA-256 officielles pour vérifier l'intégrité des fichiers téléchargés.

---

## 📁 Liste des 22 fichiers `payload/`

Tous les fichiers sources des exercices sont placés à plat dans le dossier `payload/` :
- **Images PNG (4 fichiers, partagés entre les Niveaux 1 et 3)** : `clavier.png`, `souris.png`, `ecran.png`, `unite-centrale.png`
- **Vidéos MP4 (2 fichiers, Niveau 1)** : `imprimante-demo.mp4`, `scanner-demo.mp4`
- **Sons MP3 (2 fichiers, Niveau 1)** : `pluie.mp3`, `tonnerre.mp3`
- **Images JPG (14 fichiers, Niveau 2)** : `pomme.jpg`, `banane.jpg`, `orange.jpg`, `fraise.jpg`, `citron.jpg`, `cerise.jpg`, `carotte.jpg`, `pomme_de_terre.jpg`, `brocoli.jpg`, `salade.jpg`, `eau.jpg`, `jus_de_pomme.jpg`, `cafe.jpg`, `the.jpg`
