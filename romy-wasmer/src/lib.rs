use byteorder::{LittleEndian, ReadBytesExt};
use std::fs::File;
use std::io::prelude::*;

use romy_core::output::*;
use romy_core::runtime::*;
use romy_core::*;
use wasmer_runtime::memory::MemoryView;
use wasmer_runtime::{imports, instantiate, Func, Instance, Memory};

struct RomyWasmer {
    instance: Instance,
    info: Info,
    memory: Vec<u8>,
}

impl RomyWasmer {
    fn new(mut instance: Instance) -> Self {
        let info: Info = Self::call_on_instance(&mut instance, "init", Option::<&i32>::None);
        let mut memory = Vec::new();
        Self::dump_memory(instance.context().memory(0), &mut memory);
        Self {
            instance,
            info,
            memory,
        }
    }
    fn dump_memory(_memory: &Memory, _to: &mut Vec<u8>) {
        // Fork of wasmer at https://github.com/catt-io/wasmer will allow memory save/load
        // looking for a way to do this without republishing the wasmer crates.
        // memory.get(to);
    }

    fn restore_memory(_memory: &Memory, _data: &[u8]) {
        // memory.set(data).unwrap();
    }
    fn get<'a, T: serde::Deserialize<'a>>(instance: &mut Instance, pointer: usize) -> T {
        let view: MemoryView<u8> = instance.context_mut().memory(0).view();
        let slice: Vec<_> = view[pointer..(pointer + 8)]
            .iter()
            .map(std::cell::Cell::get)
            .collect();
        let size = (&slice[0..8]).read_u64::<LittleEndian>().unwrap() as usize;
        let slice: Vec<_> = view[pointer..(pointer + size + 8)]
            .iter()
            .map(std::cell::Cell::get)
            .collect();

        let result = unsafe { serial::decode_with_size_ptr(slice.as_ptr()) };
        Self::free(instance, pointer);
        result
    }

    fn set(instance: &mut Instance, object: &impl serde::Serialize) -> usize {
        let params = serial::encode_with_size(object);

        let alloc: Func<i32, u32> = instance.func("allocate").unwrap();

        let location = alloc.call(params.len() as i32).unwrap() as usize;
        let view: MemoryView<u8> = instance.context_mut().memory(0).view();
        let slice = &view[location..(location + params.len())];
        for i in 0..params.len() {
            slice[i].set(params[i]);
        }

        location
    }

    fn free(instance: &mut Instance, pointer: usize) {
        let deallocate: Func<u32, ()> = instance.func("deallocate").unwrap();
        deallocate.call(pointer as u32).unwrap()
    }

    fn call_on_instance<'a, T: serde::Deserialize<'a>>(
        instance: &mut Instance,
        id: &str,
        arg: Option<&impl serde::Serialize>,
    ) -> T {
        let pointer = match arg {
            Some(arg) => {
                let location = Self::set(instance, arg);
                let func: Func<u32, u32> = instance.func(id).unwrap();
                let result = func.call(location as u32).unwrap() as usize;
                Self::free(instance, location);
                result
            }
            None => {
                let func: Func<(), u32> = instance.func(id).unwrap();
                func.call().unwrap() as usize
            }
        };

        Self::get(instance, pointer)
    }

    fn call<'a, T: serde::Deserialize<'a>>(
        &mut self,
        id: &str,
        arg: Option<&impl serde::Serialize>,
    ) -> T {
        Self::call_on_instance(&mut self.instance, id, arg)
    }

    fn call_without_return(&mut self, id: &str, arg: Option<&impl serde::Serialize>) {
        let instance = &mut self.instance;

        match arg {
            Some(arg) => {
                let location = Self::set(instance, arg);
                let func: Func<u32, ()> = instance.func(id).unwrap();
                func.call(location as u32).unwrap();
                Self::free(instance, location);
            }
            None => {
                let func: Func<(), ()> = instance.func(id).unwrap();
                func.call().unwrap();
            }
        };
    }
}

impl GameMut for RomyWasmer {
    fn step(&mut self, arguments: &StepArguments) {
        Self::restore_memory(self.instance.context_mut().memory(0), &self.memory);
        self.call_without_return("step", Some(arguments));
        Self::dump_memory(self.instance.context().memory(0), &mut self.memory);
    }

    fn draw(&mut self, arguments: &DrawArguments) -> Image {
        self.call("draw", Some(arguments))
    }

    fn render_audio(&mut self, arguments: &RenderAudioArguments) -> Sound {
        self.call("render_audio", Some(arguments))
    }
}

/// Load up a file and return the Game and Info data as a RunBundle
pub fn load(path: &str) -> Option<RunBundle> {
    if let Ok(mut file) = File::open(path) {
        let mut buffer = Vec::new();
        if file.read_to_end(&mut buffer).is_ok() {
            let import_object = imports! {};
            if let Ok(instance) = instantiate(&buffer, &import_object) {
                let wasm = RomyWasmer::new(instance);
                let info = wasm.info.clone();
                return Some(RunBundle::new(Box::new(wasm), info));
            }
        }
    }

    None
}
