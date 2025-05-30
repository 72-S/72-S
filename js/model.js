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
    this.terminalCanvas = null; // Add reference to terminal canvas
    this.hiddenInput = null; // Add reference to hidden input element
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
    // Get the terminal canvas directly (created by your Rust code)
    this.terminalCanvas = document.getElementById("terminal");
    // Get the hidden input element (used by Rust for input handling)
    this.hiddenInput = document.getElementById("hidden-input");

    if (!this.terminalCanvas) {
      console.error("Terminal canvas not found!");
      return;
    }

    if (!this.hiddenInput) {
      console.error("Hidden input element not found!");
      return;
    }

    // Wait a frame to ensure canvas is ready
    await new Promise((r) => requestAnimationFrame(r));

    // Create Three.js texture from the existing canvas
    this.terminalTexture = new THREE.CanvasTexture(this.terminalCanvas);
    this.terminalTexture.minFilter = THREE.LinearFilter;
    this.terminalTexture.magFilter = THREE.LinearFilter;
    this.terminalTexture.flipY = true; // Canvas is already correct orientation

    // Apply texture to screen mesh
    if (this.screenMesh) {
      this.screenMesh.material = new THREE.MeshBasicMaterial({
        map: this.terminalTexture,
        emissive: new THREE.Color(0x001a1a),
        emissiveIntensity: 0.1,
      });
    }

    // Adjust texture mapping if needed
    this.terminalTexture.offset.y = 0.0;
    this.terminalTexture.repeat.y = 1.0;

    // Set up automatic texture updates
    this.updateTerminalTexture = () => {
      try {
        // The canvas is automatically updated by your Rust terminal renderer
        // We just need to tell Three.js to update the texture
        if (this.terminalTexture) {
          this.terminalTexture.needsUpdate = true;
        }
      } catch (e) {
        console.warn("Texture update failed:", e);
      }
    };

    // Update texture regularly to sync with terminal changes
    setInterval(() => {
      this.updateTerminalTexture();
    }, 16); // ~60fps updates
  }

  setupEventListeners() {
    window.addEventListener("resize", () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);
    });

    // Set up raycasting for screen clicks
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
          this.isTerminalFocused = true;

          // Focus the hidden input element (this is what handles the actual input)
          if (this.hiddenInput) {
            this.hiddenInput.focus();
          }

          // Trigger any terminal focus events if your Rust code needs them
          const focusEvent = new CustomEvent("terminalFocus");
          window.dispatchEvent(focusEvent);
        } else {
          console.log("Clicked on non-screen object - removing focus");
          this.isTerminalFocused = false;

          // Blur the hidden input element
          if (this.hiddenInput) {
            this.hiddenInput.blur();
          }

          const blurEvent = new CustomEvent("terminalBlur");
          window.dispatchEvent(blurEvent);
        }
      } else {
        console.log("Clicked on empty space - removing focus");
        this.isTerminalFocused = false;

        // Blur the hidden input element
        if (this.hiddenInput) {
          this.hiddenInput.blur();
        }

        const blurEvent = new CustomEvent("terminalBlur");
        window.dispatchEvent(blurEvent);
      }
    });

    // Handle clicks outside the 3D scene
    document.addEventListener("click", (e) => {
      if (
        !e.target.closest("#scene-container") &&
        !e.target.closest("#terminal") &&
        !e.target.closest("#hidden-input")
      ) {
        console.log("Clicked outside - removing focus");
        this.isTerminalFocused = false;

        // Blur the hidden input element
        if (this.hiddenInput) {
          this.hiddenInput.blur();
        }

        const blurEvent = new CustomEvent("terminalBlur");
        window.dispatchEvent(blurEvent);
      }
    });

    // Listen for custom focus/blur events from Rust code if needed
    window.addEventListener("terminalFocus", () => {
      console.log("Terminal focus event received");
      this.isTerminalFocused = true;
      if (this.hiddenInput) {
        this.hiddenInput.focus();
      }
    });

    window.addEventListener("terminalBlur", () => {
      console.log("Terminal blur event received");
      this.isTerminalFocused = false;
      if (this.hiddenInput) {
        this.hiddenInput.blur();
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
    if (this.terminalTexture) this.terminalTexture.dispose();
  }
}
