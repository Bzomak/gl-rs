// Copyright 2015 Brendan Zabarauskas and the gl-rs developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::registry::Registry;
use std::io;

#[allow(missing_copy_implementations)]
pub struct DebugStructGenerator;

impl super::Generator for DebugStructGenerator {
    fn write(&self, registry: &Registry, dest: &mut dyn io::Write) -> io::Result<()> {
        super::common::write_header(dest, true)?;
        super::common::write_type_aliases(registry, dest)?;
        super::common::write_enums(registry, dest)?;
        super::common::write_fnptr_struct_def(dest, false)?;
        super::common::write_panicking_fns(registry, dest)?;
        super::common::write_struct(registry, dest, false)?;
        write_impl(registry, dest)?;
        Ok(())
    }
}

/// Creates the `impl` of the structure created by `write_struct`.
fn write_impl(registry: &Registry, dest: &mut dyn io::Write) -> io::Result<()> {
    writeln!(
        dest,
                  "impl {api} {{
            /// Load each OpenGL symbol using a custom load function. This allows for the
            /// use of functions like `glfwGetProcAddress` or `SDL_GL_GetProcAddress`.
            ///
            /// ~~~ignore
            /// let gl = Gl::load_with(|s| glfw.get_proc_address(s));
            /// ~~~
            #[allow(dead_code, unused_variables)]
            pub fn load_with<F>(mut loadfn: F) -> {api} where F: FnMut(&'static str) -> *const __gl_imports::raw::c_void {{
                #[inline(never)]
                fn do_metaloadfn(loadfn: &mut dyn FnMut(&'static str) -> *const __gl_imports::raw::c_void,
                                 symbol: &'static str,
                                 symbols: &[&'static str])
                                 -> *const __gl_imports::raw::c_void {{
                    let mut ptr = loadfn(symbol);
                    if ptr.is_null() {{
                        for &sym in symbols {{
                            ptr = loadfn(sym);
                            if !ptr.is_null() {{ break; }}
                        }}
                    }}
                    ptr
                }}
                let mut metaloadfn = |symbol: &'static str, symbols: &[&'static str]| {{
                    do_metaloadfn(&mut loadfn, symbol, symbols)
                }};
                {api} {{",
        api = super::gen_struct_name(registry.api)
    )?;

    for cmd in &registry.cmds {
        writeln!(
            dest,
            "{name}: FnPtr::new(metaloadfn(\"{symbol}\", &[{fallbacks}])),",
            name = cmd.proto.ident,
            symbol = super::gen_symbol_name(registry.api, &cmd.proto.ident),
            fallbacks = match registry.aliases.get(&cmd.proto.ident) {
                Some(fbs) => fbs
                    .iter()
                    .map(|name| format!("\"{}\"", super::gen_symbol_name(registry.api, &name)))
                    .collect::<Vec<_>>()
                    .join(", "),
                None => format!(""),
            },
        )?
    }
    writeln!(dest, "_priv: ()")?;

    writeln!(
        dest,
        "}}
        }}"
    )?;

    for cmd in &registry.cmds {
        let idents = super::gen_parameters(cmd, true, false);
        let typed_params = super::gen_parameters(cmd, false, true);
        let println = format!(
            "println!(\"[OpenGL] {}({})\" {});",
            cmd.proto.ident,
            (0..idents.len())
                .map(|_| "{:?}".to_string())
                .collect::<Vec<_>>()
                .join(", "),
            idents
                .iter()
                .zip(typed_params.iter())
                .map(|(name, ty)| if ty.contains("GLDEBUGPROC") {
                    ", \"<callback>\"".to_string()
                } else {
                    format!(", {}", name)
                })
                .collect::<Vec<_>>()
                .concat()
        );
        let no_return_value = cmd.proto.ty.clone() == "()";
        let print_err = if cmd.proto.ident != "GetError"
            && registry
                .cmds
                .iter()
                .any(|cmd| cmd.proto.ident == "GetError")
        {
            ";
                match __gl_imports::mem::transmute::<_, extern \"system\" fn() -> u32>(self.GetError.f)() {
                    0 => (),
                    r => println!(\"[OpenGL] ^ GL error triggered: {}\", r)
                }".to_string()
        } else {
            String::new()
        };

        writeln!(
            dest,
            "
            #[allow(non_snake_case, unused_variables, dead_code)]
            #[inline] pub unsafe fn {name}(&self, {params}) {return_suffix} {{
                {println}
                {let_r}__gl_imports::mem::transmute::<_, extern \"system\" fn({typed_params}) {return_suffix}>\
                (self.{name}.f)({idents}){print_err}{return_r}
            }}",
            name = cmd.proto.ident,
            params = super::gen_parameters(cmd, true, true).join(", "),
            typed_params = typed_params.join(", "),
            return_suffix = if no_return_value {
                String::new()
            } else {
                format!("-> {}", cmd.proto.ty)
            },
            idents = idents.join(", "),
            println = println,
            print_err = print_err,
            let_r = if !no_return_value && !print_err.is_empty() {
                "let r = ".to_string()
            } else {
                String::new()
            },
            return_r = if !no_return_value && !print_err.is_empty() {
                "
                r".to_string()
            } else {
                String::new()
            },
        )?
    }

    writeln!(
        dest,
        "}}

        unsafe impl __gl_imports::Send for {api} {{}}",
        api = super::gen_struct_name(registry.api)
    )
}
