import useSWRMutation from 'swr/mutation';
import * as yup from "yup";
import { useSnackbar } from 'notistack';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Box, Divider, Grid2 as Grid, Stack, Typography } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { useUser } from 'context/user';

import SideMenu from 'components/SideMenu';
import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';
import DeleteAccount from 'components/DeleteAccount';
import PasswordChange from 'components/PasswordChange';
import RequestEmailChange from 'components/RequestEmailChange';
import Manage2FA from 'components/Manage2FA';

const Account = () => {
  const { user } = useUser();
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, error, isMutating } = useSWRMutation('/api/account');

  const methods = useForm({
    resolver: yupResolver(AccountSchema),
    errors: error?.fields,
    defaultValues: {
      name: user.name,
    },
  });

  const onSubmit = (data) => {
    trigger(data)
      .then(() => enqueueSnackbar("Your account has been updated", { variant: 'success' }))
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  return (
    <Grid container spacing={4}>
      <Grid size={3}>
        <SideMenu />
      </Grid>
      <Grid size={9}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', mt: 2 }}>
          <Typography variant="h4">Account</Typography>
          <Typography variant="body1">{user.email}</Typography>
        </Box>

        <Divider sx={{ mb: 2 }} />

        <RequestEmailChange />

        <FormProvider {...methods}>
          <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap alignItems="flex-start">

            <ControlledTextField name="name" label="Your Name" placeholder="John Doe" fullWidth helperText="Max 100 characters." required />

            <LoadingButton
              type="submit"
              variant="contained"
              loading={isMutating}
              loadingPosition="start"
              startIcon={<SaveIcon />}
            >
              Save
            </LoadingButton>

            <FormServerError sx={{ width: '100%' }} />
          </Stack>
        </FormProvider>

        <Divider sx={{ mt: 4 }} />

        <Manage2FA />

        <Divider sx={{ mt: 4 }} />

        <PasswordChange />

        <Divider sx={{ mt: 4 }} />

        <DeleteAccount />
      </Grid>
    </Grid>
  );
};

const AccountSchema = yup.object({
  name: yup.string(),
}).required();

export default Account;