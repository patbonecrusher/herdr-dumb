use std::process::Command;

#[test]
fn removed_remote_entry_points_are_rejected_without_starting_a_session() {
    for args in [
        vec!["--remote", "unused"],
        vec!["--remote=unused"],
        vec!["--remote-keybindings", "server"],
        vec!["--machine", "unused", "agent", "list"],
        vec!["machine", "list"],
        vec!["remote-client-bridge", "--check"],
        vec!["remote-api-bridge", "--check"],
        vec!["plugin", "--help"],
        vec!["plugin", "install", "untrusted/repo", "--yes"],
        vec!["plugin", "link", "/nonexistent"],
        vec!["plugin", "action", "invoke", "old.action"],
        vec!["update"],
        vec!["update", "--handoff"],
        vec!["channel", "set", "preview"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_herdr-dumb"))
            .args(&args)
            .output()
            .expect("run CLI");
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("unknown"));
    }
}

#[test]
fn help_config_and_completions_advertise_only_local_connections() {
    for args in [
        vec!["--help"],
        vec!["--default-config"],
        vec!["completion", "bash"],
        vec!["completion", "zsh"],
        vec!["completion", "fish"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_herdr-dumb"))
            .args(&args)
            .output()
            .expect("run CLI");
        assert!(output.status.success(), "{args:?}: {output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        for removed in [
            "--remote",
            "--machine",
            "remote_image_paste",
            "manage_ssh_config",
            "herdr machine",
            "version_check",
            "manifest_check",
            "update-agent-manifests",
            "plugin_action",
            "plugin install",
            "herdr-dumb plugin",
        ] {
            assert!(!text.contains(removed), "{args:?} advertises {removed}");
        }
    }
}

#[test]
fn renamed_binary_has_no_manifest_download_command() {
    let version = Command::new(env!("CARGO_BIN_EXE_herdr-dumb"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("herdr-dumb "));
    let output = Command::new(env!("CARGO_BIN_EXE_herdr-dumb"))
        .args(["server", "update-agent-manifests"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn updater_processes_are_not_part_of_the_application() {
    // An architecture guard complements CLI rejection: old config/env values
    // must not silently reactivate a background downloader in release builds.
    for source in [
        include_str!("../src/update.rs"),
        include_str!("../src/detect/manifest_update.rs"),
        include_str!("../src/app/mod.rs"),
        include_str!("../src/app/runtime.rs"),
    ] {
        assert!(!source.contains("curl_command"));
        assert!(!source.contains("fn auto_update("));
        assert!(!source.contains("::auto_update("));
        assert!(!source.contains("next_auto_update_check"));
        assert!(!source.contains("next_agent_manifest_update_check"));
    }
}

#[test]
fn plugin_execution_is_not_part_of_startup_or_events() {
    for source in [
        include_str!("../src/main.rs"),
        include_str!("../src/app/mod.rs"),
        include_str!("../src/app/api.rs"),
        include_str!("../src/server/headless/bootstrap.rs"),
        include_str!("../src/persist.rs"),
    ] {
        for removed in [
            "mod plugin_command",
            "mod plugin_paths",
            "mod plugin_registry",
            "run_plugin_startup_hooks",
            "run_plugin_event_hooks",
            "load_plugin_registry",
        ] {
            assert!(
                !source.contains(removed),
                "plugin execution path returned: {removed}"
            );
        }
    }
}

#[test]
fn client_clipboard_images_are_never_staged_or_pasted() {
    for source in [
        include_str!("../src/server/mod.rs"),
        include_str!("../src/server/headless.rs"),
        include_str!("../src/server/client_transport.rs"),
    ] {
        for removed in [
            "mod clipboard_image",
            "ServerEvent::ClientClipboardImage",
            "staged_clipboard_files",
            "paste_client_clipboard_image_path",
        ] {
            assert!(
                !source.contains(removed),
                "clipboard image forwarding path returned: {removed}"
            );
        }
    }
}

#[test]
fn local_control_integrations_and_fork_guide_remain_available() {
    for args in [
        vec!["integration", "--help"],
        vec!["api", "--help"],
        vec!["--skill"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_herdr-dumb"))
            .args(&args)
            .output()
            .expect("run CLI");
        assert!(output.status.success(), "{args:?}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stdout).contains("herdr-dumb"));
    }
    // Never substitute the upstream guide, which advertises removed commands.
    let guide = include_str!("../src/agent-guide.md");
    assert!(guide.contains("only run trusted code"));
    assert!(guide
        .contains("Application updates, update channels, and detection-manifest downloads are"));
}
