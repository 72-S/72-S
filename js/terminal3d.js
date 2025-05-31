import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { ModelManager } from "./model.js";
import { DebugManager } from "./debug.js";
import { EffectsManager } from "./effects.js";
import { CRTShader } from "./crt-shader.js";
import { AnimationManager } from "./animation.js";

export class Terminal3D {
  constructor(options = {}) {
    this.debugEnabled = options.debug ?? false;

    const defaultValues = this.debugEnabled
      ? this.getDebugDefaults()
      : this.getNormalDefaults();

    this.crtSettings = {
      bulge: options.bulge ?? defaultValues.bulge,
      scanlineIntensity:
        options.scanlineIntensity ?? defaultValues.scanlineIntensity,
      scanlineCount: options.scanlineCount ?? defaultValues.scanlineCount,
      vignetteIntensity:
        options.vignetteIntensity ?? defaultValues.vignetteIntensity,
      vignetteRadius: options.vignetteRadius ?? defaultValues.vignetteRadius,
      glowIntensity: options.glowIntensity ?? defaultValues.glowIntensity,
      glowColor: options.glowColor ?? new THREE.Vector3(0.0, 0.02, 0.02),
      brightness: options.brightness ?? defaultValues.brightness,
      contrast: options.contrast ?? defaultValues.contrast,
      offsetX: options.offsetX ?? defaultValues.offsetX,
      offsetY: options.offsetY ?? defaultValues.offsetY,
    };

    this.scenePosition = {
      x: options.sceneX ?? 0,
      y: options.sceneY ?? 0,
      z: options.sceneZ ?? 0,
    };

    this.scene = null;
    this.camera = null;
    this.renderer = null;
    this.controls = null;
    this.terminalTexture = null;
    this.terminalCanvas = null;
    this.hiddenInput = null;
    this.isTerminalFocused = false;
    this.animationId = null;
    this.isHoveringScreen = false;
    this.scrollHistory = [];
    this.maxScrollHistory = 50;
    this.currentScrollPosition = 0;

    this.modelManager = null;
    this.debugManager = null;
    this.effectsManager = null;
    this.crtShader = null;
    this.animationManager = null;

    this.raycaster = null;
    this.mouse = null;

    this.defaultCameraPosition = new THREE.Vector3(4, 5, 10);
    this.defaultCameraTarget = new THREE.Vector3(0, 1.5, 0);

    this.init();
  }

  getDebugDefaults() {
    return {
      bulge: 0,
      scanlineIntensity: 0,
      scanlineCount: 0,
      vignetteIntensity: 0,
      vignetteRadius: 0,
      glowIntensity: 0,
      brightness: 1.0,
      contrast: 1.0,
      offsetX: 0,
      offsetY: 0,
    };
  }

  getNormalDefaults() {
    return {
      bulge: 0.9,
      scanlineIntensity: 0.1,
      scanlineCount: 800,
      vignetteIntensity: 0.3,
      vignetteRadius: 0.3,
      glowIntensity: 0.02,
      brightness: 1.0,
      contrast: 1.1,
      offsetX: 1.2,
      offsetY: 0.0,
    };
  }

