use wolfram_app_discovery::{discover, WolframApp, WolframAppType};

#[test]
fn test_try_default() {
    let _: WolframApp = WolframApp::try_default()
        .expect("WolframApp::try_default() could not locate any apps");
}

#[test]
fn macos_default_wolframscript_path() {
    if cfg!(not(target_os = "macos")) {
        return;
    }

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

    let engine: WolframApp = discover()
        .into_iter()
        .filter(|app: &WolframApp| app.app_type() == WolframAppType::Engine)
        .next()
        .expect("unable to locate a Wolfram Engine installation");

    let install_dir = engine.installation_directory().to_str().unwrap().to_owned();

    assert!(install_dir.contains("Wolfram Player.app"));
}

#[test]
fn macos_wolfram_engine_properties() {
    if cfg!(not(target_os = "macos")) {
        return;
    }

    let engine: WolframApp = discover()
        .into_iter()
        .filter(|app: &WolframApp| app.app_type() == WolframAppType::Engine)
        .next()
        .expect("unable to locate a Wolfram Engine installation");

    engine.wolfram_version().unwrap();
    engine.wolframscript_executable_path().unwrap();
    engine.kernel_executable_path().unwrap();
    engine.target_wstp_sdk().unwrap();
    engine.library_link_c_includes_directory().unwrap();
}
