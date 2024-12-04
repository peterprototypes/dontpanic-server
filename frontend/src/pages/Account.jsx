import useSWRMutation from 'swr/mutation';
import * as yup from "yup";
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Box, Divider, Grid2 as Grid, Stack, Typography } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { useUser } from 'context/user';

import SideMenu from 'components/SideMenu';
import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';

const Account = () => {
  const { user } = useUser();

  const { trigger, error, isMutating } = useSWRMutation('/api/auth/login');

  const methods = useForm({
    resolver: yupResolver(AccountSchema),
    errors: error?.fields,
    defaultValues: {
      name: "",
    },
  });

  const onSubmit = (data) => {
    trigger(data)
      .then(() => { })
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  return (
    <Grid container spacing={4}>
      <Grid size={3}>
        <SideMenu />
      </Grid>
      <Grid size={9}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', mt: 2 }}>
          <Typography variant="h4">
            Account
          </Typography>
          <Typography variant="body1">
            {user.email}
          </Typography>
        </Box>
        <Divider />
        <FormProvider {...methods}>
          <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap>

            <ControlledTextField name="name" label="Your Name" placeholder="John Doe" fullWidth helperText="Max 100 characters" />

            <Stack direction="row" spacing={2} alignItems="center">
              <LoadingButton
                type="submit"
                variant="contained"
                loading={isMutating}
                loadingPosition="end"
                sx={{ width: '120px' }}
                startIcon={<SaveIcon />}
              >
                Save
              </LoadingButton>
              <FormServerError />
            </Stack>

          </Stack>
        </FormProvider>
      </Grid>
    </Grid>
  );
};

const AccountSchema = yup.object({
  name: yup.string().required("Name is required"),
}).required();

export default Account;