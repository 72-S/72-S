import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";

export class ModelManager {
  constructor(scene, updateProgress) {
    this.scene = scene;
    this.updateProgress = updateProgress;
    this.pcModel = null;
    this.screenMesh = null;
    this.sceneMeshes = [];
    this.fadeStartTime = null;
    this.fadeInDuration = 4000;
  }

  async loadPCModel() {
    return new Promise((resolve) => {
      const loader = new GLTFLoader();
      loader.load(
        "./pc.glb",
        (gltf) => {
          this.pcModel = gltf.scene;
          this.pcModel.scale.setScalar(1);
          this.pcModel.position.set(0, 0, 0);

          this.pcModel.traverse((c) => {
            if (c.isMesh) {
              c.castShadow = c.receiveShadow = true;
              this.sceneMeshes.push(c);

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
              }
            }
          });

          if (!this.screenMesh) {
            this.findScreenMesh();
          }
          this.scene.add(this.pcModel);
          this.startSceneFadeIn();
          resolve();
        },
        (prog) => {
          const pct = 30 + (prog.loaded / prog.total) * 30;
          this.updateProgress(pct);
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

  startSceneFadeIn() {
    this.fadeStartTime = Date.now();

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

  easeInOutCubic(t) {
    return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
  }

  isScreenObject(object) {
    return (
      object.name === "Plane008_Material002_0" ||
      object === this.screenMesh ||
      (object.name && object.name.toLowerCase().includes("screen"))
    );
  }
}
