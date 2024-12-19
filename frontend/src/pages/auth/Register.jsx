import React from 'react';
import * as yup from "yup";
import useSWRMutation from 'swr/mutation';
import { Link as RouterLink, useSearchParams } from "react-router";
import { yupResolver } from '@hookform/resolvers/yup';
import { useForm, FormProvider } from "react-hook-form";
import { LoadingButton } from "@mui/lab";
import { Stack, Typography, Link } from "@mui/material";
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

import Logo from "components/Logo";
import ResendVerificationEmail from 'components/ResendVerificationEmail';
import { FormServerError, ControlledTextField } from "components/form";

const RegisterSchema = yup.object({
  email: yup.string().required("Email is required").email("Please enter a valid email address"),
  password: yup.string().required("Password is required").min(8, "Password must be at least 8 characters long"),
}).required();

const Register = () => {
  const [searchParams] = useSearchParams();
  const [success, setSuccess] = React.useState("");

  const { trigger, error, isMutating } = useSWRMutation('/api/auth/register');

  const methods = useForm({
    resolver: yupResolver(RegisterSchema),
    errors: error?.fields,
    defaultValues: {
      email: "",
      password: "",
      name: "",
      company: "",
      iana_timezone_name: Intl.DateTimeFormat().resolvedOptions().timeZone,
    },
  });

  const onSubmit = (data) => {
    trigger({ ...data, invite_slug: searchParams.get("slug") })
      .then((response) => setSuccess(response.require_email_verification ? "email_verification" : "login"))
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  if (success === "email_verification") {
    return (
      <Stack alignItems="center" spacing={2}>
        <Logo sx={{ width: '100px', mb: 2 }} />

        <Typography variant="h6" align="center">A link to activate your account has been emailed to the address provided.</Typography>
        <Typography align="center">Please, give it a few minutes and check your spam and junk folder.</Typography>

        <ResendVerificationEmail email={methods.watch("email")} />
      </Stack>
    );
  }

  if (success === "login") {
    return (
      <Stack alignItems="center" spacing={2}>
        <Logo sx={{ width: '100px', mb: 2 }} />

        <Typography variant="h6" align="center">
          Your account is created. You can now login and start tracking errors and panics.
        </Typography>

        <Link component={RouterLink} to="/auth/login">Go to Login</Link>
      </Stack>
    );
  }

  return (
    <FormProvider {...methods}>
      <Stack component="form" onSubmit={methods.handleSubmit(onSubmit)} noValidate alignItems="center" spacing={2} useFlexGap>
        <Logo sx={{ width: '100px', mb: 2 }} />
        <Typography variant="h5" align="center" sx={{ fontWeight: 'bold', mb: 1 }}>Don&lsquo;t Panic Account</Typography>

        <ControlledTextField name="email" type="email" label="Email" placeholder="user@example.com" fullWidth required />

        <ControlledTextField name="password" type="password" label="Password" placeholder="••••••" fullWidth required />

        <ControlledTextField name="name" type="text" label="Name" placeholder="John Doe" fullWidth />

        <ControlledTextField name="company" type="text" label="Organization" placeholder="Company Inc." fullWidth />

        <LoadingButton
          type="submit"
          variant="contained"
          loading={isMutating}
          loadingPosition="end"
          endIcon={<ChevronRightIcon />}
          fullWidth
        >
          Create Account
        </LoadingButton>

        <FormServerError />

        <Typography align="center" sx={{ my: 1 }}>
          Already have an account?
          {' '}
          <Link component={RouterLink} to="/auth/login">Login</Link>
        </Typography>
      </Stack>
    </FormProvider>
  );
};

export default Register;