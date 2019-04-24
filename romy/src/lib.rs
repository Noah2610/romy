mod exports;
pub use romy_core::input::InputDeviceType;
pub use romy_core::output::{Color, Image, Sound};
pub use romy_core::{DrawArguments, Game, Info, RenderAudioArguments, StepArguments};

#[cfg(feature = "romy-engine")]
pub use romy_engine as engine;

#[cfg(not(target_arch = "wasm32"))]
pub use romy_sdl::run_standalone;

/// Sets up the main() function for each build target
#[macro_export]
macro_rules! romy_main {
    ($x:expr, $y:expr) => {
        #[cfg(target_arch = "wasm32")]
        fn main() -> Result<(), String> {
            Ok(())
        }
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub unsafe extern "C" fn init() -> *mut u8 {
            connect(Box::new($y), $x)
        }
        #[cfg(not(target_arch = "wasm32"))]
        fn main() -> Result<(), String> {
            run_standalone(Box::new($y), $x)
        }
    };
}

/// Connects a Game to the Wasm erxports
pub fn connect(game: Box<Game>, info: Info) -> *mut u8 {
    let romy = unsafe { &mut exports::ROOT };
    romy.connect(game);
    exports::move_ownership_to_host(info)
}
