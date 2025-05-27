import init, { greet } from "../pkg/portfolio.js";

init().then(() => {
  console.log("WASM geladen.");
  greet();
});
