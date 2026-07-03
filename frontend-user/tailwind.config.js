/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Professional dark-mode palette
        base: {
          900: '#0a0e17',
          800: '#0f1623',
          700: '#161f30',
          600: '#1d2738',
          500: '#243047',
        },
        accent: {
          green: '#00d68f',
          red: '#ff5252',
          yellow: '#f5a623',
          blue: '#3b82f6',
          cyan: '#06b6d4',
        },
        text: {
          primary: '#e6e8ee',
          secondary: '#8b94a8',
          tertiary: '#5a647a',
        },
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['JetBrains Mono', 'Menlo', 'monospace'],
      },
      animation: {
        'pulse-slow': 'pulse 3s ease-in-out infinite',
        'flash-green': 'flash-green 0.4s ease-out',
        'flash-red': 'flash-red 0.4s ease-out',
      },
      keyframes: {
        'flash-green': {
          '0%': { 'background-color': 'rgba(0, 214, 143, 0.3)' },
          '100%': { 'background-color': 'transparent' },
        },
        'flash-red': {
          '0%': { 'background-color': 'rgba(255, 82, 82, 0.3)' },
          '100%': { 'background-color': 'transparent' },
        },
      },
    },
  },
  plugins: [],
};
