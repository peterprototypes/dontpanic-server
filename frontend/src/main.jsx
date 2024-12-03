import { createRoot } from 'react-dom/client';
import { SWRConfig } from 'swr';
import CssBaseline from '@mui/material/CssBaseline';
import { ThemeProvider } from '@mui/material/styles';
import { SnackbarProvider } from 'notistack';

import App from './App.jsx';
import theme from './theme';

const fetcher = async (url, data) => {
  const api_url = import.meta.env.DEV ? "http://localhost:8080" : "";

  const res = await fetch(api_url + url, {
    method: data?.arg ? 'POST' : 'GET',
    body: data?.arg ? JSON.stringify(data?.arg) : null,
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
  });

  if (!res.ok) {
    const data = await res.json();

    const error = new Error(data?.user?.message ?? 'An error occurred');
    error.user = data.user;
    error.fields = data.fields;
    error.status = res.status;
    throw error;
  }

  return res.json();
};

const authMiddleware = (useSWRNext) => {
  return (key, fetcher, config) => {
    const swr = useSWRNext(key, fetcher, config);

    if (swr?.error?.status === 401) {
      window.location.href = '/auth/login';
    }

    return swr;
  };
};

const SWR_OPTIONS = {
  fetcher,
  use: [authMiddleware],
  revalidateOnFocus: false,
};

createRoot(document.getElementById('root')).render(
  <SWRConfig value={SWR_OPTIONS}>
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <SnackbarProvider>
        <App />
      </SnackbarProvider>
    </ThemeProvider>
  </SWRConfig>,
);
