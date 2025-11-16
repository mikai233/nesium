use super::*;
use once_cell::sync::Lazy;
use std::{
    ffi::{c_void, CStr, CString},
    mem::MaybeUninit,
    ptr,
    slice,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

static EVENTS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(Vec::new()));
static VIDEO_FRAMES: Lazy<Mutex<Vec<(u32, u32, usize, bool)>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
static AUDIO_SAMPLES: Lazy<Mutex<Vec<(i16, i16)>>> = Lazy::new(|| Mutex::new(Vec::new()));
static AUDIO_BATCHES: Lazy<Mutex<Vec<Vec<(i16, i16)>>>> = Lazy::new(|| Mutex::new(Vec::new()));
static INPUT_POLLS: AtomicUsize = AtomicUsize::new(0);
static INPUT_REQUESTS: Lazy<Mutex<Vec<(u32, u32, u32, u32)>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
static ENV_CMDS: Lazy<Mutex<Vec<u32>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn log(event: impl Into<String>) {
    EVENTS.lock().unwrap().push(event.into());
}

#[derive(Default)]
struct DummyCore;

impl LibretroCore for DummyCore {
    fn construct() -> Self
    where
        Self: Sized,
    {
        log("construct");
        Self
    }

    fn system_info() -> SystemInfo {
        SystemInfo::new("DummyCore", "1.0.0").with_extensions("nes|unf")
    }

    fn init(&mut self) {
        log("init");
    }

    fn deinit(&mut self) {
        log("deinit");
    }

    fn reset(&mut self) {
        log("reset");
    }

    fn run(&mut self, runtime: &mut RuntimeHandles) {
        log("run");
        if let Some(video) = runtime.video() {
            static FRAME: [u8; 4] = [1, 2, 3, 4];
            video.submit(Frame::from_pixels(&FRAME, 2, 2, 4));
        }

        let audio = runtime.audio();
        audio.push_sample(7, -7);
        audio.push_frames(&[[1, -1], [2, -2]]);

        if let Some(input) = runtime.input() {
            input.poll();
            let _ = input.state(
                0,
                raw::RETRO_DEVICE_JOYPAD,
                0,
                raw::RETRO_DEVICE_ID_JOYPAD_A,
            );
        }

        if let Some(env) = runtime.environment() {
            let mut dummy = 0u32;
            env.request(raw::RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE, &mut dummy);
        }
    }

    fn system_av_info(&mut self) -> SystemAvInfo {
        log("av-info");
        SystemAvInfo {
            geometry: GameGeometry {
                base_width: 256,
                base_height: 240,
                max_width: 256,
                max_height: 256,
                aspect_ratio: 4.0 / 3.0,
            },
            timing: SystemTiming {
                fps: 60.0,
                sample_rate: 44100.0,
            },
        }
    }

    fn load_game(&mut self, game: &GameInfo<'_>) -> Result<(), LoadGameError> {
        log(format!(
            "load({:?},{})",
            game.path,
            game.data.as_ref().map_or(0, |d| d.len())
        ));
        if game.data.is_none() && game.path.is_none() {
            return Err(LoadGameError::MissingContent);
        }
        Ok(())
    }

    fn unload_game(&mut self) {
        log("unload");
    }

    fn serialize_size(&self) -> usize {
        log("serialize-size");
        8
    }

    fn serialize(&mut self, dst: &mut [u8]) -> Result<usize, SerializeError> {
        log(format!("serialize({})", dst.len()));
        if dst.len() < 8 {
            return Err(SerializeError::BufferTooSmall { required: 8 });
        }
        dst[..8].copy_from_slice(&0xDEADBEEFu64.to_le_bytes());
        Ok(8)
    }

    fn unserialize(&mut self, src: &[u8]) -> Result<(), SerializeError> {
        log(format!("unserialize({})", src.len()));
        if src.len() < 8 {
            return Err(SerializeError::Message("state too small".into()));
        }
        Ok(())
    }

    fn cheat_set(&mut self, index: u32, enabled: bool, code: &str) {
        log(format!("cheat({index},{enabled},{code})"));
    }

    fn set_controller_port_device(&mut self, port: u32, device: u32) {
        log(format!("controller({port},{device})"));
    }
}

crate::export_libretro_core!(DummyCore);

unsafe extern "C" fn video_cb(
    data: *const c_void,
    width: u32,
    height: u32,
    pitch: usize,
) {
    VIDEO_FRAMES
        .lock()
        .unwrap()
        .push((width, height, pitch, data.is_null()));
}

unsafe extern "C" fn audio_sample_cb(left: i16, right: i16) {
    AUDIO_SAMPLES.lock().unwrap().push((left, right));
}

