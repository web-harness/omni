import type { StorybookConfig } from "@storybook/web-components-vite";

const config: StorybookConfig = {
  stories: ["../src/**/*.stories.ts"],
  addons: ["@storybook/addon-essentials"],
  framework: {
    name: "@storybook/web-components-vite",
    options: {},
  },
  viteFinal: (config, { configType }) => {
    if (process.env.BASE_PATH) {
      config.base = process.env.BASE_PATH;
    }
    config.define = {
      ...config.define,
      __DEV__: configType === "DEVELOPMENT",
    };
    return config;
  },
};

export default config;
