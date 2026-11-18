/* Data-driven worlds and levels definition for Pixo et les Dossiers */
window.PIXO_DATA = {
  /* Pool of distinct animal/character avatars for group members */
  avatars: ['🦊', '🐼', '🐸', '🦁', '🐙', '🐯', '🐨', '🐰', '🦉', '🦄', '🐢', '🐬'],

  /* Worlds table: adding a world later = adding one object here.
     Level numbers are computed globally (1, 2, 3...) across all worlds. */
  worlds: [
    {
      title: 'Gestion des dossiers',
      icon: '📁',
      levels: [
        {
          id: 1,
          title: 'Tri',
          icon: '🗂️',
          part: '🗂️',
          pn: 'son trieur rapide',
          wipeWarning: true,
          wipeFoldersText: 'Téléchargements, Images, Musique et Vidéos',
          steps: [
            { text: 'Ouvre le dossier Téléchargements' },
            { text: 'Déplace chaque fichier au bon endroit' },
            { text: 'Mets les 2 brouillons à la Corbeille' }
          ],
          tree: {
            name: 'Ordinateur',
            icon: '📂',
            children: [
              {
                name: 'Images',
                icon: '📂',
                children: [
                  { name: 'clavier.png', kind: 'png' },
                  { name: 'souris.png', kind: 'png' },
                  { name: 'ecran.png', kind: 'png' },
                  { name: 'unite-centrale.png', kind: 'png' }
                ]
              },
              {
                name: 'Vidéos',
                icon: '📂',
                children: [
                  { name: 'imprimante-demo.mp4', kind: 'mp4' },
                  { name: 'scanner-demo.mp4', kind: 'mp4' }
                ]
              },
              {
                name: 'Musique',
                icon: '📂',
                children: [
                  { name: 'pluie.mp3', kind: 'mp3' },
                  { name: 'tonnerre.mp3', kind: 'mp3' }
                ]
              },
              {
                name: 'Corbeille (2 fichiers inutiles)',
                icon: '🗑️',
                children: []
              }
            ]
          }
        },
        {
          id: 2,
          title: 'Nourriture',
          icon: '🍎',
          part: '🍎',
          pn: 'son panier gourmand',
          wipeWarning: true,
          wipeFoldersText: 'Images',
          steps: [
            { text: 'Ouvre le dossier Images' },
            { text: 'Crée fruits, legumes et boissons' },
            { text: 'Range chaque image au bon endroit' }
          ],
          tree: {
            name: 'Images',
            icon: '📂',
            children: [
              {
                name: 'fruits',
                icon: '📂',
                create: true,
                children: [
                  { name: 'pomme.jpg', kind: 'jpg' },
                  { name: 'banane.jpg', kind: 'jpg' },
                  { name: 'orange.jpg', kind: 'jpg' },
                  { name: 'fraise.jpg', kind: 'jpg' },
                  { name: 'citron.jpg', kind: 'jpg' },
                  { name: 'cerise.jpg', kind: 'jpg' }
                ]
              },
              {
                name: 'legumes',
                icon: '📂',
                create: true,
                children: [
                  { name: 'carotte.jpg', kind: 'jpg' },
                  { name: 'pomme_de_terre.jpg', kind: 'jpg' },
                  { name: 'brocoli.jpg', kind: 'jpg' },
                  { name: 'salade.jpg', kind: 'jpg' }
                ]
              },
              {
                name: 'boissons',
                icon: '📂',
                create: true,
                children: [
                  { name: 'eau.jpg', kind: 'jpg' },
                  { name: 'jus_de_pomme.jpg', kind: 'jpg' },
                  { name: 'cafe.jpg', kind: 'jpg' },
                  { name: 'the.jpg', kind: 'jpg' }
                ]
              }
            ]
          }
        },
        {
          id: 3,
          title: 'Ordinateur',
          icon: '🖥️',
          part: '🖥️',
          pn: 'son super ordinateur',
          wipeWarning: true,
          wipeFoldersText: 'Images et le dossier ordinateur du Bureau',
          steps: [
            { text: 'Ouvre le dossier Images' },
            { text: 'Crée ordinateur sur le Bureau' },
            { text: 'Déplace les 4 images dans ordinateur' }
          ],
          tree: {
            name: 'Bureau',
            icon: '📂',
            children: [
              {
                name: 'ordinateur',
                icon: '📂',
                create: true,
                children: [
                  { name: 'unite-centrale.png', kind: 'png' },
                  { name: 'clavier.png', kind: 'png' },
                  { name: 'souris.png', kind: 'png' },
                  { name: 'ecran.png', kind: 'png' }
                ]
              }
            ]
          }
        },
        {
          id: 4,
          title: 'Présentation',
          icon: '📝',
          part: '📝',
          pn: 'sa carte de visite',
          wipeWarning: false,
          steps: [
            { text: 'Crée {group} puis {name} dans Documents' },
            { text: 'Crée un fichier texte (.txt) dedans' },
            { text: 'Écris : Bonjour, je suis {name}' }
          ],
          tree: {
            name: 'Documents',
            icon: '📂',
            children: [
              {
                name: '{group}',
                icon: '📂',
                create: true,
                children: [
                  {
                    name: '{name}',
                    icon: '📂',
                    create: true,
                    children: [
                      {
                        name: 'presentation.txt  (Texte : « Bonjour, je suis {name} »)',
                        kind: 'txt',
                        create: true
                      }
                    ]
                  }
                ]
              }
            ]
          }
        },
        {
          id: 5,
          title: 'Dessin Paint',
          icon: '🎨',
          part: '🎨',
          pn: 'son pinceau magique',
          wipeWarning: false,
          steps: [
            { text: 'Ouvre Paint et fais ton dessin' },
            { text: 'Ouvre Documents/{group}/{name}' },
            { text: 'Enregistre sous : mon-dessin.png' }
          ],
          tree: {
            name: 'Documents',
            icon: '📂',
            children: [
              {
                name: '{group}',
                icon: '📂',
                create: true,
                children: [
                  {
                    name: '{name}',
                    icon: '📂',
                    create: true,
                    children: [
                      {
                        name: 'mon-dessin.png',
                        kind: 'png',
                        create: true
                      }
                    ]
                  }
                ]
              }
            ]
          }
        }
      ]
    }
  ]
};
