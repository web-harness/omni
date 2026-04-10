import type { Meta, StoryObj } from "@storybook/web-components";
import { html } from "lit";
import "./omni-harness.js";
import type { OmniHarness } from "./omni-harness.js";

declare const __DEV__: boolean;

const appSrc = __DEV__ ? "http://127.0.0.1:8080" : "./app/";

const meta: Meta<OmniHarness> = {
  title: "Components/OmniHarness",
  component: "omni-harness",
  tags: ["autodocs"],
  parameters: {
    layout: "fullscreen",
  },
  argTypes: {
    src: { control: "text" },
    theme: { control: { type: "select" }, options: ["dark", "light"] },
    agents: { control: "object" },
    dicebearStyle: {
      control: { type: "select" },
      options: ["bottts-neutral", "thumbs"],
    },
  },
};

export default meta;
type Story = StoryObj<OmniHarness>;

export const Default: Story = {
  args: {
    src: appSrc,
    theme: "dark",
    dicebearStyle: "bottts-neutral",
    agents: [{ url: "https://agent1.example.com/api", apiKey: "sk-fake-key-1" }],
  },
  render: (args) => html`
    <omni-harness
      style="display:block;width:100%;height:100vh;"
      .agents=${args.agents}
      .dicebearStyle=${args.dicebearStyle}
      src=${args.src}
      theme=${args.theme}
    ></omni-harness>
  `,
};

export const Light: Story = {
  args: {
    src: appSrc,
    theme: "light",
    dicebearStyle: "bottts-neutral",
    agents: [
      { url: "https://agent1.example.com/api", apiKey: "sk-fake-key-1" },
      { url: "https://agent2.example.com/api", apiKey: "sk-fake-key-2" },
    ],
  },
  render: (args) => html`
    <omni-harness
      style="display:block;width:100%;height:100vh;"
      .agents=${args.agents}
      .dicebearStyle=${args.dicebearStyle}
      src=${args.src}
      theme=${args.theme}
    ></omni-harness>
  `,
};
