import init from "../pkg/portfolio.js";
import { Terminal3D } from "./model.js";

init().then(() => {
  console.log("Portfolio loaded successfully!");
});

document.addEventListener("DOMContentLoaded", () => {
  setTimeout(() => {
    window.terminal3d = new Terminal3D();
  }, 1000);
});

window.addEventListener("beforeunload", () => {
  if (window.terminal3d) {
    window.terminal3d.dispose();
  }
});
