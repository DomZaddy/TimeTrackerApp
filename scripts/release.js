import { readFileSync, writeFileSync } from "fs";
import { join } from "path";

const BUNDLE_DIR = join("src-tauri", "target", "release", "bundle", "nsis");
const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const version = conf.version;

const zipSig = readFileSync(
  join(BUNDLE_DIR, `TimeTracker_${version}_x64-setup.nsis.zip.sig`),
  "utf8"
).trim();

const latest = {
  version,
  notes: `TimeTracker v${version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    "windows-x86_64": {
      signature: zipSig,
      url: `https://github.com/DomZaddy/TimeTrackerApp/releases/download/v${version}/TimeTracker_${version}_x64-setup.nsis.zip`,
    },
  },
};

const outPath = join(BUNDLE_DIR, "latest.json");
writeFileSync(outPath, JSON.stringify(latest, null, 2));
console.log(`Generated ${outPath}`);
console.log(`Version: ${version}`);
console.log(`\nUpload these files to GitHub Release v${version}:`);
console.log(`  - TimeTracker_${version}_x64-setup.exe`);
console.log(`  - TimeTracker_${version}_x64-setup.nsis.zip`);
console.log(`  - TimeTracker_${version}_x64-setup.nsis.zip.sig`);
console.log(`  - latest.json`);
