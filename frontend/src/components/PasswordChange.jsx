import { Stack, Typography } from '@mui/material';
import { LoadingButton } from '@mui/lab';
import useSWRMutation from 'swr/mutation';
import { ControlledTextField, FormServerError } from 'components/form';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { SaveIcon } from 'components/ConsistentIcons';
import { useSnackbar } from 'notistack';

const PasswordChange = () => {
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, error, isMutating } = useSWRMutation('/api/account/update-password');

  const methods = useForm({
    resolver: yupResolver(PasswordChangeSchema),
    errors: error?.fields,
    defaultValues: {
      old_password: '',
      new_password: '',
      new_password_repeat: '',
    },
  });

  const onSubmit = (data) => {
    trigger(data)
      .then(() => {
        enqueueSnackbar("Your password has been updated", { variant: 'success' });
        methods.reset();
      })
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  return (
    <FormProvider {...methods}>
      <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap>
        <Typography variant="h5">Password</Typography>

        <ControlledTextField name="old_password" label="Current Password" type="password" fullWidth required />
        <ControlledTextField name="new_password" label="New Password" type="password" fullWidth required helperText="Must be at least 8 characters. Make sure your password is strong." />
        <ControlledTextField name="new_password_repeat" label="Retype New Password" type="password" fullWidth required />

        <Stack direction="row" spacing={2} alignItems="center">

          <LoadingButton
            type="submit"
            variant="contained"
            loading={isMutating}
            loadingPosition="start"
            sx={{ width: '120px' }}
            startIcon={<SaveIcon />}
          >
            Save
          </LoadingButton>

          <FormServerError />

        </Stack>
      </Stack>
    </FormProvider>
  );
};

//PasswordChangeSchema
const PasswordChangeSchema = yup.object().shape({
  old_password: yup.string().required('Your current password is required.'),
  new_password: yup.string().required('Please provide a new password.').min(8, 'Must be at least 8 characters.'),
  new_password_repeat: yup.string().required('Please repeat the new password.').oneOf([yup.ref('new_password'), null], 'Passwords must match'),
});

export default PasswordChange;