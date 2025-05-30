import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

export class Terminal3D {
  constructor() {
    this.scene = null;
    this.camera = null;
    this.renderer = null;
    this.controls = null;
    this.pcModel = null;
    this.screenMesh = null;
    this.terminalTexture = null;
    this.isTerminalFocused = false;
    this.animationId = null;
    this.init();
  }

  async init() {
    try {
      this.updateLoadingProgress(10);
      await this.setupScene();
      this.updateLoadingProgress(30);
      await this.loadPCModel();
      this.updateLoadingProgress(60);
      await this.setupTerminalTexture();
      this.updateLoadingProgress(80);
      this.setupEventListeners();
      this.updateLoadingProgress(100);
      this.hideLoading();
      this.showTerminal();
      this.animate();
    } catch (e) {
      console.error("3D init failed:", e);
      this.showError();
    }
  }

  updateLoadingProgress(p) {
    const bar = document.getElementById("loading-progress");
    if (bar) bar.style.width = p + "%";
  }

  hideLoading() {
    setTimeout(() => {
      const L = document.getElementById("loading");
      if (L) L.classList.add("hidden");
    }, 500);
  }

  showError() {
    const txt = document.querySelector(".loading-text");
    if (txt) {
      txt.textContent = "Failed to load 3D. Showing terminal.";
      txt.style.color = "#ff5555";
    }
    setTimeout(() => {
      const L = document.getElementById("loading");
      if (L) L.classList.add("hidden");
      const term = document.getElementById("terminal");
      if (term) term.style.visibility = "visible";
    }, 2000);
  }

  showTerminal() {
    const terminal = document.getElementById("terminal");
    if (terminal) {
      terminal.style.visibility = "visible";
    }
  }

  async setupScene() {
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x0a0a0a);
    this.camera = new THREE.PerspectiveCamera(
      45,
      window.innerWidth / window.innerHeight,
      0.1,
      100,
    );
    this.camera.position.set(0, 1.5, 3);
    this.renderer = new THREE.WebGLRenderer({ antialias: true });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    document
      .getElementById("scene-container")
      .appendChild(this.renderer.domElement);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.target.set(0, 1, 0);

