module.exports = {
  content: ['*.html','./src/**/*.rs'],
  theme: {
    screens: {
      sm: '480px',
      md: '768px',
      lg: '976px',
      xl: '1440px'
    },
    extend: {
      // Tints are made with https://maketintsandshades.com
      colors: {
        'kvm-green': '#96bf0c',
        'kvm-yellow': '#ffdd00',
        'kvm-pink': '#e56091',
        'kvm-raspberry': {
            light: '#e6c3d3',
            DEFAULT: '#aa386b',
        },
        'kvm-bluegray': '#637a84',
        'kvm-blue': {
           light: '#ccebef',
           DEFAULT: '#0099ad',
        }
      },
    },
  },
  plugins: [],
}
