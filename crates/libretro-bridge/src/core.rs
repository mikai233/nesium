use crate::{raw, runtime::RuntimeHandles};
use std::{
    error::Error,
    ffi::{CStr, c_char},
    fmt,
    marker::PhantomData,
    path::PathBuf,
    slice,
};

/// High-level libretro trait implemented by emulator cores.
///
/// Types implementing this trait describe themselves via [`system_info`],
/// drive the frontend through [`run`], and handle the lifecycle hooks the
/// libretro API expects. The [`export_libretro_core`](crate::export_libretro_core)
/// macro wires an implementation into the global C entry points that a frontend
/// like RetroArch calls.
pub trait LibretroCore: Send + 'static {
    /// Constructs a new core instance.
    ///
    /// Called automatically after [`retro_init`](crate::raw::retro_init) and
    /// before any other method on the trait is used.
    fn construct() -> Self
    where
        Self: Sized;

    /// Returns static metadata about the core such as its name and version.
    fn system_info() -> SystemInfo;

    /// Reports the libretro API version implemented by the core.
    ///
    /// Override this only if you need to target an older revision of the
    /// libretro ABI. The default matches the bindings generated from
    /// `libretro.h`.
    fn api_version() -> u32 {
        raw::RETRO_API_VERSION
    }

    /// Initializes global state.
    ///
    /// Called exactly once after [`construct`](Self::construct). The default
    /// implementation does nothing.
    fn init(&mut self) {}

    /// Releases any resources that outlive `run` calls.
    ///
    /// Invoked when the frontend unloads the core. The default implementation
    /// does nothing.
    fn deinit(&mut self) {}

    /// Resets the emulated hardware.
    ///
    /// Frontends call this when the user requests a soft reset or before
    /// re-running the current game.
    fn reset(&mut self) {}

    /// Runs a single emulation step.
    ///
    /// Libretro dictates that `retro_run` renders one frame and produces the
    /// corresponding audio samples. The [`RuntimeHandles`] argument grants
    /// access to the callbacks that push video, audio, input, and environment
    /// data back to the frontend.
    fn run(&mut self, runtime: &mut RuntimeHandles);

    /// Describes current geometry and timing values.
    ///
    /// Called when the frontend queries video/audio parameters; the values are
    /// cached and reused across frames unless updated via environment calls.
    fn system_av_info(&mut self) -> SystemAvInfo;

    /// Loads regular content into the core.
    ///
    /// The [`GameInfo`] argument mirrors the `retro_game_info` structure. Return
    /// an error if the data cannot be understood or deserialized.
    fn load_game(&mut self, game: &GameInfo<'_>) -> Result<(), LoadGameError>;

    /// Loads multi-part or special content.
    ///
    /// Override this if the core supports `retro_load_game_special`. The
    /// default implementation returns an error describing that the feature is
    /// unsupported.
    fn load_game_special(
        &mut self,
        _game_type: u32,
        _games: &[GameInfo<'_>],
    ) -> Result<(), LoadGameError> {
        Err(LoadGameError::Unsupported(
            "special content not implemented".into(),
        ))
    }

    /// Unloads the currently running game and frees associated resources.
    fn unload_game(&mut self);

    /// Reports the amount of memory (in bytes) needed to serialize state.
    ///
    /// Returning zero indicates that serialization is not supported.
    fn serialize_size(&self) -> usize {
        0
    }

    /// Serializes the core state into the provided buffer.
    ///
    /// Implementations should return the number of bytes written or an error
    /// describing why serialization failed. The default implementation reports
    /// [`SerializeError::Unsupported`].
    fn serialize(&mut self, _dst: &mut [u8]) -> Result<usize, SerializeError> {
        Err(SerializeError::Unsupported)
    }

    /// Restores previously serialized state.
    fn unserialize(&mut self, _src: &[u8]) -> Result<(), SerializeError> {
        Err(SerializeError::Unsupported)
    }

    /// Clears all active cheats.
    fn cheat_reset(&mut self) {}

    /// Enables or disables a cheat code.
    ///
    /// The parameters mirror `retro_cheat_set`.
    fn cheat_set(&mut self, _index: u32, _enabled: bool, _code: &str) {}

    /// Configures the device connected to the specified controller port.
    fn set_controller_port_device(&mut self, _port: u32, _device: u32) {}

    /// Returns the television standard of the current game.
    fn region(&self) -> u32 {
        raw::RETRO_REGION_NTSC
    }

    /// Provides raw access to core-managed memory (save RAM, etc.).
    ///
    /// Return `None` if the requested region does not exist.
    fn get_memory_data(&mut self, _id: u32) -> Option<MemoryRegion<'_>> {
        None
    }
}

