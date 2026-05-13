mod cli;
mod configuration;
mod forwarding;
mod handlers;
mod kick_messages;
mod server;
mod server_brand;
mod server_state;

use crate::cli::Cli;
use clap::Parser;
use std::ffi::{CStr, c_char, c_int};
use std::slice;
use tokio_util::sync::CancellationToken;

/// Creates a token used for telling the app to stop listening for new connections.
/// This token can be used only once.
///
/// # Returns
/// Raw pointer to a `CancellationToken` used for server shutdown,
#[unsafe(no_mangle)]
pub extern "C" fn get_cancellation_token() -> *mut CancellationToken {
    let token = CancellationToken::new();
    Box::into_raw(Box::new(token))
}

/// Cleanup a reference of the token used for cancellation.
///
/// # Arguments
/// * `ptr` - The handle returned by [`get_cancellation_token`].
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cleanup_token(token_ptr: *mut CancellationToken) {
    if token_ptr.is_null() {
        return;
    }
    let _ = unsafe { Box::from_raw(token_ptr) };
}

/// Initializes and starts the PicoLimbo server.
///
/// # Arguments
/// * `ptr` - The handle returned by [`get_cancellation_token`].
/// * `argc` - The number of arguments in the `argv` array.
/// * `argv` - A pointer to an array of C-style strings (null-terminated).
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. The caller must
/// ensure that `argv` is a valid pointer and that `argc` correctly represents
/// the number of elements in the array.
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
            let token = unsafe { &*token_ptr };

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            let _ = rt.block_on(server::start_server::start_server(
                cli.config_path,
                cli.verbose,
                Some(token),
            ));
        }
        Err(e) => {
            e.print().expect("Failed to print error");
        }
    }
}

/// Shuts down the PicoLimbo server and releases the allocation of the token.
///
/// # Arguments
/// * `token_ptr` - The handle returned by [`get_cancellation_token`].
///
/// # Safety
/// This function is unsafe because it reconstructs a `Box` from a raw pointer.
/// The caller must ensure that `token_ptr` is a valid pointer previously returned by `start_app`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_app(token_ptr: *mut CancellationToken) {
    if token_ptr.is_null() {
        return;
    }
    let wrapper = unsafe { &*token_ptr };
    wrapper.cancel();
}
