import { Stack, Typography, Alert, Link } from '@mui/material';
import { Link as RouterLink } from 'react-router';
import { useConfirm } from "material-ui-confirm";
import { useSnackbar } from 'notistack';
import useSWR, { mutate } from 'swr';
import useSWRMutation from 'swr/mutation';

import { useUser } from 'context/user';
import { LoadingButton } from '@mui/lab';

const DeleteAccount = () => {
  const confirm = useConfirm();
  const { enqueueSnackbar } = useSnackbar();
  const { user } = useUser();
  const { trigger, error, isMutating } = useSWRMutation('/api/account/delete');
  const { data: organizations } = useSWR('/api/organizations');

  const soleOwnerOrgs = organizations?.filter((org) =>
    // orgs with single owner
    org.members.filter((member) => member.role === 'owner').length === 1 &&
    // that single owner is the current user
    org.members.filter((member) => member.role === 'owner' && member.user_id == user.user_id).length > 0
  ) ?? [];

  const onDeleteAccount = () => {
    let config = {
      title: 'Are you sure?',
      description: 'You\'re about to permanently delete your account. This action cannot be undone.',
      acknowledgement: 'I understand',
      confirmationText: 'Delete Account'
    };

    confirm(config)
      .then(() => trigger({})
        .then(() => mutate('/api/account'))
        .catch((e) => enqueueSnackbar(e.message, { variant: 'error' }))
      )
      .catch(() => { });
  };

  return (
    <Stack sx={{ mt: 4 }} spacing={1} alignItems="flex-start" useFlexGap>
      <Typography variant="h5" color="error">Account termination</Typography>

      {soleOwnerOrgs.length > 0 ? (
        <>
          <Typography variant="body1" color="textSecondary">Your account is currently the sole owner in the following organizations:</Typography>

          <ul>
            {soleOwnerOrgs.map((org) => (
              // TODO: revisit this link
              <li key={org.organization_id}><Link component={RouterLink} to={`/organization/${org.organization_id}/members`}>{org.name}</Link></li>
            ))}
          </ul>

          <Alert severity="warning" sx={{ width: '100%' }}>
            You must add another owner or delete these before you can delete your account.
          </Alert>
        </>
      ) : (
        <Typography variant="body1" color="textSecondary">Deleting your account will remove all your data and cannot be undone.</Typography>
      )}

      <LoadingButton
        variant="outlined"
        color="error"
        sx={{ mt: 2 }}
        onClick={onDeleteAccount}
        loading={isMutating}
        disabled={soleOwnerOrgs.length > 0}
      >
        Delete Account
      </LoadingButton>

      {error && <Alert severity="error" sx={{ width: '100%', mt: 2 }}>{error.message}</Alert>}
    </Stack>
  );
};

export default DeleteAccount;