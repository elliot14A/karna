/** @type {import('tailwindcss').Config} */
module.exports = {
  content: {
    files: [
      "./app/src/**/*.rs",
    ],
  },
  theme: {
    extend: {},
  },
  plugins: [
    require("@tailwindcss/typography"),
    require("daisyui")
  ],
  daisyui: {
    themes: ["light", "dark"], 
    darkTheme: "dark",
    base: true, 
    styled: true,
    utils: true,
    prefix: "",
    logs: true,
  }
}