/// Metadata returned by [`LibretroCore::system_info`].
#[derive(Debug, Clone)]
pub struct SystemInfo {
    /// Human-readable name of the core (e.g. "ExampleCore").
    pub library_name: String,
    /// Semantic version string presented to frontends.
    pub library_version: String,
    /// Pipe-delimited list of file extensions the core can load.
    pub valid_extensions: Option<String>,
    /// Whether the core requires frontends to supply file paths instead of raw
    /// buffers.
    pub need_fullpath: bool,
    /// When true, frontends must not unzip or otherwise preprocess archives
    /// before handing them to the core.
    pub block_extract: bool,
}

impl SystemInfo {
    /// Creates a new [`SystemInfo`] with the provided name and version.
    pub fn new(library_name: impl Into<String>, library_version: impl Into<String>) -> Self {
        Self {
            library_name: library_name.into(),
            library_version: library_version.into(),
            valid_extensions: None,
            need_fullpath: false,
            block_extract: false,
        }
    }

    /// Sets the file extensions supported by the core.
    pub fn with_extensions(mut self, extensions: impl Into<String>) -> Self {
        self.valid_extensions = Some(extensions.into());
        self
    }

    /// Marks whether content must be provided as a path on disk.
    pub fn need_fullpath(mut self, need: bool) -> Self {
        self.need_fullpath = need;
        self
    }

    /// Indicates whether frontends may extract archives automatically.
    pub fn block_extract(mut self, block: bool) -> Self {
        self.block_extract = block;
        self
    }
}

/// Combined AV geometry/timing data.
#[derive(Debug, Clone, Copy)]
pub struct SystemAvInfo {
    /// The dimensions and aspect ratio of the emulated display.
    pub geometry: GameGeometry,
    /// Audio/video timing details.
    pub timing: SystemTiming,
}

impl SystemAvInfo {
    /// Converts the safe abstraction into the raw `retro_system_av_info`.
    pub fn to_raw(&self) -> raw::retro_system_av_info {
        raw::retro_system_av_info {
            geometry: raw::retro_game_geometry {
                base_width: self.geometry.base_width,
                base_height: self.geometry.base_height,
                max_width: self.geometry.max_width,
                max_height: self.geometry.max_height,
                aspect_ratio: self.geometry.aspect_ratio,
            },
            timing: raw::retro_system_timing {
                fps: self.timing.fps,
                sample_rate: self.timing.sample_rate,
            },
        }
    }
}

/// Matches `retro_game_geometry`.
#[derive(Debug, Clone, Copy)]
pub struct GameGeometry {
    /// The default width of the emulated display in pixels.
    pub base_width: u32,
    /// The default height of the emulated display in pixels.
    pub base_height: u32,
    /// The maximum width the core may output when dynamic resizing is used.
    pub max_width: u32,
    /// The maximum height the core may output when dynamic resizing is used.
    pub max_height: u32,
    /// The aspect ratio reported to the frontend.
    pub aspect_ratio: f32,
}

impl GameGeometry {
    /// Creates a square viewport that is `size` pixels wide and tall.
    pub fn square(size: u32) -> Self {
        Self {
            base_width: size,
            base_height: size,
            max_width: size,
            max_height: size,
            aspect_ratio: 1.0,
        }
    }
}

/// Matches `retro_system_timing`.
#[derive(Debug, Clone, Copy)]
pub struct SystemTiming {
    /// Frames per second produced by the core.
    pub fps: f64,
    /// Audio sample rate used by the emulator.
    pub sample_rate: f64,
}

