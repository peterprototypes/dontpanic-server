import { createTheme } from '@mui/material/styles';
import { grey } from "@mui/material/colors";

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
    grey: {
      main: grey[300],
      dark: grey[400],
    },
    accentBackground: "#f2f2f7"
  },
  typography: {
    fontFamily: '"Inter", sans-serif',
    fontSize: 12,
    h5: {
      // fontWeight: 500
    }
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
    MuiSelect: {
      defaultProps: {
        size: 'small',
      },
    },
    MuiOutlinedInput: {
      defaultProps: {
        size: 'small',
      },
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
    MuiTab: {
      styleOverrides: {
        root: {
          textTransform: 'none',
          fontSize: '1rem',
          minHeight: 'auto',
          padding: '4px 16px',
        },
      },
      defaultProps: {
        disableRipple: true,
      }
    },
    MuiTabs: {
      styleOverrides: {
        root: {
          minHeight: 'auto',
        }
      }
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
    MuiPaper: {
      defaultProps: {
        variant: 'outlined',
      },
    },
    MuiDataGrid: {
      defaultProps: {
        disableColumnMenu: true,
      },
    },
    MuiTableCell: {
      styleOverrides: {
        head: {
          fontWeight: 300,
          textTransform: 'uppercase',
        },
      }
    },
    MuiTable: {
      defaultProps: {
        size: 'small',
      }
    }
  }
});

export default theme;