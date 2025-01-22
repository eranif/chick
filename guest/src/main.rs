#![no_std]
#![no_main]
extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use hyperlight_common::flatbuffer_wrappers::function_call::FunctionCall;
use hyperlight_common::flatbuffer_wrappers::function_types::{
    ParameterType, ParameterValue, ReturnType,
};
use hyperlight_common::flatbuffer_wrappers::guest_error::ErrorCode;
use hyperlight_common::flatbuffer_wrappers::util::get_flatbuffer_result_from_string;

use hyperlight_guest::error::{HyperlightGuestError, Result};
use hyperlight_guest::guest_function_definition::GuestFunctionDefinition;
use hyperlight_guest::guest_function_register::register_function;
use serde::Serialize;

struct LineReader<'a> {
    data: &'a [u8],
}

impl<'a> LineReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Iterator for LineReader<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }

        let newline_pos = self.data.iter().position(|&b| b == b'\n');

        let (line, rest) = match newline_pos {
            Some(pos) => {
                let line = &self.data[..pos];
                let rest = &self.data[pos + 1..];
                (line, rest)
            }
            None => {
                let line = self.data;
                let rest: &[u8] = &[];
                (line, rest)
            }
        };

        self.data = rest;
        core::str::from_utf8(line)
            .ok()
            .map(|s| s.trim_end_matches('\r'))
    }
}

#[derive(Default, Serialize)]
struct DpkgRecord {
    pub package: String,
    pub status: String,
    pub version: String,
}

fn read_package<'a>(reader: &mut LineReader) -> Result<Option<DpkgRecord>> {
    let mut package = DpkgRecord::default();

    let mut line = reader.next();
    if line.is_none() {
        return Ok(None);
    }

    loop {
        if let Some(line_str) = line {
            if line_str.is_empty() {
                break;
            }
            let (key, value) = line_str.split_once(':').unwrap_or(("", ""));
            match key {
                "Package" => package.package = value.trim().to_string(),
                "Status" => package.status = value.trim().to_string(),
                "Version" => package.version = value.trim().to_string(),
                _ => {}
            }
        }

        line = reader.next();
        if line.is_none() {
            break;
        }
    }

    Ok(Some(package))
}

fn inspect(function_call: &FunctionCall) -> Result<Vec<u8>> {
    if let ParameterValue::VecBytes(value) = function_call.parameters.clone().unwrap()[0].clone() {
        let mut packages: Vec<DpkgRecord> = Vec::new();

        let mut reader = LineReader::new(&value.as_slice());
        while let Some(package) = read_package(&mut reader)? {
            packages.push(package);
        }
        // while let Some(package) = read_package(&mut reader)? {
        //     packages.push(package);
        // }

        let res = serde_json::to_string(&packages).unwrap();
        Ok(get_flatbuffer_result_from_string(&res))
    } else {
        Err(HyperlightGuestError::new(
            ErrorCode::GuestFunctionParameterTypeMismatch,
            "Invalid parameters passed to inspect".to_string(),
        ))
    }
}

#[no_mangle]
pub extern "C" fn hyperlight_main() {
    let inspect_def = GuestFunctionDefinition::new(
        "Inspect".to_string(),
        Vec::from(&[ParameterType::VecBytes]),
        ReturnType::String,
        inspect as i64,
    );
    register_function(inspect_def);
}

#[no_mangle]
pub fn guest_dispatch_function(function_call: FunctionCall) -> Result<Vec<u8>> {
    let function_name = function_call.function_name.clone();
    return Err(HyperlightGuestError::new(
        ErrorCode::GuestFunctionNotFound,
        function_name,
    ));
}
