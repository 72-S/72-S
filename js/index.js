import init from "../pkg/portfolio.js";
import { Terminal3D } from "./terminal3d.js";

init().then(() => {
  console.log("Portfolio loaded successfully!");
});

document.addEventListener("DOMContentLoaded", () => {
  setTimeout(() => {
    window.terminal3d = new Terminal3D({
      bulge: 0.9,
      scanlineIntensity: 0.02,
      scanlineCount: 640,
      vignetteIntensity: 0.3,
      vignetteRadius: 0.26,
      glowIntensity: 0.005,
      glowColor: {
        x: 0,
        y: 0.01,
        z: 0.01,
      },
      brightness: 0.85,
      contrast: 1.05,
      offsetX: 0.54,
      offsetY: 0.7,
      sceneX: 0,
      sceneY: 0,
      sceneZ: 0,
    });

    window.toggleDebug = () => {
      if (window.terminal3d) {
        window.terminal3d.toggleDebugPanel();
      }
    };

    document.addEventListener("keydown", (event) => {
      if ((event.ctrlKey || event.metaKey) && event.key === "d") {
        event.preventDefault();
        window.toggleDebug();
      }
    });

    console.log("3D Terminal initialized!");
    console.log("Use Ctrl+D or Cmd+D to toggle debug panel");
    console.log("Or call window.toggleDebug() in console");
  }, 1000);
});

window.addEventListener("beforeunload", () => {
  if (window.terminal3d) {
    window.terminal3d.dispose();
  }
});

export { Terminal3D };
