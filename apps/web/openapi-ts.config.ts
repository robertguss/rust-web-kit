import { defineConfig } from "@hey-api/openapi-ts";

export default defineConfig({
  input: "./openapi.json",
  output: {
    clean: true,
    path: "src/api/generated",
  },
  plugins: [
    {
      name: "@hey-api/client-fetch",
      baseUrl: "/api",
      runtimeConfigPath: "./src/api/client.ts",
    },
    "@hey-api/sdk",
    "@hey-api/typescript",
    "@tanstack/react-query",
  ],
});
