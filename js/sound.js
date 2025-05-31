export class SoundManager {
  constructor() {
    this.sounds = {
      boot: new Audio("../audio/boot.wav"),
    };
    // this.sounds.ambient.loop = true;
    this.ambientStarted = false;
    this.setVolume(0.4);
  }

  setVolume(volume) {
    Object.values(this.sounds).forEach((s) => {
      s.volume = volume;
    });
  }

  play(name) {
    if (!this.sounds[name]) return;
    if (name === "ambient") {
      if (!this.ambientStarted) {
        this.sounds.ambient.play();
        this.ambientStarted = true;
      }
    } else {
      const snd = this.sounds[name].cloneNode();
      snd.volume = this.sounds[name].volume;
      snd.play();
    }
  }

  stop(name) {
    if (this.sounds[name]) {
      this.sounds[name].pause();
      this.sounds[name].currentTime = 0;
    }
  }
}

export const soundManager = new SoundManager();
