// @cloudflare/vitest-pool-workers 0.18.4(vitest 4 系)の実際のエクスポートには
// `defineWorkersConfig`/`/config` サブパスが存在しない(v3→v4 移行で
// `cloudflareTest` プラグイン方式に変更された。パッケージ同梱の
// dist/codemods/vitest-v3-to-v4.mjs が移行内容の正)。ここではその新 API に合わせる。
import { defineConfig } from "vitest/config";
import { cloudflareTest } from "@cloudflare/vitest-pool-workers";

export default defineConfig({
  plugins: [
    cloudflareTest({
      wrangler: { configPath: "./wrangler.jsonc" },
    }),
  ],
});
