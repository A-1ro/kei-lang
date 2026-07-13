// @cloudflare/vitest-pool-workers 0.18.4(vitest 4 系)の実際のエクスポートには
// `defineWorkersConfig`/`/config` サブパスが存在しない(v3→v4 移行で
// `cloudflareTest` プラグイン方式に変更された。パッケージ同梱の
// dist/codemods/vitest-v3-to-v4.mjs が移行内容の正)。ここではその新 API に合わせる。
import { fileURLToPath } from "node:url";
import path from "node:path";
import { defineConfig } from "vitest/config";
import { cloudflareTest } from "@cloudflare/vitest-pool-workers";

// cwd が examples/workers-api 以外(モノレポ root からの並列実行等)でも
// config を解決できるよう __dirname 相当を作り、絶対パスで wrangler.jsonc を指す。
const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: path.resolve(__dirname, "./wrangler.jsonc") },
      // e2e 時のみ debug ルートを有効化する。本番 wrangler.jsonc の vars には書かない。
      // src/index.ts の `/debug/*` ミドルウェアが env.ENABLE_DEBUG_ROUTES === "true" のときのみ配線する。
      miniflare: {
        bindings: {
          ENABLE_DEBUG_ROUTES: "true",
        },
      },
    }),
  ],
});
