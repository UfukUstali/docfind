import * as fs from "fs";

function main() {
  const [, , wasmFile, jsFile] = process.argv;

  if (!wasmFile || !jsFile) {
    console.error("Usage: node script.js <wasmFile> <jsFile>");
    console.error(process.argv);
    process.exit(1);
  }

  const base64Wasm = fs.readFileSync(wasmFile).toString("base64");
  let jsContent = fs.readFileSync(jsFile, "utf8");

  jsContent = jsContent.replace(
    /const\s+wasmPath\s*=\s*[^;]+;[\s\S]*?readFileSync\s*\(\s*wasmPath\s*\)\s*;/,
    "const wasmBytes = Buffer.from('" + base64Wasm + "', 'base64');",
  );

  fs.writeFileSync(jsFile, jsContent);
  fs.unlinkSync(wasmFile);
}

main();
