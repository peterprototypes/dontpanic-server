import React from "react";
import useSWRMutation from "swr/mutation";
import * as yup from "yup";
import { yupResolver } from '@hookform/resolvers/yup';
import { useForm, FormProvider } from "react-hook-form";
import { useConfirm } from "material-ui-confirm";
import { useSnackbar } from 'notistack';
import { FormControl, FormLabel, InputAdornment, Button, OutlinedInput, FormHelperText, Dialog, DialogTitle, DialogContent, DialogContentText, DialogActions, Link } from "@mui/material";
import { LoadingButton } from "@mui/lab";

import { useUser } from 'context/user';
import { useConfig } from "context/config";
import { ControlledTextField, FormServerError } from "./form";
import { SendEmailIcon } from "./ConsistentIcons";

const ChangeEmailSchema = yup.object({
  new_email: yup.string().required("Email is required").email("Please enter a valid email address"),
}).required();

const RequestEmailChange = () => {
  const { config } = useConfig();
  const confirm = useConfirm();
  const { enqueueSnackbar } = useSnackbar();
  const [open, setOpen] = React.useState(false);
  const { user } = useUser();
  const { trigger, error, isMutating } = useSWRMutation('/api/account/update-email');

  const methods = useForm({
    resolver: yupResolver(ChangeEmailSchema),
    errors: error?.fields,
    defaultValues: {
      new_email: "",
    }
  });

  const onSubmit = (data) => {
    trigger(data).then(() => {
      setOpen(false);
      enqueueSnackbar("Confirmation email sent", { variant: 'success' });
      methods.reset();
    }).catch((e) => {
      methods.setError('root.serverError', { message: e.message });
    });
  };

  const handleClickOpen = () => {
    if (!config?.can_send_emails) {
      confirm({
        title: 'Emails are disabled',
        description: <>
          Email sending is not configured.
          Changing emails requires a confirmation email to be sent.
          Please set <strong>EMAIL_URL</strong> environment variable
          (<Link href="https://github.com/peterprototypes/dontpanic-server/tree/main?tab=readme-ov-file#environment-variables" target="_blank">README</Link>).
        </>,
        confirmationText: 'Close',
        hideCancelButton: true
      });
    } else {
      setOpen(true);
    }
  };

  const handleClose = () => {
    setOpen(false);
  };

  return (
    <FormProvider {...methods}>
      <FormControl fullWidth>
        <FormLabel required sx={{ mb: 1 }}>Email</FormLabel>
        <OutlinedInput
          disabled
          value={user.email}
          endAdornment={(
            <InputAdornment position="end">
              <Button variant="text" color="primary" onClick={handleClickOpen}>Change Email</Button>
            </InputAdornment>
          )}
        />
        <FormHelperText>
          Used for logging in and notifications.
        </FormHelperText>
      </FormControl>

      <Dialog open={open} onClose={handleClose} component="form" noValidate onSubmit={methods.handleSubmit(onSubmit)}>
        <DialogTitle>Change Email</DialogTitle>
        <DialogContent>
          <DialogContentText sx={{ mb: 2 }}>
            Please enter your new email address below. You will receive a confirmation email at the new address.
            To complete the email change process, click the link in the confirmation email.
            For security reasons, you will be required to log in again after confirming your new email address.
          </DialogContentText>

          <ControlledTextField
            required
            autoFocus
            fullWidth
            name="new_email"
            label="New Email"
            helperText="Please enter your new email address."
          />

          <FormServerError />
        </DialogContent>
        <DialogActions sx={{ justifyContent: 'space-between' }}>
          <Button onClick={handleClose} color="inherit">Cancel</Button>
          <LoadingButton
            type="submit"
            loading={isMutating}
            loadingPosition="end"
            endIcon={<SendEmailIcon />}
          >
            Send Confirmation
          </LoadingButton>
        </DialogActions>
      </Dialog>
    </FormProvider>
  );
};

export default RequestEmailChange;