/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'bg-main': 'rgb(30, 35, 50)',
        'bg-card': 'rgb(38, 45, 62)',
        'bg-input': 'rgb(45, 52, 70)',
        'primary': 'rgb(80, 160, 255)',
        'primary-dim': 'rgb(60, 120, 200)',
        'success': 'rgb(80, 200, 120)',
        'warning': 'rgb(255, 180, 80)',
        'error': 'rgb(255, 100, 100)',
        'text-primary': 'rgb(220, 225, 235)',
        'text-secondary': 'rgb(160, 170, 190)',
        'text-muted': 'rgb(120, 130, 150)',
        'text-bright': 'rgb(255, 255, 255)',
        'user-msg': 'rgb(60, 100, 160)',
        'assistant-msg': 'rgb(45, 55, 75)',
      },
    },
  },
  plugins: [],
}
