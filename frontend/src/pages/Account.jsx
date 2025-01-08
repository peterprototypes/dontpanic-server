import useSWRMutation from 'swr/mutation';
import * as yup from "yup";
import { useSnackbar } from 'notistack';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Box, Divider, Grid2 as Grid, Stack, Typography, Link } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { useUser } from 'context/user';
import { useConfig } from 'context/config';

import SideMenu from 'components/SideMenu';
import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';
import DeleteAccount from 'components/DeleteAccount';
import PasswordChange from 'components/PasswordChange';
import RequestEmailChange from 'components/RequestEmailChange';
import Manage2FA from 'components/Manage2FA';

const Account = () => {
  const { config } = useConfig();
  const { user } = useUser();
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, error, isMutating } = useSWRMutation('/api/account');

  const methods = useForm({
    resolver: yupResolver(AccountSchema),
    errors: error?.fields,
    defaultValues: {
      name: user.name,
      pushover_user_key: user.pushover_user_key,
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

            {config.pushover_enabled && (
              <ControlledTextField
                name="pushover_user_key"
                label="Pushover user key"
                fullWidth
                helperText={(
                  <>
                    <Typography variant="body2" sx={{ mt: 1 }}>
                      Pushover is a third-party service that allows you to receive push notifications on your mobile device.
                    </Typography>
                    <Typography variant="body2" sx={{ mt: 1 }}>
                      You can find your user key in the <Link href="https://pushover.net" target="_blank" rel="noreferrer">Pushover website</Link>.
                    </Typography>
                  </>
                )}
              />
            )}

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
  pushover_user_key: yup.string().max(60),
}).required();

export default Account;