/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // 语义色（与 src/styles.css 的 --color-* 对应，随主题切换）
        canvas: "var(--color-canvas)",
        surface: "var(--color-surface)",
        "surface-2": "var(--color-surface-2)",
        "surface-3": "var(--color-surface-3)",
        ink: "var(--color-ink)",
        muted: "var(--color-muted)",
        faint: "var(--color-faint)",
        fainter: "var(--color-fainter)",
        line: "var(--color-line)",
        "line-2": "var(--color-line-2)",
        "line-3": "var(--color-line-3)",
        divider: "var(--color-divider)",
        primary: {
          DEFAULT: "var(--color-primary)",
          hover: "var(--color-primary-hover)",
          soft: "var(--color-primary-soft)",
          text: "var(--color-primary-text)",
        },
        active: "var(--color-active)",
      },
      fontFamily: {
        // 与 src/styles.css 的 --font-display / --font-sans / --font-mono 保持一致
        display: [
          "Cormorant Garamond",
          "Tiempos Headline",
          "Songti SC",
          "SimSun",
          "Georgia",
          "Times New Roman",
          "serif",
        ],
        sans: [
          "Inter",
          "StyreneB",
          "-apple-system",
          "BlinkMacSystemFont",
          "PingFang SC",
          "Microsoft YaHei",
          "Segoe UI",
          "Roboto",
          "sans-serif",
        ],
        serif: [
          "Cormorant Garamond",
          "Tiempos Headline",
          "Songti SC",
          "SimSun",
          "Georgia",
          "Times New Roman",
          "serif",
        ],
        mono: [
          "JetBrains Mono",
          "ui-monospace",
          "SF Mono",
          "Cascadia Mono",
          "Segoe UI Mono",
          "Menlo",
          "Consolas",
          "monospace",
        ],
      },
    },
  },
  plugins: [],
};
