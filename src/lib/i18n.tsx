import { create } from "zustand";

export type Locale = "fr" | "en";

interface I18nStore {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (key: string) => string;
}

const translations: Record<Locale, Record<string, string>> = {
  fr: {
    // Titlebar
    "app.name": "Castly",

    // Sidebar
    "sidebar.devices": "Appareils",
    "sidebar.scan": "Scanner",
    "sidebar.scanning": "Recherche en cours...",
    "sidebar.no_device": "Aucun appareil detecte",
    "sidebar.help_hint": "Cliquez sur ? pour le guide de connexion",
    "sidebar.start_scan": "Lancer la detection",
    "sidebar.hint": "Double-cliquer pour lancer le miroir",
    "sidebar.wifi": "Wi-Fi",
    "sidebar.wifi_auto": "Passer en Wi-Fi",
    "sidebar.wifi_manual": "Connexion par IP",
    "sidebar.wifi_ip_placeholder": "IP:port",
    "sidebar.wifi_connect": "Connecter",
    "sidebar.wifi_connecting": "Connexion...",
    "sidebar.wifi_success": "Connecte !",
    "sidebar.wifi_error": "Echec de connexion",
    "sidebar.wifi_pair_addr": "Port assoc.",
    "sidebar.wifi_pair_code": "Code 6 chiffres",
    "sidebar.wifi_connect_port": "Port connexion",
    "sidebar.wifi_pair": "Associer",
    "sidebar.debug_title": "Activer le debogage USB :",
    "sidebar.debug_1": "1. Ouvrir {Parametres > A propos}",
    "sidebar.debug_2": "2. Taper 7x sur {Numero de build}",
    "sidebar.debug_3": "3. Retour dans {Options developpeur}",
    "sidebar.debug_4": "4. Activer {Debogage USB}",
    "sidebar.debug_5": "5. Brancher le telephone en USB",
    "sidebar.debug_6": "6. Autoriser le debogage sur le telephone",

    // Viewport
    "viewport.no_mirror": "Aucun miroir actif",
    "viewport.hint": "Selectionnez un appareil et double-cliquez",
    "viewport.connecting": "Connexion en cours...",
    "viewport.connecting_hint": "Lancement du serveur sur l'appareil",

    // ControlBar
    "control.back": "Retour",
    "control.home": "Accueil",
    "control.recent": "Recent",
    "control.record": "Enregistrer",
    "control.stop_record": "Arreter",
    "control.screenshot": "Capture",
    "control.disconnect": "Deconnecter",
    "control.screen_off": "Eteindre l'ecran",
    "control.screen_on": "Allumer l'ecran",

    // StatusBar
    "status.ready": "Pret",
    "status.rec": "REC",

    // Language
    // Settings
    "settings.quality": "Qualite",
    "settings.performance": "Eco",
    "settings.performance_desc": "720p 30fps — Economie batterie",
    "settings.balanced": "Equilibre",
    "settings.balanced_desc": "1280p 30fps — Usage normal",
    "settings.quality_preset": "Qualite",
    "settings.quality_desc": "1920p 60fps — Meilleure image",

    // Help
    "help.title": "Guide de connexion",
    "help.tab_usb": "USB (Android)",
    "help.tab_wifi": "Wi-Fi (Android)",
    "help.tab_airplay": "AirPlay (iOS)",
    "help.usb_intro": "Connectez votre telephone Android en USB pour le mirroring avec la latence la plus faible.",
    "help.usb_step1": "Sur votre telephone, allez dans Parametres > A propos du telephone",
    "help.usb_step2": "Tapez 7 fois sur Numero de build pour activer les options developpeur",
    "help.usb_step3": "Retournez dans Parametres > Options developpeur",
    "help.usb_step4": "Activez Debogage USB",
    "help.usb_step5": "Branchez votre telephone au PC avec un cable USB",
    "help.usb_step6": "Sur votre telephone, autorisez le debogage USB quand la popup apparait",
    "help.usb_step7": "Votre appareil apparait dans la liste — double-cliquez pour lancer le miroir",
    "help.usb_screenshot": "Capture : options developpeur",
    "help.wifi_intro": "Connectez votre telephone Android en Wi-Fi pour un mirroring sans fil. Les deux appareils doivent etre sur le meme reseau.",
    "help.wifi_step1": "Sur votre telephone, allez dans Parametres > Options developpeur > Debogage sans fil",
    "help.wifi_step2": "Activez Debogage sans fil",
    "help.wifi_step3": "Appuyez sur Associer l'appareil avec un code d'association",
    "help.wifi_step4": "Notez l'adresse IP, le port d'association, le code, et le port de connexion",
    "help.wifi_step5": "Entrez ces informations dans la section Wi-Fi de Castly",
    "help.wifi_step6": "Cliquez Associer — votre appareil apparait dans la liste",
    "help.wifi_screenshot": "Capture : debogage sans fil",
    "help.airplay_intro": "Partagez l'ecran de votre iPhone ou iPad via AirPlay. Aucune installation requise sur l'appareil Apple.",
    "help.airplay_step1": "Assurez-vous que le PC et l'iPhone sont sur le meme reseau Wi-Fi",
    "help.airplay_step2": "Sur l'iPhone, ouvrez le Centre de controle (glissez depuis le coin superieur droit)",
    "help.airplay_step3": "Appuyez sur Recopie de l'ecran",
    "help.airplay_step4": "Selectionnez Castly dans la liste",
    "help.airplay_note": "Le mirroring AirPlay est en affichage uniquement — le controle tactile n'est pas disponible.",
    "help.airplay_screenshot": "Capture : recopie de l'ecran",

    "lang.label": "Langue",
  },
  en: {
    // Titlebar
    "app.name": "Castly",

    // Sidebar
    "sidebar.devices": "Devices",
    "sidebar.scan": "Scan",
    "sidebar.scanning": "Scanning...",
    "sidebar.no_device": "No device detected",
    "sidebar.help_hint": "Click ? for the connection guide",
    "sidebar.start_scan": "Start scanning",
    "sidebar.hint": "Double-click to start mirroring",
    "sidebar.wifi": "Wi-Fi",
    "sidebar.wifi_auto": "Switch to Wi-Fi",
    "sidebar.wifi_manual": "Connect by IP",
    "sidebar.wifi_ip_placeholder": "IP:port",
    "sidebar.wifi_connect": "Connect",
    "sidebar.wifi_connecting": "Connecting...",
    "sidebar.wifi_success": "Connected!",
    "sidebar.wifi_error": "Connection failed",
    "sidebar.wifi_pair_addr": "Pair port",
    "sidebar.wifi_pair_code": "6-digit code",
    "sidebar.wifi_connect_port": "Connect port",
    "sidebar.wifi_pair": "Pair",
    "sidebar.debug_title": "Enable USB debugging:",
    "sidebar.debug_1": "1. Open {Settings > About phone}",
    "sidebar.debug_2": "2. Tap 7 times on {Build number}",
    "sidebar.debug_3": "3. Go back to {Developer options}",
    "sidebar.debug_4": "4. Enable {USB debugging}",
    "sidebar.debug_5": "5. Plug your phone via USB",
    "sidebar.debug_6": "6. Allow debugging on your phone",

    // Viewport
    "viewport.no_mirror": "No active mirror",
    "viewport.hint": "Select a device and double-click",
    "viewport.connecting": "Connecting...",
    "viewport.connecting_hint": "Starting server on device",

    // ControlBar
    "control.back": "Back",
    "control.home": "Home",
    "control.recent": "Recent",
    "control.record": "Record",
    "control.stop_record": "Stop",
    "control.screenshot": "Screenshot",
    "control.disconnect": "Disconnect",
    "control.screen_off": "Turn screen off",
    "control.screen_on": "Turn screen on",

    // StatusBar
    "status.ready": "Ready",
    "status.rec": "REC",

    // Language
    // Settings
    "settings.quality": "Quality",
    "settings.performance": "Eco",
    "settings.performance_desc": "720p 30fps — Battery saver",
    "settings.balanced": "Balanced",
    "settings.balanced_desc": "1280p 30fps — Normal use",
    "settings.quality_preset": "Quality",
    "settings.quality_desc": "1920p 60fps — Best image",

    // Help
    "help.title": "Connection Guide",
    "help.tab_usb": "USB (Android)",
    "help.tab_wifi": "Wi-Fi (Android)",
    "help.tab_airplay": "AirPlay (iOS)",
    "help.usb_intro": "Connect your Android phone via USB for the lowest latency mirroring.",
    "help.usb_step1": "On your phone, go to Settings > About phone",
    "help.usb_step2": "Tap Build number 7 times to enable Developer options",
    "help.usb_step3": "Go back to Settings > Developer options",
    "help.usb_step4": "Enable USB debugging",
    "help.usb_step5": "Plug your phone into the PC with a USB cable",
    "help.usb_step6": "On your phone, allow USB debugging when the popup appears",
    "help.usb_step7": "Your device appears in the list — double-click to start mirroring",
    "help.usb_screenshot": "Screenshot: developer options",
    "help.wifi_intro": "Connect your Android phone via Wi-Fi for wireless mirroring. Both devices must be on the same network.",
    "help.wifi_step1": "On your phone, go to Settings > Developer options > Wireless debugging",
    "help.wifi_step2": "Enable Wireless debugging",
    "help.wifi_step3": "Tap Pair device with pairing code",
    "help.wifi_step4": "Note the IP address, pairing port, code, and connection port",
    "help.wifi_step5": "Enter this information in the Wi-Fi section of Castly",
    "help.wifi_step6": "Click Pair — your device appears in the list",
    "help.wifi_screenshot": "Screenshot: wireless debugging",
    "help.airplay_intro": "Share your iPhone or iPad screen via AirPlay. No installation needed on the Apple device.",
    "help.airplay_step1": "Make sure the PC and iPhone are on the same Wi-Fi network",
    "help.airplay_step2": "On the iPhone, open Control Center (swipe down from top-right corner)",
    "help.airplay_step3": "Tap Screen Mirroring",
    "help.airplay_step4": "Select Castly from the list",
    "help.airplay_note": "AirPlay mirroring is display-only — touch control is not available.",
    "help.airplay_screenshot": "Screenshot: screen mirroring",

    "lang.label": "Language",
  },
};

export const useI18n = create<I18nStore>((set, get) => ({
  locale: (localStorage.getItem("locale") as Locale) || "fr",
  setLocale: (locale) => {
    localStorage.setItem("locale", locale);
    set({ locale });
  },
  t: (key) => {
    const { locale } = get();
    return translations[locale][key] ?? key;
  },
}));

/** Render a translation string with {highlighted} segments */
export function renderHighlighted(text: string) {
  const parts = text.split(/\{([^}]+)\}/);
  return parts.map((part, i) =>
    i % 2 === 1 ? (
      <span key={i} className="text-text-secondary">
        {part}
      </span>
    ) : (
      part
    ),
  );
}
