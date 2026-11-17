mod banner;
mod cli;
mod configuration;
mod floodgate;
mod forwarding;
mod handlers;
mod kick_messages;
mod plugins;
mod registry_provider;
mod server;
mod server_brand;
mod server_state;

use crate::cli::Cli;
use clap::Parser;
use std::ffi::{CStr, c_char, c_int};
use std::process::ExitCode;
use std::slice;
use tokio_util::sync::CancellationToken;

/// Creates a token used for telling the app to stop listening for new connections.
/// This token can be used only once.
///
/// # Returns
/// Raw pointer to a `CancellationToken` used for server shutdown.
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
/// * `token_ptr` - The handle returned by [`get_cancellation_token`].
/// * `argc` - The number of arguments in the `argv` array (0 to 1024).
/// * `argv` - A pointer to an array of C-style strings (null-terminated).
///
/// # Returns
/// 0 on successful shutdown, 1 on runtime/startup failure, or 2 on CLI parsing failure.
///
/// # Safety
/// This function is unsafe because it dereferences raw pointers. The caller must
/// ensure that `token_ptr` is a live token returned by [`get_cancellation_token`],
/// and that `argv` is valid and `argc` correctly represents the number of elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_app(
    token_ptr: *mut CancellationToken,
    argc: c_int,
    argv: *const *const c_char,
) -> c_int {
    if argv.is_null() || token_ptr.is_null() || !(0..=1024).contains(&argc) {
        eprintln!("Error: invalid start_app arguments");
        return 1;
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
            return 1;
        }
    }

    match Cli::try_parse_from(&rust_args) {
        Ok(cli) => {
            let cancellation_token = unsafe { &*token_ptr };

            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(err) => {
                    eprintln!("Error creating Tokio runtime: {err}");
                    return 1;
                }
            };

            match rt.block_on(server::start_server::start_server(
                &cli,
                Some(cancellation_token),
            )) {
                ExitCode::SUCCESS => 0,
                _ => 1,
            }
        }
        Err(e) => {
            e.print().expect("Failed to print error");
            2
        }
    }
}

/// Requests graceful shutdown of the PicoLimbo server associated with the token.
///
/// # Arguments
/// * `token_ptr` - The handle returned by [`get_cancellation_token`].
///
/// # Safety
/// This function is unsafe because it reconstructs a `Box` from a raw pointer.
/// The caller must ensure that `token_ptr` is a live pointer returned by [`get_cancellation_token`]
/// and must call [`cleanup_token`] exactly once after the server has stopped.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_app(token_ptr: *mut CancellationToken) {
    if token_ptr.is_null() {
        return;
    }
    let wrapper = unsafe { &*token_ptr };
    wrapper.cancel();
}