    this.scene.add(new THREE.HemisphereLight(0x8be9fd, 0x444444, 0.3));
  }

  async loadPCModel() {
    console.log("Loading PC model...");
    return new Promise((resolve) => {
      const loader = new GLTFLoader();
      loader.load(
        "./pc.glb",
        (gltf) => {
          console.log("Model loaded");
          this.pcModel = gltf.scene;
          this.pcModel.scale.setScalar(1);
          this.pcModel.position.set(0, 0, 0);

          this.pcModel.traverse((c) => {
            if (c.isMesh) {
              c.castShadow = c.receiveShadow = true;
              const n = c.name.toLowerCase();
              const m = c.material?.name?.toLowerCase() || "";
              if (
                n.includes("screen") ||
                n.includes("monitor") ||
                m.includes("screen") ||
                m.includes("monitor") ||
                c.name === "Plane008_Material002_0"
              ) {
                this.screenMesh = c;
                console.log("Found screen:", c.name);
              }
            }
          });
          if (!this.screenMesh) {
            this.findScreenMesh();
          }
          this.scene.add(this.pcModel);
          resolve();
        },
        (prog) => {
          const pct = 30 + (prog.loaded / prog.total) * 30;
          this.updateLoadingProgress(pct);
        },
        (err) => {
          console.warn("Model load failed:", err);
          resolve();
        },
      );
    });
  }

  findScreenMesh() {
    const candidates = [];
    this.pcModel.traverse((c) => {
      if (c.isMesh && c.geometry) {
        const box = new THREE.Box3().setFromObject(c);
        const s = box.getSize(new THREE.Vector3());
        const flat = Math.min(s.x, s.y, s.z) < Math.max(s.x, s.y, s.z) * 0.1;
        const big = Math.max(s.x, s.y, s.z) > 0.5;
        if (flat && big) candidates.push({ mesh: c, area: s.x * s.y * s.z });
      }
    });
    if (candidates.length) {
      candidates.sort((a, b) => b.area - a.area);
      this.screenMesh = candidates[0].mesh;
    } else {
      this.pcModel.traverse((c) => {
        if (!this.screenMesh && c.isMesh) this.screenMesh = c;
      });
    }
  }

  async setupTerminalTexture() {
    const t = document.getElementById("terminal");
    if (t) t.classList.add("texture-mode");
    await new Promise((r) => requestAnimationFrame(r));

    const canvas = document.createElement("canvas");
    canvas.width = 1024;
    canvas.height = 768;
    canvas.style.position = "absolute";
    canvas.style.top = "0";
    canvas.style.left = "0";
    canvas.style.zIndex = "1000";
    canvas.style.border = "1px solid red";
    document.body.appendChild(canvas);
    const ctx = canvas.getContext("2d");
    this.terminalTexture = new THREE.CanvasTexture(canvas);
    this.terminalTexture.minFilter = THREE.LinearFilter;
    this.terminalTexture.magFilter = THREE.LinearFilter;
    this.terminalTexture.flipY = true;

    if (this.screenMesh) {
      this.screenMesh.material = new THREE.MeshBasicMaterial({
        map: this.terminalTexture,
        emissive: new THREE.Color(0x001a1a),
        emissiveIntensity: 0.1,
      });
    }

    this.terminalTexture.offset.y = 0.2;
    this.terminalTexture.repeat.y = 0.9;

    this.updateTerminalTexture = () => {
      try {
        const terminalBody = document.getElementById("terminal-body");
        const terminalOutput = document.getElementById("terminal-output");

        if (!terminalBody || !terminalOutput) return;

        ctx.fillStyle = "#0a0a0a";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        const scrollTop = terminalBody.scrollTop;
        const bodyHeight = terminalBody.clientHeight;

        const lineHeight = parseInt(ctx.font.match(/\d+/)[0], 10) + 4;

        const startLine = Math.floor(scrollTop / lineHeight);
        const visibleLines = Math.ceil(bodyHeight / lineHeight) + 2; // +2 buffer

        const outputDivs = Array.from(terminalOutput.children);

        ctx.font = "16px 'JetBrains Mono', monospace";

        ctx.strokeStyle = "#ff0000";
        ctx.strokeRect(0, 0, canvas.width, canvas.height);

        let y = 20;
        let lineIndex = 0;

        if (y + lineHeight > canvas.height) {
          y = canvas.height - lineHeight - 10; // give a margin
        }

        for (let i = 0; i < outputDivs.length; i++) {
          const div = outputDivs[i];
          const text = div.textContent || "";

          const maxChars = Math.floor((canvas.width - 40) / 10);
          const lines = [];
          if (text.length > maxChars) {
            for (let j = 0; j < text.length; j += maxChars) {
              lines.push(text.substring(j, j + maxChars));
            }
          } else {
            lines.push(text);
          }

          for (const line of lines) {
            if (
              lineIndex >= startLine &&
              lineIndex < startLine + visibleLines
            ) {
              if (div.classList.contains("command")) {
                ctx.fillStyle = "#8be9fd";
              } else if (div.classList.contains("error")) {
                ctx.fillStyle = "#ff5555";
              } else if (div.classList.contains("success")) {
                ctx.fillStyle = "#50fa7b";
              } else if (div.classList.contains("warning")) {
                ctx.fillStyle = "#ffb86c";
              } else if (div.classList.contains("info")) {
                ctx.fillStyle = "#bd93f9";
              } else {
                ctx.fillStyle = "#e6e6e6";
              }

              ctx.fillText(line, 20, y);
              y += lineHeight;
            }
            lineIndex++;
          }

          console.log({
            scrollTop,
            clientHeight: terminalBody.clientHeight,
            scrollHeight: terminalBody.scrollHeight,
            outputLines: outputDivs.length,
            startLine,
            visibleLines,
          });
        }

        const terminalInput = document.getElementById("terminal-input");
        if (terminalInput) {
          const promptElements = document.querySelectorAll(".prompt");
          const currentPrompt = promptElements[promptElements.length - 1];
          const promptText = currentPrompt
            ? currentPrompt.textContent
            : "objz:~$ ";
          const inputValue = terminalInput.value;

          if (lineIndex >= startLine && lineIndex < startLine + visibleLines) {
            ctx.fillStyle = "#8be9fd";
            ctx.fillText(promptText, 20, y);

            ctx.fillStyle = "#f8f8f2";
            const promptWidth = ctx.measureText(promptText).width;
            ctx.fillText(inputValue, 20 + promptWidth, y);

            if (document.activeElement === terminalInput) {
              const inputWidth = ctx.measureText(inputValue).width;
              ctx.fillStyle = "#ffffff";
              ctx.fillRect(20 + promptWidth + inputWidth, y - 15, 2, 18);
            }
          }
        }

        this.terminalTexture.needsUpdate = true;
      } catch (e) {
        console.warn("Texture update failed:", e);
      }
    };

    this.updateTerminalTexture();
    setInterval(() => {
      this.updateTerminalTexture();
    }, 100);
  }

  setupEventListeners() {
    window.addEventListener("resize", () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);
    });

    window.objzEnsureAutoscroll = () => {
      const terminalBody = document.getElementById("terminal-body");
      if (terminalBody) {
        terminalBody.scrollTop = terminalBody.scrollHeight;
      }
    };

    const terminalOutput = document.getElementById("terminal-output");
    if (terminalOutput) {
      const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
          if (mutation.type === "childList" && mutation.addedNodes.length > 0) {
            setTimeout(() => {
              window.objzEnsureAutoscroll();
            }, 0);
          }
        });
      });

      observer.observe(terminalOutput, {
        childList: true,
        subtree: true,
      });
    }

    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();

    this.renderer.domElement.addEventListener("click", (event) => {
      this.mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
      this.mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;

      this.raycaster.setFromCamera(this.mouse, this.camera);

      const intersectables = [];
      if (this.pcModel) {
        this.pcModel.traverse((child) => {
          if (child.isMesh) {
            intersectables.push(child);
          }
        });
      }
      if (this.screenMesh && !intersectables.includes(this.screenMesh)) {
        intersectables.push(this.screenMesh);
      }

      const intersects = this.raycaster.intersectObjects(intersectables);

      const terminalInput = document.getElementById("terminal-input");

      if (intersects.length > 0) {
        const clickedObject = intersects[0].object;
        console.log(
          "Clicked object:",
          clickedObject.name || "unnamed",
          clickedObject,
        );

        const isScreen =
          clickedObject.name === "Plane008_Material002_0" ||
          clickedObject === this.screenMesh ||
          clickedObject.material?.map === this.terminalTexture ||
          (clickedObject.name &&
            clickedObject.name.toLowerCase().includes("screen"));

        if (isScreen) {
          console.log("Screen clicked - focusing terminal input");
          if (terminalInput) {
            terminalInput.focus();
            this.isTerminalFocused = true;
          }
        } else {
          console.log("Clicked on non-screen object - removing focus");
          if (terminalInput) {
            terminalInput.blur();
            this.isTerminalFocused = false;
          }
        }
      } else {
        console.log("Clicked on empty space - removing focus");
        if (terminalInput) {
          terminalInput.blur();
          this.isTerminalFocused = false;
        }
      }
    });

    document.addEventListener("click", (e) => {
      const terminalInput = document.getElementById("terminal-input");

      if (
        !e.target.closest("#scene-container") &&
        !e.target.closest("#terminal")
      ) {
        console.log("Clicked outside - removing focus");
        if (terminalInput) {
          terminalInput.blur();
          this.isTerminalFocused = false;
        }
      }
    });

    document.addEventListener("keydown", (e) => {
      const terminalInput = document.getElementById("terminal-input");

      if (
        terminalInput &&
        this.isTerminalFocused &&
        document.activeElement !== terminalInput &&
        !e.ctrlKey &&
        !e.altKey &&
        !e.metaKey &&
        e.key.length === 1
      ) {
        terminalInput.focus();
      }
    });
  }

  animate() {
    this.animationId = requestAnimationFrame(() => this.animate());
    this.controls.update();
    this.renderer.render(this.scene, this.camera);
  }

  dispose() {
    if (this.animationId) cancelAnimationFrame(this.animationId);
    if (this.renderer) this.renderer.dispose();
  }
}
