//! Clears a system proxy that sing-box set but could not undo.
//!
//! sing-box's `set_system_proxy` restores the OS setting only on a clean
//! shutdown. On macOS and Linux the core is asked to exit with SIGTERM first
//! (see `process::terminate_gracefully`), which is enough. Windows has no
//! signal to send a windowless child, so there the proxy is reset from here
//! once the core is gone: after every exit, and at startup for a session that
//! crashed. Only a proxy pointing at our own listen port is touched, so one
//! the user set up for something else survives.
//!
//! WinINet rather than the `Internet Settings` registry values: WinINet keeps
//! its own per-connection blob, and writing the legacy values alone does not
//! reliably take effect. This is also how sing-box sets the proxy itself.
//!
//! The Windows half is built from the WinINet docs and type-checked against
//! the Windows target, but has not been run on a Windows host.

/// Resets the system proxy to direct if it still points at the local mixed
/// inbound on `port`. Best effort: a failure leaves the proxy as it was, and
/// there is nothing more useful to do with it than there was before.
#[cfg(windows)]
pub fn clear_if_ours(port: u16) {
    if let Some(server) = wininet::enabled_proxy_server() {
        if points_at_local_port(&server, port) {
            wininet::set_direct();
        }
    }
}

/// sing-box undoes the proxy itself here; see the module docs.
#[cfg(not(windows))]
pub fn clear_if_ours(_port: u16) {}

/// Whether a WinINet proxy server string names `127.0.0.1:port`. sing-box
/// writes a bare `host:port`, but a per-scheme list such as
/// `http=127.0.0.1:2080;https=127.0.0.1:2080` is valid too, and so is a
/// scheme prefix, so every entry is checked in every form.
#[cfg_attr(not(windows), allow(dead_code))]
fn points_at_local_port(server: &str, port: u16) -> bool {
    let target = format!("127.0.0.1:{port}");
    server.split(';').any(|entry| {
        let address = entry.split_once('=').map_or(entry, |(_, a)| a).trim();
        let address = address.split_once("://").map_or(address, |(_, a)| a);
        address.trim_end_matches('/') == target
    })
}

#[cfg(windows)]
mod wininet {
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    use windows_sys::Win32::Foundation::GlobalFree;
    use windows_sys::Win32::Networking::WinInet::{
        InternetQueryOptionW, InternetSetOptionW, INTERNET_OPTION_PER_CONNECTION_OPTION,
        INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED, INTERNET_PER_CONN_FLAGS,
        INTERNET_PER_CONN_OPTIONW, INTERNET_PER_CONN_OPTIONW_0, INTERNET_PER_CONN_OPTION_LISTW,
        INTERNET_PER_CONN_PROXY_SERVER, PROXY_TYPE_DIRECT, PROXY_TYPE_PROXY,
    };

    /// The LAN connection's proxy server, if a proxy is enabled on it. A
    /// null connection name is the LAN settings, which is what sing-box sets.
    pub fn enabled_proxy_server() -> Option<String> {
        let mut options = [
            option(INTERNET_PER_CONN_FLAGS, 0),
            option(INTERNET_PER_CONN_PROXY_SERVER, 0),
        ];
        let mut list = list(&mut options);
        let mut size = list.dwSize;
        // SAFETY: `list` points at `options`, both live for the call, and
        // `size` is the size of the list structure as the API expects.
        let ok = unsafe {
            InternetQueryOptionW(
                null(),
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                (&mut list as *mut INTERNET_PER_CONN_OPTION_LISTW).cast::<c_void>(),
                &mut size,
            )
        };
        if ok == 0 {
            return None;
        }
        // SAFETY: a successful query filled both unions with the member
        // matching its option: a DWORD for the flags, a string for the server.
        let (flags, server) = unsafe { (options[0].Value.dwValue, options[1].Value.pszValue) };
        if server.is_null() {
            return None;
        }
        // SAFETY: WinINet returns a NUL-terminated string that the caller
        // owns and must release with GlobalFree.
        let text = unsafe {
            let len = (0..).take_while(|&i| *server.add(i) != 0).count();
            let text = String::from_utf16_lossy(std::slice::from_raw_parts(server, len));
            GlobalFree(server.cast());
            text
        };
        (flags & PROXY_TYPE_PROXY != 0).then_some(text)
    }

    /// Switches the LAN connection to direct, then tells running WinINet
    /// clients to reread their settings, as sing-box does when it clears it.
    pub fn set_direct() {
        let mut options = [option(INTERNET_PER_CONN_FLAGS, PROXY_TYPE_DIRECT)];
        let list = list(&mut options);
        // SAFETY: as in `enabled_proxy_server`; the two notifications take no
        // buffer at all.
        unsafe {
            InternetSetOptionW(
                null(),
                INTERNET_OPTION_PER_CONNECTION_OPTION,
                (&list as *const INTERNET_PER_CONN_OPTION_LISTW).cast::<c_void>(),
                list.dwSize,
            );
            InternetSetOptionW(null(), INTERNET_OPTION_SETTINGS_CHANGED, null(), 0);
            InternetSetOptionW(null(), INTERNET_OPTION_REFRESH, null(), 0);
        }
    }

    fn option(which: u32, value: u32) -> INTERNET_PER_CONN_OPTIONW {
        INTERNET_PER_CONN_OPTIONW {
            dwOption: which,
            Value: INTERNET_PER_CONN_OPTIONW_0 { dwValue: value },
        }
    }

    fn list(options: &mut [INTERNET_PER_CONN_OPTIONW]) -> INTERNET_PER_CONN_OPTION_LISTW {
        INTERNET_PER_CONN_OPTION_LISTW {
            dwSize: std::mem::size_of::<INTERNET_PER_CONN_OPTION_LISTW>() as u32,
            pszConnection: null_mut(),
            dwOptionCount: options.len() as u32,
            dwOptionError: 0,
            pOptions: options.as_mut_ptr(),
        }
    }
}

#[cfg(test)]
mod tests;
