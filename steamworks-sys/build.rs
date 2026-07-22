fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::path::{Path, PathBuf};

    let sdk_loc = if let Ok(sdk_loc) = env::var("STEAM_SDK_LOCATION") {
        Path::new(&sdk_loc).to_path_buf()
    } else {
        let mut path = PathBuf::new();
        path.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.push("lib");
        path.push("steam");
        path
    };
    println!("cargo:rerun-if-env-changed=STEAM_SDK_LOCATION");

    let binding_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let binding_path = binding_path.join("bindings.rs");
    let bindings = bindgen::Builder::default()
        .header(
            sdk_loc
                .join("public/steam/steam_api_flat.h")
                .to_string_lossy(),
        )
        .header(
            sdk_loc
                .join("public/steam/steam_gameserver.h")
                .to_string_lossy(),
        )
        .clang_arg("-xc++")
        .clang_arg("-std=c++11")
        .clang_arg(format!("-I{}", sdk_loc.join("public").display()))
        .allowlist_file(".*[/\\\\]public[/\\\\]steam[/\\\\].*")
        .allowlist_function("Steam.*")
        .allowlist_var("^(k.*|STEAM.*|Steam.*|MASTERSERVER.*|INVALID_.*)$")
        .blocklist_item("^(__security_cookie|k_SteamItemInstanceIDInvalid)$")
        .raw_line(
            "pub const k_SteamItemInstanceIDInvalid: SteamItemInstanceID_t = SteamItemInstanceID_t::MAX;",
        )
        // bindgen 0.72.0+ has a bug that causes it to generate invalid code for some of the Steam API types.
        // See https://github.com/rust-lang/rust-bindgen/issues/380#issuecomment-2655035849
        .opaque_type("^(ISteamParties|ISteamUGC|ISteamTimeline|ISteamVideo|ISteamInventory)$")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        .bitfield_enum("EMarketNotAllowedReasonFlags")
        .bitfield_enum("EBetaBranchFlags")
        .bitfield_enum("EFriendFlags")
        .bitfield_enum("EPersonaChange")
        .bitfield_enum("ERemoteStoragePlatform")
        .bitfield_enum("EChatSteamIDInstanceFlags")
        .bitfield_enum("ESteamItemFlags")
        .bitfield_enum("EOverlayToStoreFlag")
        .bitfield_enum("EChatSteamIDInstanceFlags")
        .dynamic_library_name("SteamAPI")
        .dynamic_link_require_all(false)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(binding_path)
        .expect("Couldn't write bindings!");

    Ok(())
}
