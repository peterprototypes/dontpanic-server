import { Link, Stack, Typography } from '@mui/material';
import { LoadingButton } from '@mui/lab';
import useSWRMutation from 'swr/mutation';
import { ControlledTextField, FormServerError } from 'components/form';
import { useForm, FormProvider, useFormContext } from 'react-hook-form';
import { useConfirm } from "material-ui-confirm";
import { yupResolver } from '@hookform/resolvers/yup';
import * as yup from 'yup';
import { useSnackbar } from 'notistack';

import { SaveIcon } from 'components/ConsistentIcons';
import { useUser } from 'context/user';

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
      <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap alignItems="flex-start">
        <Typography variant="h5">Password</Typography>

        <ControlledTextField
          name="old_password"
          label="Current Password"
          type="password"
          fullWidth
          required
          helperText="You must provide your current password in order to change it."
        />

        <ControlledTextField
          name="new_password"
          label="New Password"
          type="password"
          fullWidth
          required
          helperText="Must be at least 8 characters. Make sure your password is strong."
        />

        <ControlledTextField name="new_password_repeat" label="Retype New Password" type="password" fullWidth required />

        <Stack direction="row" alignItems="center" spacing={2}>
          <LoadingButton
            type="submit"
            variant="contained"
            loading={isMutating}
            loadingPosition="start"
            startIcon={<SaveIcon />}
          >
            Save Password
          </LoadingButton>

          <ForgotPasswordLink />
        </Stack>

        <FormServerError sx={{ width: '100%' }} />

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

const ForgotPasswordLink = () => {
  const { user } = useUser();
  const confirm = useConfirm();
  const methods = useFormContext();
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, isMutating } = useSWRMutation('/api/auth/request-password-reset');

  const onForgotPassword = () => {
    let config = {
      title: 'Forgotten Password',
      description: (
        <Typography>
          {'We will send an email to '}
          <Typography fontWeight="bold" display="inline">{user.email}</Typography>
          {' with instructions on how to reset your password.'}
        </Typography>
      ),
      confirmationText: 'Send Instructions'
    };

    confirm(config)
      .then(() => trigger({})
        .then(() => enqueueSnackbar(`Email sent to ${user.email}`, { variant: 'success' }))
        .catch((e) => methods.setError('root.serverError', { message: e.message })))
      .catch(() => { });
  };

  return (
    <Link component="button" type="button" onClick={onForgotPassword} disabled={isMutating}>
      {isMutating ? 'Sending email...' : 'I forgot my password'}
    </Link>
  );
};



export default PasswordChange;