import { build } from "../../wasm/build_index/pkg-node/docfind_build_index.js";
import { readFileSync, writeFileSync } from "fs";

function main() {
  let file = "demo/build_index/documents.json";
  if (process.argv[2] === "size") {
    file = "demo/build_index/size.json";
  }
  const documentsJson = readFileSync(file, "utf-8");
  const index = build(documentsJson);
  writeFileSync("demo/build_index/index.bin", index);
}

main();
