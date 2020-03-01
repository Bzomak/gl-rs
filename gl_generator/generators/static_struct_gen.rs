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

use registry::Registry;
use std::io;

#[allow(missing_copy_implementations)]
pub struct StaticStructGenerator;

impl super::Generator for StaticStructGenerator {
    fn write<W>(&self, registry: &Registry, dest: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        super::common::write_header(dest, false)?;
        super::common::write_type_aliases(registry, dest)?;
        super::common::write_enums(registry, dest)?;
        write_struct(registry, dest)?;
        write_impl(registry, dest)?;
        write_fns(registry, dest)?;
        Ok(())
    }
}

/// Creates a stub structure.
///
/// The name of the struct corresponds to the namespace.
fn write_struct<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        "
        #[allow(non_camel_case_types, non_snake_case, dead_code)]
        #[derive(Copy, Clone)]
        pub struct {api};",
        api = super::gen_struct_name(registry.api),
    )
}

/// Creates the `impl` of the structure created by `write_struct`.
fn write_impl<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(dest,
        "impl {api} {{
            /// Stub function.
            #[allow(dead_code)]
            pub fn load_with<F>(mut _loadfn: F) -> {api} where F: FnMut(&'static str) -> *const __gl_imports::raw::c_void {{
                {api}
            }}",
        api = super::gen_struct_name(registry.api),
    )?;

    for cmd in &registry.cmds {
        writeln!(
            dest,
            "#[allow(non_snake_case)]
            // #[allow(unused_variables)]
            #[allow(dead_code)]
            #[inline]
            pub unsafe fn {name}(&self, {typed_params}) -> {return_suffix} {{
                {name}({idents})
            }}",
            name = cmd.proto.ident,
            typed_params = super::gen_parameters(cmd, true, true).join(", "),
            return_suffix = cmd.proto.ty,
            idents = super::gen_parameters(cmd, true, false).join(", "),
        )?;
    }

    writeln!(dest, "}}")
}

/// io::Writes all functions corresponding to the GL bindings.
///
/// These are foreign functions, they don't have any content.
fn write_fns<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        "
        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        #[allow(dead_code)]
        extern \"system\" {{"
    )?;

    for cmd in &registry.cmds {
        writeln!(
            dest,
            "#[link_name=\"{symbol}\"] fn {name}({params}) -> {return_suffix};",
            symbol = super::gen_symbol_name(registry.api, &cmd.proto.ident),
            name = cmd.proto.ident,
            params = super::gen_parameters(cmd, true, true).join(", "),
            return_suffix = cmd.proto.ty,
        )?;
    }

    writeln!(dest, "}}")
}
