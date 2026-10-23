import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Tangent",
  description: "Piano practice, MIDI playback and live chord recognition",
  base: "/Neothesia/",
  head: [["link", { rel: "icon", type: "image/svg+xml", href: "/Neothesia/tangent-icon.svg" }]],
  themeConfig: {
    logo: "/tangent-icon.svg",
    nav: [
      { text: "Home", link: "/" },
      { text: "How-to", link: "/pages/installation" },
    ],

    sidebar: [
      {
        text: "How-to",
        items: [
          { text: "Installation", link: "/pages/installation" },
          { text: "Shortcuts", link: "/pages/shortcuts" },
          { text: "MIDI Library", link: "/pages/midi-library" },
          { text: "Arcade", link: "/pages/arcade" },
          { text: "Live Chords", link: "/pages/chords" },
          { text: "Customization", link: "/pages/customization" },
          { text: "Branding", link: "/pages/branding" },
          { text: "Video Encoding", link: "/pages/video-encoding" },
        ],
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/passeride/Neothesia" },
    ],
  },
});
