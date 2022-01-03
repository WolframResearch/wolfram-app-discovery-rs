use wolfram_app_discovery::WolframApp;

#[test]
fn test() {
    let app = WolframApp::try_default().expect("failed to locate Wolfram app");

    let wolframscript_path = app
        .wolframscript_executable_path()
        .expect("failed to locate wolframscript");

    assert!(wolframscript_path.ends_with("MacOS/wolframscript"));
}
