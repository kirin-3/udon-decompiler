use std::path::{Path, PathBuf};
use udon_decompiler::decompiler::DecompileContext;
use udon_decompiler::odin::UdonProgramBinary;
use udon_decompiler::util::read_program_bytes;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
#[test]
fn artsy_toggle_multiple_udonsharp_golden() {
    let path = fixture("ArtsyToggleMultiple.hex");
    let bytes = read_program_bytes(&path).unwrap();
    let program = UdonProgramBinary::parse_bytes(&bytes).unwrap();
    let mut ctx = DecompileContext::from_file(&path).unwrap();
    let formatter = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("node_modules/.bin")
        .join(if cfg!(windows) {
            "clang-format.cmd"
        } else {
            "clang-format"
        });
    ctx.set_clang_format_override(Some(formatter));
    ctx.enable_udonsharp(&program, &fixture("UdonEvents.json"), None)
        .unwrap();
    let source = ctx.run_decompile().unwrap().generated_code;
    assert!(source.contains("public override void Interact()"));
    assert!(source.contains("global::System.Int32 i = 0;"));
    assert_eq!(
        source
            .matches("(global::VRC.SDKBase.VRC_Pickup.PickupHand)")
            .count(),
        2
    );
    assert!(!source.contains("unserializable"));
    assert_eq!(
        source.replace("\r\n", "\n").trim(),
        include_str!("fixtures/ArtsyToggleMultiple.us.cs").trim()
    );
}

#[test]
fn sync_modes_and_builtin_event_parameters_survive() {
    let path = fixture("StringSync.hex");
    let program = UdonProgramBinary::parse_bytes(&read_program_bytes(&path).unwrap()).unwrap();
    let mut ctx = DecompileContext::from_file(&path).unwrap();
    ctx.set_clang_format_enabled(false);
    ctx.enable_udonsharp(&program, &fixture("UdonEvents.json"), Some("Manual".into()))
        .unwrap();
    assert_eq!(
        ctx.udonsharp.as_ref().unwrap().synced_fields["syncedString"],
        "None"
    );
    let source = ctx.run_decompile().unwrap().generated_code;
    assert!(source.contains("[UdonSynced(UdonSyncMode.None)]"));
    assert!(source.contains("[UdonBehaviourSyncMode(BehaviourSyncMode.Manual)]"));
    let path = fixture("Sensor.hex");
    let program = UdonProgramBinary::parse_bytes(&read_program_bytes(&path).unwrap()).unwrap();
    let mut ctx = DecompileContext::from_file(&path).unwrap();
    ctx.set_clang_format_enabled(false);
    ctx.enable_udonsharp(&program, &fixture("UdonEvents.json"), None)
        .unwrap();
    let source = ctx.run_decompile().unwrap().generated_code;
    assert!(
        source.contains(
            "public override void OnPlayerJoined(global::VRC.SDKBase.VRCPlayerApi player)"
        )
    );
    assert!(!source.contains("VRCPlayerApi onPlayerJoinedPlayer ="));
}
