mod banner;
mod cli;
mod configuration;
mod floodgate;
mod forwarding;
mod handlers;
mod kick_messages;
mod registry_provider;
mod server;
mod server_brand;
mod server_state;

use crate::cli::Cli;
use clap::Parser;
use std::ffi::{CStr, c_char, c_int};
use std::slice;
use tokio_util::sync::CancellationToken;

#[unsafe(no_mangle)]
pub extern "C" fn get_cancellation_token() -> *mut CancellationToken {
    let token = CancellationToken::new();
    Box::into_raw(Box::new(token))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cleanup_token(token_ptr: *mut CancellationToken) {
    if token_ptr.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(token_ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_app(
    token_ptr: *mut CancellationToken,
    argc: c_int,
    argv: *const *const c_char,
) {
    if argv.is_null() {
        eprintln!("Error: argv is null");
        return;
    }

    let mut rust_args: Vec<String> = Vec::new();
    let c_args_slice = unsafe { slice::from_raw_parts(argv, argc as usize) };

    for &ptr in c_args_slice {
        if ptr.is_null() {
            continue;
        }
        let c_str = unsafe { CStr::from_ptr(ptr) };
        if let Ok(str_slice) = c_str.to_str() {
            rust_args.push(str_slice.to_owned());
        } else {
            eprintln!("Error: Argument not valid UTF-8");
            return;
        }
    }

    match Cli::try_parse_from(&rust_args) {
        Ok(cli) => {
            let cancellation_token = unsafe { &*token_ptr };
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let _ = rt.block_on(server::start_server::start_server(
                &cli,
                Some(cancellation_token),
            ));
        }
        Err(e) => {
            e.print().expect("Failed to print error");
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_app(token_ptr: *mut CancellationToken) {
    if token_ptr.is_null() {
        return;
    }
    let wrapper = unsafe { &*token_ptr };
    wrapper.cancel();
}
