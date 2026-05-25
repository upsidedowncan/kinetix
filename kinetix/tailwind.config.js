/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        background: '#09090b', // Zinc 950
        surface: '#18181b',    // Zinc 900
        'surface-lighter': '#27272a', // Zinc 800
        'surface-border': '#3f3f46',  // Zinc 700
        primary: {
          DEFAULT: '#3b82f6', // Blue 500
          hover: '#2563eb',   // Blue 600
        },
        accent: {
          DEFAULT: '#f43f5e', // Rose 500
          hover: '#e11d48',   // Rose 600
        },
        muted: '#71717a',      // Zinc 500
      },
      borderRadius: {
        'xl': '12px',
        '2xl': '16px',
      },
      boxShadow: {
        'pro': '0 0 20px -5px rgba(0, 0, 0, 0.5)',
        'pro-lg': '0 0 40px -10px rgba(0, 0, 0, 0.7)',
      },
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'],
      }
    },
  },
  plugins: [],
}
