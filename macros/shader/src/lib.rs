use std::path::PathBuf;

use proc_macro2::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use shaderc::EnvVersion;
use shaderc::ShaderKind;
use shaderc::SpirvVersion;
use syn::bracketed;
use syn::parenthesized;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
use syn::Error;
use syn::LitStr;
use syn::Result;

mod codegen;

#[proc_macro]
pub fn shader(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);

    shader_inner(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn shader_inner(input: MacroInput) -> Result<TokenStream> {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let relative_path_error_msg = "to your Cargo.toml".to_owned();

    let root_path = std::path::Path::new(&root);
    let MacroInput {
        macro_defines,
        include_directories,
        shader: (shader_kind, source_kind),
        spirv_version,
        vulkan_version,
    } = input;

    let code = match source_kind {
        SourceKind::Src(source) => {
            let (artifact, includes) = codegen::compile(
                macro_defines,
                include_directories,
                spirv_version,
                vulkan_version,
                None,
                root_path,
                &source.value(),
                shader_kind.unwrap(),
            )
            .map_err(|err| Error::new_spanned(&source, err))?;

            let words = artifact.as_binary();

            codegen::reflect(words, includes)?
        }
        SourceKind::Path(path) => {
            let full_path = root_path.join(path.value());

            if !full_path.is_file() {
                bail!(
                    path,
                    "file `{full_path:?}` was not found, note that the path must be relative \
                        {relative_path_error_msg}",
                );
            }

            let source_code = std::fs::read_to_string(&full_path)
                .or_else(|err| bail!(path, "failed to read source `{full_path:?}`: {err}"))?;

            let (artifact, mut includes) = codegen::compile(
                macro_defines,
                include_directories,
                spirv_version,
                vulkan_version,
                Some(path.value()),
                root_path,
                &source_code,
                shader_kind.unwrap(),
            )
            .map_err(|err| Error::new_spanned(&path, err))?;

            let words = artifact.as_binary();

            includes.push(full_path.into_os_string().into_string().unwrap());

            codegen::reflect(words, includes)?
        }
    };

    let result = quote! {
        #code
    };

    Ok(result)
}

enum SourceKind {
    Src(LitStr),
    Path(LitStr),
}

struct MacroInput {
    macro_defines: Vec<(String, String)>,
    include_directories: Vec<PathBuf>,
    shader: (Option<ShaderKind>, SourceKind),
    spirv_version: Option<SpirvVersion>,
    vulkan_version: Option<EnvVersion>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());

        let mut include_directories = Vec::new();
        let mut macro_defines = Vec::new();
        let mut shader = (None, None);
        let mut vulkan_version = None;
        let mut spirv_version = None;

        fn parse_shader_fields(
            output: &mut (Option<ShaderKind>, Option<SourceKind>),
            name: &str,
            input: ParseStream<'_>,
        ) -> Result<()> {
            match name {
                "ty" => {
                    let lit = input.parse::<LitStr>()?;
                    if output.0.is_some() {
                        bail!(lit, "field `ty` is already defined");
                    }

                    output.0 = Some(match lit.value().as_str() {
                        "vertex" => ShaderKind::Vertex,
                        "tess_ctrl" => ShaderKind::TessControl,
                        "tess_eval" => ShaderKind::TessEvaluation,
                        "geometry" => ShaderKind::Geometry,
                        "task" => ShaderKind::Task,
                        "mesh" => ShaderKind::Mesh,
                        "fragment" => ShaderKind::Fragment,
                        "compute" => ShaderKind::Compute,
                        "raygen" => ShaderKind::RayGeneration,
                        "anyhit" => ShaderKind::AnyHit,
                        "closesthit" => ShaderKind::ClosestHit,
                        "miss" => ShaderKind::Miss,
                        "intersection" => ShaderKind::Intersection,
                        "callable" => ShaderKind::Callable,
                        ty => bail!(
                            lit,
                            "expected `vertex`, `tess_ctrl`, `tess_eval`, `geometry`, `task`, \
                            `mesh`, `fragment` `compute`, `raygen`, `anyhit`, `closesthit`, \
                            `miss`, `intersection` or `callable`, found `{ty}`",
                        ),
                    });
                }
                "path" => {
                    let lit = input.parse::<LitStr>()?;
                    if output.1.is_some() {
                        bail!(lit, "field `path` is already defined");
                    }

                    output.1 = Some(SourceKind::Path(lit));
                }
                _ => unreachable!(),
            }

            Ok(())
        }

        while !input.is_empty() {
            let field_ident = input.parse::<Ident>()?;
            input.parse::<Token![:]>()?;
            let field = field_ident.to_string();

            match field.as_str() {
                "ty" | "path" => {
                    parse_shader_fields(&mut shader, &field, input)?;
                }
                "define" => {
                    let array_input;
                    bracketed!(array_input in input);

                    while !array_input.is_empty() {
                        let tuple_input;
                        parenthesized!(tuple_input in array_input);

                        let name = tuple_input.parse::<LitStr>()?;
                        tuple_input.parse::<Token![,]>()?;
                        let value = tuple_input.parse::<LitStr>()?;
                        macro_defines.push((name.value(), value.value()));

                        if !array_input.is_empty() {
                            array_input.parse::<Token![,]>()?;
                        }
                    }
                }
                "include" => {
                    let in_brackets;
                    bracketed!(in_brackets in input);

                    while !in_brackets.is_empty() {
                        let path = in_brackets.parse::<LitStr>()?;

                        include_directories.push([&root, &path.value()].into_iter().collect());

                        if !in_brackets.is_empty() {
                            in_brackets.parse::<Token![,]>()?;
                        }
                    }
                }
                "vulkan_version" => {
                    let lit = input.parse::<LitStr>()?;
                    if vulkan_version.is_some() {
                        bail!(lit, "field `vulkan_version` is already defined");
                    }

                    vulkan_version = Some(match lit.value().as_str() {
                        "1.0" => EnvVersion::Vulkan1_0,
                        "1.1" => EnvVersion::Vulkan1_1,
                        "1.2" => EnvVersion::Vulkan1_2,
                        "1.3" => EnvVersion::Vulkan1_3,
                        ver => bail!(lit, "expected `1.0`, `1.1`, `1.2` or `1.3`, found `{ver}`"),
                    });
                }
                "spirv_version" => {
                    let lit = input.parse::<LitStr>()?;
                    if spirv_version.is_some() {
                        bail!(lit, "field `spirv_version` is already defined");
                    }

                    spirv_version = Some(match lit.value().as_str() {
                        "1.0" => SpirvVersion::V1_0,
                        "1.1" => SpirvVersion::V1_1,
                        "1.2" => SpirvVersion::V1_2,
                        "1.3" => SpirvVersion::V1_3,
                        "1.4" => SpirvVersion::V1_4,
                        "1.5" => SpirvVersion::V1_5,
                        "1.6" => SpirvVersion::V1_6,
                        ver => bail!(
                            lit,
                            "expected `1.0`, `1.1`, `1.2`, `1.3`, `1.4`, `1.5` or `1.6`, found \
                            `{ver}`",
                        ),
                    });
                }
                field => bail!(
                    field_ident,
                    "expected `ty`, `path`,`define`, `include`, \
                    `vulkan_version`, `spirv_version` or \
                    `generate_structs` as a field, found `{field}`",
                ),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        if shader.0.is_none() || shader.1.is_none() {
            bail!(
                r#"please specify at least one shader e.g. `ty: "vertex", path: "path to shader"`"#
            );
        }

        match shader {
            // if source is bytes, the shader type should not be declared
            (None, _) => {
                bail!(r#"please specify the type of the shader e.g. `ty: "vertex"`"#);
            }
            (_, None) => {
                bail!(r#"please specify the source of the shader e.g. `src: "<GLSL code>"`"#);
            }
            _ => {}
        }

        Ok(MacroInput {
            include_directories,
            macro_defines,
            shader: (shader.0, shader.1.unwrap()),
            vulkan_version,
            spirv_version,
        })
    }
}

macro_rules! bail {
    ($msg:literal $(,)?) => {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!($msg),
        ))
    };
    ($span:expr, $msg:literal $(,)?) => {
        return Err(syn::Error::new_spanned(&$span, format!($msg)))
    };
}
use bail;
use syn::Token;
