game-settings-description = Gère les paramètres en jeu et les sessions
sandbox-settings-description = Lance le jeu dans une sandbox bubblewrap, qui fonctionne comme Flatpak
environment-settings-description = Spécifie les variables d'environnement et la commande qui lance le jeu

wine = Wine

synchronization = Synchronisation
wine-sync-description = Technologie utilisée pour synchroniser les évènements wine internes

language = Langue
wine-lang-description = Langue utilisée dans l'environnement wine. Peut résoudre des problèmes de clavier
system = Système

borderless-window = Utiliser une fenêtre sans bordure
virtual-desktop = Bureau virtuel

map-drive-c = Map drive C:
map-drive-c-description = Lien symbolique automatique du drive_c du préfixe wine vers celui de dosdevices

map-game-folder = Dossier des maps de jeux
map-game-folder-description = Lien symbolique automatique du dossier de jeu vers les dosdevices

game = Jeu

hud = HUD

fsr = FSR
fsr-description = Permet d'upscale le jeu à la taille de l'écran. Pour l'utiliser, sélectionnez une résolution plus basse en jeu, et appuyez sur Alt+Entrée
ultra-quality = Qualité ultra
quality = Qualité
balanced = Équilibré
performance = Performances

gamemode = Gamemode
gamemode-description = Donne la priorité au jeu sur le reste des processus du système

gamescope = Gamescope
gamescope-description = Gamescope est un outil fait par Valve qui permet aux jeux de se lancer dans une instance Xwayland isolée, et qui est compatible avec les cartes graphiques AMD, Intel et NVidia

gpu-device = GPU
gpu-description = Select which GPU to launch the game on. Forces the Vulkan device so the game uses the selected GPU on hybrid laptops. Restart the game to apply.
gpu-integrated = Integrated (default)
gpu-dedicated-nvidia = Dedicated (NVIDIA)

discord-rpc = Activité Discord
discord-rpc-description = Permet à Discord d'afficher à vos amis des informations sur le jeu auquel vous jouez actuellement
icon = Icône
title = Titre
description = Description

fps-unlocker = Déblocage des FPS
fps-unlocker-description = Enlève les limitations de FPS en modifiant la mémoire du jeu. Peut être détecté par l'anticheat

enabled = Activer

fps-unlocker-interval = Overwrite interval
fps-unlocker-interval-description = Délai en millisecondes entre les écrasages de la valeur limite de FPS. Des écrasages périodiques sont nécessaires l'empêcher de se réinitialiser

window-mode = Type de fenêtre
borderless = Sans bordure
headless = Sans fenêtre
popup = Popup
fullscreen = Plein écran

winewayland = Activer Winewayland
winewayland-description = Utiliser le pilote Wayland au lieu de X11 (définit DISPLAY="")
winewayland-unavailable-tooltip = Wayland n'est pas disponible. Si vous utilisez flatpak, vérifiez dans FlatSeal que l'appli a bien accès au socket Wayland.
