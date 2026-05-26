import type { Config } from 'tailwindcss';

const config: Config = {
  darkMode: 'class',
  content: ['./src/**/*.{ts,tsx,html}'],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          '"SF Pro Text"',
          '"SF Pro Display"',
          'system-ui',
          'sans-serif',
        ],
        mono: ['"SF Mono"', 'ui-monospace', 'Menlo', 'monospace'],
      },
      colors: {
        brand: {
          DEFAULT: '#0a84ff',
        },
      },
      borderRadius: {
        sm: '4px',
        md: '6px',
        lg: '10px',
      },
    },
  },
  plugins: [],
};

export default config;
