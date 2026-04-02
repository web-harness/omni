import type { Meta, StoryObj } from "@storybook/web-components";
import { html } from "lit";
import "./omni-harness.js";
import type { OmniHarness } from "./omni-harness.js";

const DEV_APP_SRC = "http://127.0.0.1:8080";
const appSrc = import.meta.env.DEV ? DEV_APP_SRC : "./app/";

const meta: Meta<OmniHarness> = {
  title: "Components/OmniHarness",
  component: "omni-harness",
  tags: ["autodocs"],
  argTypes: {
    src: { control: "text" },
    theme: { control: { type: "select" }, options: ["dark", "light"] },
    agents: { control: "object" },
  },
};

export default meta;
type Story = StoryObj<OmniHarness>;

export const Default: Story = {
  args: {
    src: appSrc,
    theme: "dark",
    agents: [{ url: "https://agent1.example.com/api", apiKey: "sk-fake-key-1" }],
  },
  render: (args) => html`
    <omni-harness
      style="display:block;width:100%;height:600px;"
      .agents=${args.agents}
      src=${args.src}
      theme=${args.theme}
    ></omni-harness>
  `,
};

export const Light: Story = {
  args: {
    src: appSrc,
    theme: "light",
    agents: [
      { url: "https://agent1.example.com/api", apiKey: "sk-fake-key-1" },
      { url: "https://agent2.example.com/api", apiKey: "sk-fake-key-2" },
    ],
  },
  render: (args) => html`
    <omni-harness
      style="display:block;width:100%;height:600px;"
      .agents=${args.agents}
      src=${args.src}
      theme=${args.theme}
    ></omni-harness>
  `,
};
