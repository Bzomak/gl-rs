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
pub struct StaticGenerator;

impl super::Generator for StaticGenerator {
    fn write(&self, registry: &Registry, dest: &mut dyn io::Write) -> io::Result<()> {
        super::common::write_header(dest, false)?;
        super::common::write_type_aliases(registry, dest)?;
        super::common::write_enums(registry, dest)?;
        write_fns(registry, dest)?;
        Ok(())
    }
}

/// io::Writes all functions corresponding to the GL bindings.
///
/// These are foreign functions, they don't have any content.
fn write_fns(registry: &Registry, dest: &mut dyn io::Write) -> io::Result<()> {
    writeln!(
        dest,
        "
        #[allow(non_snake_case, unused_variables, dead_code)]
        extern \"system\" {{"
    )?;

    for cmd in &registry.cmds {
        writeln!(
            dest,
            "#[link_name=\"{symbol}\"]
            pub fn {name}({params}){return_suffix};",
            symbol = super::gen_symbol_name(registry.api, &cmd.proto.ident),
            name = cmd.proto.ident,
            params = super::gen_parameters(cmd, true, true).join(", "),
            return_suffix = if cmd.proto.ty.clone() == "()" {
                String::new()
            } else {
                format!("-> {}", cmd.proto.ty)
            },
        )?;
    }

    writeln!(dest, "}}")
}
