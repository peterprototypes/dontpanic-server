import React from "react";
import * as yup from "yup";
import { useSWRConfig } from "swr";
import useSWRMutation from "swr/mutation";
import { useSnackbar } from "notistack";
import { FormProvider, useForm } from "react-hook-form";
import { yupResolver } from '@hookform/resolvers/yup';
import { Paper, SvgIcon, Typography, Stack, Box, Button, Dialog, DialogTitle, DialogContent, IconButton, DialogActions, Menu, MenuItem, Divider, ListItemText, ListItemIcon, DialogContentText, CircularProgress } from "@mui/material";

import RadioButtonCheckedIcon from '@mui/icons-material/RadioButtonChecked';
import RadioButtonUncheckedIcon from '@mui/icons-material/RadioButtonUnchecked';

import MoreVertIcon from '@mui/icons-material/MoreVert';

import WebhookIcon from "./assets/webhooks.svg?react";

import { ControlledTextField } from "components/form";
import { SaveIcon, DeleteIcon, TestIcon } from "components/ConsistentIcons";
import { LoadingButton } from "@mui/lab";

const Webhook = ({ project }) => {
  const { enqueueSnackbar } = useSnackbar();

  const [menuOpen, setMenuOpen] = React.useState(null);
  const [dialogOpen, setDialogOpen] = React.useState(false);

  const { mutate } = useSWRConfig();

  const { trigger: triggerRemove, isMutating: isRemoving } = useSWRMutation(`/api/notifications/${project.project_id}/webhook/delete`);
  const { trigger: triggerTest, isMutating: isTesting } = useSWRMutation(`/api/notifications/${project.project_id}/webhook/test`);

  const { trigger, error, isMutating } = useSWRMutation(`/api/notifications/${project.project_id}/webhook/save`);

  const methods = useForm({
    resolver: yupResolver(WebhookSchema),
    errors: error?.fields,
    values: {
      webhook_url: project?.webhook || "",
    },
  });

  const isConfigured = Boolean(project.webhook);

  const onSubmit = (data) => {
    setMenuOpen(null);

    trigger(data)
      .then(() => {
        mutate(`/api/notifications/project/${project.project_id}`);
        enqueueSnackbar("Webhook configured", { variant: 'success' });
        setDialogOpen(false);
      })
      .catch((e) => {
        methods.setError('root.serverError', { message: e.message });
      });
  };

  const onRemove = () => {
    triggerRemove({})
      .then(() => {
        mutate(`/api/notifications/project/${project.project_id}`);
        enqueueSnackbar("Webhook successfully removed", { variant: 'success' });
      });
  };

  const onTest = () => {
    setMenuOpen(null);

    triggerTest({})
      .then(() => {
        enqueueSnackbar("Event sent", { variant: 'success' });
      });
  };

  const loading = isMutating || isRemoving || isTesting;

  return (
    <FormProvider {...methods}>
      <Paper sx={{ p: 2 }}>
        <Stack direction="row" spacing={2} alignItems="center">

          {!loading && (isConfigured ? <RadioButtonCheckedIcon color="success" /> : <RadioButtonUncheckedIcon />)}

          {loading && <CircularProgress size="20px" />}

          <SvgIcon sx={{ fontSize: 40, filter: isConfigured ? '' : 'grayscale() opacity(0.5)' }} component={WebhookIcon} inheritViewBox />

          <Box>
            <Typography variant="h6">Webhook</Typography>
            <Typography color="textSecondary">
              Configure a webhook to receive real-time notifications directly in your system.
              Ideal for custom automation and advanced workflows.
            </Typography>
          </Box>

          {isConfigured ? (
            <>
              <IconButton onClick={(e) => setMenuOpen(e.currentTarget)}>
                <MoreVertIcon />
              </IconButton>
              <Menu
                anchorEl={menuOpen}
                open={Boolean(menuOpen)}
                onClose={() => setMenuOpen(null)}
              >
                <MenuItem onClick={() => setMenuOpen(null) || setDialogOpen(true)}>
                  <ListItemIcon><SaveIcon /></ListItemIcon>
                  <ListItemText>Edit</ListItemText>
                </MenuItem>
                <MenuItem onClick={onTest}>
                  <ListItemIcon><TestIcon /></ListItemIcon>
                  <ListItemText>Send Test Event</ListItemText>
                </MenuItem>
                <Divider />
                <MenuItem onClick={onRemove}>
                  <ListItemIcon><DeleteIcon /></ListItemIcon>
                  <ListItemText>Remove</ListItemText>
                </MenuItem>
              </Menu>
            </>
          ) : (
            <Button variant="outlined" color="primary" sx={{ minWidth: '100px' }} onClick={() => setDialogOpen(true)}>Configure</Button>
          )}
        </Stack>
      </Paper>

      <Dialog open={dialogOpen} onClose={() => setDialogOpen(false)} component="form" noValidate onSubmit={methods.handleSubmit(onSubmit)}>
        <DialogTitle>Webhook Setup</DialogTitle>
        <DialogContent>
          <ControlledTextField required name="webhook_url" label="Webhook Url" fullWidth />
          <DialogContentText sx={{ mt: 2, minWidth: '400px' }}>
            This URL will receive a <strong>POST request</strong> whenever a new report is generated.
            Ensure your endpoint is configured to handle incoming requests.
          </DialogContentText>
        </DialogContent>
        <DialogActions sx={{ justifyContent: 'space-between' }}>
          <Button onClick={() => setDialogOpen(false)} color="inherit">Cancel</Button>
          <LoadingButton
            type="submit"
            loading={isMutating}
            loadingPosition="end"
            endIcon={<SaveIcon />}
          >
            Save Webhook
          </LoadingButton>
        </DialogActions>
      </Dialog>
    </FormProvider >
  );
};

const WebhookSchema = yup.object({
  webhook_url: yup.string().url("Must be a valid URL").required("Webhook URL is required"),
}).required();

export default Webhook;