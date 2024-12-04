import { Button, Stack, Typography, Alert, Link } from '@mui/material';
import { Link as RouterLink } from 'react-router';

const DeleteAccount = () => {
  let { user } = useUser();

  return (
    <Stack sx={{ mt: 4 }} spacing={1} alignItems="flex-start" useFlexGap>
      <Typography variant="h6" color="error">Account termination</Typography>
      <Typography variant="body1" color="textSecondary">Your account is currently the sole owner in the following organizations:</Typography>

      <ul>
        <li><Link component={RouterLink} to="/organization/1">Organization 1</Link></li>
      </ul>

      <Alert severity="warning" sx={{ width: '100%' }}>
        You must transfer ownership or delete these before you can delete your account.
      </Alert>

      <Button variant="outlined" color="error" sx={{ mt: 4 }}>Delete Account</Button>
    </Stack>
  );
};

export default DeleteAccount;