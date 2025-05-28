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
    this.terminalTexture.flipY = true; // Fix upside-down content

    if (this.screenMesh) {
      this.screenMesh.material = new THREE.MeshBasicMaterial({
        map: this.terminalTexture,
        emissive: new THREE.Color(0x001a1a), // Dark cyan glow
        emissiveIntensity: 0.1,
      });
    }

    this.updateTerminalTexture = () => {
      try {
        // Match original terminal background color
        ctx.fillStyle = "#0a0a0a";
        ctx.fillRect(0, 0, canvas.width, canvas.height);

        // No header - start content directly
        ctx.fillStyle = "#8be9fd"; // Original cyan color
        ctx.font = "16px 'JetBrains Mono'";

        const lines = Array.from(
          document.querySelectorAll("#terminal-output div"),
        ).map((d) => d.textContent || "");

        lines.forEach((ln, i) => {
          // Use original terminal colors
          ctx.fillStyle = "#e6e6e6"; // Default text color
          if (ln.includes("objz@")) {
            ctx.fillStyle = "#8be9fd"; // Prompt color
          }
          if (ln.includes("error") || ln.includes("Error")) {
            ctx.fillStyle = "#ff5555"; // Error color
          }
          if (ln.includes("success") || ln.includes("Success")) {
            ctx.fillStyle = "#50fa7b"; // Success color
          }
          ctx.fillText(ln, 10, 20 + i * 20);
        });

        this.terminalTexture.needsUpdate = true;
      } catch (e) {
        console.warn("Texture update failed:", e);
      }
    };

    this.updateTerminalTexture();
    setInterval(() => {
      if (!this.isTerminalFocused) this.updateTerminalTexture();
    }, 500);
  }

  setupEventListeners() {
    window.addEventListener("resize", () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);
    });

    const ray = new THREE.Raycaster();
    const mouse = new THREE.Vector2();
    this.renderer.domElement.addEventListener("click", (e) => {
      mouse.x = (e.clientX / window.innerWidth) * 2 - 1;
      mouse.y = -(e.clientY / window.innerHeight) * 2 + 1;
      ray.setFromCamera(mouse, this.camera);
      ray.intersectObjects(this.scene.children, true).forEach((inter) => {
        if (inter.object === this.screenMesh) {
          this.updateTerminalTexture();
        }
      });
    });
  }

  animate() {
    this.animationId = requestAnimationFrame(() => this.animate());
    this.controls.update();
    // Auto-spinning removed as requested
    this.renderer.render(this.scene, this.camera);
  }

  dispose() {
    if (this.animationId) cancelAnimationFrame(this.animationId);
    if (this.renderer) this.renderer.dispose();
  }
}
