import React from 'react';
import useSWRMutation from 'swr/mutation';
import { Navigate, Link as RouterLink, useSearchParams } from "react-router";
import { Stack, Typography, Link, CircularProgress } from "@mui/material";
import Logo from "components/Logo";

const ChangeEmail = () => {
  const [searchParams] = useSearchParams();
  const payload = searchParams.get('payload');

  const [error, setError] = React.useState(null);

  const { trigger, isMutating } = useSWRMutation('/api/auth/change-email?payload=' + payload);

  React.useEffect(() => {
    trigger().catch((e) => setError(e.message));
  }, [trigger]);

  if (!payload) {
    return <Navigate to="/auth/login" />;
  }

  return (
    <Stack alignItems="center" spacing={2}>
      <Logo sx={{ width: '100px', mb: 2 }} />

      {isMutating && <CircularProgress />}

      {error && <Typography variant="h6" align="center" color="error">{error}</Typography>}

      {!error && (<>
        <Typography variant="h6" align="center">Your email has been updated.</Typography>
        <Typography variant="h6" align="center">
          Please
          {' '}
          <Link component={RouterLink} to="/auth/login">login</Link>
          {' '}
          again to your account to ensure everything is working as expected.
        </Typography>
      </>)}
    </Stack>
  );
};

export default ChangeEmail;