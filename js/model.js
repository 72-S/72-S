import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

export class Terminal3D {
  constructor() {
    this.scene = null;
    this.camera = null;
    this.renderer = null;
    this.controls = null;
    this.composer = null; // Post-processing composer (fallback)
    this.pcModel = null;
    this.screenMesh = null;
    this.terminalTexture = null;
    this.terminalCanvas = null;
    this.hiddenInput = null;
    this.isTerminalFocused = false;
    this.animationId = null;
    this.isHoveringScreen = false;
    this.scrollHistory = [];
    this.maxScrollHistory = 50;
    this.currentScrollPosition = 0;

    // Camera animation states - closer focus on monitor
    this.cameraStates = {
      default: {
        position: new THREE.Vector3(4, 5, 10), // Side angle, much further
        target: new THREE.Vector3(0, 1.5, 0),
      },
      focused: {
        position: new THREE.Vector3(0.5, 3.5, 4.5), // Much closer to monitor, centered view
        target: new THREE.Vector3(0, 2.2, 0), // Look directly at monitor center
      },
      overview: {
        position: new THREE.Vector3(6, 6, 12), // Wide overview, very far
        target: new THREE.Vector3(0, 1.5, 0),
      },
      idle: {
        position: new THREE.Vector3(3, 4.5, 9), // Gentle side angle for idle rotation
        target: new THREE.Vector3(0, 1.5, 0),
      },
    };
    this.currentCameraState = "default";
    this.cameraAnimation = { isAnimating: false, progress: 0, duration: 1500 };

    // Startup animation system
    this.startupAnimation = {
      isActive: false,
      hasCompleted: false,
      delayDuration: 1000, // 1 second delay after loading
      focusDuration: 2000, // 2 seconds focus animation
      startTime: 0,
      phase: "waiting", // 'waiting', 'focusing', 'complete'
    };

    // Idle rotation system
    this.idleRotation = {
      enabled: false, // Start disabled until startup animation completes
      radius: 8, // Distance from center
      speed: 0.0003, // Very slow rotation
      angle: Math.PI * 0.25, // Start at 45 degrees
      height: 4.5,
      lastInteraction: Date.now(),
    };

    // Scene fade animation
    this.sceneOpacity = 0;
    this.fadeAnimation = { isAnimating: true, progress: 0, duration: 2000 };

    // Post-processing support detection
    this.supportsPostProcessing = false;

    // Constraint system for camera movement - wider bounds
    this.constraintBounds = {
      position: {
        x: { min: -15, max: 15 },
        y: { min: 2, max: 12 },
        z: { min: 3, max: 20 },
      },
      target: {
        x: { min: -4, max: 4 },
        y: { min: 0, max: 4 },
        z: { min: -3, max: 3 },
      },
    };
    this.constraintAnimation = { isAnimating: false };

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
      await this.trySetupPostProcessing();
      this.updateLoadingProgress(100);
      this.hideLoading();
      this.showTerminal();
      this.startFadeInAnimation();
      this.animate();

      // Start the startup animation sequence after everything is loaded
      this.startStartupSequence();
    } catch (e) {
      console.error("3D init failed:", e);
      this.showError();
    }
  }

  startStartupSequence() {
    console.log("Starting startup animation sequence...");
    this.startupAnimation.isActive = true;
    this.startupAnimation.startTime = performance.now();
    this.startupAnimation.phase = "waiting";
  }

  updateStartupAnimation() {
    if (!this.startupAnimation.isActive || this.startupAnimation.hasCompleted)
      return;

    const elapsed = performance.now() - this.startupAnimation.startTime;

    switch (this.startupAnimation.phase) {
      case "waiting":
        // Wait for the delay period
        if (elapsed >= this.startupAnimation.delayDuration) {
          console.log("Starting focus animation...");
          this.startupAnimation.phase = "focusing";
          this.animateCamera("focused", this.startupAnimation.focusDuration);

          // Trigger terminal focus events
          this.isTerminalFocused = true;
          if (this.hiddenInput) {
            this.hiddenInput.focus();
          }
          const focusEvent = new CustomEvent("terminalFocus");
          window.dispatchEvent(focusEvent);
        }
        break;

      case "focusing":
        // Wait for focus animation to complete
        if (!this.cameraAnimation.isAnimating) {
          console.log("Startup sequence completed!");
          this.startupAnimation.phase = "complete";
          this.startupAnimation.hasCompleted = true;
          this.startupAnimation.isActive = false;

          // Enable idle rotation now that startup is complete
          this.idleRotation.enabled = true;
          this.resetIdleTimer();
        }
        break;
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

  startFadeInAnimation() {
    this.fadeAnimation.isAnimating = true;
    this.fadeAnimation.startTime = performance.now();
  }

  async setupScene() {
    this.scene = new THREE.Scene();
    this.scene.background = new THREE.Color(0x0a0a0a);

    // Enhanced camera setup with much further initial position
    this.camera = new THREE.PerspectiveCamera(
      45,
      window.innerWidth / window.innerHeight,
      0.1,
      100,
    );
    this.camera.position.copy(this.cameraStates.default.position);

    // Enhanced renderer with better settings
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

    // Add manual gamma correction for better visuals
    this.renderer.gammaFactor = 2.2;

    document
      .getElementById("scene-container")
      .appendChild(this.renderer.domElement);

    // Enhanced controls with wider constraints
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.05;
    this.controls.enableZoom = false; // Disable zoom
    this.controls.enablePan = true; // Allow panning
    this.controls.enableRotate = true; // Allow rotation
    this.controls.target.copy(this.cameraStates.default.target);
    this.controls.minDistance = 3; // Closer minimum distance for focused view
    this.controls.maxDistance = 20; // Much larger maximum distance
    this.controls.minPolarAngle = Math.PI * 0.05;
    this.controls.maxPolarAngle = Math.PI * 0.75;

    // Wider azimuth angle limits
    this.controls.minAzimuthAngle = -Math.PI * 0.75; // -135 degrees
    this.controls.maxAzimuthAngle = Math.PI * 0.75; // +135 degrees

    // Enhanced lighting setup
    this.setupLighting();
  }

  setupLighting() {
    // Remove existing basic lighting
    this.scene.children = this.scene.children.filter((child) => !child.isLight);

    // Ambient light for overall illumination
    const ambientLight = new THREE.AmbientLight(0x6c7b95, 0.9);
    this.scene.add(ambientLight);

    // Main directional light (key light) - positioned for side lighting
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

    // Fill light from opposite side
    const fillLight = new THREE.DirectionalLight(0x8be9fd, 1.0);
    fillLight.position.set(-5, 6, -4);
    this.scene.add(fillLight);

    // Monitor area lighting - focused on screen
    const monitorLight = new THREE.PointLight(0x50fa7b, 2.2, 15);
    monitorLight.position.set(0, 3, 2);
    this.scene.add(monitorLight);

    // Back rim lighting
    const rimLight = new THREE.DirectionalLight(0x50fa7b, 0.8);
    rimLight.position.set(0, 4, -8);
    this.scene.add(rimLight);

    // Additional accent lights
    const accentLight1 = new THREE.PointLight(0x8be9fd, 1.8, 18);
    accentLight1.position.set(4, 4, 4);
    this.scene.add(accentLight1);

    const accentLight2 = new THREE.PointLight(0x50fa7b, 1.5, 15);
    accentLight2.position.set(-4, 3, 2);
    this.scene.add(accentLight2);

    // Hemisphere light for realistic environment
    const hemiLight = new THREE.HemisphereLight(0x8be9fd, 0x6c5ce7, 0.8);
    hemiLight.position.set(0, 25, 0);
    this.scene.add(hemiLight);

    // Front lighting for better visibility at distance
    const frontLight = new THREE.DirectionalLight(0xf8f8ff, 1.2);
    frontLight.position.set(0, 8, 12);
    this.scene.add(frontLight);
  }

  // Idle rotation system
  updateIdleRotation() {
    if (
      !this.idleRotation.enabled ||
      this.cameraAnimation.isAnimating ||
      this.constraintAnimation.isAnimating
    ) {
      return;
    }

    // Check if user has been inactive for 3 seconds and startup is complete
    const timeSinceInteraction = Date.now() - this.idleRotation.lastInteraction;
    if (timeSinceInteraction > 3000 && this.startupAnimation.hasCompleted) {
      // Update rotation angle
      this.idleRotation.angle += this.idleRotation.speed;

      // Calculate new position
      const x = Math.cos(this.idleRotation.angle) * this.idleRotation.radius;
      const z = Math.sin(this.idleRotation.angle) * this.idleRotation.radius;

      // Only update if we're in default mode and not focused
      if (this.currentCameraState === "default" && !this.isTerminalFocused) {
        this.camera.position.set(x, this.idleRotation.height, z);
        this.controls.update();
      }
    }
  }

  resetIdleTimer() {
    this.idleRotation.lastInteraction = Date.now();
  }

  // Constraint checking and animation system
  checkConstraints() {
    const position = this.camera.position;
    const target = this.controls.target;

    let needsCorrection = false;
    let targetPosition = position.clone();
    let targetTarget = target.clone();

    // Check camera position constraints
    if (
      position.x < this.constraintBounds.position.x.min ||
      position.x > this.constraintBounds.position.x.max
    ) {
      targetPosition.x = THREE.MathUtils.clamp(
        position.x,
        this.constraintBounds.position.x.min,
        this.constraintBounds.position.x.max,
      );
      needsCorrection = true;
    }

    if (
      position.y < this.constraintBounds.position.y.min ||
      position.y > this.constraintBounds.position.y.max
    ) {
      targetPosition.y = THREE.MathUtils.clamp(
        position.y,
        this.constraintBounds.position.y.min,
        this.constraintBounds.position.y.max,
      );
      needsCorrection = true;
    }

    if (
      position.z < this.constraintBounds.position.z.min ||
      position.z > this.constraintBounds.position.z.max
    ) {
      targetPosition.z = THREE.MathUtils.clamp(
        position.z,
        this.constraintBounds.position.z.min,
        this.constraintBounds.position.z.max,
      );
      needsCorrection = true;
    }

    // Check target constraints
    if (
      target.x < this.constraintBounds.target.x.min ||
      target.x > this.constraintBounds.target.x.max
    ) {
      targetTarget.x = THREE.MathUtils.clamp(
        target.x,
        this.constraintBounds.target.x.min,
        this.constraintBounds.target.x.max,
      );
      needsCorrection = true;
    }

    if (
      target.y < this.constraintBounds.target.y.min ||
      target.y > this.constraintBounds.target.y.max
    ) {
      targetTarget.y = THREE.MathUtils.clamp(
        target.y,
        this.constraintBounds.target.y.min,
        this.constraintBounds.target.y.max,
      );
      needsCorrection = true;
    }

    if (
      target.z < this.constraintBounds.target.z.min ||
      target.z > this.constraintBounds.target.z.max
    ) {
      targetTarget.z = THREE.MathUtils.clamp(
        target.z,
        this.constraintBounds.target.z.min,
        this.constraintBounds.target.z.max,
      );
      needsCorrection = true;
    }

    if (needsCorrection && !this.constraintAnimation.isAnimating) {
      this.animateToConstraints(targetPosition, targetTarget);
    }
  }

  animateToConstraints(targetPosition, targetTarget) {
    if (this.constraintAnimation.isAnimating) return;

    const startPosition = this.camera.position.clone();
    const startTarget = this.controls.target.clone();

    this.constraintAnimation = {
      isAnimating: true,
      progress: 0,
      duration: 1000,
      startTime: performance.now(),
      startPosition: startPosition,
      startTarget: startTarget,
      targetPosition: targetPosition,
      targetTarget: targetTarget,
      onComplete: () => {
        this.constraintAnimation.isAnimating = false;
      },
    };
  }

  updateConstraintAnimation() {
    if (!this.constraintAnimation.isAnimating) return;

    const elapsed = performance.now() - this.constraintAnimation.startTime;
    this.constraintAnimation.progress = Math.min(
      elapsed / this.constraintAnimation.duration,
      1,
    );

    // Smooth easing function
    const easeOutCubic = (t) => {
      return 1 - Math.pow(1 - t, 3);
    };

    const easedProgress = easeOutCubic(this.constraintAnimation.progress);

    // Interpolate camera position
    this.camera.position.lerpVectors(
      this.constraintAnimation.startPosition,
      this.constraintAnimation.targetPosition,
      easedProgress,
    );

    // Interpolate controls target
    this.controls.target.lerpVectors(
      this.constraintAnimation.startTarget,
      this.constraintAnimation.targetTarget,
      easedProgress,
    );

    if (this.constraintAnimation.progress >= 1) {
      this.constraintAnimation.onComplete();
    }
  }

  // Try to load post-processing with fallback
  async trySetupPostProcessing() {
    try {
      // Try to dynamically import post-processing modules
      const [
        { EffectComposer },
        { RenderPass },
        { UnrealBloomPass },
        { OutputPass },
      ] = await Promise.all([
        import(
          "https://cdn.skypack.dev/three@0.160.0/examples/jsm/postprocessing/EffectComposer.js"
        ),
        import(
          "https://cdn.skypack.dev/three@0.160.0/examples/jsm/postprocessing/RenderPass.js"
        ),
        import(
          "https://cdn.skypack.dev/three@0.160.0/examples/jsm/postprocessing/UnrealBloomPass.js"
        ),
        import(
          "https://cdn.skypack.dev/three@0.160.0/examples/jsm/postprocessing/OutputPass.js"
        ),
      ]);

      this.setupPostProcessing(
        EffectComposer,
        RenderPass,
        UnrealBloomPass,
        OutputPass,
      );
      this.supportsPostProcessing = true;
      console.log("Post-processing enabled");
    } catch (error) {
      console.warn(
        "Post-processing not available, using fallback rendering:",
        error,
      );
      this.supportsPostProcessing = false;
      // Manual bloom effect fallback
      this.setupManualBloom();
    }
  }

  setupPostProcessing(EffectComposer, RenderPass, UnrealBloomPass, OutputPass) {
    // Create composer for post-processing effects
    this.composer = new EffectComposer(this.renderer);

    // Base render pass
    const renderPass = new RenderPass(this.scene, this.camera);
    this.composer.addPass(renderPass);

    // Bloom effect for glow
    const bloomPass = new UnrealBloomPass(
      new THREE.Vector2(window.innerWidth, window.innerHeight),
      0.4, // strength
      0.6, // radius
      0.8, // threshold
    );
    this.composer.addPass(bloomPass);

    // Output pass
    const outputPass = new OutputPass();
    this.composer.addPass(outputPass);
  }

  // Manual bloom effect using render targets (fallback)
  setupManualBloom() {
    // Create render targets for bloom effect
    this.bloomRenderTarget = new THREE.WebGLRenderTarget(
      window.innerWidth * 0.5,
      window.innerHeight * 0.5,
      {
        minFilter: THREE.LinearFilter,
        magFilter: THREE.LinearFilter,
        format: THREE.RGBAFormat,
        type: THREE.FloatType,
      },
    );

    // Simple bloom shader material
    this.bloomMaterial = new THREE.ShaderMaterial({
      uniforms: {
        tDiffuse: { value: null },
        strength: { value: 0.3 },
      },
      vertexShader: `
        varying vec2 vUv;
        void main() {
          vUv = uv;
          gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
        }
      `,
      fragmentShader: `
        uniform sampler2D tDiffuse;
        uniform float strength;
        varying vec2 vUv;
        
        void main() {
          vec4 color = texture2D(tDiffuse, vUv);
          
          // Simple bloom approximation
          float brightness = dot(color.rgb, vec3(0.299, 0.587, 0.114));
          vec3 bloom = color.rgb * max(0.0, brightness - 0.5) * strength * 2.0;
          
          gl_FragColor = vec4(color.rgb + bloom, color.a);
        }
      `,
    });
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

          // Enhanced material processing
          this.pcModel.traverse((c) => {
            if (c.isMesh) {
              c.castShadow = c.receiveShadow = true;

              // Enhance materials for better lighting response
              if (c.material) {
                if (c.material.isMeshStandardMaterial) {
                  // Make materials more responsive to light
                  c.material.envMapIntensity = 1.0;
                  c.material.roughness = Math.max(
                    0.2,
                    c.material.roughness * 0.9,
                  );
                  c.material.metalness = Math.min(
                    0.9,
                    c.material.metalness * 1.1,
                  );

                  // Increase emissive for self-illumination
                  if (c.material.emissive) {
                    c.material.emissive.multiplyScalar(1.2);
                  }
                } else if (c.material.isMeshBasicMaterial) {
                  // Convert basic materials to standard for better lighting
                  const newMaterial = new THREE.MeshStandardMaterial({
                    color: c.material.color,
                    map: c.material.map,
                    transparent: c.material.transparent,
                    opacity: c.material.opacity,
                  });
                  c.material = newMaterial;
                }
              }

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

    if (this.screenMesh) {
      // Enhanced screen material with stronger emissive for visibility
      this.screenMesh.material = new THREE.MeshStandardMaterial({
        map: this.terminalTexture,
        emissive: new THREE.Color(0x004444),
        emissiveIntensity: 0.7,
        roughness: 0.05,
        metalness: 0.05,
        transparent: true,
        opacity: 1.0,
        // Add some self-illumination
        emissiveMap: this.terminalTexture,
      });
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

  // Camera animation system
  animateCamera(targetState, duration = 1500) {
    if (this.cameraAnimation.isAnimating) return;

    // Disable idle rotation when animating
    this.idleRotation.enabled = false;

    const startPosition = this.camera.position.clone();
    const startTarget = this.controls.target.clone();
    const targetPosition = this.cameraStates[targetState].position.clone();
    const targetTargetPos = this.cameraStates[targetState].target.clone();

    this.cameraAnimation = {
      isAnimating: true,
      progress: 0,
      duration: duration,
      startTime: performance.now(),
      startPosition: startPosition,
      startTarget: startTarget,
      targetPosition: targetPosition,
      targetTarget: targetTargetPos,
      onComplete: () => {
        this.currentCameraState = targetState;
        this.cameraAnimation.isAnimating = false;

        // Re-enable idle rotation for default state only after startup completes
        if (targetState === "default" && this.startupAnimation.hasCompleted) {
          this.idleRotation.enabled = true;
          this.resetIdleTimer();
        }
      },
    };
  }

  updateCameraAnimation(deltaTime) {
    if (!this.cameraAnimation.isAnimating) return;

    const elapsed = performance.now() - this.cameraAnimation.startTime;
    this.cameraAnimation.progress = Math.min(
      elapsed / this.cameraAnimation.duration,
      1,
    );

    // Smooth easing function
    const easeInOutCubic = (t) => {
      return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
    };

    const easedProgress = easeInOutCubic(this.cameraAnimation.progress);

    // Interpolate camera position
    this.camera.position.lerpVectors(
      this.cameraAnimation.startPosition,
      this.cameraAnimation.targetPosition,
      easedProgress,
    );

    // Interpolate controls target
    this.controls.target.lerpVectors(
      this.cameraAnimation.startTarget,
      this.cameraAnimation.targetTarget,
      easedProgress,
    );

    if (this.cameraAnimation.progress >= 1) {
      this.cameraAnimation.onComplete();
    }
  }

  updateFadeAnimation() {
    if (!this.fadeAnimation.isAnimating) return;

    const elapsed = performance.now() - this.fadeAnimation.startTime;
    this.fadeAnimation.progress = Math.min(
      elapsed / this.fadeAnimation.duration,
      1,
    );

    // Smooth fade in
    this.sceneOpacity = this.fadeAnimation.progress;

    // Apply fade to scene
    if (this.pcModel) {
      this.pcModel.traverse((child) => {
        if (child.isMesh && child.material && child !== this.screenMesh) {
          if (Array.isArray(child.material)) {
            child.material.forEach((mat) => {
              if (mat.transparent !== undefined) {
                mat.transparent = true;
                mat.opacity = this.sceneOpacity;
              }
            });
          } else {
            child.material.transparent = true;
            child.material.opacity = this.sceneOpacity;
          }
        }
      });
    }

    if (this.fadeAnimation.progress >= 1) {
      this.fadeAnimation.isAnimating = false;
      // Restore original opacity settings
      if (this.pcModel) {
        this.pcModel.traverse((child) => {
          if (child.isMesh && child.material && child !== this.screenMesh) {
            if (Array.isArray(child.material)) {
              child.material.forEach((mat) => {
                mat.transparent = false;
                mat.opacity = 1.0;
              });
            } else {
              child.material.transparent = false;
              child.material.opacity = 1.0;
            }
          }
        });
      }
    }
  }

  setupEventListeners() {
    window.addEventListener("resize", () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);

      if (this.composer) {
        this.composer.setSize(window.innerWidth, window.innerHeight);
      }

      if (this.bloomRenderTarget) {
        this.bloomRenderTarget.setSize(
          window.innerWidth * 0.5,
          window.innerHeight * 0.5,
        );
      }
    });

    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();

    // Enhanced mouse move handler with interaction tracking
    this.renderer.domElement.addEventListener("mousemove", (event) => {
      // Only reset idle timer if startup sequence is complete
      if (this.startupAnimation.hasCompleted) {
        this.resetIdleTimer();
      }

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

      const intersects = this.raycaster.intersectObjects(intersectables);

      if (intersects.length > 0) {
        const hoveredObject = intersects[0].object;
        const isScreen = this.isScreenObject(hoveredObject);

        if (isScreen) {
          this.renderer.domElement.style.cursor = "pointer";
          this.isHoveringScreen = true;

          // Enhanced glow effect when hovering screen
          if (this.screenMesh && this.screenMesh.material) {
            this.screenMesh.material.emissiveIntensity = 1.0;
          }
        } else {
          this.renderer.domElement.style.cursor = "grab";
          this.isHoveringScreen = false;

          // Reset screen glow
          if (this.screenMesh && this.screenMesh.material) {
            this.screenMesh.material.emissiveIntensity = 0.7;
          }
        }
      } else {
        this.renderer.domElement.style.cursor = "default";
        this.isHoveringScreen = false;

        // Reset screen glow
        if (this.screenMesh && this.screenMesh.material) {
          this.screenMesh.material.emissiveIntensity = 0.7;
        }
      }
    });

    // Track mouse interactions for idle timer
    this.renderer.domElement.addEventListener("mousedown", () => {
      if (this.startupAnimation.hasCompleted) {
        this.resetIdleTimer();
      }
    });

    // Enhanced click handler with camera animations (only after startup)
    this.renderer.domElement.addEventListener("click", (event) => {
      // Don't allow manual camera control during startup
      if (!this.startupAnimation.hasCompleted) return;

      this.resetIdleTimer();

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

      const intersects = this.raycaster.intersectObjects(intersectables);

      if (intersects.length > 0) {
        const clickedObject = intersects[0].object;
        const isScreen = this.isScreenObject(clickedObject);

        if (isScreen) {
          console.log("Screen clicked - focusing on monitor");
          this.isTerminalFocused = true;
          this.animateCamera("focused", 1500);

          if (this.hiddenInput) {
            this.hiddenInput.focus();
          }

          const focusEvent = new CustomEvent("terminalFocus");
          window.dispatchEvent(focusEvent);
        } else {
          console.log("Clicked on PC model - switching to overview");
          this.isTerminalFocused = false;
          this.animateCamera("overview", 1200);

          if (this.hiddenInput) {
            this.hiddenInput.blur();
          }

          const blurEvent = new CustomEvent("terminalBlur");
          window.dispatchEvent(blurEvent);
        }
      } else {
        console.log("Clicked on empty space - returning to default view");
        this.isTerminalFocused = false;
        this.animateCamera("default", 1800);

        if (this.hiddenInput) {
          this.hiddenInput.blur();
        }

        const blurEvent = new CustomEvent("terminalBlur");
        window.dispatchEvent(blurEvent);
      }
    });

    // Handle clicks outside the 3D scene (only after startup)
    document.addEventListener("click", (e) => {
      if (!this.startupAnimation.hasCompleted) return;

      if (
        !e.target.closest("#scene-container") &&
        !e.target.closest("#terminal") &&
        !e.target.closest("#hidden-input")
      ) {
        console.log("Clicked outside - returning to default view");
        this.isTerminalFocused = false;
        this.animateCamera("default", 1500);

        if (this.hiddenInput) {
          this.hiddenInput.blur();
        }

        const blurEvent = new CustomEvent("terminalBlur");
        window.dispatchEvent(blurEvent);
      }
    });

    // Disable wheel zoom while preserving other wheel events
    this.renderer.domElement.addEventListener("wheel", (event) => {
      if (this.startupAnimation.hasCompleted) {
        this.resetIdleTimer();
      }

      if (this.isHoveringScreen) {
        // Allow terminal scrolling (your existing logic)
        event.preventDefault();

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
      } else {
        // Prevent camera zoom
        event.preventDefault();
      }
    });

    // Custom event listeners
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

  isScreenObject(object) {
    return (
      object.name === "Plane008_Material002_0" ||
      object === this.screenMesh ||
      object.material?.map === this.terminalTexture ||
      (object.name && object.name.toLowerCase().includes("screen"))
    );
  }

  // Scroll functionality (keeping your existing logic)
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
      console.log(`Scrolled up to position ${this.currentScrollPosition}`);

      const scrollEvent = new CustomEvent("terminalScrollUp", {
        detail: { position: this.currentScrollPosition },
      });
      window.dispatchEvent(scrollEvent);
    }
  }

  scrollDown() {
    if (this.currentScrollPosition > 0) {
      this.currentScrollPosition--;
      console.log(`Scrolled down to position ${this.currentScrollPosition}`);

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

  animate() {
    this.animationId = requestAnimationFrame(() => this.animate());

    const deltaTime = 16; // Approximate 60fps

    // Update animations
    this.updateCameraAnimation(deltaTime);
    this.updateFadeAnimation();
    this.updateConstraintAnimation();
    this.updateIdleRotation(); // Add idle rotation
    this.updateStartupAnimation(); // Add startup animation

    // Check constraints every few frames when not animating
    if (
      !this.cameraAnimation.isAnimating &&
      !this.constraintAnimation.isAnimating &&
      this.startupAnimation.hasCompleted
    ) {
      // Only check constraints occasionally to avoid constant corrections
      if (Math.random() < 0.02) {
        // ~2% chance per frame = roughly every 3 seconds at 60fps
        this.checkConstraints();
      }
    }

    this.controls.update();

    // Use composer for enhanced rendering if available, otherwise fallback
    if (this.composer && this.supportsPostProcessing) {
      this.composer.render();
    } else {
      // Manual bloom rendering if available
      if (this.bloomRenderTarget && this.bloomMaterial) {
        this.renderWithManualBloom();
      } else {
        // Standard rendering
        this.renderer.render(this.scene, this.camera);
      }
    }
  }

  renderWithManualBloom() {
    // Simple manual bloom implementation
    const originalBackground = this.scene.background;
    this.scene.background = null;

    // Render to bloom target
    this.renderer.setRenderTarget(this.bloomRenderTarget);
    this.renderer.render(this.scene, this.camera);

    // Render to screen with bloom
    this.renderer.setRenderTarget(null);
    this.scene.background = originalBackground;
    this.renderer.render(this.scene, this.camera);
  }

  dispose() {
    if (this.animationId) cancelAnimationFrame(this.animationId);
    if (this.renderer) this.renderer.dispose();
    if (this.composer) this.composer.dispose();
    if (this.terminalTexture) this.terminalTexture.dispose();
    if (this.bloomRenderTarget) this.bloomRenderTarget.dispose();
    if (this.bloomMaterial) this.bloomMaterial.dispose();
  }
}
