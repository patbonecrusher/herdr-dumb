import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";

const workflow: any = Bun.YAML.parse(readFileSync(
  new URL("../.github/workflows/fork-build-release.yml", import.meta.url), "utf8",
));
const { validate, build, publish } = workflow.jobs;

describe("fork build and release boundaries", () => {
  test("builds master, pull requests, tags, and manual requests", () => {
    expect(workflow.on.push).toEqual({ branches: ["master"], tags: ["v*"] });
    expect(workflow.on.pull_request).toEqual({ branches: ["master"] });
    expect("workflow_dispatch" in workflow.on).toBe(true);
    expect(workflow.on.pull_request_target).toBeUndefined();
  });

  test("only the fork publishes, only after both native builds, only on a tag push", () => {
    expect(publish.if).toContain("github.repository == 'patbonecrusher/herdr-dumb'");
    expect(publish.if).toContain("github.event_name == 'push'");
    expect(publish.if).toContain("startsWith(github.ref, 'refs/tags/v')");
    expect(publish.needs).toEqual(["validate", "build"]);
    expect(build.needs).toBe("validate");
    expect(workflow.permissions).toEqual({ contents: "read" });
    expect(publish.permissions).toEqual({ contents: "write" });
    expect(build.permissions).toBeUndefined();
    expect(validate.permissions).toBeUndefined();
    expect(workflow.concurrency["cancel-in-progress"]).toContain("!startsWith(github.ref, 'refs/tags/')");
  });

  test("uses the two requested hosted platforms and tests before building", () => {
    const python = build.steps.find((s: any) => s.uses?.startsWith("actions/setup-python@"));
    expect(python.with["python-version"]).toBe("3.12");
    expect(build.strategy.matrix.include).toEqual([
      { os: "macos-15", target: "aarch64-apple-darwin", asset: "herdr-dumb-macos-arm64.tar.gz" },
      { os: "windows-2022", target: "x86_64-pc-windows-msvc", asset: "herdr-dumb-windows-x86_64.zip" },
    ]);
    const testing = build.steps.findIndex((s: any) => s.run?.includes("just test-release-target"));
    const compiling = build.steps.findIndex((s: any) => s.run === "just build");
    expect(testing).toBeGreaterThan(-1);
    expect(compiling).toBeGreaterThan(testing);
    expect(build.steps[compiling].env.CARGO_BUILD_TARGET).toBe("${{ matrix.target }}");
    const windows = build.steps.find((s: any) => s.name === "Package Windows and verify Microsoft signatures");
    expect(windows.run).toContain("package_windows_conpty.ps1");
    expect(windows.run).toContain("-ExecutableName herdr-dumb.exe");
    expect(windows.run).toContain('"herdr-dumb.exe") --version');
  });

  test("checks version, master ancestry, admin actors, and complete assets", () => {
    const source = validate.steps.find((s: any) => s.name === "Check release version and source");
    expect(source.run).toContain('check-version --tag "$GITHUB_REF_NAME"');
    expect(source.run).toContain('git merge-base --is-ancestor "$GITHUB_SHA" origin/master');
    expect(publish.steps[0].run).toContain('"$GITHUB_ACTOR" "$GITHUB_TRIGGERING_ACTOR"');
    expect(publish.steps[0].run).toContain('test "$permission" = admin');
    expect(publish.steps[0].run).toContain('test "$tag_commit" = "$GITHUB_SHA"');
    const release = publish.steps.at(-1).run;
    expect(release).toContain('gh release create "$GITHUB_REF_NAME"');
    expect(release).toContain("--verify-tag");
    expect(release).not.toContain("--clobber");
    expect(publish.steps.some((s: any) => s.run?.includes("checksums --directory dist"))).toBe(true);
  });

  test("passes target triples without literal quotes through Windows cmd.exe", () => {
    const justfile = readFileSync(new URL("../justfile", import.meta.url), "utf8");
    const recipe = justfile.split("\ntest-release-target target:\n")[1].split("\n\n")[0];
    expect(recipe).toContain("--target={{target}}");
    expect(recipe).not.toContain('"{{target}}"');
  });

  test("existing Windows CI uses the renamed Cargo binary", () => {
    const ci = readFileSync(new URL("../.github/workflows/ci.yml", import.meta.url), "utf8");
    expect(ci).toContain('target\\debug\\herdr-dumb.exe');
    expect(ci).toContain('target\\x86_64-pc-windows-msvc\\debug\\herdr-dumb.exe');
    expect(ci).not.toMatch(/target[^\r\n]*\\herdr\.exe/);
  });

  test("pins every action and does not interpolate event data into shell programs", () => {
    for (const job of Object.values(workflow.jobs) as any[]) {
      for (const step of job.steps) {
        if (step.uses) expect(step.uses).toMatch(/@[a-f0-9]{40}$/);
        if (step.run) expect(step.run).not.toContain("${{");
        if (step.uses?.startsWith("actions/checkout@")) {
          expect(step.with["persist-credentials"]).toBe(false);
        }
      }
    }
  });
});
