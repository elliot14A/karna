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
    themes: ["retro", "dark"], 
    darkTheme: "dark",
    base: true, 
    styled: true,
    utils: true,
    logs: true,
  }
}
