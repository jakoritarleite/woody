use std::cell::RefCell;
use std::path::Path;
use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::format_ident;
use quote::quote;
use shaderc::CompilationArtifact;
use shaderc::CompileOptions;
use shaderc::Compiler;
use shaderc::EnvVersion;
use shaderc::IncludeType;
use shaderc::ResolvedInclude;
use shaderc::ShaderKind;
use shaderc::SpirvVersion;
use shaderc::TargetEnv;
use syn::Error;
use syn::LitStr;

//use crate::structs;
//use crate::structs::TypeRegistry;

pub struct Shader {
    pub source: LitStr,
    pub name: String,
    // pub spirv: Spirv,
}

pub(super) fn compile(
    // input: &MacroInput,
    macro_defines: Vec<(String, String)>,
    include_directories: Vec<PathBuf>,
    spirv_version: Option<SpirvVersion>,
    vulkan_version: Option<EnvVersion>,
    path: Option<String>,
    base_path: &Path,
    code: &str,
    shader_kind: ShaderKind,
) -> Result<(CompilationArtifact, Vec<String>), String> {
    let includes = RefCell::new(Vec::new());
    let compiler = Compiler::new().ok_or("failed to create GLSL compiler")?;
    let mut compile_options =
        CompileOptions::new().ok_or("failed to initialize compile options")?;

    compile_options.set_target_env(
        TargetEnv::Vulkan,
        vulkan_version.unwrap_or(EnvVersion::Vulkan1_0) as u32,
    );

    if let Some(spirv_version) = spirv_version {
        compile_options.set_target_spirv(spirv_version);
    }

    let root_source_path = path.as_deref().unwrap_or(
        // An arbitrary placeholder file name for embedded shaders.
        "shader.glsl",
    );

    // Specify the file resolution callback for the `#include` directive.
    compile_options.set_include_callback(
        |requested_source_path, directive_type, contained_within_path, recursion_depth| {
            include_callback(
                requested_source_path,
                directive_type,
                contained_within_path,
                recursion_depth,
                &include_directories,
                path.is_some(),
                base_path,
                &mut includes.borrow_mut(),
            )
        },
    );

    for (macro_name, macro_value) in &macro_defines {
        compile_options.add_macro_definition(macro_name, Some(macro_value));
    }

    #[cfg(feature = "shaderc-debug")]
    compile_options.set_generate_debug_info();

    let content = compiler
        .compile_into_spirv(
            code,
            shader_kind,
            root_source_path,
            "main",
            Some(&compile_options),
        )
        .map_err(|e| e.to_string().replace("(s): ", "(s):\n"))?;

    drop(compile_options);

    Ok((content, includes.into_inner()))
}

pub(super) fn reflect(words: &[u32], input_paths: Vec<String>) -> Result<TokenStream, Error> {
    let include_bytes = input_paths.into_iter().map(|s| {
        quote! {
            // Using `include_bytes` here ensures that changing the shader will force recompilation.
            // The bytes themselves can be optimized out by the compiler as they are unused.
            ::std::include_bytes!( #s )
        }
    });

    let load_name = format_ident!("load");

    let shader_code = quote! {
        /// Loads the shader as a `ShaderModule`.
        #[allow(unsafe_code)]
        #[inline]
        pub fn #load_name(
            device: ::std::sync::Arc<::woody::graphics2::vulkan::device::Device>,
        ) -> ::std::result::Result<
            ::woody::graphics2::vulkan::shader::ShaderModule,
            ::woody::graphics2::vulkan::Error,
        > {
            let _bytes = ( #( #include_bytes ),* );

            static WORDS: &[u32] = &[ #( #words ),* ];

            unsafe {
                ::woody::graphics2::vulkan::shader::ShaderModule::new(device, ::ash::vk::ShaderModuleCreateInfo::builder().code(&WORDS).build())
            }
        }
    };

    Ok(shader_code)
}

#[allow(clippy::too_many_arguments)]
fn include_callback(
    requested_source_path_raw: &str,
    directive_type: IncludeType,
    contained_within_path_raw: &str,
    recursion_depth: usize,
    include_directories: &[PathBuf],
    root_source_has_path: bool,
    base_path: &Path,
    includes: &mut Vec<String>,
) -> Result<ResolvedInclude, String> {
    let file_to_include = match directive_type {
        IncludeType::Relative => {
            let requested_source_path = Path::new(requested_source_path_raw);
            // If the shader source is embedded within the macro, abort unless we get an absolute
            // path.
            if !root_source_has_path && recursion_depth == 1 && !requested_source_path.is_absolute()
            {
                let requested_source_name = requested_source_path
                    .file_name()
                    .expect("failed to get the name of the requested source file")
                    .to_string_lossy();
                let requested_source_directory = requested_source_path
                    .parent()
                    .expect("failed to get the directory of the requested source file")
                    .to_string_lossy();

                return Err(format!(
                    "usage of relative paths in imports in embedded GLSL is not allowed, try \
                    using `#include <{}>` and adding the directory `{}` to the `include` array in \
                    your `shader!` macro call instead",
                    requested_source_name, requested_source_directory,
                ));
            }

            let mut resolved_path = if recursion_depth == 1 {
                Path::new(contained_within_path_raw)
                    .parent()
                    .map(|parent| base_path.join(parent))
            } else {
                Path::new(contained_within_path_raw)
                    .parent()
                    .map(|parent| parent.to_owned())
            }
            .unwrap_or_else(|| {
                panic!(
                    "the file `{}` does not reside in a directory, this is an implementation \
                    error",
                    contained_within_path_raw,
                )
            });
            resolved_path.push(requested_source_path);

            if !resolved_path.is_file() {
                return Err(format!(
                    "invalid inclusion path `{}`, the path does not point to a file",
                    requested_source_path_raw,
                ));
            }

            resolved_path
        }
        IncludeType::Standard => {
            let requested_source_path = Path::new(requested_source_path_raw);

            if requested_source_path.is_absolute() {
                // This message is printed either when using a missing file with an absolute path
                // in the relative include directive or when using absolute paths in a standard
                // include directive.
                return Err(format!(
                    "no such file found as specified by the absolute path; keep in mind that \
                    absolute paths cannot be used with inclusion from standard directories \
                    (`#include <...>`), try using `#include \"...\"` instead; requested path: {}",
                    requested_source_path_raw,
                ));
            }

            let found_requested_source_path = include_directories
                .iter()
                .map(|include_directory| include_directory.join(requested_source_path))
                .find(|resolved_requested_source_path| resolved_requested_source_path.is_file());

            if let Some(found_requested_source_path) = found_requested_source_path {
                found_requested_source_path
            } else {
                return Err(format!(
                    "failed to include the file `{}` from any include directories",
                    requested_source_path_raw,
                ));
            }
        }
    };

    let content = std::fs::read_to_string(file_to_include.as_path()).map_err(|err| {
        format!(
            "failed to read the contents of file `{file_to_include:?}` to be included in the \
            shader source: {err}",
        )
    })?;
    let resolved_name = file_to_include
        .into_os_string()
        .into_string()
        .map_err(|_| {
            "failed to stringify the file to be included; make sure the path consists of valid \
            unicode characters"
        })?;

    includes.push(resolved_name.clone());

    Ok(ResolvedInclude {
        resolved_name,
        content,
    })
}
