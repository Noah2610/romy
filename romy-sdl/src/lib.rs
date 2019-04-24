use romy_core::input::*;
use romy_core::runtime::*;
use romy_core::*;

use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::controller::Axis;
use sdl2::controller::Button;
use sdl2::controller::GameController;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::keyboard::Scancode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

struct AudioQueue {
    samples: std::sync::Arc<std::sync::RwLock<std::collections::VecDeque<f32>>>,
}

impl AudioCallback for AudioQueue {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut lock = self.samples.write().unwrap();

        if lock.len() < out.len() {
            for sample in out.iter_mut() {
                *sample = 0.0;
            }
            return;
        }

        let slice: Vec<_> = lock.drain(..out.len()).collect();
        out.copy_from_slice(&slice);
    }
}

struct ControllerMapper {
    sdl_controller: GameController,
}

impl ControllerMapper {
    fn new(sdl_controller: GameController) -> Self {
        Self { sdl_controller }
    }
    fn map_axis(value: i16) -> f32 {
        if value > 0 {
            f32::from(value) / 32767.0
        }
        else {
            f32::from(value) / 32768.0
        }
    }
    fn to_standard_controller(&self) -> Controller {
        Controller::new(ControllerInit {
            a: self.sdl_controller.button(Button::A),
            b: self.sdl_controller.button(Button::A),
            x: self.sdl_controller.button(Button::X),
            y: self.sdl_controller.button(Button::Y),
            left: self.sdl_controller.button(Button::DPadLeft),
            right: self.sdl_controller.button(Button::DPadRight),
            up: self.sdl_controller.button(Button::DPadUp),
            down: self.sdl_controller.button(Button::DPadDown),
            start: self.sdl_controller.button(Button::Start),
            select: self.sdl_controller.button(Button::Back),
            guide: self.sdl_controller.button(Button::Guide),
            left_shoulder: self.sdl_controller.button(Button::LeftShoulder),
            right_shoulder: self.sdl_controller.button(Button::RightShoulder),
            left_stick: self.sdl_controller.button(Button::LeftStick),
            right_stick: self.sdl_controller.button(Button::RightStick),
            left_stick_x: Self::map_axis(self.sdl_controller.axis(Axis::LeftX)),
            left_stick_y: Self::map_axis(self.sdl_controller.axis(Axis::LeftY)),
            right_stick_x: Self::map_axis(self.sdl_controller.axis(Axis::RightX)),
            right_stick_y: Self::map_axis(self.sdl_controller.axis(Axis::RightY)),
            left_trigger: Self::map_axis(self.sdl_controller.axis(Axis::TriggerLeft)),
            right_trigger: Self::map_axis(self.sdl_controller.axis(Axis::TriggerRight)),
        })
    }
}

fn convert_scan_code(scan_code: Scancode) -> Option<KeyCode> {
    match scan_code {
        Scancode::Num1 => Some(KeyCode::_1),
        Scancode::Num2 => Some(KeyCode::_2),
        Scancode::Num3 => Some(KeyCode::_3),
        Scancode::Num4 => Some(KeyCode::_4),
        Scancode::Num5 => Some(KeyCode::_5),
        Scancode::Num6 => Some(KeyCode::_6),
        Scancode::Num7 => Some(KeyCode::_7),
        Scancode::Num8 => Some(KeyCode::_8),
        Scancode::Num9 => Some(KeyCode::_9),
        Scancode::Num0 => Some(KeyCode::_0),
        Scancode::A => Some(KeyCode::A),
        Scancode::B => Some(KeyCode::B),
        Scancode::C => Some(KeyCode::C),
        Scancode::D => Some(KeyCode::D),
        Scancode::E => Some(KeyCode::E),
        Scancode::F => Some(KeyCode::F),
        Scancode::G => Some(KeyCode::G),
        Scancode::H => Some(KeyCode::H),
        Scancode::I => Some(KeyCode::I),
        Scancode::J => Some(KeyCode::J),
        Scancode::K => Some(KeyCode::K),
        Scancode::L => Some(KeyCode::L),
        Scancode::M => Some(KeyCode::M),
        Scancode::N => Some(KeyCode::N),
        Scancode::O => Some(KeyCode::O),
        Scancode::P => Some(KeyCode::P),
        Scancode::Q => Some(KeyCode::Q),
        Scancode::R => Some(KeyCode::R),
        Scancode::S => Some(KeyCode::S),
        Scancode::T => Some(KeyCode::T),
        Scancode::U => Some(KeyCode::U),
        Scancode::V => Some(KeyCode::V),
        Scancode::W => Some(KeyCode::W),
        Scancode::X => Some(KeyCode::X),
        Scancode::Y => Some(KeyCode::Y),
        Scancode::Z => Some(KeyCode::Z),
        Scancode::Up => Some(KeyCode::Up),
        Scancode::Down => Some(KeyCode::Down),
        Scancode::Left => Some(KeyCode::Left),
        Scancode::Right => Some(KeyCode::Right),
        Scancode::Return => Some(KeyCode::Enter),
        Scancode::Tab => Some(KeyCode::Tab),
        Scancode::LeftBracket => Some(KeyCode::LeftBracket),
        Scancode::RightBracket => Some(KeyCode::RightBracket),
        Scancode::Slash => Some(KeyCode::Slash),
        Scancode::Backslash => Some(KeyCode::Backslash),
        Scancode::Comma => Some(KeyCode::Comma),
        Scancode::Period => Some(KeyCode::Period),
        Scancode::Semicolon => Some(KeyCode::Semicolon),
        Scancode::Apostrophe => Some(KeyCode::Quote),
        _ => None,
    }
}

