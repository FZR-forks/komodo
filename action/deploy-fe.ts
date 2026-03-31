const result = await new Deno.Command("km", {
  args: ["run", "-y", "action", "deploy-komodo-fe-change"],
}).output();

if (!result.success) {
  const stderr = new TextDecoder("utf-8").decode(result.stderr).trim();
  const stdout = new TextDecoder("utf-8").decode(result.stdout).trim();
  console.error(
    `Command failed with exit code ${result.code}: ${stderr || stdout}`
  );
  Deno.exit(result.code ?? 1);
}
