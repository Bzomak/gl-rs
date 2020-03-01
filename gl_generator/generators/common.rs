use registry::Registry;
use std::io;

/// Creates a `__gl_imports` module which contains all the external symbols that we need for the
///  bindings.
///
/// send == true: DebugStructGenerator, StructGenerator
/// send == false: GlobalGenerator, StaticGenerator, StaticStructGenerator
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

/// Creates a `FnPtr` structure which contains the store for a single binding.
///
/// global == true: GlobalGenerator
/// global == false: DebugStructGenerator, StructGenerator
pub fn write_fnptr_struct_def<W>(dest: &mut W, global: bool) -> io::Result<()>
where
    W: io::Write,
{
    if global {
        writeln!(dest,
            "
       #[allow(missing_copy_implementations)]
       pub struct FnPtr {{
           /// The function pointer that will be used when calling the function.
           f: *const __gl_imports::raw::c_void,
           /// True if the pointer points to a real function, false if points to a `panic!` fn.
           is_loaded: bool,
       }}

       impl FnPtr {{
           /// Creates a `FnPtr` from a load attempt.
           pub fn new(ptr: *const __gl_imports::raw::c_void) -> FnPtr {{
               if ptr.is_null() {{
                   FnPtr {{ f: missing_fn_panic as *const __gl_imports::raw::c_void, is_loaded: false }}
               }} else {{
                   FnPtr {{ f: ptr, is_loaded: true }}
               }}
           }}
       }}
   ")
    } else {
        writeln!(
            dest,
            "
        #[allow(dead_code, missing_copy_implementations)]
        #[derive(Clone)]
        pub struct FnPtr {{
            /// The function pointer that will be used when calling the function.
            f: *const __gl_imports::raw::c_void,
            /// True if the pointer points to a real function, false if points to a `panic!` fn.
            is_loaded: bool,
        }}

        impl FnPtr {{
            /// Creates a `FnPtr` from a load attempt.
            fn new(ptr: *const __gl_imports::raw::c_void) -> FnPtr {{
                if ptr.is_null() {{
                    FnPtr {{
                        f: missing_fn_panic as *const __gl_imports::raw::c_void,
                        is_loaded: false
                    }}
                }} else {{
                    FnPtr {{ f: ptr, is_loaded: true }}
                }}
            }}

            /// Returns `true` if the function has been successfully loaded.
            ///
            /// If it returns `false`, calling the corresponding function will fail.
            #[inline]
            #[allow(dead_code)]
            pub fn is_loaded(&self) -> bool {{
                self.is_loaded
            }}
        }}
    "
        )
    }
}

/// Creates a `panicking` module which contains one function per GL command.
///
/// These functions are the mocks that are called if the real function could not be loaded.
///
/// Used by DebugStructGenerator, GlobalGenerator, StructGenerator
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

/// If stat == true creates a stub structure.
/// if stat == false creates a structure which stores all the `FnPtr` of the bindings.
///
/// The name of the struct corresponds to the namespace.
///
/// stat == true: StaticStructGenerator
/// stat == false: DebugStructGenerator, StructGenerator
pub fn write_struct<W>(registry: &Registry, dest: &mut W, stat: bool) -> io::Result<()>
where
    W: io::Write,
{
    if stat {
        writeln!(
            dest,
            "
            #[allow(non_camel_case_types, non_snake_case, dead_code)]
            #[derive(Copy, Clone)]
            pub struct {api};",
            api = super::gen_struct_name(registry.api),
        )
    } else {
        writeln!(
            dest,
            "
        #[allow(non_camel_case_types, non_snake_case, dead_code)]
        #[derive(Clone)]
        pub struct {api} {{",
            api = super::gen_struct_name(registry.api)
        )?;

        for cmd in &registry.cmds {
            if let Some(v) = registry.aliases.get(&cmd.proto.ident) {
                writeln!(dest, "/// Fallbacks: {}", v.join(", "))?;
            }
            writeln!(dest, "pub {name}: FnPtr,", name = cmd.proto.ident)?;
        }
        writeln!(dest, "_priv: ()")?;

        writeln!(dest, "}}")
    }
}
