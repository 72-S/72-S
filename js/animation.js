import * as THREE from "three";

export class AnimationManager {
  constructor(camera, controls, onStateChange) {
    this.camera = camera;
    this.controls = controls;
    this.onStateChange = onStateChange;

    this.startupPhase = "waiting";
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
    this.lastFocusedInteractionTime = Date.now();
    this.inactivityDelay = 10000;
    this.focusedInactivityDelay = 180000;
    this.idleRotationSpeed = 0.0002;
    this.idleRadius = 7;
    this.idleHeight = 4.2;
    this.idleAngle = 0;
    this.maxIdleRotation = Math.PI * 1.2;
    this.idleRotationCount = 0;

    this.cameraBounds = {
      position: { x: [-15, 15], y: [2, 12], z: [3, 20] },
      target: { x: [-5, 5], y: [0, 5], z: [-3, 3] },
    };
    this.lastValidCameraPosition = new THREE.Vector3();
    this.lastValidCameraTarget = new THREE.Vector3();
  }

  startupAnimation() {
    this.startupPhase = "waiting";

    setTimeout(() => {
      this.startupPhase = "focusing";
      this.animateToState("focused", 2000, () => {
        this.startupPhase = "complete";
        this.lastInteractionTime = Date.now();
        this.lastFocusedInteractionTime = Date.now();
      });
    }, 1000);
  }

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
      const easeProgress = this.easeInOutCubic(progress);

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
        if (this.onStateChange) this.onStateChange(stateName);
      }
    };

    animateStep();
  }

  updateIdleRotation(isTerminalFocused) {
    if (this.startupPhase !== "complete") return;
    if (this.isAnimatingCamera) return;

    const currentTime = Date.now();
    const timeSinceLastInteraction = currentTime - this.lastInteractionTime;
    const timeSinceLastFocusedInteraction =
      currentTime - this.lastFocusedInteractionTime;

    if (this.currentCameraState === "focused" && isTerminalFocused) {
      if (
        timeSinceLastFocusedInteraction > this.focusedInactivityDelay &&
        !this.idleRotationActive
      ) {
        this.startIdleRotation();
      }
    } else if (this.currentCameraState !== "focused") {
      if (
        timeSinceLastInteraction > this.inactivityDelay &&
        !this.idleRotationActive
      ) {
        this.startIdleRotation();
      }
    }

    if (this.idleRotationActive) {
      this.idleAngle += this.idleRotationSpeed;
      this.idleRotationCount += this.idleRotationSpeed;

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

  stopIdleRotation() {
    this.idleRotationActive = false;
    this.lastInteractionTime = Date.now();
    this.lastFocusedInteractionTime = Date.now();

    if (this.currentCameraState === "idle") {
      this.animateToState("default", 1000);
    }
  }

  startIdleRotation() {
    this.idleRotationActive = false;
    this.isAnimatingCamera = true;
    this.idleRotationCount = 0;

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

    const startPosition = this.camera.position.clone();
    const startTarget = this.controls.target.clone();
    const targetTarget = new THREE.Vector3(0, 1.5, 0);
    const startTime = Date.now();
    const duration = 1500;

    const animateToIdleStart = () => {
      const elapsed = Date.now() - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const easeProgress = this.easeInOutCubic(progress);

      this.camera.position.lerpVectors(
        startPosition,
        idealStartPosition,
        easeProgress,
      );
      this.controls.target.lerpVectors(startTarget, targetTarget, easeProgress);

      if (progress < 1) {
        requestAnimationFrame(animateToIdleStart);
      } else {
        this.isAnimatingCamera = false;
        this.idleRotationActive = true;
        this.currentCameraState = "idle";
      }
    };

    animateToIdleStart();
  }

  resetIdleRotation() {
    this.idleRotationActive = false;
    this.idleRotationCount = 0;

    this.animateToState("default", 1500, () => {
      setTimeout(() => {
        if (
          this.currentCameraState === "default" &&
          this.startupPhase === "complete"
        ) {
          this.startIdleRotation();
        }
      }, 2000);
    });
  }

  checkAndCorrectCameraBounds() {
    const pos = this.camera.position;
    const target = this.controls.target;
    let needsCorrection = false;

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

  easeInOutCubic(t) {
    return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  }
}
