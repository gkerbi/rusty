// Copyright (c) 2020 Ghaith Hachem and Mathias Rieder
//! A Structured Text LLVM Frontent
//!
//! RuSTy is an [`ST`] Compiler using LLVM
//!
//! # Features
//! ## Standard language support
//! Most of the [`IEC61131-3`] standard for ST and general programing is supported.
//! ## Native compilation
//! A (currently) single ST files into object code using LLVM.
//! A compiled object can be linked statically or dynamically linked
//!     with other programs using standard compiler linkers (ld, clang, gcc)
//! ## IR Output
//! An [`IR`] file can be generated from any given ST file in order to examin the generated LLVM IR code.
//! For a usage guide refer to the [User Documentation](../../)
//!
//! [`ST`]: https://en.wikipedia.org/wiki/Structured_text
//! [`IEC61131-3`]: https://en.wikipedia.org/wiki/IEC_61131-3
//! [`IR`]: https://llvm.org/docs/LangRef.html
use glob::glob;
use rusty::{
    cli::{parse_parameters, CompileParameters, ParameterError},
    compile_error::CompileError,
    compile_to_bitcode, compile_to_ir, compile_to_shared_object, compile_to_static_obj,
};
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let compile_parameters: Result<CompileParameters, ParameterError> = parse_parameters(args);
    match compile_parameters {
        Ok(cp) => main_compile(cp),
        Err(err) => err.exit(), // prints the nice message to std-out
    }
}

fn read_contents(input: &str) -> Result<String, String> {
    let paths =
        glob(input).map_err(|e| format!("Failed to read glob pattern: {}, ({})", input, e))?;

    let contents: Result<Vec<String>, String>  = paths
        .map(read_content)
        .map(|p| p.map(|(_, content)| content))
        .collect();

    Ok(contents?.join("\n"))
}

fn read_content(path_buf: Result<std::path::PathBuf, glob::GlobError>) -> Result<(String, String), String> {
    path_buf
        .map_err(|e| format!("Invalid Path: {}", e))
        .map(|p| p.to_string_lossy().to_string())
        .and_then(|p| {
            fs::read_to_string(p.as_str())
                .map(|content| (p.to_string(), content))
                .map_err(|e| format!("Cannot read file {}: {}", p, e))
        })
}

fn main_compile(parameters: CompileParameters) {
    let file_path = parameters.input.as_str();

    let contents = read_contents(file_path).unwrap();

    if parameters.output_bit_code {
        compile_to_bitcode(file_path, contents, parameters.output.as_str()).unwrap();
    } else if parameters.output_ir {
        generate_ir(file_path, contents, parameters.output.as_str()).unwrap();
    } else if parameters.output_pic_obj {
        compile_to_shared_object(
            file_path,
            contents,
            parameters.output.as_str(),
            parameters.target,
        )
        .unwrap();
    } else if parameters.output_shared_obj {
        compile_to_shared_object(
            file_path,
            contents,
            parameters.output.as_str(),
            parameters.target,
        )
        .unwrap()
    } else if parameters.output_obj_code {
        compile_to_static_obj(
            file_path,
            contents,
            parameters.output.as_str(),
            parameters.target,
        )
        .unwrap();
    } else {
        //none is set, so we use default
        panic!("no output format defined");
    }
}
fn generate_ir(file_path: &str, content: String, output: &str) -> Result<(), CompileError> {
    let ir = compile_to_ir(file_path, content)?;
    fs::write(output, ir).unwrap();
    Ok(())
}
