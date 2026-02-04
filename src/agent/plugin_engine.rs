use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use wasmtime::{Engine, Module, Store, Linker};
use wasmtime_wasi::WasiCtxBuilder;
use std::io::Write;

/// Simple stdout capturer implementing `Write`
struct WriteCapturer {
    buf: Vec<u8>,
}
impl WriteCapturer {
    fn new() -> Self { Self { buf: Vec::new() } }
    fn take(&mut self) -> Vec<u8> { std::mem::take(&mut self.buf) }
}
impl Write for WriteCapturer {
    fn write(&mut self, data: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

pub struct PluginEngine {
    engine: Engine,
    modules: HashMap<String, Module>,
    skills_dir: PathBuf,
}

impl PluginEngine {
    pub fn new(skills_dir: Option<PathBuf>) -> Result<Self> {
        let engine = Engine::default();
        let skills_dir = skills_dir.unwrap_or_else(|| {
            let mut p = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("./"));
            p.push(".supercode/skills");
            p
        });
        Ok(Self { engine, modules: HashMap::new(), skills_dir })
    }

    /// Scan the skills directory, compile wasm modules and cache them as `Module`
    pub fn load_skills(&mut self) -> Result<()> {
        fs::create_dir_all(&self.skills_dir)?;
        for entry in fs::read_dir(&self.skills_dir)? {
            let e = entry?;
            let p = e.path();
            if p.is_file() {
                let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");
                let name = p.file_stem().and_then(|s| s.to_str()).unwrap_or("skill").to_string();
                if ext == "wasm" {
                    let module = Module::from_file(&self.engine, &p)?;
                    self.modules.insert(name, module);
                } else if ext == "wat" {
                    // parse WAT text into wasm bytes
                    let s = std::fs::read_to_string(&p)?;
                    let wasm = wat::parse_str(&s)?;
                    let module = Module::new(&self.engine, &wasm)?;
                    self.modules.insert(name, module);
                }
            }
        }
        Ok(())
    }

    /// Call a skill by name. The skill can call host-provided functions to communicate
    /// with the host. We provide a `host.write(ptr, len)` function which reads linear memory
    /// from the guest and appends it to a host-side buffer which is returned as the output.
    pub fn call_skill(&self, name: &str, _input: Option<&str>) -> Result<String> {
        use wasmtime::{Caller, Extern};
        use std::sync::{Arc, Mutex};

        let module = self.modules.get(name).ok_or_else(|| anyhow::anyhow!("skill not found"))?;

        // Host state: a simple buffer to collect strings emitted by the wasm module.
        struct HostData {
            out: Arc<Mutex<String>>,
            wasi: wasmtime_wasi::WasiCtx,
        }

        let host_state = HostData {
            out: Arc::new(Mutex::new(String::new())),
            wasi: WasiCtxBuilder::new().inherit_stdio().build(),
        };
        let mut store = Store::new(&self.engine, host_state);
        let mut linker: Linker<HostData> = Linker::new(&self.engine);

        // Add a host function `host.write(ptr: i32, len: i32)`
        linker.func_wrap("host", "write", move |mut caller: Caller<'_, HostData>, ptr: i32, len: i32| {
            // get memory
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(m)) => m,
                _ => return Ok(()),
            };
            let mut buf = vec![0u8; len as usize];
            if mem.read(&caller, ptr as usize, &mut buf).is_err() {
                return Ok(());
            }
            let s = String::from_utf8_lossy(&buf).to_string();
            if let Ok(mut out) = caller.data().out.lock() {
                out.push_str(&s);
            }
            Ok(())
        })?;

        // Add `host.readdir(ptr, len)` which reads a path from memory and writes a JSON array of filenames
        linker.func_wrap("host", "readdir", move |mut caller: Caller<'_, HostData>, ptr: i32, len: i32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(m)) => m,
                _ => return Ok(()),
            };
            let mut buf = vec![0u8; len as usize];
            if mem.read(&caller, ptr as usize, &mut buf).is_err() {
                return Ok(());
            }
            let path = String::from_utf8_lossy(&buf).to_string();
            let mut list = vec![];
            if let Ok(entries) = std::fs::read_dir(&path) {
                for e in entries.flatten() {
                    if let Some(n) = e.file_name().to_str() { list.push(n.to_string()); }
                }
            }
            let json = match serde_json::to_string(&list) {
                Ok(json) => json,
                Err(_) => return Ok(()),
            };
            if let Ok(mut out) = caller.data().out.lock() {
                out.push_str(&json);
            }
            Ok(())
        })?;

        // Add WASI support as well (optional) so modules can use standard libs if desired
        wasmtime_wasi::add_to_linker(&mut linker, |data: &mut HostData| &mut data.wasi)?;

        // Instantiate and call
        let instance = linker.instantiate(&mut store, module)?;
        if let Some(func) = instance.get_func(&mut store, "run") {
            let call = func.typed::<(), ()>(&store)?;
            call.call(&mut store, ())?;
        } else if let Some(start) = instance.get_func(&mut store, "_start") {
            let call = start.typed::<(), ()>(&store)?;
            call.call(&mut store, ())?;
        }

        // Extract host buffer
        let out = store.data().out.lock().unwrap().clone();
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_skill_host_write() -> anyhow::Result<()> {
        // a tiny WAT module that has an exported memory and calls host.write(0, 11)
        let wat = r#"(module
            (import "host" "write" (func $write (param i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "Hello Wasm!")
            (func (export "run")
                i32.const 0
                i32.const 11
                call $write
            )
        )"#;

        let engine = Engine::default();
        let mut pe = PluginEngine { engine: engine.clone(), modules: HashMap::new(), skills_dir: dirs::home_dir().unwrap_or_default() };
        let wasm = wat::parse_str(wat)?;
        let module = Module::new(&engine, &wasm)?;
        pe.modules.insert("test".into(), module);
        let out = pe.call_skill("test", None)?;
        assert!(out.contains("Hello Wasm"));
        Ok(())
    }

    #[test]
    fn test_call_skill_readdir() -> anyhow::Result<()> {
        // create a tempdir with files
        let td = tempfile::tempdir()?;
        std::fs::write(td.path().join("a.txt"), b"x")?;
        std::fs::write(td.path().join("b.txt"), b"y")?;

        // wat module with path data segment
        let path = td.path().to_str().unwrap();
        let wat = format!(r#"(module
            (import "host" "readdir" (func $readdir (param i32 i32)))
            (memory (export "memory") 1)
            (data (i32.const 0) "{}")
            (func (export "run")
                i32.const 0
                i32.const {}
                call $readdir
            )
        )"#, path, path.len());

        let engine = Engine::default();
        let mut pe = PluginEngine { engine: engine.clone(), modules: HashMap::new(), skills_dir: dirs::home_dir().unwrap_or_default() };
        let wasm = wat::parse_str(&wat)?;
        let module = Module::new(&engine, &wasm)?;
        pe.modules.insert("readdir_test".into(), module);
        let out = pe.call_skill("readdir_test", None)?;
        assert!(out.contains("a.txt"));
        assert!(out.contains("b.txt"));
        Ok(())
    }
}
