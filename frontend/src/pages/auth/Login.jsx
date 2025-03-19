import React from 'react';
import * as yup from "yup";
import useSWRMutation from 'swr/mutation';
import { useNavigate, Link as RouterLink } from "react-router";
import { yupResolver } from '@hookform/resolvers/yup';
import { useForm, FormProvider } from "react-hook-form";
import { LoadingButton } from "@mui/lab";
import { Stack, Typography, Link } from "@mui/material";
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

import { useConfig } from "context/config";
import Logo from "components/Logo";
import ResendVerificationEmail from 'components/ResendVerificationEmail';
import { FormServerError, ControlledTextField } from "components/form";

const LoginSchema = yup.object({
  email: yup.string().required("Email is required").email("Please enter a valid email address"),
  password: yup.string().required("Password is required").min(8, "Password must be at least 8 characters long"),
}).required();

const Login = () => {
  const navigate = useNavigate();
  const { config } = useConfig();

  const [totpRequired, setTotpRequired] = React.useState(false);

  const [showResendVerification, setShowResendVerification] = React.useState(false);

  const { trigger, error, isMutating } = useSWRMutation('/api/auth/login');

  const methods = useForm({
    resolver: yupResolver(LoginSchema),
    errors: error?.fields,
    defaultValues: {
      email: "",
      password: "",
      totp: "",
    },
  });

  const onSubmit = React.useCallback((data) => {
    setShowResendVerification(false);

    trigger(data)
      .then((response) => {
        // first time users should go to org project to create a new project
        if (!response?.has_projects && response?.org_id) {
          navigate(`/organization/${response?.org_id}/projects`);
        } else {
          navigate("/reports");
        }
      })
      .catch((e) => {
        if (e?.user?.type === 'totp_required') {
          setTotpRequired(true);
          return;
        }

        methods.setError('root.serverError', { message: e.message });
        setShowResendVerification(e?.user?.type === 'email_unverified');
      });
  }, [trigger, navigate, methods]);

  const totp = methods.watch("totp");

  React.useEffect(() => {
    if (totpRequired && totp.length === 6) {
      methods.handleSubmit(onSubmit)();
    }
  }, [totp, methods, totpRequired, onSubmit]);

  return (
    <FormProvider {...methods}>
      <Stack component="form" onSubmit={methods.handleSubmit(onSubmit)} noValidate alignItems="center" spacing={2} useFlexGap>
        <Logo sx={{ width: '100px', mb: 2 }} />
        <Typography variant="h5" align="center" sx={{ fontWeight: 'bold', mb: 1 }}>Login to your account</Typography>

        <ControlledTextField name="email" type="email" label="Email" placeholder="user@example.com" fullWidth />

        <ControlledTextField name="password" type="password" label="Password" placeholder="••••••" fullWidth />

        {totpRequired && (
          <ControlledTextField
            name="totp"
            label="Two-factor authentication code"
            placeholder="123456"
            fullWidth
            helperText="Please enter the 6-digit code from your authenticator app"
          />
        )}

        <LoadingButton
          type="submit"
          variant="contained"
          loading={isMutating}
          loadingPosition="end"
          endIcon={<ChevronRightIcon />}
          fullWidth
        >
          Login
        </LoadingButton>

        <FormServerError />

        {showResendVerification && <ResendVerificationEmail email={methods.watch("email")} initialWait={10} variant="text" />}

        {config?.registration_enabled && (
          <Typography align="center" sx={{ my: 1 }}>
            Don&lsquo;t have an account?
            {' '}
            <Link component={RouterLink} to="/auth/register">Register</Link>
          </Typography>
        )}

        <Link component={RouterLink} to="/auth/request-password-reset">Forgot your password?</Link>
      </Stack>
    </FormProvider>
  );
};

export default Login;