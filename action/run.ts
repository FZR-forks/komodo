import * as TOML from "@std/toml";

const decoder = new TextDecoder("utf-8");

const runCommand = async (
  command: string,
  args: string[],
  errorMessage: string
) => {
  const result = await new Deno.Command(command, { args }).output();
  if (!result.success) {
    const stderr = decoder.decode(result.stderr).trim();
    const stdout = decoder.decode(result.stdout).trim();
    throw new Error(
      `${errorMessage}. Exit code: ${result.code}. ${stderr || stdout}`
    );
  }
  return result;
};

export const run = async (action: string) => {
  if (!/^[A-Za-z0-9](?:[A-Za-z0-9-]*[A-Za-z0-9])?$/.test(action)) {
    throw new Error(`Invalid action name: ${action}`);
  }

  const branch = decoder.decode(
    (
      await runCommand(
        "git",
        ["rev-parse", "--abbrev-ref", "HEAD"],
        "Failed to determine git branch"
      )
    ).stdout
  ).trim();

  const cargo_toml_str = await Deno.readTextFile("Cargo.toml");
  const prev_version = (
    TOML.parse(cargo_toml_str) as {
      workspace: { package: { version: string } };
    }
  ).workspace.package.version;

  const version_with_count_match = prev_version.match(
    /^(\d+\.\d+\.\d+)-(.+?)-(\d+)$/
  );
  const version = version_with_count_match?.[1] ?? prev_version;
  const tag = version_with_count_match?.[2] ?? "build";
  const count = Number(version_with_count_match?.[3] ?? 0);
  const next_count = count + 1;
  const next_version = `${version}-${tag}-${next_count}`;

  const next_cargo_toml_str = cargo_toml_str.replace(
    /(\[workspace\.package\][\s\S]*?\bversion\s*=\s*")([^"]+)(")/,
    `$1${next_version}$3`
  );
  if (next_cargo_toml_str === cargo_toml_str) {
    throw new Error("Failed to update [workspace.package].version in Cargo.toml");
  }

  await Deno.writeTextFile("Cargo.toml", next_cargo_toml_str);

  // Cargo check first here to make sure lock file is updated before commit.
  await runCommand("cargo", ["check"], "cargo check failed");
  await runCommand("git", ["add", "--all"], "git add failed");
  await runCommand(
    "git",
    ["commit", "--all", "--message", `deploy ${version}-${tag}-${next_count}`],
    "git commit failed"
  );
  await runCommand("git", ["push"], "git push failed");
  await runCommand(
    "km",
    [
      "run",
      "-y",
      "action",
      action,
      `KOMODO_BRANCH=${branch}&KOMODO_VERSION=${version}&KOMODO_TAG=${tag}-${next_count}`,
    ],
    `Failed to run action "${action}"`
  );
};
