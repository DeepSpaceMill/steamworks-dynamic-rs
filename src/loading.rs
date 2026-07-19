use std::sync::LazyLock;

use steamworks_sys::SteamAPI;

static STEAMAPI: LazyLock<Option<SteamAPI>> = LazyLock::new(|| {
    let lib = if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            "steam_api64"
        } else {
            "steam_api"
        }
    } else {
        "steam_api"
    };

    // load the SteamAPI dynamic library from the current working directory, or from the system library path
    let p = if cfg!(target_os = "windows") {
        format!("{}.dll", lib)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", lib)
    } else {
        format!("lib{}.so", lib)
    };

    let api = unsafe { SteamAPI::new(&p).ok() };

    api
});

pub fn steam_api() -> &'static SteamAPI {
    STEAMAPI.as_ref().unwrap()
}
