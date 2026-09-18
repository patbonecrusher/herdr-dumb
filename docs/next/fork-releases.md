# herdr-dumb automated builds and releases

This custom fork uses `.github/workflows/fork-build-release.yml`. The upstream
stable/preview workflows remain restricted to `herdrdev/herdr`; do not use
`just release` or `just preview` to publish this fork.

## Build downloads

Pushes to `master`, pull requests targeting `master`, and manual dispatches run
native builds on GitHub-hosted Windows x86-64 and macOS Apple Silicon runners.
They do not install a runner on your personal computer.

After a successful run, download the artifacts from the workflow's Actions page:

- `herdr-dumb-windows-x86_64.zip`: `herdr-dumb.exe`, the pinned ConPTY runtime,
  and its third-party notices. Keep the `conpty` directory beside the executable.
- `herdr-dumb-macos-arm64.tar.gz`: `herdr-dumb` and `LICENSE`.

Actions artifacts expire after 14 days. Publishing a tagged release uploads
the archives as release assets instead. Both builds must pass native unit and
local-only CLI tests. Windows packaging verifies Microsoft's package signature,
Authenticode signatures, pinned hashes, and static CRT linkage.

## Publish a new version

1. Update the package version in `Cargo.toml` and regenerate `Cargo.lock` with
   Cargo. Include the version change in the commit you intend to release.
2. Test, commit, and push that commit to this fork's `master` branch. Follow the
   repository's commit-message approval rules.
3. Create an annotated tag matching the package version, then push only that tag.
   For example, **after** both manifests say `0.9.2`:

   ```sh
   git tag -a v0.9.2 -m "herdr-dumb 0.9.2"
   git push origin v0.9.2
   ```

The tag must be `vMAJOR.MINOR.PATCH`, exactly match `Cargo.toml`, and point to a
commit reachable from `origin/master`. Only repository admins may publish or
rerun publication. Tag pushes rebuild both targets, then create a GitHub Release
with both archives and `SHA256SUMS`. Branch pushes and manual runs never publish.
The workflow does not increment versions, create tags, overwrite existing
releases, publish upstream, update distribution manifests, or close issues.

If SSH authentication selects a different GitHub account, use your correctly
configured account when pushing the tag, just as when pushing normal commits.
Protect `v*` tags in repository settings to prevent unintended edits or deletion.

These binaries are not Developer ID / publisher signed or macOS-notarized;
the bundled Microsoft runtime is signature-verified. Gatekeeper or SmartScreen
may warn. Signing/notarization would require separately configured credentials.

Publishing releases does not restore application auto-updates or remote control.

## Local verification

```sh
just maintenance-test
just test-release-target aarch64-apple-darwin
```

Use a recent Bun with `Bun.YAML` support for the workflow contract tests. Native
Windows testing and package-signature verification run on the Windows runner.
The fork packaging helper requires Python 3.11 or later.
