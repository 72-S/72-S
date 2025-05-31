import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

export class Terminal3D {
  constructor() {
    this.scene = null;
    this.camera = null;
    this.renderer = null;
    this.controls = null;
    this.composer = null;
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
    this.supportsPostProcessing = false;

    // Animation System Properties
    this.startupPhase = "waiting"; // waiting, focusing, complete
    this.cameraStates = {
      default: {
        position: new THREE.Vector3(4, 5, 10),
        target: new THREE.Vector3(0, 1.5, 0),
      },
      focused: {
        position: new THREE.Vector3(0, 3.0, 5.5),
        target: new THREE.Vector3(0, 2.3, 0),
      },
      overview: {
        position: new THREE.Vector3(6, 6, 12),
        target: new THREE.Vector3(0, 1.5, 0),
      },
      idle: {
        position: new THREE.Vector3(3, 4.5, 9),
        target: new THREE.Vector3(0, 1.5, 0),
      },
    };
    this.currentCameraState = "default";
    this.isAnimatingCamera = false;
    this.idleRotationActive = false;
    this.lastInteractionTime = Date.now();
    this.lastFocusedInteractionTime = Date.now(); // Track focused state interactions separately
    this.inactivityDelay = 10000; // Regular idle delay (10 seconds)
    this.focusedInactivityDelay = 180000; // 3 minutes for focused state
    this.idleRotationSpeed = 0.0002;
    this.idleRadius = 7;
    this.idleHeight = 4.2;
    this.idleAngle = 0;
    this.fadeInDuration = 4000;
    this.fadeStartTime = null;
    this.maxIdleRotation = Math.PI * 1.2;
    this.idleRotationCount = 0;
    this.sceneMeshes = [];
    this.cameraBounds = {
      position: { x: [-15, 15], y: [2, 12], z: [3, 20] },
      target: { x: [-5, 5], y: [0, 5], z: [-3, 3] },
    };
    this.lastValidCameraPosition = new THREE.Vector3();
    this.lastValidCameraTarget = new THREE.Vector3();

    this.defaultCameraPosition = new THREE.Vector3(4, 5, 10);
    this.defaultCameraTarget = new THREE.Vector3(0, 1.5, 0);

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
      this.startupAnimation();
      this.animate();
    } catch (e) {
      console.error("3D init failed:", e);
      this.showError();
    }
  }

  // 1. Startup Animation Sequence
  startupAnimation() {
    this.startupPhase = "waiting";

    // Phase 1: Wait 1 second
    setTimeout(() => {
      this.startupPhase = "focusing";
      // Phase 2: Focus on monitor (2 seconds)
      this.animateToState("focused", 2000, () => {
        this.startupPhase = "complete";
        // Phase 3: Enable idle rotation
        this.lastInteractionTime = Date.now();
        this.lastFocusedInteractionTime = Date.now();
      });
    }, 1000);
  }

  // 2. Camera State Animation System
  animateToState(stateName, duration = 1500, onComplete = null) {
    if (!this.cameraStates[stateName] || this.isAnimatingCamera) return;

    this.isAnimatingCamera = true;
    this.idleRotationActive = false;
    this.currentCameraState = stateName;

    const targetState = this.cameraStates[stateName];
    const startPosition = this.camera.position.clone();
    const startTarget = this.controls.target.clone();
    const startTime = Date.now();

    const animateStep = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);

      // Smooth easing function
      const easeProgress = this.easeInOutCubic(progress);

      // Interpolate position
      this.camera.position.lerpVectors(
        startPosition,
        targetState.position,
        easeProgress,
      );
      this.controls.target.lerpVectors(
        startTarget,
        targetState.target,
        easeProgress,
      );

      if (progress < 1) {
        requestAnimationFrame(animateStep);
      } else {
        this.isAnimatingCamera = false;
        if (onComplete) onComplete();
      }
    };

    animateStep();
  }

  easeInOutCubic(t) {
    return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  }

  // 3. Enhanced Idle Rotation System with Focused State Support
  updateIdleRotation() {
    // Don't start idle if startup isn't complete
    if (this.startupPhase !== "complete") return;

    // Don't start idle if currently animating
    if (this.isAnimatingCamera) return;

    const currentTime = Date.now();
    const timeSinceLastInteraction = currentTime - this.lastInteractionTime;
    const timeSinceLastFocusedInteraction =
      currentTime - this.lastFocusedInteractionTime;

    // Different logic for focused vs non-focused states
    if (this.currentCameraState === "focused" && this.isTerminalFocused) {
      // In focused state, need 3 minutes of inactivity
      if (
        timeSinceLastFocusedInteraction > this.focusedInactivityDelay &&
        !this.idleRotationActive
      ) {
        console.log(
          "Starting idle rotation from focused state after 3 minutes",
        );
        this.startIdleRotation();
      }
    } else if (this.currentCameraState !== "focused") {
      // Regular idle logic for non-focused states
      if (
        timeSinceLastInteraction > this.inactivityDelay &&
        !this.idleRotationActive
      ) {
        this.startIdleRotation();
      }
    }

    // Update idle rotation if active
    if (this.idleRotationActive) {
      this.idleAngle += this.idleRotationSpeed;
      this.idleRotationCount += this.idleRotationSpeed;

      // Check if we've rotated enough to reset
      if (this.idleRotationCount >= this.maxIdleRotation) {
        this.resetIdleRotation();
        return;
      }

      const x = Math.cos(this.idleAngle) * this.idleRadius;
      const z = Math.sin(this.idleAngle) * this.idleRadius;

      this.camera.position.set(x, this.idleHeight, z);
      this.controls.target.set(0, 1.5, 0);
    }
  }

  resetIdleRotation() {
    this.idleRotationActive = false;
    this.idleRotationCount = 0;

    // Smoothly return to a new starting position
    this.animateToState("default", 1500, () => {
      // After returning to default, wait a bit then start idle again
      setTimeout(() => {
        if (
          this.currentCameraState === "default" &&
          !this.isTerminalFocused &&
          this.startupPhase === "complete"
        ) {
          this.startIdleRotation();
        }
      }, 2000); // Wait 2 seconds before starting again
    });
  }

  // Enhanced idle rotation start - now allows starting from focused state
  startIdleRotation() {
    // Allow idle rotation to start from focused state after long inactivity
    this.idleRotationActive = false; // Ensure it's off during transition
    this.isAnimatingCamera = true;
    this.idleRotationCount = 0; // Reset rotation counter

    // Calculate the starting position for smooth transition
    const startAngle = Math.atan2(
      this.camera.position.z - 0,
      this.camera.position.x - 0,
    );
    this.idleAngle = startAngle;

    const idealStartPosition = new THREE.Vector3(
      Math.cos(this.idleAngle) * this.idleRadius,
      this.idleHeight,
      Math.sin(this.idleAngle) * this.idleRadius,
    );

    // Smooth transition to ideal idle starting position
    const startPosition = this.camera.position.clone();
    const startTarget = this.controls.target.clone();
    const targetTarget = new THREE.Vector3(0, 1.5, 0);
    const startTime = Date.now();
    const duration = 1500;

    const animateToIdleStart = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const easeProgress = this.easeInOutCubic(progress);

      // Interpolate to ideal starting position
      this.camera.position.lerpVectors(
        startPosition,
        idealStartPosition,
        easeProgress,
      );
      this.controls.target.lerpVectors(startTarget, targetTarget, easeProgress);

      if (progress < 1) {
        requestAnimationFrame(animateToIdleStart);
      } else {
        // Now start the smooth continuous rotation
        this.isAnimatingCamera = false;
        this.idleRotationActive = true;
        this.currentCameraState = "idle";
        // Clear focused state when going to idle
        this.isTerminalFocused = false;
      }
    };

    animateToIdleStart();
  }

  stopIdleRotation() {
    this.idleRotationActive = false;
    this.lastInteractionTime = Date.now();
    this.lastFocusedInteractionTime = Date.now();

    // If we were in idle state, return to default
    if (this.currentCameraState === "idle") {
      this.animateToState("default", 1000);
    }
  }

  // 4. Scene Fade-In Animation
  startSceneFadeIn() {
    this.fadeStartTime = Date.now();

    // Set all meshes to transparent initially
    this.sceneMeshes.forEach((mesh) => {
      if (mesh !== this.screenMesh && mesh.material) {
        mesh.material.transparent = true;
        mesh.material.opacity = 0;
      }
    });
  }

  updateSceneFadeIn() {
    if (!this.fadeStartTime) return;

    const elapsed = Date.now() - this.fadeStartTime;
    const progress = Math.min(elapsed / this.fadeInDuration, 1);
    const opacity = this.easeInOutCubic(progress);

    this.sceneMeshes.forEach((mesh) => {
      if (
        mesh !== this.screenMesh &&
        mesh.material &&
        mesh.material.transparent
      ) {
        mesh.material.opacity = opacity;

        if (progress >= 1) {
          mesh.material.transparent = false;
          mesh.material.opacity = 1;
        }
      }
    });

    if (progress >= 1) {
      this.fadeStartTime = null;
    }
  }

  // 5. Constraint Animation System
  checkAndCorrectCameraBounds() {
    const pos = this.camera.position;
    const target = this.controls.target;
    let needsCorrection = false;

    // Check position bounds
    if (
      pos.x < this.cameraBounds.position.x[0] ||
      pos.x > this.cameraBounds.position.x[1] ||
      pos.y < this.cameraBounds.position.y[0] ||
      pos.y > this.cameraBounds.position.y[1] ||
      pos.z < this.cameraBounds.position.z[0] ||
      pos.z > this.cameraBounds.position.z[1]
    ) {
      needsCorrection = true;
    }

    // Check target bounds
    if (
      target.x < this.cameraBounds.target.x[0] ||
      target.x > this.cameraBounds.target.x[1] ||
      target.y < this.cameraBounds.target.y[0] ||
      target.y > this.cameraBounds.target.y[1] ||
      target.z < this.cameraBounds.target.z[0] ||
      target.z > this.cameraBounds.target.z[1]
    ) {
      needsCorrection = true;
    }

    if (needsCorrection && !this.isAnimatingCamera) {
      this.correctCameraPosition();
    } else if (!needsCorrection) {
      this.lastValidCameraPosition.copy(pos);
      this.lastValidCameraTarget.copy(target);
    }
  }

  correctCameraPosition() {
    this.animateToState("default", 1000);
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

    // Camera setup
    this.camera = new THREE.PerspectiveCamera(
      45,
      window.innerWidth / window.innerHeight,
      0.1,
      100,
    );
    this.camera.position.copy(this.defaultCameraPosition);
    this.lastValidCameraPosition.copy(this.defaultCameraPosition);
    this.lastValidCameraTarget.copy(this.defaultCameraTarget);

    // Renderer setup
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

    // Controls setup (ZOOM DISABLED)
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.05;
    this.controls.enableZoom = false; // DISABLED ZOOM
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
    // Ambient light
    const ambientLight = new THREE.AmbientLight(0x6c7b95, 0.9);
    this.scene.add(ambientLight);

    // Main directional light
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

    // Fill light
    const fillLight = new THREE.DirectionalLight(0x8be9fd, 1.0);
    fillLight.position.set(-5, 6, -4);
    this.scene.add(fillLight);

    // Monitor area lighting
    const monitorLight = new THREE.PointLight(0x50fa7b, 2.2, 15);
    monitorLight.position.set(0, 3, 2);
    this.scene.add(monitorLight);

    // Back rim lighting
    const rimLight = new THREE.DirectionalLight(0x50fa7b, 0.8);
    rimLight.position.set(0, 4, -8);
    this.scene.add(rimLight);

    // Accent lights
    const accentLight1 = new THREE.PointLight(0x8be9fd, 1.8, 18);
    accentLight1.position.set(4, 4, 4);
    this.scene.add(accentLight1);

    const accentLight2 = new THREE.PointLight(0x50fa7b, 1.5, 15);
    accentLight2.position.set(-4, 3, 2);
    this.scene.add(accentLight2);

    // Hemisphere light
    const hemiLight = new THREE.HemisphereLight(0x8be9fd, 0x6c5ce7, 0.8);
    hemiLight.position.set(0, 25, 0);
    this.scene.add(hemiLight);

    // Front lighting
    const frontLight = new THREE.DirectionalLight(0xf8f8ff, 1.2);
    frontLight.position.set(0, 8, 12);
    this.scene.add(frontLight);
  }

  // 6. Enhanced Post-Processing with Bloom Effects
  async trySetupPostProcessing() {
    try {
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
      console.warn("Post-processing not available:", error);
      this.supportsPostProcessing = false;
      this.setupManualBloom();
    }
  }

  setupPostProcessing(EffectComposer, RenderPass, UnrealBloomPass, OutputPass) {
    this.composer = new EffectComposer(this.renderer);

    const renderPass = new RenderPass(this.scene, this.camera);
    this.composer.addPass(renderPass);

    // Enhanced bloom settings
    const bloomPass = new UnrealBloomPass(
      new THREE.Vector2(window.innerWidth, window.innerHeight),
      0.6, // strength
      0.8, // radius
      0.7, // threshold
    );
    this.composer.addPass(bloomPass);

    const outputPass = new OutputPass();
    this.composer.addPass(outputPass);
  }

  // Manual bloom fallback
  setupManualBloom() {
    if (this.screenMesh && this.screenMesh.material) {
      this.screenMesh.material.emissiveIntensity = 1.2;

      // Add pulsing effect
      this.bloomPulse = 0;
      this.bloomDirection = 1;
    }
  }

  updateManualBloom() {
    if (
      !this.supportsPostProcessing &&
      this.screenMesh &&
      this.screenMesh.material
    ) {
      this.bloomPulse += 0.02 * this.bloomDirection;

      if (this.bloomPulse > 1) {
        this.bloomPulse = 1;
        this.bloomDirection = -1;
      } else if (this.bloomPulse < 0.5) {
        this.bloomPulse = 0.5;
        this.bloomDirection = 1;
      }

      this.screenMesh.material.emissiveIntensity = 0.7 + this.bloomPulse * 0.5;
    }
  }

  // 7. Enhanced CRT Shader with Dynamic Parameters
  createCRTMaterial() {
    const vertexShader = `
      varying vec2 vUv;
      void main() {
        vUv = uv;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }
    `;

    const fragmentShader = `
      uniform sampler2D map;
      uniform float bulge;
      uniform float scanlineIntensity;
      uniform float scanlineCount;
      uniform float vignetteIntensity;
      uniform float vignetteRadius;
      uniform float glowIntensity;
      uniform vec3 glowColor;
      uniform float brightness;
      uniform float contrast;
      uniform float time;
      varying vec2 vUv;

      vec2 crtDistort(vec2 uv) {
        // Center the UV coordinates
        uv = uv * 2.0 - 1.0;
        
        // Apply barrel distortion
        float r2 = dot(uv, uv);
        float distortion = 1.0 + bulge * r2;
        uv *= distortion;
        
        // Convert back to 0-1 range
        uv = uv * 0.5 + 0.5;
        return uv;
      }

      void main() {
        vec2 distortedUV = crtDistort(vUv);
        
        // Sample the texture with distorted UV
        vec4 color = texture2D(map, distortedUV);
        
        // Apply brightness and contrast
        color.rgb = color.rgb * brightness;
        color.rgb = ((color.rgb - 0.5) * contrast) + 0.5;
        
        // Add scanlines
        float scanlinePattern = sin(distortedUV.y * scanlineCount) * scanlineIntensity;
        color.rgb += scanlinePattern;
        
        // Add glow effect
        color.rgb += glowColor * glowIntensity;
        
        // Add vignette effect
        float vignette = smoothstep(vignetteRadius, 1.0, distance(vUv, vec2(0.5)));
        color.rgb *= (1.0 - vignette * vignetteIntensity);
        
        // Add subtle flicker effect
        float flicker = 1.0 + sin(time * 60.0) * 0.005;
        color.rgb *= flicker;
        
        gl_FragColor = color;
      }
    `;

    return new THREE.ShaderMaterial({
      uniforms: {
        map: { value: this.terminalTexture },
        bulge: { value: 0.15 },
        scanlineIntensity: { value: 0.04 },
        scanlineCount: { value: 800.0 },
        vignetteIntensity: { value: 0.3 },
        vignetteRadius: { value: 0.3 },
        glowIntensity: { value: 0.02 },
        glowColor: { value: new THREE.Vector3(0.0, 0.02, 0.02) },
        brightness: { value: 1.0 },
        contrast: { value: 1.1 },
        time: { value: 0.0 },
      },
      vertexShader: vertexShader,
      fragmentShader: fragmentShader,
      transparent: true,
    });
  }

  // 8. Dynamic Shader Parameter Control Methods
  setBulge(value) {
    if (
      this.screenMesh &&
      this.screenMesh.material &&
      this.screenMesh.material.uniforms
    ) {
      this.screenMesh.material.uniforms.bulge.value = Math.max(
        0,
        Math.min(0.5, value),
      );
      console.log(
        `Bulge set to: ${this.screenMesh.material.uniforms.bulge.value}`,
      );
    }
  }

  setScanlines(intensity, count = null) {
    if (
      this.screenMesh &&
      this.screenMesh.material &&
      this.screenMesh.material.uniforms
    ) {
      this.screenMesh.material.uniforms.scanlineIntensity.value = Math.max(
        0,
        Math.min(0.2, intensity),
      );
      if (count !== null) {
        this.screenMesh.material.uniforms.scanlineCount.value = Math.max(
          100,
          Math.min(2000, count),
        );
      }
      console.log(
        `Scanlines: intensity=${this.screenMesh.material.uniforms.scanlineIntensity.value}, count=${this.screenMesh.material.uniforms.scanlineCount.value}`,
      );
    }
  }

  setVignette(intensity, radius = null) {
    if (
      this.screenMesh &&
      this.screenMesh.material &&
      this.screenMesh.material.uniforms
    ) {
      this.screenMesh.material.uniforms.vignetteIntensity.value = Math.max(
        0,
        Math.min(1, intensity),
      );
      if (radius !== null) {
        this.screenMesh.material.uniforms.vignetteRadius.value = Math.max(
          0,
          Math.min(1, radius),
        );
      }
      console.log(
        `Vignette: intensity=${this.screenMesh.material.uniforms.vignetteIntensity.value}, radius=${this.screenMesh.material.uniforms.vignetteRadius.value}`,
      );
    }
  }

  setGlow(intensity, color = null) {
    if (
      this.screenMesh &&
      this.screenMesh.material &&
      this.screenMesh.material.uniforms
    ) {
      this.screenMesh.material.uniforms.glowIntensity.value = Math.max(
        0,
        Math.min(0.1, intensity),
      );
      if (color !== null) {
        this.screenMesh.material.uniforms.glowColor.value.copy(color);
      }
      console.log(
        `Glow intensity set to: ${this.screenMesh.material.uniforms.glowIntensity.value}`,
      );
    }
  }

  setBrightnessContrast(brightness, contrast = null) {
    if (
      this.screenMesh &&
      this.screenMesh.material &&
      this.screenMesh.material.uniforms
    ) {
      this.screenMesh.material.uniforms.brightness.value = Math.max(
        0.1,
        Math.min(3, brightness),
      );
      if (contrast !== null) {
        this.screenMesh.material.uniforms.contrast.value = Math.max(
          0.1,
          Math.min(3, contrast),
        );
      }
      console.log(
        `Brightness: ${this.screenMesh.material.uniforms.brightness.value}, Contrast: ${this.screenMesh.material.uniforms.contrast.value}`,
      );
    }
  }

  // Preset configurations
  setCRTPreset(preset) {
    switch (preset) {
      case "modern":
        this.setBulge(0.05);
        this.setScanlines(0.01, 1200);
        this.setVignette(0.1, 0.4);
        this.setGlow(0.005);
        this.setBrightnessContrast(1.0, 1.0);
        break;
      case "vintage":
        this.setBulge(0.25);
        this.setScanlines(0.06, 600);
        this.setVignette(0.4, 0.2);
        this.setGlow(0.03);
        this.setBrightnessContrast(0.9, 1.3);
        break;
      case "retro":
        this.setBulge(0.35);
        this.setScanlines(0.08, 400);
        this.setVignette(0.5, 0.15);
        this.setGlow(0.04);
        this.setBrightnessContrast(0.8, 1.5);
        break;
      case "flat":
        this.setBulge(0.0);
        this.setScanlines(0.0, 800);
        this.setVignette(0.0, 0.3);
        this.setGlow(0.0);
        this.setBrightnessContrast(1.0, 1.0);
        break;
      default:
        this.setBulge(0.15);
        this.setScanlines(0.04, 800);
        this.setVignette(0.3, 0.3);
        this.setGlow(0.02);
        this.setBrightnessContrast(1.0, 1.1);
    }
    console.log(`CRT preset '${preset}' applied`);
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
              this.sceneMeshes.push(c); // Store for fade-in animation

              if (c.material) {
                if (c.material.isMeshStandardMaterial) {
                  c.material.envMapIntensity = 1.0;
                  c.material.roughness = Math.max(
                    0.2,
                    c.material.roughness * 0.9,
                  );
                  c.material.metalness = Math.min(
                    0.9,
                    c.material.metalness * 1.1,
                  );

                  if (c.material.emissive) {
                    c.material.emissive.multiplyScalar(1.2);
                  }
                } else if (c.material.isMeshBasicMaterial) {
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
          this.startSceneFadeIn(); // Start fade-in animation
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
      // Use CRT shader material instead of standard material
      this.screenMesh.material = this.createCRTMaterial();
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

  setupEventListeners() {
    window.addEventListener("resize", () => {
      this.camera.aspect = window.innerWidth / window.innerHeight;
      this.camera.updateProjectionMatrix();
      this.renderer.setSize(window.innerWidth, window.innerHeight);

      if (this.composer) {
        this.composer.setSize(window.innerWidth, window.innerHeight);
      }
    });

    this.raycaster = new THREE.Raycaster();
    this.mouse = new THREE.Vector2();

    this.renderer.domElement.addEventListener("mousemove", (event) => {
      this.stopIdleRotation(); // Stop idle rotation on interaction

      // Track different interaction times
      this.lastInteractionTime = Date.now();
      if (this.isTerminalFocused) {
        this.lastFocusedInteractionTime = Date.now();
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
      this.stopIdleRotation(); // Stop idle rotation on interaction

      // Track interaction
      this.lastInteractionTime = Date.now();
      this.lastFocusedInteractionTime = Date.now();

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
          console.log("Screen clicked");
          this.isTerminalFocused = true;
          this.animateToState("focused"); // Animate to focused state

          if (this.hiddenInput) {
            this.hiddenInput.focus();
          }

          const focusEvent = new CustomEvent("terminalFocus");
          window.dispatchEvent(focusEvent);
        } else {
          console.log("Clicked on PC model");
          this.isTerminalFocused = false;
          this.animateToState("default"); // Animate to default state

          if (this.hiddenInput) {
            this.hiddenInput.blur();
          }

          const blurEvent = new CustomEvent("terminalBlur");
          window.dispatchEvent(blurEvent);
        }
      }
    });

    // Mouse wheel disabled for zoom, but kept for terminal scrolling
    this.renderer.domElement.addEventListener("wheel", (event) => {
      if (this.isHoveringScreen) {
        event.preventDefault();
        this.stopIdleRotation(); // Stop idle rotation on interaction

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

    // Enhanced keyboard shortcuts with CRT parameter controls
    window.addEventListener("keydown", (event) => {
      this.stopIdleRotation(); // Stop idle rotation on interaction

      // Track interaction
      this.lastInteractionTime = Date.now();
      if (this.isTerminalFocused) {
        this.lastFocusedInteractionTime = Date.now();
      }

      switch (event.key) {
        // Camera controls
        case "1":
          this.animateToState("default");
          break;
        case "2":
          this.animateToState("focused");
          break;
        case "3":
          this.animateToState("overview");
          break;
        case "0":
          this.startIdleRotation();
          break;

        // CRT Bulge controls
        case "-":
        case "_":
          if (this.screenMesh?.material?.uniforms?.bulge) {
            const currentBulge = this.screenMesh.material.uniforms.bulge.value;
            this.setBulge(currentBulge - 0.05);
          }
          break;
        case "+":
        case "=":
          if (this.screenMesh?.material?.uniforms?.bulge) {
            const currentBulge = this.screenMesh.material.uniforms.bulge.value;
            this.setBulge(currentBulge + 0.05);
          }
          break;

        // Scanline controls
        case "[":
          if (this.screenMesh?.material?.uniforms?.scanlineIntensity) {
            const current =
              this.screenMesh.material.uniforms.scanlineIntensity.value;
            this.setScanlines(current - 0.01);
          }
          break;
        case "]":
          if (this.screenMesh?.material?.uniforms?.scanlineIntensity) {
            const current =
              this.screenMesh.material.uniforms.scanlineIntensity.value;
            this.setScanlines(current + 0.01);
          }
          break;

        // Brightness controls
        case ",":
        case "<":
          if (this.screenMesh?.material?.uniforms?.brightness) {
            const current = this.screenMesh.material.uniforms.brightness.value;
            this.setBrightnessContrast(current - 0.1);
          }
          break;
        case ".":
        case ">":
          if (this.screenMesh?.material?.uniforms?.brightness) {
            const current = this.screenMesh.material.uniforms.brightness.value;
            this.setBrightnessContrast(current + 0.1);
          }
          break;

        // CRT Presets (Ctrl + number)
        case "4":
          if (event.ctrlKey) this.setCRTPreset("flat");
          break;
        case "5":
          if (event.ctrlKey) this.setCRTPreset("modern");
          break;
        case "6":
          if (event.ctrlKey) this.setCRTPreset("default");
          break;
        case "7":
          if (event.ctrlKey) this.setCRTPreset("vintage");
          break;
        case "8":
          if (event.ctrlKey) this.setCRTPreset("retro");
          break;
      }
    });

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

    if (this.controls) {
      this.controls.update();
    }

    // Update CRT shader time uniform for subtle animations
    if (
      this.screenMesh &&
      this.screenMesh.material &&
      this.screenMesh.material.uniforms
    ) {
      this.screenMesh.material.uniforms.time.value = Date.now() * 0.001;
    }

    // Update all animation systems
    this.updateIdleRotation();
    this.updateSceneFadeIn();
    this.checkAndCorrectCameraBounds();
    this.updateManualBloom();

    if (this.composer && this.supportsPostProcessing) {
      this.composer.render();
    } else {
      this.renderer.render(this.scene, this.camera);
    }
  }

  dispose() {
    if (this.animationId) cancelAnimationFrame(this.animationId);
    if (this.renderer) this.renderer.dispose();
    if (this.composer) this.composer.dispose();
    if (this.terminalTexture) this.terminalTexture.dispose();
  }
}
