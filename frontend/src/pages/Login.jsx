import * as yup from "yup";
import useSWRMutation from 'swr/mutation';
import { yupResolver } from '@hookform/resolvers/yup';
import { LoadingButton } from "@mui/lab";
import { Stack, Typography, Link } from "@mui/material";
import { FormContainer, TextFieldElement, PasswordElement } from 'react-hook-form-mui';
import ChevronRightIcon from '@mui/icons-material/ChevronRight';

import Logo from "components/Logo";

const Login = () => {
  const { trigger } = useSWRMutation('/api/auth/login');

  return (
    <FormContainer onSuccess={(data) => trigger(data)} resolver={yupResolver(LoginSchema)}>
      <Stack alignItems="center" spacing={2} useFlexGap>
        <Logo sx={{ width: '100px', mb: 2 }} />
        <Typography variant="h5" align="center" sx={{ fontWeight: 'bold', mb: 1 }}>Login to your account</Typography>

        <TextFieldElement name="email" label="Email" placeholder="user@example.com" fullWidth helperText=" " type="email" />

        <PasswordElement name="password" label="Password" fullWidth helperText=" " />

        <LoadingButton
          type="submit"
          variant="contained"
          loading={false}
          loadingPosition="end"
          endIcon={<ChevronRightIcon />}
          fullWidth
        >
          Login
        </LoadingButton>

        <Typography align="center" sx={{ my: 1 }}>
          Don&lsquo;t have an account?
          {' '}
          <Link href="#">Register</Link>
        </Typography>

        <Link href="#">Forgot your password?</Link>
      </Stack>
    </FormContainer>
  );
};

const LoginSchema = yup.object({
  email: yup.string().required("Email is required").email("Please enter a valid email address"),
  password: yup.string().required(),
}).required();

export default Login;