use js_sys::WebAssembly::Memory;
use js_sys::{Array, ArrayBuffer, Function, Object, Promise, Reflect, Uint8Array, WebAssembly};
use romy_core::input::*;
use romy_core::output::*;
use romy_core::runtime::*;
use romy_core::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::{
    AudioContext, AudioContextState, Blob, BlobPropertyBag, Event, Gamepad, GamepadButton, Request,
    RequestInit, RequestMode, Response, Url, Window,
};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn window() -> Window {
    web_sys::window().unwrap()
}

struct ControllerMapper {
    gamepad: Gamepad,
}

impl ControllerMapper {
    fn new(gamepad: Gamepad) -> Self {
        Self { gamepad }
    }
    fn get_button(&self, button: i32) -> bool {
        Reflect::get(self.gamepad.buttons().as_ref(), &button.into())
            .unwrap()
            .dyn_into::<GamepadButton>()
            .unwrap()
            .pressed()
    }
    fn get_axes(&self, axes: i32) -> f32 {
        Reflect::get(self.gamepad.axes().as_ref(), &axes.into())
            .unwrap()
            .as_f64()
            .unwrap() as f32
    }
    fn build_standard_controller(&mut self) -> Controller {
        let gamepads = window().navigator().get_gamepads().unwrap();
        self.gamepad = Reflect::get(gamepads.as_ref(), &self.gamepad.index().into())
            .unwrap()
            .dyn_into::<Gamepad>()
            .unwrap();

        Controller::new(ControllerInit {
            a: self.get_button(0),
            b: self.get_button(1),
            x: self.get_button(2),
            y: self.get_button(3),
            left: self.get_button(14),
            right: self.get_button(15),
            up: self.get_button(12),
            down: self.get_button(13),
            start: self.get_button(9),
            select: self.get_button(8),
            guide: self.get_button(16),
            left_shoulder: self.get_button(4),
            right_shoulder: self.get_button(5),
            left_stick: self.get_button(10),
            right_stick: self.get_button(11),
            left_stick_x: self.get_axes(0),
            left_stick_y: self.get_axes(1),
            right_stick_x: self.get_axes(2),
            right_stick_y: self.get_axes(3),
            left_trigger: if self.get_button(6) { 1.0 } else { 0.0 },
            right_trigger: if self.get_button(7) { 1.0 } else { 0.0 },
        })
    }
}

fn convert_key_code(key: &str) -> Option<KeyCode> {
    match key {
        "Digit1" => Some(KeyCode::_1),
        "Digit2" => Some(KeyCode::_2),
        "Digit3" => Some(KeyCode::_3),
        "Digit4" => Some(KeyCode::_4),
        "Digit5" => Some(KeyCode::_5),
        "Digit6" => Some(KeyCode::_6),
        "Digit7" => Some(KeyCode::_7),
        "Digit8" => Some(KeyCode::_8),
        "Digit9" => Some(KeyCode::_9),
        "Digit0" => Some(KeyCode::_0),
        "KeyA" | "a" | "A" => Some(KeyCode::A),
        "KeyB" | "b" | "B" => Some(KeyCode::B),
        "KeyC" | "c" | "C" => Some(KeyCode::C),
        "KeyD" | "d" | "D" => Some(KeyCode::D),
        "KeyE" | "e" | "E" => Some(KeyCode::E),
        "KeyF" | "f" | "F" => Some(KeyCode::F),
        "KeyG" | "g" | "G" => Some(KeyCode::G),
        "KeyH" | "h" | "H" => Some(KeyCode::H),
        "KeyI" | "i" | "I" => Some(KeyCode::I),
        "KeyJ" | "j" | "J" => Some(KeyCode::J),
        "KeyK" | "k" | "K" => Some(KeyCode::K),
        "KeyL" | "l" | "L" => Some(KeyCode::L),
        "KeyM" | "m" | "M" => Some(KeyCode::M),
        "KeyN" | "n" | "N" => Some(KeyCode::N),
        "KeyO" | "o" | "O" => Some(KeyCode::O),
        "KeyP" | "p" | "P" => Some(KeyCode::P),
        "KeyQ" | "q" | "Q" => Some(KeyCode::Q),
        "KeyR" | "r" | "R" => Some(KeyCode::R),
        "KeyS" | "s" | "S" => Some(KeyCode::S),
        "KeyT" | "t" | "T" => Some(KeyCode::T),
        "KeyU" | "u" | "U" => Some(KeyCode::U),
        "KeyV" | "v" | "V" => Some(KeyCode::V),
        "KeyW" | "w" | "W" => Some(KeyCode::W),
        "KeyX" | "x" | "X" => Some(KeyCode::X),
        "KeyY" | "y" | "Y" => Some(KeyCode::Y),
        "KeyZ" | "z" | "Z" => Some(KeyCode::Z),
        "ArrowUp" => Some(KeyCode::Up),
        "ArrowDown" => Some(KeyCode::Down),
        "ArrowLeft" => Some(KeyCode::Left),
        "ArrowRight" => Some(KeyCode::Right),
        "BracketLeft" => Some(KeyCode::LeftBracket),
        "BracketRight" => Some(KeyCode::RightBracket),
        "Slash" => Some(KeyCode::Slash),
        "Backslash" => Some(KeyCode::Backslash),
        "Comma" => Some(KeyCode::Comma),
        "Period" => Some(KeyCode::Period),
        "Semicolon" => Some(KeyCode::Semicolon),
        "Quote" => Some(KeyCode::Quote),
        _ => None,
    }
}