unsafe extern "C" fn audio_batch_cb(data: *const i16, frames: usize) -> usize {
    let slice = unsafe { slice::from_raw_parts(data, frames * 2) };
    let mut chunk = Vec::with_capacity(frames);
    for pair in slice.chunks_exact(2) {
        chunk.push((pair[0], pair[1]));
    }
    AUDIO_BATCHES.lock().unwrap().push(chunk);
    frames
}

unsafe extern "C" fn input_poll_cb() {
    INPUT_POLLS.fetch_add(1, Ordering::SeqCst);
}

unsafe extern "C" fn input_state_cb(
    port: u32,
    device: u32,
    index: u32,
    id: u32,
) -> i16 {
    INPUT_REQUESTS.lock().unwrap().push((port, device, index, id));
    1
}

unsafe extern "C" fn environment_cb(cmd: u32, _data: *mut c_void) -> bool {
    ENV_CMDS.lock().unwrap().push(cmd);
    true
}

fn reset_logs() {
    EVENTS.lock().unwrap().clear();
    VIDEO_FRAMES.lock().unwrap().clear();
    AUDIO_SAMPLES.lock().unwrap().clear();
    AUDIO_BATCHES.lock().unwrap().clear();
    INPUT_POLLS.store(0, Ordering::SeqCst);
    INPUT_REQUESTS.lock().unwrap().clear();
    ENV_CMDS.lock().unwrap().clear();
}

#[test]
fn exports_delegate_to_trait() {
    reset_logs();

    unsafe {
        retro_set_environment(Some(environment_cb));
        retro_set_video_refresh(Some(video_cb));
        retro_set_audio_sample(Some(audio_sample_cb));
        retro_set_audio_sample_batch(Some(audio_batch_cb));
        retro_set_input_poll(Some(input_poll_cb));
        retro_set_input_state(Some(input_state_cb));
    }

    let mut info = MaybeUninit::<raw::retro_system_info>::uninit();
    unsafe { retro_get_system_info(info.as_mut_ptr()) };
    let info = unsafe { info.assume_init() };
    let name = unsafe { CStr::from_ptr(info.library_name) }
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(name, "DummyCore");
    assert!(!info.valid_extensions.is_null());

    unsafe { retro_init() };

    let rom = [0xAAu8; 4];
    let game = raw::retro_game_info {
        path: ptr::null(),
        data: rom.as_ptr() as *const c_void,
        size: rom.len(),
        meta: ptr::null(),
    };

    assert!(unsafe { retro_load_game(&game) });

    let mut av = MaybeUninit::<raw::retro_system_av_info>::uninit();
    unsafe { retro_get_system_av_info(av.as_mut_ptr()) };
    let av = unsafe { av.assume_init() };
    assert_eq!(av.geometry.base_width, 256);
    assert_eq!(av.timing.sample_rate, 44100.0);

    assert!(!unsafe { retro_load_game_special(0, ptr::null(), 0) });

    unsafe { retro_run() };

    assert_eq!(unsafe { retro_serialize_size() }, 8);
    let mut state = [0u8; 8];
    assert!(unsafe { retro_serialize(state.as_mut_ptr() as *mut c_void, state.len()) });
    assert_eq!(state, 0xDEADBEEFu64.to_le_bytes());
    assert!(unsafe { retro_unserialize(state.as_ptr() as *const c_void, state.len()) });

    let cheat = CString::new("XYZ").unwrap();
    unsafe { retro_cheat_set(1, true, cheat.as_ptr()) };
    unsafe { retro_set_controller_port_device(0, raw::RETRO_DEVICE_JOYPAD) };

    unsafe { retro_unload_game() };
    unsafe { retro_deinit() };

    let events = EVENTS.lock().unwrap().clone();
    assert!(events.iter().any(|e| e == "construct"));
    assert!(events.iter().any(|e| e == "init"));
    assert!(events.iter().any(|e| e == "run"));
    assert!(events.iter().any(|e| e == "deinit"));
    assert!(events.iter().any(|e| e.starts_with("cheat(")));
    assert!(events.iter().any(|e| e.starts_with("controller(")));

    assert_eq!(VIDEO_FRAMES.lock().unwrap().len(), 1);
    assert_eq!(AUDIO_SAMPLES.lock().unwrap().len(), 1);
    assert_eq!(AUDIO_BATCHES.lock().unwrap().len(), 1);
    assert_eq!(INPUT_POLLS.load(Ordering::SeqCst), 1);
    assert_eq!(INPUT_REQUESTS.lock().unwrap().len(), 1);
    assert_eq!(ENV_CMDS.lock().unwrap().len(), 1);
}
