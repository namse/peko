import { fileURLToPath } from "node:url";
import { resolve, dirname } from "node:path";
import { spawn, type SpawnOptions } from "node:child_process";
import fs from "node:fs/promises";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export interface BuildConfig {
  server: URL;
  client: URL;
}

export async function buildServer(buildConfig: BuildConfig) {
  console.log("🚀 Building WASM component with suisei...");

  const serverDir = fileURLToPath(buildConfig.server);

  // 1. Stub 파일 생성 (사용금지 모듈 무력화용)
  const stubPath = resolve(serverDir, "stub.js");
  await fs.writeFile(stubPath, "export default {};");

  // 2. Buffer 패키지 경로 찾기
  let bufferPath: string;
  try {
    bufferPath = require.resolve("buffer/");
  } catch (e) {
    console.error("❌ Failed to resolve 'buffer' package.");
    throw e;
  }

  // 3. Shim Entry 파일 생성 (Buffer 및 Entry 연결용)
  // Intl은 여기서 제거하고 banner로 옮김 (실행 순서 문제 해결)
  const shimEntryPath = resolve(serverDir, "shim.mjs");
  const shimContent = `
import { Buffer } from 'buffer'; 

// Buffer Polyfill
globalThis.Buffer = Buffer;

// Re-export Astro Entry
export * from './entry.mjs';
`;
  await fs.writeFile(shimEntryPath, shimContent);

  // 4. Intl Polyfill 코드 (Banner용 - Import 없이 순수 JS로 작성)
  // 이 코드는 번들 파일의 최상단에 물리적으로 박히므로 가장 먼저 실행됨.
  const intlBannerCode = `
// [Shim] Intl for WASM (Injected via Banner)
if (typeof Intl === 'undefined') {
  globalThis.Intl = {
    DateTimeFormat: class {
      constructor(locales, options) {}
      format(date) { return new Date(date || Date.now()).toISOString(); }
      static supportedLocalesOf(locales, options) { return []; }
      formatToParts(date) { return []; }
      resolvedOptions() { return {}; }
    },
    NumberFormat: class {
      constructor(locales, options) {}
      format(number) { return String(number); }
      resolvedOptions() { return { locale: "en-US" }; }
    },
    Segmenter: class {
      segment(input) { return [input]; }
    },
    PluralRules: class {
      select() { return 'other'; }
    },
    getCanonicalLocales: (l) => l ? (Array.isArray(l) ? l : [l]) : []
  };
}
`;

  // 5. Rolldown 설정 파일 생성
  const rolldownConfigPath = resolve(serverDir, "rolldown.config.mjs");
  const componentOutput = resolve(serverDir, "component.js");

  const safeStubPath = JSON.stringify(stubPath);
  const safeBufferPath = JSON.stringify(bufferPath);

  await fs.writeFile(
    rolldownConfigPath,
    `export default {
  input: 'shim.mjs',
  external: /wasi:.*/,
  resolve: {
    alias: {
      "es-module-lexer": "es-module-lexer/js",
      "buffer": ${safeBufferPath},
      "node:buffer": ${safeBufferPath},
      
      // Node Built-ins Stubbing
      "sharp": ${safeStubPath},
      "node:util": ${safeStubPath},
      "node:stream": ${safeStubPath},
      "node:path": ${safeStubPath},
      "node:child_process": ${safeStubPath},
      "node:crypto": ${safeStubPath},
      "node:events": ${safeStubPath},
      "node:os": ${safeStubPath},
      "node:fs": ${safeStubPath},
      "fs": ${safeStubPath},
      "path": ${safeStubPath},
      "events": ${safeStubPath},
      "util": ${safeStubPath},
      "stream": ${safeStubPath},
      "child_process": ${safeStubPath},
      "crypto": ${safeStubPath},
      "os": ${safeStubPath},
    },
  },
  output: {
    file: 'component.js',
    format: 'esm',
    inlineDynamicImports: true,
    // ✅ 여기에 Intl 코드를 넣습니다. Import 구문이 없으므로 경로 에러가 안 납니다.
    banner: ${JSON.stringify(intlBannerCode)}
  },
};
`
  );

  const suiseiRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const witDir = resolve(suiseiRoot, "wit");
  const rolldownBin = resolve(suiseiRoot, "node_modules", ".bin", "rolldown");
  const jcoBin = resolve(suiseiRoot, "node_modules", ".bin", "jco");

  console.log("📦 Running Rolldown bundler...");
  await runCommand(rolldownBin, ["-c", rolldownConfigPath], { cwd: serverDir });

  console.log("🔧 Running JCO componentization...");
  const wasmOutput = resolve(serverDir, "component.wasm");
  const projectRoot = resolve(serverDir, "..", "..");

  const nodeModulesPath = resolve(projectRoot, "node_modules");
  await runCommand(
    jcoBin,
    ["componentize", "-w", witDir, "-o", wasmOutput, componentOutput],
    {
      cwd: projectRoot,
      env: { ...process.env, NODE_PATH: nodeModulesPath },
    }
  );

  console.log("✅ WASM component built successfully!");
  console.log(`   Output: ${wasmOutput}`);
}

function runCommand(
  cmd: string,
  args: string[],
  options: SpawnOptions = {}
): Promise<void> {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: "inherit", ...options });
    child.on("close", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`Command failed with code ${code}`));
      }
    });
    child.on("error", reject);
  });
}
