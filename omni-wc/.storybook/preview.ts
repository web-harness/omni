import type { Preview } from "@storybook/web-components";

const preview: Preview = {
  parameters: {
    backgrounds: {
      default: "dark",
      values: [
        { name: "dark", value: "#0d0d0f" },
        { name: "light", value: "#f5f5f7" },
      ],
    },
  },
};

export default preview;
