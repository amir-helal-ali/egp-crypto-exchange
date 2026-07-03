/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // لوحة ألوان احترافية للوضع الداكن
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
          purple: '#a855f7',
        },
        text: {
          primary: '#e6e8ee',
          secondary: '#8b94a8',
          tertiary: '#5a647a',
        },
      },
      fontFamily: {
        // خط عربي أساسي + سانس اللاتيني
        sans: ['Cairo', 'Tajawal', 'Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'Menlo', 'monospace'],
        display: ['Cairo', 'Tajawal', 'sans-serif'],
      },
      animation: {
        'pulse-slow': 'pulse 3s ease-in-out infinite',
        'flash-green': 'flash-green 0.4s ease-out',
        'flash-red': 'flash-red 0.4s ease-out',
        'slide-in': 'slide-in 0.3s ease-out',
        'fade-in': 'fade-in 0.3s ease-out',
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
        'slide-in': {
          '0%': { transform: 'translateY(-10px)', opacity: '0' },
          '100%': { transform: 'translateY(0)', opacity: '1' },
        },
        'fade-in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
      },
    },
  },
  plugins: [],
};