  async init() {
    try {
      this.updateLoadingProgress(10);
      await this.setupScene();
      this.updateLoadingProgress(30);

      this.modelManager = new ModelManager(this.scene, (progress) =>
        this.updateLoadingProgress(progress),
      );
      this.effectsManager = new EffectsManager(
        this.renderer,
        this.scene,
        this.camera,
      );
      this.debugManager = new DebugManager(this);

      await this.modelManager.loadPCModel();
      this.updateLoadingProgress(60);

      await this.setupTerminalTexture();
      this.updateLoadingProgress(80);

      this.setupEventListeners();
      await this.effectsManager.trySetupPostProcessing();

      this.animationManager = new AnimationManager(
        this.camera,
        this.controls,
        (newState) => {},
      );

      if (this.debugEnabled) {
        this.debugManager.createDebugGUI();
      }

      this.updateLoadingProgress(100);
      this.hideLoading();
      this.showTerminal();
      this.animationManager.startupAnimation();
      this.animate();
    } catch (e) {
      console.error("3D init failed:", e);
      this.showError();
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
    this.camera.position.copy(this.defaultCameraPosition);

    this.renderer = new THREE.WebGLRenderer({
      antialias: true,
      powerPreference: "high-performance",
      alpha: true,
    });
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.5;
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.gammaFactor = 2.2;

    document
      .getElementById("scene-container")
      .appendChild(this.renderer.domElement);

    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.05;
    this.controls.enableZoom = false;
    this.controls.enablePan = true;
    this.controls.enableRotate = true;
    this.controls.target.copy(this.defaultCameraTarget);
    this.controls.minDistance = 3;
    this.controls.maxDistance = 20;
    this.controls.minPolarAngle = Math.PI * 0.05;
    this.controls.maxPolarAngle = Math.PI * 0.75;
    this.controls.minAzimuthAngle = -Math.PI * 0.75;
    this.controls.maxAzimuthAngle = Math.PI * 0.75;

    this.setupLighting();
  }

  setupLighting() {
    const ambientLight = new THREE.AmbientLight(0x6c7b95, 0.9);
    this.scene.add(ambientLight);

    const mainLight = new THREE.DirectionalLight(0xffffff, 2.2);
    mainLight.position.set(8, 10, 6);
    mainLight.castShadow = true;
    mainLight.shadow.mapSize.setScalar(2048);
    mainLight.shadow.bias = -0.0001;
    mainLight.shadow.camera.near = 0.1;
    mainLight.shadow.camera.far = 50;
    mainLight.shadow.camera.left = mainLight.shadow.camera.bottom = -15;
    mainLight.shadow.camera.right = mainLight.shadow.camera.top = 15;
    this.scene.add(mainLight);

    const fillLight = new THREE.DirectionalLight(0x8be9fd, 1.0);
    fillLight.position.set(-5, 6, -4);
    this.scene.add(fillLight);

    const monitorLight = new THREE.PointLight(0x50fa7b, 2.2, 15);
    monitorLight.position.set(0, 3, 2);
    this.scene.add(monitorLight);

    const rimLight = new THREE.DirectionalLight(0x50fa7b, 0.8);
    rimLight.position.set(0, 4, -8);
    this.scene.add(rimLight);

    const accentLight1 = new THREE.PointLight(0x8be9fd, 1.8, 18);
    accentLight1.position.set(4, 4, 4);
    this.scene.add(accentLight1);

    const accentLight2 = new THREE.PointLight(0x50fa7b, 1.5, 15);
    accentLight2.position.set(-4, 3, 2);
    this.scene.add(accentLight2);

    const hemiLight = new THREE.HemisphereLight(0x8be9fd, 0x6c5ce7, 0.8);
    hemiLight.position.set(0, 25, 0);
    this.scene.add(hemiLight);

    const frontLight = new THREE.DirectionalLight(0xf8f8ff, 1.2);
    frontLight.position.set(0, 8, 12);
    this.scene.add(frontLight);
  }

  async setupTerminalTexture() {
    this.terminalCanvas = document.getElementById("terminal");
    this.hiddenInput = document.getElementById("hidden-input");

    if (!this.terminalCanvas || !this.hiddenInput) {
      console.error("Terminal elements not found!");
      return;
    }

    await new Promise((r) => requestAnimationFrame(r));

    this.terminalTexture = new THREE.CanvasTexture(this.terminalCanvas);
    this.terminalTexture.minFilter = THREE.LinearFilter;
    this.terminalTexture.magFilter = THREE.LinearFilter;
    this.terminalTexture.flipY = true;

    this.crtShader = new CRTShader(this.terminalTexture, this.crtSettings);

    if (this.modelManager.screenMesh) {
      this.modelManager.screenMesh.material =
        this.crtShader.createCRTMaterial();
      this.crtShader.applyRealBulgeToScreen(this.modelManager.screenMesh);
    }

    this.terminalTexture.offset.y = 0.0;
    this.terminalTexture.repeat.y = 1.0;

    this.updateTerminalTexture = () => {
      try {
        if (this.terminalTexture) {
          this.terminalTexture.needsUpdate = true;
        }
      } catch (e) {
        console.warn("Texture update failed:", e);
      }
    };

    setInterval(() => {
      this.updateTerminalTexture();
    }, 16);
  }

  updateCRTSettings(newSettings) {
    if (this.crtShader) {
      this.crtShader.updateSettings(newSettings, this.modelManager.screenMesh);
    }

    if (this.debugManager) {
      this.debugManager.updateDebugCenter();
    }
  }

  updateScenePosition(axis, value) {
    this.scenePosition[axis] = value;

    if (this.modelManager && this.modelManager.pcModel) {
      if (axis === "x") {
        this.modelManager.pcModel.position.x = this.scenePosition.x;
      } else if (axis === "y") {
        this.modelManager.pcModel.position.y = this.scenePosition.y;
      } else if (axis === "z") {
        this.modelManager.pcModel.position.z = this.scenePosition.z;
      }
    }
  }

  toggleDebugPanel() {
    if (this.debugManager) {
      this.debugManager.toggleDebugPanel();
    }
  }

  setupEventListeners() {
    window.addEventListener("resize", () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);
      this.effectsManager.onResize();
    });

    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();

