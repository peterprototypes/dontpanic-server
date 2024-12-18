import React from 'react';
import * as yup from "yup";
import useSWRMutation from 'swr/mutation';
import { yupResolver } from '@hookform/resolvers/yup';
import { useForm, FormProvider } from "react-hook-form";
import { LoadingButton } from "@mui/lab";
import { Stack, Typography } from "@mui/material";
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

import Logo from "components/Logo";
import { FormServerError, ControlledTextField } from "components/form";

const PasswordResetRequestSchema = yup.object({
  email: yup.string().required("Email is required").email("Please enter a valid email address"),
}).required();

const PasswordResetRequest = () => {
  const { trigger, error, isMutating } = useSWRMutation('/api/auth/request-password-reset');

  const [requestSubmitted, setRequestSubmitted] = React.useState(false);

  const methods = useForm({
    resolver: yupResolver(PasswordResetRequestSchema),
    errors: error?.fields,
    defaultValues: {
      email: "",
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
        <Typography variant="h6" align="center">A link to recover your password has been emailed to the address provided.</Typography>
        <Typography align="center">Please, give it a few minutes and check your spam and junk folder.</Typography>
      </Stack>
    );
  }

  return (
    <FormProvider {...methods}>
      <Stack component="form" onSubmit={methods.handleSubmit(onSubmit)} noValidate alignItems="center" spacing={2} useFlexGap>
        <Logo sx={{ width: '100px', mb: 2 }} />
        <Typography variant="h5" align="center" sx={{ fontWeight: 'bold', mb: 1 }}>Recover your password</Typography>

        <ControlledTextField name="email" type="email" label="Email" placeholder="user@example.com" fullWidth />

        <LoadingButton
          type="submit"
          variant="contained"
          loading={isMutating}
          loadingPosition="end"
          endIcon={<ChevronRightIcon />}
          fullWidth
        >
          Recover Password
        </LoadingButton>

        <FormServerError />

        <Typography align="center" sx={{ my: 1 }}>
          Enter your the email address you registered with, and we&lsquo;ll contact you with further instructions.
        </Typography>
      </Stack>
    </FormProvider>
  );
};

export default PasswordResetRequest;