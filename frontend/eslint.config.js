import pluginVue from "eslint-plugin-vue";
import vueTsEslintConfig from "@vue/eslint-config-typescript";
import prettierConfig from "@vue/eslint-config-prettier";

export default [
  {
    ignores: ["dist/**", "node_modules/**"]
  },
  ...pluginVue.configs["flat/essential"],
  ...vueTsEslintConfig(),
  prettierConfig,
  {
    rules: {
      "no-console": process.env.NODE_ENV === "production" ? "warn" : "off",
      "no-debugger": process.env.NODE_ENV === "production" ? "warn" : "off",
      "max-len": ["warn", { code: 120 }]
    }
  }
];