impl SystemTiming {
    /// Convenience constructor for a 60 FPS system with the provided sample rate.
    pub fn ntsc(sample_rate: f64) -> Self {
        Self {
            fps: 60.0,
            sample_rate,
        }
    }
}

/// Safe representation of `retro_game_info`.
#[derive(Debug, Clone)]
pub struct GameInfo<'a> {
    /// Filesystem path to the content, when available.
    pub path: Option<PathBuf>,
    /// Raw bytes of the ROM or disk image.
    pub data: Option<&'a [u8]>,
    /// Optional metadata string supplied by the frontend.
    pub meta: Option<String>,
}

#[allow(dead_code)]
impl<'a> GameInfo<'a> {
    pub(crate) unsafe fn from_ptr(ptr: *const raw::retro_game_info) -> Self {
        if ptr.is_null() {
            return Self {
                path: None,
                data: None,
                meta: None,
            };
        }

        unsafe { Self::from_raw(&*ptr) }
    }

    pub(crate) unsafe fn from_slice(raw: &'a [raw::retro_game_info]) -> Vec<Self> {
        raw.iter()
            .map(|info| unsafe { Self::from_raw(info) })
            .collect()
    }

    unsafe fn from_raw(raw: &'a raw::retro_game_info) -> Self {
        Self {
            path: c_path_to_pathbuf(raw.path),
            data: if raw.data.is_null() {
                None
            } else {
                Some(unsafe { slice::from_raw_parts(raw.data as *const u8, raw.size) })
            },
            meta: c_str_to_string(raw.meta),
        }
    }
}

#[allow(dead_code)]
fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(|s| s.to_owned())
}

#[allow(dead_code)]
fn c_path_to_pathbuf(ptr: *const c_char) -> Option<PathBuf> {
    if ptr.is_null() {
        return None;
    }

    let cstr = unsafe { CStr::from_ptr(ptr) };
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(std::ffi::OsStr::from_bytes(cstr.to_bytes()).into())
    }
    #[cfg(not(unix))]
    {
        cstr.to_str().ok().map(PathBuf::from)
    }
}

/// Error returned by [`LibretroCore::load_game`].
#[derive(Debug)]
pub enum LoadGameError {
    /// The frontend failed to provide content data or a path.
    MissingContent,
    /// The content format is not supported by the core.
    Unsupported(String),
    /// A catch-all error for situations the other variants do not cover.
    Message(String),
}

impl fmt::Display for LoadGameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadGameError::MissingContent => write!(f, "missing content data"),
            LoadGameError::Unsupported(msg) => write!(f, "unsupported content: {msg}"),
            LoadGameError::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for LoadGameError {}

/// Error returned by serialization routines.
#[derive(Debug)]
pub enum SerializeError {
    /// Provided buffer was too small to store the serialized state.
    BufferTooSmall { required: usize },
    /// The core does not support serialization.
    Unsupported,
    /// A catch-all error for situations the other variants do not cover.
    Message(String),
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializeError::BufferTooSmall { required } => {
                write!(f, "buffer too small (requires {required} bytes)")
            }
            SerializeError::Unsupported => write!(f, "serialization is not supported"),
            SerializeError::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for SerializeError {}

/// Safe wrapper for memory regions returned by [`LibretroCore::get_memory_data`].
pub struct MemoryRegion<'a> {
    ptr: *mut u8,
    len: usize,
    _marker: PhantomData<&'a mut [u8]>,
}

impl<'a> MemoryRegion<'a> {
    /// Exposes a mutable slice as a [`MemoryRegion`].
    pub fn from_slice(slice: &'a mut [u8]) -> Self {
        Self {
            ptr: slice.as_mut_ptr(),
            len: slice.len(),
            _marker: PhantomData,
        }
    }

    /// Returns the number of bytes in the mapped region.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` when the region is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a mutable reference to the underlying memory.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    #[allow(dead_code)]
    pub(crate) fn into_raw(self) -> (*mut std::ffi::c_void, usize) {
        (self.ptr.cast(), self.len)
    }
}
