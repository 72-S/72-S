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

    // lighting
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

          // enable shadows & find screen
          this.pcModel.traverse((c) => {
            if (c.isMesh) {
              c.castShadow = c.receiveShadow = true;
              const n = c.name.toLowerCase();
              const m = c.material?.name?.toLowerCase() || "";
              if (
                n.includes("screen") ||
                n.includes("monitor") ||
                m.includes("screen") ||
                m.includes("monitor")
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
          console.warn("Model load failed, using fallback:", err);
          this.createFallbackPC();
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

  createFallbackPC() {
    // Create a simple fallback PC model if GLB fails to load
    const group = new THREE.Group();

    // Monitor
    const monitorGeometry = new THREE.BoxGeometry(2, 1.2, 0.1);
    const monitorMaterial = new THREE.MeshBasicMaterial({ color: 0x333333 });
    const monitor = new THREE.Mesh(monitorGeometry, monitorMaterial);
    monitor.position.set(0, 1, 0);
    group.add(monitor);

    // Screen (this will be our texture target)
    const screenGeometry = new THREE.PlaneGeometry(1.8, 1);
    const screenMaterial = new THREE.MeshBasicMaterial({ color: 0x000000 });
    const screen = new THREE.Mesh(screenGeometry, screenMaterial);
    screen.position.set(0, 1, 0.06);
    this.screenMesh = screen;
    group.add(screen);

    // Base
    const baseGeometry = new THREE.CylinderGeometry(0.3, 0.3, 0.1, 8);
    const baseMaterial = new THREE.MeshBasicMaterial({ color: 0x444444 });
    const base = new THREE.Mesh(baseGeometry, baseMaterial);
    base.position.set(0, 0.05, 0);
    group.add(base);

    this.pcModel = group;
    this.scene.add(this.pcModel);
  }

  async setupTerminalTexture() {
    const t = document.getElementById("terminal");
    if (t) t.classList.add("texture-mode");
    await new Promise((r) => requestAnimationFrame(r));

    const canvas = document.createElement("canvas");
    canvas.width = 1024;
    canvas.height = 768;
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

    this.updateTerminalTexture = () => {
      try {
        const terminalBody = document.getElementById("terminal-body");
        const terminalOutput = document.getElementById("terminal-output");

        if (!terminalBody || !terminalOutput) return;

        // Clear canvas
        ctx.fillStyle = "#0a0a0a";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        // Get scroll info
        const scrollTop = terminalBody.scrollTop;
        const bodyHeight = terminalBody.clientHeight;
        const lineHeight = 20;

        // Calculate visible lines
        const startLine = Math.floor(scrollTop / lineHeight);
        const visibleLines = Math.ceil(bodyHeight / lineHeight) + 2; // +2 buffer

        // Get all terminal lines
        const outputDivs = Array.from(terminalOutput.children);

        // Set font
        ctx.font = "16px 'JetBrains Mono', monospace";

        let y = 20;
        let lineIndex = 0;

        // Draw visible content
        for (let i = 0; i < outputDivs.length; i++) {
          const div = outputDivs[i];
          const text = div.textContent || "";

          // Split long lines
          const maxChars = Math.floor((canvas.width - 40) / 10);
          const lines = [];
          if (text.length > maxChars) {
            for (let j = 0; j < text.length; j += maxChars) {
              lines.push(text.substring(j, j + maxChars));
            }
          } else {
            lines.push(text);
          }

          // Draw lines that are in visible range
          for (const line of lines) {
            if (
              lineIndex >= startLine &&
              lineIndex < startLine + visibleLines
            ) {
              // Set color based on CSS classes
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
        }

        // Draw current input line
        const terminalInput = document.getElementById("terminal-input");
        if (terminalInput) {
          const promptElements = document.querySelectorAll(".prompt");
          const currentPrompt = promptElements[promptElements.length - 1];
          const promptText = currentPrompt
            ? currentPrompt.textContent
            : "objz:~$ ";
          const inputValue = terminalInput.value;

          // Only draw if this line would be visible
          if (lineIndex >= startLine && lineIndex < startLine + visibleLines) {
            ctx.fillStyle = "#8be9fd"; // Prompt color
            ctx.fillText(promptText, 20, y);

            ctx.fillStyle = "#f8f8f2"; // Input color
            const promptWidth = ctx.measureText(promptText).width;
            ctx.fillText(inputValue, 20 + promptWidth, y);

            // Draw cursor if focused
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

    // Update texture regularly
    this.updateTerminalTexture();
    setInterval(() => {
      this.updateTerminalTexture();
    }, 100); // More frequent updates for better responsiveness
  }

  setupEventListeners() {
    // Window resize
    window.addEventListener("resize", () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);
    });

    // Simple click handling - focus terminal when clicking anywhere
    const terminalInput = document.getElementById("terminal-input");

    // Click anywhere to focus terminal
    document.addEventListener("click", (e) => {
      if (terminalInput && !e.target.closest("#scene-container canvas")) {
        terminalInput.focus();
      }
    });

    // Key press to focus terminal
    document.addEventListener("keydown", (e) => {
      if (
        terminalInput &&
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
