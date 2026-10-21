const results = await Promise.all([
  Bun.build({ entrypoints: ["src/main.ts"], outdir: "dist", target: "browser", minify: true, naming: "app.[ext]" }),
  Bun.build({ entrypoints: ["styles/tokens.css"], outdir: "dist", minify: true, naming: "app.[ext]" }),
]);

for (const r of results) {
  if (!r.success) {
    console.error(r.logs);
    process.exit(1);
  }
}
