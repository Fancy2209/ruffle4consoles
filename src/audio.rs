use ruffle_core::backend::audio::{
    swf, AudioBackend, AudioMixer, DecodeError, RegisterError, SoundHandle, SoundInstanceHandle,
    SoundStreamInfo, SoundTransform,
};

use ruffle_core::impl_audio_mixer_backend;
use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};

pub struct SdlAudioBackend {
    pub device: AudioDevice<MixerCallback>,
    pub mixer: AudioMixer,
    pub paused: bool,
}

pub struct MixerCallback {
    proxy: ruffle_core::backend::audio::AudioMixerProxy,
}

impl AudioCallback for MixerCallback {
    // Ruffle mixer works with f32 PCM
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.proxy.mix(out);
    }
}

type Error = Box<dyn std::error::Error>;

impl SdlAudioBackend {
pub fn new(sdl2_audio: sdl2::AudioSubsystem) -> Result<Self, Error> {
        let mixer = AudioMixer::new(2, 44100);
        let mixer_proxy = mixer.proxy();
        let spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(2),
            samples: None
        };

        let device = sdl2_audio.open_playback(None, &spec, |_spec|MixerCallback { proxy: mixer_proxy });
        let result = Self {
            device: device.unwrap(),
            mixer,
            paused: false,
        };
        result.device.resume();
        Ok(result)
    
}
}

impl AudioBackend for SdlAudioBackend
{
    impl_audio_mixer_backend!(mixer);

    fn play(&mut self)
    {
        self.paused = false;
        self.device.resume();
    }

    fn pause(&mut self)
    {
        self.device.pause();
        self.paused = true;

    }
}
