import React from 'react';
import * as yup from "yup";
import useSWRMutation from 'swr/mutation';
import { yupResolver } from '@hookform/resolvers/yup';
import { useForm, FormProvider } from "react-hook-form";
import { LoadingButton } from "@mui/lab";
import { Stack, Typography, Link } from "@mui/material";
import { Link as RouterLink, useParams } from "react-router";
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

import Logo from "components/Logo";
import { FormServerError, ControlledTextField } from "components/form";

const PasswordResetSchema = yup.object({
  new_password: yup.string().required("Password is required").min(8, "Password must be at least 8 characters long"),
  new_password_repeat: yup.string().oneOf([yup.ref('new_password'), null], 'Passwords must match')
}).required();

const PasswordReset = () => {
  let { hash } = useParams();

  const { trigger, error, isMutating } = useSWRMutation('/api/auth/password-reset/' + hash);

  const [requestSubmitted, setRequestSubmitted] = React.useState(false);

  const methods = useForm({
    resolver: yupResolver(PasswordResetSchema),
    errors: error?.fields,
    defaultValues: {
      new_password: "",
      new_password_repeat: ""
    },
  });

  const onSubmit = (data) => {
    trigger(data)
      .then(() => setRequestSubmitted(true))
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  if (requestSubmitted) {
    return (
      <Stack alignItems="center" spacing={2}>
        <Logo sx={{ width: '100px', mb: 2 }} />
        <Typography variant="h6" align="center">Your password is updated.</Typography>
        <Typography align="center">
          You can now
          {' '}
          <Link component={RouterLink} to="/auth/login">login</Link>
          {' '}
          to your account.
        </Typography>
      </Stack>
    );
  }

  return (
    <FormProvider {...methods}>
      <Stack component="form" onSubmit={methods.handleSubmit(onSubmit)} noValidate alignItems="center" spacing={2} useFlexGap>
        <Logo sx={{ width: '100px', mb: 2 }} />
        <Typography variant="h5" align="center" sx={{ fontWeight: 'bold', mb: 1 }}>Recover your password</Typography>

        <ControlledTextField name="new_password" type="password" label="New Password" placeholder="••••••" fullWidth required />

        <ControlledTextField name="new_password_repeat" type="password" label="Repeat New Password" placeholder="••••••" fullWidth required />

        <LoadingButton
          type="submit"
          variant="contained"
          loading={isMutating}
          loadingPosition="end"
          endIcon={<ChevronRightIcon />}
          fullWidth
        >
          Update Password
        </LoadingButton>

        <FormServerError />
      </Stack>
    </FormProvider>
  );
};

export default PasswordReset;