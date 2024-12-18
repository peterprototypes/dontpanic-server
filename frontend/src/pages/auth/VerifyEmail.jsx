import React from 'react';
import useSWRMutation from 'swr/mutation';
import { Link as RouterLink, useParams } from "react-router";
import { Stack, Typography, Link } from "@mui/material";
import Logo from "components/Logo";

const VerifyEmail = () => {
  let { hash } = useParams();

  const [error, setError] = React.useState(null);

  const { trigger } = useSWRMutation('/api/auth/verify-email/' + hash);

  React.useEffect(() => {
    trigger().catch((e) => setError(e.message));
  }, [trigger]);

  return (
    <Stack alignItems="center" spacing={2}>
      <Logo sx={{ width: '100px', mb: 2 }} />

      {error && <Typography variant="h6" align="center" color="error">{error}</Typography>}

      {!error && (<>
        <Typography variant="h6" align="center">Your email is confirmed. You&lsquo;re ready to start tracking panics and error messages in your Rust apps.</Typography>
        <Typography variant="h6" align="center">
          <Link component={RouterLink} to="/auth/login">Login</Link>
          {' '}
          to your account.
        </Typography>
      </>)}
    </Stack>
  );
};

export default VerifyEmail;