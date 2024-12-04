import { createTheme } from '@mui/material/styles';

import '@fontsource/inter/300.css';
import '@fontsource/inter/400.css';
import '@fontsource/inter/500.css';
import '@fontsource/inter/700.css';

const theme = createTheme({
  palette: {
    primary: {
      main: '#0071e3',
    },
    background: {
      default: '#fbfbfd',
    },
    text: {
      primary: "#1d1d1f"
    },
    accentBackground: "#f2f2f7"
  },
  typography: {
    fontFamily: '"Inter", sans-serif',
    fontSize: 12
  },
  components: {
    MuiLink: {
      defaultProps: {
        underline: 'hover',
      },
    },
    MuiTextField: {
      defaultProps: {
        size: 'small',
      },
    },
    MuiOutlinedInput: {
      styleOverrides: {
        root: {
          borderRadius: '3px',
          backgroundColor: '#fbfbfd'
        },
      },
    },
    MuiButton: {
      styleOverrides: {
        root: {
          padding: "4px 16px",
        },
      },
      defaultProps: {
        disableElevation: true
      }
    },
    MuiButtonBase: {
      styleOverrides: {
        // root: {
        //   padding: 0,
        // },
      },
    },
    MuiFormLabel: {
      styleOverrides: {
        root: {
          fontWeight: 500,
          fontSize: '0.78rem',
          color: '#1d1d1f',
        },
      },
    },
  }
});

export default theme;