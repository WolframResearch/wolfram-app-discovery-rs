use std::path::PathBuf;

use wolfram_app_discovery::WolframApp;

#[test]
fn macos_default_wolframscript_path() {
    let app = WolframApp::try_default().expect("failed to locate Wolfram app");

    let wolframscript_path = app
        .wolframscript_executable_path()
        .expect("failed to locate wolframscript");

    assert!(wolframscript_path.ends_with("MacOS/wolframscript"));
}

/// Test that the WolframApp representing a Wolfram Engine application correctly resolves
/// paths to the Wolfram Player.app that is used to support Wolfram Engine.
#[test]
fn macos_wolfram_engine_contains_wolfram_player() {
    if cfg!(not(target_os = "macos")) {
        return;
    }

    let engine =
        WolframApp::from_app_directory(PathBuf::from("/Applications/Wolfram Engine.app"))
            .unwrap();

    let install_dir = engine.installation_directory().to_str().unwrap().to_owned();

    assert!(install_dir.contains("Wolfram Player.app"));
}

#[test]
fn macos_wolfram_engine_properties() {
    let engine =
        WolframApp::from_app_directory(PathBuf::from("/Applications/Wolfram Engine.app"))
            .unwrap();

    engine.wolfram_version().unwrap();
    engine.wolframscript_executable_path().unwrap();
    engine.kernel_executable_path().unwrap();
    engine.wstp_c_header_path().unwrap();
    engine.wstp_static_library_path().unwrap();
    engine.library_link_c_includes_path().unwrap();
}