fn convert_key_code(scan_code: sdl2::keyboard::Keycode) -> Option<KeyCode> {
    match scan_code {
        Keycode::Num1 => Some(KeyCode::_1),
        Keycode::Num2 => Some(KeyCode::_2),
        Keycode::Num3 => Some(KeyCode::_3),
        Keycode::Num4 => Some(KeyCode::_4),
        Keycode::Num5 => Some(KeyCode::_5),
        Keycode::Num6 => Some(KeyCode::_6),
        Keycode::Num7 => Some(KeyCode::_7),
        Keycode::Num8 => Some(KeyCode::_8),
        Keycode::Num9 => Some(KeyCode::_9),
        Keycode::Num0 => Some(KeyCode::_0),
        Keycode::A => Some(KeyCode::A),
        Keycode::B => Some(KeyCode::B),
        Keycode::C => Some(KeyCode::C),
        Keycode::D => Some(KeyCode::D),
        Keycode::E => Some(KeyCode::E),
        Keycode::F => Some(KeyCode::F),
        Keycode::G => Some(KeyCode::G),
        Keycode::H => Some(KeyCode::H),
        Keycode::I => Some(KeyCode::I),
        Keycode::J => Some(KeyCode::J),
        Keycode::K => Some(KeyCode::K),
        Keycode::L => Some(KeyCode::L),
        Keycode::M => Some(KeyCode::M),
        Keycode::N => Some(KeyCode::N),
        Keycode::O => Some(KeyCode::O),
        Keycode::P => Some(KeyCode::P),
        Keycode::Q => Some(KeyCode::Q),
        Keycode::R => Some(KeyCode::R),
        Keycode::S => Some(KeyCode::S),
        Keycode::T => Some(KeyCode::T),
        Keycode::U => Some(KeyCode::U),
        Keycode::V => Some(KeyCode::V),
        Keycode::W => Some(KeyCode::W),
        Keycode::X => Some(KeyCode::X),
        Keycode::Y => Some(KeyCode::Y),
        Keycode::Z => Some(KeyCode::Z),
        Keycode::Up => Some(KeyCode::Up),
        Keycode::Down => Some(KeyCode::Down),
        Keycode::Left => Some(KeyCode::Left),
        Keycode::Right => Some(KeyCode::Right),
        Keycode::LeftBracket => Some(KeyCode::LeftBracket),
        Keycode::RightBracket => Some(KeyCode::RightBracket),
        Keycode::Slash => Some(KeyCode::Slash),
        Keycode::Backslash => Some(KeyCode::Backslash),
        Keycode::Comma => Some(KeyCode::Comma),
        Keycode::Period => Some(KeyCode::Period),
        Keycode::Semicolon => Some(KeyCode::Semicolon),
        Keycode::Quote => Some(KeyCode::Quote),
        _ => None,
    }
}

fn convert_key(
    scan_code: sdl2::keyboard::Scancode,
    key_code: sdl2::keyboard::Keycode,
) -> Option<Key> {
    if let Some(scan_code) = convert_scan_code(scan_code) {
        if let Some(key_code) = convert_key_code(key_code) {
            return Some(Key::new(scan_code, key_code));
        }
    }

    None
}

pub fn run_standalone(app: Box<Game>, info: Info) -> Result<(), String> {
    run(
        Some(RunBundle {
            game: Box::new(GameMutMap::new(app)),
            info,
        }),
        |_| None,
    )
}

struct RomyGame {
    bundle: RunBundle,
    start_time: Instant,
    step: Duration,
    steps: u128,
}

impl RomyGame {
    fn new(bundle: RunBundle) -> Self {
        let step = Duration::from_nanos(u64::from(bundle.info.step_interval()));

        Self {
            bundle,
            start_time: Instant::now(),
            step,
            steps: 0,
        }
    }
}

