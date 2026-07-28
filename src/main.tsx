import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles/tokens.css";
import "./styles/skins.css";
import "./styles/tailwind.css";
import "streamdown/styles.css";
import "./styles/app.css";
import "./styles/setup-wizard.css";
import {
  applyNativeWindowTheme,
  applyThemeToDocument,
  loadTheme,
} from "./lib/theme";
import {
  applySkinToDocument,
  applyWallpaperFlag,
  applyWallpaperScrimToDocument,
  loadSkin,
  loadWallpaperMeta,
  loadWallpaperScrim,
} from "./lib/themeSkin";

// Apply persisted theme + color skin + wallpaper flag before first paint of React tree.
const bootTheme = loadTheme(localStorage);
applyThemeToDocument(bootTheme);
applySkinToDocument(loadSkin(localStorage));
// Only the data-wallpaper flag is set synchronously (so the shell flips to
// transparent + scrim instantly). The media layer is rendered by App after
// the IndexedDB blob is loaded — no synchronous access to IDB is possible.
applyWallpaperFlag(loadWallpaperMeta(localStorage) !== null);
applyWallpaperScrimToDocument(loadWallpaperScrim(localStorage));
// Sync macOS NSAppearance / vibrancy with app theme (avoids dark glass under light UI).
void applyNativeWindowTheme(bootTheme);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
