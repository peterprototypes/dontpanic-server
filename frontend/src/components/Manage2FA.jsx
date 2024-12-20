import React from 'react';
import { useSWRConfig } from "swr";
import useSWRMutation from 'swr/mutation';
import * as yup from "yup";
import { yupResolver } from '@hookform/resolvers/yup';
import { useForm, FormProvider } from "react-hook-form";
import { useSnackbar } from 'notistack';
import { useConfirm } from "material-ui-confirm";
import { Alert, Stack, Typography, Button, Dialog, DialogTitle, DialogContent, DialogActions, TextField, Stepper, Step, StepLabel, StepContent, Link } from '@mui/material';
import { LoadingButton } from '@mui/lab';
import QRCode from "react-qr-code";

import { useUser } from 'context/user';
import { SaveIcon } from 'components/ConsistentIcons';
import { ControlledTextField, FormServerError } from "components/form";

const Manage2FA = () => {
  const { user } = useUser();
  const confirm = useConfirm();
  const { enqueueSnackbar } = useSnackbar();

  const { mutate } = useSWRConfig();
  const { trigger: disableTotp } = useSWRMutation("/api/account/totp/disable");

  const onDisable2FA = () => {
    let config = {
      title: 'Are you sure?',
      description: 'You\'re about to disable two-factor authentication.',
      confirmationText: 'Disable 2FA'
    };

    confirm(config)
      .then(() => disableTotp({})
        .then(() => {
          enqueueSnackbar("Two-factor authentication disabled", { variant: 'success' });
          mutate("/api/account");
        })
        .catch((e) => enqueueSnackbar(e.message, { variant: 'error' }))
      )
      .catch(() => { });
  };

  return (
    <Stack spacing={2} sx={{ mt: 2 }} useFlexGap>
      <Typography variant="h5">Two-factor authentication</Typography>
      <Typography variant="body1" color="textSecondary">
        Two-factor authentication adds an extra layer of security to your account. Once enabled, you will need to provide a code from your authenticator app in addition to your password when logging in.
      </Typography>

      {user.totp_enabled ? (
        <Alert
          severity="success"
          action={
            <Button color="error" size="small" variant="outlined" onClick={onDisable2FA}>
              Disable 2FA
            </Button>
          }
        >
          <strong>Status:</strong> Enabled
        </Alert>
      ) : (
        <Alert
          severity="inherit"
          action={
            <Enable2FA />
          }
        >
          <strong>Status:</strong> Disabled
        </Alert>
      )}
    </Stack>
  );
};

const Enable2FA = () => {
  const { enqueueSnackbar } = useSnackbar();

  const [secret, setSecret] = React.useState(null);

  const { mutate } = useSWRConfig();
  const { trigger: loadSecret, isMutating: isLoadingSecret } = useSWRMutation("/api/account/totp/secret");
  const { trigger: enableTotp, error, isMutating } = useSWRMutation("/api/account/totp/enable");

  const methods = useForm({
    resolver: yupResolver(EnableTotpSchema),
    errors: error?.fields,
    defaultValues: {
      code: "",
    }
  });

  const onOpenDialog = () => {
    loadSecret().then((res) => {
      setSecret(res);
    }).catch((e) => {
      enqueueSnackbar(e.message, { variant: 'error' });
    });
  };

  const onSubmit = (data) => {
    enableTotp({ ...secret, ...data }).then(() => {
      setSecret(null);
      enqueueSnackbar("Two-factor authentication enabled", { variant: 'success' });
      mutate("/api/account");
    }).catch((e) => {
      methods.setError('root.serverError', { message: e.message });
    });
  };

  return (
    <FormProvider {...methods}>
      <LoadingButton color="success" size="small" variant="contained" onClick={onOpenDialog} loading={isLoadingSecret}>
        Enable 2FA
      </LoadingButton>

      <Dialog open={secret !== null} onClose={() => setSecret(null)} component="form" noValidate onSubmit={methods.handleSubmit(onSubmit)}>
        <DialogTitle>Register Two-Factor Authenticator</DialogTitle>
        <DialogContent>

          <Stepper activeStep={-1} orientation="vertical">

            <Step expanded={true}>
              <StepLabel>Install Google Authenticator or a compatible app on your mobile device</StepLabel>
              <StepContent>
                <Link href="https://play.google.com/store/apps/details?id=com.google.android.apps.authenticator2&hl=en" target="_blank" rel="noopener noreferrer">Google Authenticator for Android</Link>
                <br />
                <Link href="https://apps.apple.com/us/app/google-authenticator/id388497605" target="_blank" rel="noopener noreferrer">Google Authenticator for iOS</Link>
              </StepContent>
            </Step>

            <Step expanded={true}>
              <StepLabel>
                Use your virtual MFA app and your device&lsquo;s camera to scan the QR code
              </StepLabel>
              <StepContent>
                {secret?.url && <QRCode value={secret?.url} />}

                <Typography my={2}>Alternatively, you can type the secret key:</Typography>

                <TextField value={secret?.secret || ""} size="small" fullWidth disabled />
              </StepContent>
            </Step>

            <Step expanded={true}>
              <StepLabel>Type one MFA code from your authenticator app.</StepLabel>
              <StepContent>
                <ControlledTextField
                  autoFocus
                  fullWidth
                  name="code"
                  placeholder="123456"
                />

                <FormServerError sx={{ mt: 1 }} />
              </StepContent>
            </Step>
          </Stepper>
        </DialogContent>
        <DialogActions sx={{ justifyContent: 'space-between' }}>
          <Button onClick={() => setSecret(null)} color="inherit">Cancel</Button>
          <LoadingButton
            type="submit"
            loading={isMutating}
            loadingPosition="end"
            endIcon={<SaveIcon />}
          >
            Save
          </LoadingButton>
        </DialogActions>
      </Dialog>
    </FormProvider>
  );
};

const EnableTotpSchema = yup.object({
  code: yup.string().required(),
}).required();

export default Manage2FA;