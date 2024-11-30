import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { SWRConfig } from 'swr';
import CssBaseline from '@mui/material/CssBaseline';
import { ThemeProvider } from '@mui/material/styles';

import App from './App.jsx';
import theme from './theme';

const fetcher = async (url, { arg }) => {
  const api_url = import.meta.env.VITE_API_URL ?? "";

  const res = await fetch(api_url + url, {
    method: arg ? 'POST' : 'GET',
    body: arg ? JSON.stringify(arg) : null,
  });

  if (!res.ok) {
    const error = new Error('An error occurred while executing request');
    error.data = await res.json();
    error.status = res.status;
    throw error;
  }

  return res.json();
};

const authMiddleware = (useSWRNext) => {
  return (key, fetcher, config) => {
    const swr = useSWRNext(key, fetcher, config);

    if (swr?.error?.status === 401) {
      window.location.href = '/login';
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
  <StrictMode>
    <SWRConfig value={SWR_OPTIONS}>
      <ThemeProvider theme={theme}>
        <CssBaseline />
        <App />
      </ThemeProvider>
    </SWRConfig>
  </StrictMode>,
);
