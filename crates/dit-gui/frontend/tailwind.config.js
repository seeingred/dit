/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        dit: {
          bg: "rgb(var(--dit-bg) / <alpha-value>)",
          surface: "rgb(var(--dit-surface) / <alpha-value>)",
          border: "rgb(var(--dit-border) / <alpha-value>)",
          accent: "rgb(var(--dit-accent) / <alpha-value>)",
          "accent-hover": "rgb(var(--dit-accent-hover) / <alpha-value>)",
          text: "rgb(var(--dit-text) / <alpha-value>)",
          "text-muted": "rgb(var(--dit-text-muted) / <alpha-value>)",
          success: "rgb(var(--dit-success) / <alpha-value>)",
          warning: "rgb(var(--dit-warning) / <alpha-value>)",
          danger: "rgb(var(--dit-danger) / <alpha-value>)",
        },
      },
    },
  },
  plugins: [],
};
