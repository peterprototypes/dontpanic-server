import * as yup from "yup";
import useSWRMutation from 'swr/mutation';
import { yupResolver } from '@hookform/resolvers/yup';
import { useForm, FormProvider } from "react-hook-form";
import { LoadingButton } from "@mui/lab";
import { Stack, Typography, Link } from "@mui/material";
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

import Logo from "components/Logo";
import { FormServerError, ControlledTextField } from "components/form";

const LoginSchema = yup.object({
  email: yup.string().required("Email is required").email("Please enter a valid email address"),
  password: yup.string().required("Password is required").min(8, "Password must be at least 8 characters long"),
}).required();

const Login = () => {
  const { trigger, error, isMutating } = useSWRMutation('/api/auth/login');

  const methods = useForm({
    resolver: yupResolver(LoginSchema),
    errors: error?.fields,
    defaultValues: {
      email: "",
      password: "",
    },
  });

  const onSubmit = (data) => {
    trigger(data).catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  return (
    <FormProvider {...methods}>
      <Stack component="form" onSubmit={methods.handleSubmit(onSubmit)} noValidate alignItems="center" spacing={2} useFlexGap>
        <Logo sx={{ width: '100px', mb: 2 }} />
        <Typography variant="h5" align="center" sx={{ fontWeight: 'bold', mb: 1 }}>Login to your account</Typography>

        <ControlledTextField name="email" type="email" label="Email" placeholder="user@example.com" fullWidth />

        <ControlledTextField name="password" type="password" label="Password" placeholder="••••••" fullWidth />

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

        <Typography align="center" sx={{ my: 1 }}>
          Don&lsquo;t have an account?
          {' '}
          <Link href="#">Register</Link>
        </Typography>

        <Link href="#">Forgot your password?</Link>
      </Stack>
    </FormProvider>
  );
};

export default Login;