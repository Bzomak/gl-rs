use registry::Registry;
use std::io;

/// Creates a `__gl_imports` module which contains all the external symbols that we need for the
///  bindings.
pub fn write_header<W>(dest: &mut W, send: bool) -> io::Result<()>
where
    W: io::Write,
{
    if send {
        writeln!(
            dest,
            r#"
        mod __gl_imports {{
            pub use std::marker::Send;
            pub use std::mem;
            pub use std::os::raw;
        }}
    "#
        )
    } else {
        writeln!(
            dest,
            r#"
        mod __gl_imports {{
            pub use std::mem;
            pub use std::os::raw;
        }}
    "#
        )
    }
}

/// Creates a `types` module which contains all the type aliases.
///
/// See also `generators::gen_types`.
pub fn write_type_aliases<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        r#"
        pub mod types {{
            #![allow(non_camel_case_types, non_snake_case, dead_code, missing_copy_implementations)]
    "#
    )?;

    super::gen_types(registry.api, dest)?;

    writeln!(dest, "}}")
}

/// Creates all the `<enum>` elements at the root of the bindings.
pub fn write_enums<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    for enm in &registry.enums {
        super::gen_enum_item(enm, "types::", dest)?;
    }

    Ok(())
}

/// Creates a `panicking` module which contains one function per GL command.
///
/// These functions are the mocks that are called if the real function could not be loaded.
pub fn write_panicking_fns<W>(registry: &Registry, dest: &mut W) -> io::Result<()>
where
    W: io::Write,
{
    writeln!(
        dest,
        "#[inline(never)]
        fn missing_fn_panic() -> ! {{
            panic!(\"{api} function was not loaded\")
        }}",
        api = registry.api
    )
}