fn convert_key(scan_code: &str, key_code: &str) -> Option<Key> {
    if let Some(scan_code) = convert_key_code(scan_code) {
        if let Some(key_code) = convert_key_code(key_code) {
            return Some(Key::new(scan_code, key_code));
        }
    }

    None
}

struct InstanceWrapper {
    instance: WebAssembly::Instance,
    memory: Option<ArrayBuffer>,
    scratch: Vec<u8>,
}

impl InstanceWrapper {
    fn new(instance: WebAssembly::Instance) -> Self {
        Self {
            instance,
            memory: None,
            scratch: Vec::new(),
        }
    }
    fn memory(&self) -> WebAssembly::Memory {
        Reflect::get(self.instance.exports().as_ref(), &"memory".into())
            .unwrap()
            .dyn_into::<WebAssembly::Memory>()
            .unwrap()
    }
    fn function(&self, name: &str) -> Function {
        Reflect::get(self.instance.exports().as_ref(), &name.into())
            .unwrap()
            .dyn_into::<Function>()
            .unwrap()
    }
    pub fn free(&self, ptr: u32) {
        self.function("deallocate")
            .call1(&JsValue::undefined(), &JsValue::from_f64(f64::from(ptr)))
            .unwrap();
    }
    pub fn encode(&self, object: &impl serde::Serialize) -> u32 {
        let params = serial::encode_with_size(object);
        let pointer = self
            .function("allocate")
            .call1(
                &JsValue::undefined(),
                &JsValue::from_f64(params.len() as f64),
            )
            .unwrap()
            .as_f64()
            .unwrap() as u32;

        let mem = self.memory();
        let buffer = mem.buffer().dyn_into::<ArrayBuffer>().unwrap();
        let buffer =
            Uint8Array::new_with_byte_offset_and_length(&buffer, pointer, params.len() as u32);
        unsafe {
            buffer.set(&Uint8Array::view(&params), 0);
        }

        pointer
    }
    fn decode<'a, T: serde::Deserialize<'a>>(&'a mut self, pointer: u32) -> T {
        let mem = self.memory();

        let buffer = mem.buffer().dyn_into::<ArrayBuffer>().unwrap();
        let mut size_buffer: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        Uint8Array::new_with_byte_offset_and_length(&buffer, pointer, 8).copy_to(&mut size_buffer);
        let size = u64::from_le_bytes(size_buffer);

        self.scratch.resize(size as usize, 0);
        Uint8Array::new_with_byte_offset_and_length(&buffer, pointer + 8, size as u32)
            .copy_to(&mut self.scratch);
        self.free(pointer);
        serial::decode::<T>(&self.scratch)
    }
    fn call<'a, T: serde::Deserialize<'a>>(&'a mut self, name: &str) -> T {
        let func = self.function(name);
        let pointer = func.call0(&JsValue::undefined()).unwrap().as_f64().unwrap() as u32;
        self.decode(pointer)
    }
    fn call_with_arg<'a, T: serde::Deserialize<'a>>(
        &'a mut self,
        name: &str,
        arg: &impl serde::Serialize,
    ) -> T {
        let arg_pointer = self.encode(arg);
        let result_pointer = self
            .function(name)
            .call1(
                &JsValue::undefined(),
                &JsValue::from_f64(f64::from(arg_pointer)),
            )
            .unwrap()
            .as_f64()
            .unwrap() as u32;
        self.free(arg_pointer);
        self.decode(result_pointer)
    }
    fn call_with_arg_no_return(&self, name: &str, arg: &impl serde::Serialize) {
        let arg_pointer = self.encode(arg);
        self.function(name)
            .call1(
                &JsValue::undefined(),
                &JsValue::from_f64(f64::from(arg_pointer)),
            )
            .unwrap();
        self.free(arg_pointer);
    }
    fn save(&mut self) {
        let mem = self.memory();
        let buffer = mem.buffer().dyn_into::<ArrayBuffer>().unwrap();
        self.memory = Some(buffer.slice(0));
    }
    fn load(&mut self) {
        if let Some(save) = &mut self.memory {
            let pages = save.byte_length() / 65536;
            let desc = Object::new();
            Reflect::set(desc.as_ref(), &"initial".into(), &pages.into()).unwrap();
            let new_mem = Memory::new(&desc).unwrap();

            let buffer = new_mem.buffer().dyn_into::<ArrayBuffer>().unwrap();
            let dest = Uint8Array::new(&buffer);
            let source = Uint8Array::new(save);
            dest.set(&source, 0);

            Reflect::set(
                self.instance.exports().as_ref(),
                &"memory".into(),
                &new_mem.into(),
            )
            .unwrap();
        }
    }
}