/// Runs a RunBundle using SDL2
///
/// # Arguments
/// * `bundle` - Optional bundle to run, if none is supplied the sdl window will open and wait
/// for a game do be dropped onto it.
/// * `load_new` - Callback to get a new bundle from a file path, this will be called if a file is
/// dragged onto the game window.
pub fn run<F>(bundle: Option<RunBundle>, load_new: F) -> Result<(), String>
where
    F: Fn(&str) -> Option<RunBundle>,
{
    let mut title = "Romy".to_string();

    let mut game = match bundle {
        Some(bundle) => {
            title = format!("Romy: {}", bundle.info.name());
            Some(RomyGame::new(bundle))
        }
        None => None,
    };

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(&title, 960, 720)
        .resizable()
        .allow_highdpi()
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::ABGR8888, 320, 240)
        .map_err(|e| e.to_string())?;

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: Some(1024),
    };
    let game_controller_subsystem = sdl_context.game_controller()?;

    let samples = Arc::new(RwLock::new(VecDeque::new()));
    let samples_clone = samples.clone();

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_| AudioQueue {
            samples: samples_clone,
        })
        .unwrap();
    device.resume();

    let mut keyboard = Keyboard::default();
    let mut controllers = Vec::new();

    'mainloop: loop {
        for event in sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(keycode),
                    scancode: Some(scancode),
                    keymod,
                    ..
                } => {
                    if keymod.contains(sdl2::keyboard::Mod::LALTMOD)
                        || keymod.contains(sdl2::keyboard::Mod::RALTMOD)
                    {
                        if keycode == sdl2::keyboard::Keycode::Return {
                            let new_fullscreen_mode = match canvas.window().fullscreen_state() {
                                sdl2::video::FullscreenType::Off => {
                                    sdl2::video::FullscreenType::Desktop
                                }
                                sdl2::video::FullscreenType::Desktop => {
                                    sdl2::video::FullscreenType::Off
                                }
                                sdl2::video::FullscreenType::True => {
                                    sdl2::video::FullscreenType::Off
                                }
                            };

                            canvas
                                .window_mut()
                                .set_fullscreen(new_fullscreen_mode)
                                .unwrap();
                        }
                    } else {
                        let key = convert_key(scancode, keycode);
                        if let Some(key) = key {
                            keyboard.key_down(key);
                        }
                    }
                }
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => {
                    let scancode = convert_scan_code(scancode);
                    if let Some(scancode) = scancode {
                        keyboard.key_up(scancode);
                    }
                }
                Event::DropFile { filename, .. } => {
                    if let Some(bundle) = load_new(&filename) {
                        canvas
                            .window_mut()
                            .set_title(format!("Romy: {}", bundle.info.name()).as_str())
                            .unwrap();

                        game = Some(RomyGame::new(bundle));
                    }
                }
                Event::ControllerDeviceAdded { which, .. } => {
                    if let Ok(c) = game_controller_subsystem.open(which) {
                        controllers.push(ControllerMapper::new(c));
                    }
                }
                Event::ControllerDeviceRemoved { which, .. } => {
                    controllers
                        .retain(|controller| controller.sdl_controller.instance_id() != which);
                }
                Event::Quit { .. } => break 'mainloop,
                _ => {}
            }
        }

        let mut input = InputCollection::new();
        input.add_input(InputDevice::Keyboard(keyboard.clone()));

        for controller in &controllers {
            input.add_input(InputDevice::Controller(controller.to_standard_controller()));
        }

        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        canvas.clear();

        if let Some(game) = &mut game {
            let app = &mut game.bundle.game;
            let info = &game.bundle.info;

            let time_span = Instant::now().duration_since(game.start_time);
            let expected_steps = time_span.as_micros() / game.step.as_micros();
            while game.steps < expected_steps {
                app.step(&StepArguments::new(input.get_input_arguments(&info)));

                let audio = app.render_audio(&RenderAudioArguments {});

                {
                    let mut samples = samples.write().unwrap();
                    let new_samples = audio.samples();
                    for sample in new_samples {
                        samples.push_back(*sample);
                    }

                    //TODO: Don't let the audio get more than 10 steps out, need better solution:
                    if samples.len() > new_samples.len()*10 {
                        samples.clear();
                    }
                }

                game.steps += 1;
            }

            let step_offset = (time_span.as_micros() % game.step.as_micros()) as f32
                / game.step.as_micros() as f32;

            let (width, height) = canvas.output_size().unwrap();
            let render = app.draw(&DrawArguments::new(
                width as i32,
                height as i32,
                step_offset,
            ));

            let t = texture.query();
            if t.width != render.width() as u32 || t.height != render.height() as u32 {
                texture = creator
                    .create_texture_streaming(
                        PixelFormatEnum::ABGR8888,
                        render.width() as u32,
                        render.height() as u32,
                    )
                    .map_err(|e| e.to_string())?;
            }

            texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                let source = render.pixels8();
                buffer.clone_from_slice(&source[..buffer.len()])
            })?;

            let scale =
                (width as f32 / render.width() as f32).min(height as f32 / render.height() as f32);
            let new_width = (render.width() as f32 * scale) as u32;
            let new_height = (render.height() as f32 * scale) as u32;
            let dest = Rect::new(
                ((width - new_width) / 2) as i32,
                ((height - new_height) / 2) as i32,
                new_width,
                new_height,
            );

            canvas.copy(&texture, None, dest)?;
        }

        canvas.present();
    }

    Ok(())
}
