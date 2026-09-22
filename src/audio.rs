use macroquad::audio::{PlaySoundParams, Sound, load_sound_from_bytes, play_sound};

pub struct AudioManager {
    pub sound_ak47: Sound,
    pub sound_knife: Sound,
    pub sound_pipe: Sound,
    pub sound_pistol: Sound,
    pub sound_swosh: Sound,
    pub sound_shotgun: Sound,
    pub volume: f32,
}

impl AudioManager {
    pub async fn load() -> Self {
        let sound_ak47 = load_sound_from_bytes(include_bytes!("../assets/ak47.wav"))
            .await
            .unwrap();
        let sound_knife = load_sound_from_bytes(include_bytes!("../assets/knife.wav"))
            .await
            .unwrap();
        let sound_pipe = load_sound_from_bytes(include_bytes!("../assets/pipe.wav"))
            .await
            .unwrap();
        let sound_pistol = load_sound_from_bytes(include_bytes!("../assets/pistol.wav"))
            .await
            .unwrap();
        let sound_swosh = load_sound_from_bytes(include_bytes!("../assets/swosh.wav"))
            .await
            .unwrap();
        let sound_shotgun = load_sound_from_bytes(include_bytes!("../assets/shotgun.wav"))
            .await
            .unwrap();

        Self {
            sound_ak47,
            sound_knife,
            sound_pipe,
            sound_pistol,
            sound_swosh,
            sound_shotgun,
            volume: 0.5,
        }
    }

    pub fn play(&self, sound: &Sound) {
        if self.volume > 0.0 {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: self.volume,
                },
            );
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }
}