struct RomyGame {
    instance: InstanceWrapper,
    info: Info,
    start_time: f64,
    steps: i32,
}

impl GameMut for RomyGame {
    fn step(&mut self, arguments: &StepArguments) {
        self.instance.load();
        self.instance.call_with_arg_no_return("step", arguments);
        self.instance.save();
    }

    fn draw(&mut self, arguments: &DrawArguments) -> Image {
        self.instance.call_with_arg("draw", arguments)
    }

    fn render_audio(&mut self, arguments: &RenderAudioArguments) -> Sound {
        self.instance.call_with_arg("render_audio", arguments)
    }
}

impl RomyGame {
    fn new(instance: WebAssembly::Instance) -> Self {
        let mut instance = InstanceWrapper::new(instance);

        let info: Info = instance.call("init");
        let window = window();
        let start_time = window.performance().unwrap().now();
        instance.save();

        Self {
            instance,
            info,
            start_time,
            steps: 0,
        }
    }
}

fn request_animation_frame(f: &Closure<FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}

struct Audio {
    started: bool,
    samples: Rc<RefCell<VecDeque<f32>>>,
    audio_context: AudioContext,
}

impl Audio {
    fn new(samples: Rc<RefCell<VecDeque<f32>>>) -> Self {
        let mut audio = Audio {
            started: false,
            samples,
            audio_context: AudioContext::new().unwrap(),
        };

        audio.start();

        audio
    }
    fn start(&mut self) {
        if self.started {
            return;
        }
        self.audio_context.resume().unwrap();
        if self.audio_context.state() == AudioContextState::Suspended {
            return;
        }
        self.started = true;
        {
            self.samples.borrow_mut().clear();
        }
        let processor = self.audio_context.create_script_processor_with_buffer_size_and_number_of_input_channels_and_number_of_output_channels(1024, 2, 2).unwrap();
        let samples_inner = self.samples.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::AudioProcessingEvent| {
            let output_buffer = event.output_buffer().unwrap();
            let mut samples = samples_inner.borrow_mut();
            if samples.len() < output_buffer.length() as usize {
                return;
            }
            let mut samples: Vec<_> = samples.drain(..output_buffer.length() as usize).collect();
            for channel in 0..output_buffer.number_of_channels() {
                output_buffer
                    .copy_to_channel(&mut samples, channel as i32)
                    .unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        processor.set_onaudioprocess(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
        let destination = self.audio_context.destination();
        processor.connect_with_audio_node(&destination).unwrap();
    }
}

fn load_wasm(path: &str, romy_game: Rc<RefCell<Option<RomyGame>>>, streaming: bool) {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);
    let imports = Object::new();

    let request = Request::new_with_str_and_init(&path, &opts).unwrap();
    let request_promise = window().fetch_with_request(&request);

    let romy_game_inner = romy_game.clone();
    if streaming {
        let wasm_stream = WebAssembly::instantiate_streaming(&request_promise, &imports);

        //TODO: These future closures are a bit funky to deal with, there has to be a better way
        let wasm_stream_closure = std::rc::Rc::new(std::cell::RefCell::new(None));
        let wasm_stream_closure_inner = wasm_stream_closure.clone();
        *wasm_stream_closure.borrow_mut() = Some(Closure::wrap(Box::new(move |obj: JsValue| {
            let instance = Reflect::get(obj.as_ref(), &"instance".into())
                .unwrap()
                .dyn_into::<WebAssembly::Instance>()
                .unwrap();

            *romy_game_inner.borrow_mut() = Some(RomyGame::new(instance));
            wasm_stream_closure_inner.borrow().as_ref().unwrap();
        }) as Box<FnMut(JsValue)>));
        wasm_stream.then(wasm_stream_closure.borrow().as_ref().unwrap());
    } else {
        let request_closure = std::rc::Rc::new(std::cell::RefCell::new(None));
        let request_closure_inner = request_closure.clone();
        *request_closure.borrow_mut() = Some(Closure::wrap(Box::new(move |obj: JsValue| {
            let buffer_func = Reflect::get(obj.as_ref(), &"arrayBuffer".into())
                .unwrap()
                .dyn_into::<Function>()
                .unwrap();
            let bytes_promise = buffer_func
                .call0(&obj)
                .unwrap()
                .dyn_into::<Promise>()
                .unwrap();
            let bytes_closure = std::rc::Rc::new(std::cell::RefCell::new(None));
            let bytes_closure_inner = bytes_closure.clone();
            let romy_game_inner = romy_game.clone();
            *bytes_closure.borrow_mut() = Some(Closure::wrap(Box::new(move |obj: JsValue| {
                let array = obj.dyn_into::<ArrayBuffer>().unwrap();
                let module = WebAssembly::Module::new(&array).unwrap();
                let instance = WebAssembly::Instance::new(&module, &Object::new()).unwrap();
                *romy_game_inner.borrow_mut() = Some(RomyGame::new(instance));

                bytes_closure_inner.borrow().as_ref().unwrap();
            })
                as Box<FnMut(JsValue)>));
            bytes_promise.then(bytes_closure.borrow().as_ref().unwrap());

            request_closure_inner.borrow().as_ref().unwrap();
        }) as Box<FnMut(JsValue)>));
        request_promise.then(request_closure.borrow().as_ref().unwrap());
    }
}

#[wasm_bindgen]
pub fn bind(
    element: &web_sys::HtmlElement,
    args: Option<String>,
    streaming: Option<bool>,
) -> Result<(), JsValue> {
    let window = window();
    let document = window.document().unwrap();
    let element = element.clone();

    let canvas = document.create_element("canvas")?;
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    element.append_child(&canvas)?;

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let samples = Rc::new(RefCell::new(VecDeque::new()));
    let audio = Rc::new(RefCell::new(Audio::new(samples.clone())));

    let romy_game = Rc::new(RefCell::new(None));

    if let Some(args) = args {
        load_wasm(&args, romy_game.clone(), streaming.unwrap_or(true));
    }

    let keyboard = Rc::new(RefCell::new(Keyboard::default()));
    let controllers = Rc::new(RefCell::new(Vec::new()));

    let audio_inner = audio.clone();
    let keyboard_inner = keyboard.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let key = convert_key(&event.code(), &event.key());
        if let Some(key) = key {
            keyboard_inner.borrow_mut().key_down(key);
        }
        audio_inner.borrow_mut().start();
    }) as Box<dyn FnMut(_)>);
    window.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let audio_inner = audio.clone();
    let keyboard_inner = keyboard.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let key = convert_key_code(&event.code());
        if let Some(key) = key {
            keyboard_inner.borrow_mut().key_up(key);
        }
        audio_inner.borrow_mut().start();
    }) as Box<dyn FnMut(_)>);
    window.add_event_listener_with_callback("keyup", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let controllers_inner = controllers.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::GamepadEvent| {
        let mut controllers = controllers_inner.borrow_mut();
        controllers.push(ControllerMapper::new(event.gamepad().unwrap()));
    }) as Box<dyn FnMut(_)>);
    window
        .add_event_listener_with_callback("gamepadconnected", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let controllers_inner = controllers.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::GamepadEvent| {
        let mut controllers = controllers_inner.borrow_mut();
        controllers.retain(|controller| controller.gamepad.id() != event.gamepad().unwrap().id());
    }) as Box<dyn FnMut(_)>);
    window.add_event_listener_with_callback(
        "gamepaddisconnected",
        closure.as_ref().unchecked_ref(),
    )?;
    closure.forget();

    let romy_game_inner = romy_game.clone();
    let audio_inner = audio.clone();
    let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        audio_inner.borrow_mut().start();

        let file = event
            .data_transfer()
            .unwrap()
            .files()
            .unwrap()
            .item(0)
            .unwrap();
        let data: &Blob = file.as_ref();
        let data = Array::of1(data);
        let mut props = BlobPropertyBag::new();
        props.type_("application/wasm");
        let data = web_sys::Blob::new_with_blob_sequence_and_options(&data, &props).unwrap();
        let url = Url::create_object_url_with_blob(&data).unwrap();

        load_wasm(&url, romy_game_inner.clone(), streaming.unwrap_or(true));

        let event: &Event = event.as_ref();
        event.prevent_default();
    }) as Box<dyn FnMut(_)>);
    element.add_event_listener_with_callback("drop", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
        let event: &Event = event.as_ref();
        event.prevent_default();
    }) as Box<dyn FnMut(_)>);
    element.add_event_listener_with_callback("dragover", closure.as_ref().unchecked_ref())?;
    closure.forget();

    let animation_closure = std::rc::Rc::new(std::cell::RefCell::new(None));
    let animation_closure_inner = animation_closure.clone();
    let samples_inner = samples.clone();
    let romy_game_inner = romy_game.clone();
    let keyboard_inner = keyboard.clone();
    let controllers_inner = controllers.clone();
    *animation_closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let mut input = InputCollection::new();
        input.add_input(InputDevice::Keyboard(keyboard_inner.borrow().clone()));

        let mut controllers = controllers_inner.borrow_mut();
        for controller in controllers.iter_mut() {
            input.add_input(InputDevice::Controller(
                controller.build_standard_controller(),
            ));
        }

        let mut r = romy_game_inner.borrow_mut();
        if let Some(ref mut romy_game) = *r {
            let now = crate::window().performance().unwrap().now();
            let time_span = Duration::from_millis((now - romy_game.start_time) as u64);
            let step = Duration::from_nanos(u64::from(romy_game.info.step_interval()));
            let expected_steps = (time_span.as_micros() / step.as_micros()) as i32;
            while romy_game.steps < expected_steps {
                romy_game.step(&StepArguments::new(
                    input.get_input_arguments(&romy_game.info),
                ));

                let audio = romy_game.render_audio(&RenderAudioArguments {});

                {
                    let mut samples = samples_inner.borrow_mut();
                    let new_samples = audio.samples();
                    for sample in new_samples {
                        samples.push_back(*sample);
                    }

                    //TODO: Don't let the audio get more than 10 steps out, need better solution:
                    if samples.len() > new_samples.len()*10 {
                        samples.clear();
                    }
                }

                romy_game.steps += 1;
            }

            let step_offset =
                (time_span.as_micros() % step.as_micros()) as f32 / step.as_micros() as f32;

            let mut image = romy_game.draw(&DrawArguments::new(320, 240, step_offset));

            let render_width = image.width();
            let render_height = image.height();
            canvas.set_width(render_width as u32);
            canvas.set_height(render_height as u32);

            let image = web_sys::ImageData::new_with_u8_clamped_array(
                Clamped(image.pixels8_mut()),
                render_width as u32,
            )
            .unwrap();
            context.put_image_data(&image, 0.0, 0.0).unwrap();
            let width = element.offset_width();
            let height = element.offset_height();

            let scale =
                (width as f32 / render_width as f32).min(height as f32 / render_height as f32);
            let new_width = (render_width as f32 * scale) as i32;
            let new_height = (render_height as f32 * scale) as i32;

            let padding_left = ((width - new_width) / 2) as i32;
            let padding_top = ((height - new_height) / 2) as i32;

            canvas
                .set_attribute(
                    "style",
                    format!(
                        "width: {}px;
                        height: {}px;
                        position: relative;
                        left: {}px;
                        top: {}px; 
                        image-rendering: -moz-crisp-edges;
                        image-rendering: -webkit-crisp-edges;
                        image-rendering: pixelated;
                        image-rendering: crisp-edges;",
                        new_width, new_height, padding_left, padding_top
                    )
                    .as_str(),
                )
                .unwrap();
        }

        request_animation_frame(animation_closure_inner.borrow().as_ref().unwrap());
    }) as Box<FnMut()>));

    request_animation_frame(animation_closure.borrow().as_ref().unwrap());

    Ok(())
}
