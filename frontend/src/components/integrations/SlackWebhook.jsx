import React from "react";
import * as yup from "yup";
import { useSWRConfig } from "swr";
import useSWRMutation from "swr/mutation";
import { useSnackbar } from "notistack";
import { FormProvider, useForm } from "react-hook-form";
import { yupResolver } from '@hookform/resolvers/yup';
import { Paper, SvgIcon, Typography, Stack, Box, Button, Dialog, DialogTitle, DialogContent, IconButton, DialogActions, Menu, MenuItem, Divider, ListItemText, ListItemIcon, DialogContentText, CircularProgress, Link } from "@mui/material";

import RadioButtonCheckedIcon from '@mui/icons-material/RadioButtonChecked';
import RadioButtonUncheckedIcon from '@mui/icons-material/RadioButtonUnchecked';

import MoreVertIcon from '@mui/icons-material/MoreVert';

import SlackIcon from "./assets/Slack.svg?react";

import { ControlledTextField } from "components/form";
import { SaveIcon, DeleteIcon, TestIcon } from "components/ConsistentIcons";
import { LoadingButton } from "@mui/lab";

const SlackWebhook = ({ project }) => {
  const { enqueueSnackbar } = useSnackbar();

  const [menuOpen, setMenuOpen] = React.useState(null);
  const [dialogOpen, setDialogOpen] = React.useState(false);

  const { mutate } = useSWRConfig();

  const { trigger: triggerRemove, isMutating: isRemoving } = useSWRMutation(`/api/notifications/${project.project_id}/slack-webhook/delete`);
  const { trigger: triggerTest, isMutating: isTesting } = useSWRMutation(`/api/notifications/${project.project_id}/slack-webhook/test`);

  const { trigger, error, isMutating } = useSWRMutation(`/api/notifications/${project.project_id}/slack-webhook/save`);

  const methods = useForm({
    resolver: yupResolver(WebhookSchema),
    errors: error?.fields,
    values: {
      webhook_url: project?.slack_webhook || "",
    },
  });

  const isConfigured = Boolean(project.slack_webhook);

  const onSubmit = (data) => {
    setMenuOpen(null);

    trigger(data)
      .then(() => {
        mutate(`/api/notifications/project/${project.project_id}`);
        enqueueSnackbar("Slack webhook configured", { variant: 'success' });
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
        enqueueSnackbar("Slack webhook removed", { variant: 'success' });
      });
  };

  const onTest = () => {
    setMenuOpen(null);

    triggerTest({})
      .then(() => {
        enqueueSnackbar("Message sent", { variant: 'success' });
      });
  };

  const loading = isMutating || isRemoving || isTesting;

  return (
    <FormProvider {...methods}>
      <Paper sx={{ p: 2 }}>
        <Stack direction="row" spacing={2} alignItems="center">

          {!loading && (isConfigured ? <RadioButtonCheckedIcon color="success" /> : <RadioButtonUncheckedIcon />)}

          {loading && <CircularProgress size="20px" />}

          <SvgIcon sx={{ fontSize: 40, filter: isConfigured ? '' : 'grayscale() opacity(0.5)' }} component={SlackIcon} inheritViewBox />

          <Box>
            <Typography variant="h6">Slack Webhook</Typography>
            <Typography color="textSecondary">Slack webhook is the legacy, although still supported, method for sending messages to a Slack Channel.</Typography>
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
                  <ListItemText>Test</ListItemText>
                </MenuItem>
                <Divider />
                <MenuItem onClick={onRemove}>
                  <ListItemIcon><DeleteIcon /></ListItemIcon>
                  <ListItemText>Remove</ListItemText>
                </MenuItem>
              </Menu>
            </>
          ) : (
            <Button variant="outlined" color="primary" onClick={() => setDialogOpen(true)}>Configure</Button>
          )}
        </Stack>
      </Paper>

      <Dialog open={dialogOpen} onClose={() => setDialogOpen(false)} component="form" noValidate onSubmit={methods.handleSubmit(onSubmit)}>
        <DialogTitle>Configure Webhook</DialogTitle>
        <DialogContent>
          <ControlledTextField required name="webhook_url" label="Webhook Url" fullWidth />
          <DialogContentText sx={{ mt: 2, minWidth: '400px' }}>
            You can get Slack Incoming Webhook URL in Slack&lsquo;s <Link href="https://api.slack.com/messaging/webhooks#getting-started">Apps {">"} Incoming WebHooks</Link>.
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
            Save
          </LoadingButton>
        </DialogActions>
      </Dialog>
    </FormProvider >
  );
};

const WebhookSchema = yup.object({
  webhook_url: yup.string().url("Must be a valid URL").required("Webhook URL is required"),
}).required();

export default SlackWebhook;