    this.renderer.domElement.addEventListener("mousemove", (event) => {
      this.animationManager.stopIdleRotation();

      this.mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
      this.mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;

      this.raycaster.setFromCamera(this.mouse, this.camera);

      const intersectables = [];
      if (this.modelManager.pcModel) {
        this.modelManager.pcModel.traverse((child) => {
          if (child.isMesh) {
            intersectables.push(child);
          }
        });
      }

      const intersects = this.raycaster.intersectObjects(intersectables);

      if (intersects.length > 0) {
        const hoveredObject = intersects[0].object;
        const isScreen = this.modelManager.isScreenObject(hoveredObject);

        if (isScreen) {
          this.renderer.domElement.style.cursor = "pointer";
          this.isHoveringScreen = true;
        } else {
          this.renderer.domElement.style.cursor = "grab";
          this.isHoveringScreen = false;
        }
      } else {
        this.renderer.domElement.style.cursor = "default";
        this.isHoveringScreen = false;
      }
    });

    this.renderer.domElement.addEventListener("click", (event) => {
      this.animationManager.stopIdleRotation();

      this.mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
      this.mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;

      this.raycaster.setFromCamera(this.mouse, this.camera);

      const intersectables = [];
      if (this.modelManager.pcModel) {
        this.modelManager.pcModel.traverse((child) => {
          if (child.isMesh) {
            intersectables.push(child);
          }
        });
      }

      const intersects = this.raycaster.intersectObjects(intersectables);

      if (intersects.length > 0) {
        const clickedObject = intersects[0].object;
        const isScreen = this.modelManager.isScreenObject(clickedObject);

        if (isScreen) {
          this.isTerminalFocused = true;
          this.animationManager.animateToState("focused");

          if (this.hiddenInput) {
            this.hiddenInput.focus();
          }

          const focusEvent = new CustomEvent("terminalFocus");
          window.dispatchEvent(focusEvent);
        } else {
          this.isTerminalFocused = false;
          this.animationManager.animateToState("default");

          if (this.hiddenInput) {
            this.hiddenInput.blur();
          }

          const blurEvent = new CustomEvent("terminalBlur");
          window.dispatchEvent(blurEvent);
        }
      }
    });

    this.renderer.domElement.addEventListener("wheel", (event) => {
      if (this.isHoveringScreen) {
        event.preventDefault();
        this.animationManager.stopIdleRotation();

        const currentSnapshot = this.captureTerminalSnapshot();
        if (
          this.scrollHistory.length === 0 ||
          this.scrollHistory[this.scrollHistory.length - 1] !== currentSnapshot
        ) {
          this.scrollHistory.push(currentSnapshot);
          if (this.scrollHistory.length > this.maxScrollHistory) {
            this.scrollHistory.shift();
          }
        }

        if (event.deltaY < 0) {
          this.scrollUp();
        } else {
          this.scrollDown();
        }
      }
    });

    window.addEventListener("terminalFocus", () => {
      this.isTerminalFocused = true;
      if (this.hiddenInput) {
        this.hiddenInput.focus();
      }
    });

    window.addEventListener("terminalBlur", () => {
      this.isTerminalFocused = false;
      if (this.hiddenInput) {
        this.hiddenInput.blur();
      }
    });
  }

  captureTerminalSnapshot() {
    if (!this.terminalCanvas) return null;

    try {
      return {
        timestamp: Date.now(),
      };
    } catch (e) {
      console.warn("Failed to capture terminal snapshot:", e);
      return null;
    }
  }

  scrollUp() {
    if (this.scrollHistory.length === 0) return;

    if (this.currentScrollPosition < this.scrollHistory.length - 1) {
      this.currentScrollPosition++;

      const scrollEvent = new CustomEvent("terminalScrollUp", {
        detail: { position: this.currentScrollPosition },
      });
      window.dispatchEvent(scrollEvent);
    }
  }

  scrollDown() {
    if (this.currentScrollPosition > 0) {
      this.currentScrollPosition--;

      const scrollEvent = new CustomEvent("terminalScrollDown", {
        detail: { position: this.currentScrollPosition },
      });
      window.dispatchEvent(scrollEvent);
    } else {
      this.currentScrollPosition = 0;
      const scrollEvent = new CustomEvent("terminalScrollToBottom");
      window.dispatchEvent(scrollEvent);
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

  animate() {
    this.animationId = requestAnimationFrame(() => this.animate());

    if (this.controls) {
      this.controls.update();
    }

    if (this.crtShader) {
      this.crtShader.updateTime(
        Date.now() * 0.001,
        this.modelManager.screenMesh,
      );
    }

    if (this.animationManager) {
      this.animationManager.updateIdleRotation(this.isTerminalFocused);
      this.animationManager.checkAndCorrectCameraBounds();
    }

    if (this.modelManager) {
      this.modelManager.updateSceneFadeIn();
    }

    if (this.effectsManager) {
      this.effectsManager.updateManualBloom(this.modelManager.screenMesh);
      this.effectsManager.render();
    }
  }

  dispose() {
    if (this.animationId) cancelAnimationFrame(this.animationId);
    if (this.renderer) this.renderer.dispose();
    if (this.effectsManager) this.effectsManager.dispose();
    if (this.terminalTexture) this.terminalTexture.dispose();
    if (this.crtShader) this.crtShader.dispose();
    if (this.debugManager) this.debugManager.dispose();
  }
}
