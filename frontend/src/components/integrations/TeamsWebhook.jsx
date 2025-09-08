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

import TeamsIcon from "./assets/Teams.svg?react";

import { ControlledTextField } from "components/form";
import { SaveIcon, DeleteIcon, TestIcon } from "components/ConsistentIcons";
import { LoadingButton } from "@mui/lab";

const TeamsWebhook = ({ project }) => {
  const { enqueueSnackbar } = useSnackbar();

  const [menuOpen, setMenuOpen] = React.useState(null);
  const [dialogOpen, setDialogOpen] = React.useState(false);

  const { mutate } = useSWRConfig();

  const { trigger: triggerRemove, isMutating: isRemoving } = useSWRMutation(`/api/notifications/${project.project_id}/teams-webhook/delete`);
  const { trigger: triggerTest, isMutating: isTesting } = useSWRMutation(`/api/notifications/${project.project_id}/teams-webhook/test`);

  const { trigger, error, isMutating } = useSWRMutation(`/api/notifications/${project.project_id}/teams-webhook/save`);

  const methods = useForm({
    resolver: yupResolver(WebhookSchema),
    errors: error?.fields,
    values: {
      webhook_url: project?.teams_webhook || "",
      environments: project?.environments.map(e => ({
        project_environment_id: e.project_environment_id,
        teams_webhook_config: (e.teams_webhook != '-1' && e.teams_webhook != null) ? 'as_defined' : e.teams_webhook ?? '',
        teams_webhook_override: (e.teams_webhook != '-1' && e.teams_webhook != null) ? e.teams_webhook : ''
      })) ?? [],
    },
  });

  const isConfigured = Boolean(project.teams_webhook);

  const onSubmit = (data) => {
    setMenuOpen(null);

    const req = {
      webhook_url: data.webhook_url,
      environments: data.environments.map(e => ({
        project_environment_id: e.project_environment_id,
        teams_webhook: e.teams_webhook_config == 'as_defined' ? e.teams_webhook_override : (e.teams_webhook_config == '' ? null : e.teams_webhook_config)
      }))
    };

    trigger(req)
      .then(() => {
        mutate(`/api/notifications/project/${project.project_id}`);
        enqueueSnackbar("Teams webhook configured", { variant: 'success' });
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
        enqueueSnackbar("Microsoft Teams webhook successfully removed", { variant: 'success' });
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

          <SvgIcon sx={{ fontSize: 40, filter: isConfigured ? '' : 'grayscale() opacity(0.5)' }} component={TeamsIcon} inheritViewBox />

          <Box sx={{ flexGrow: 1 }}>
            <Typography variant="h6">Microsoft Teams Integration</Typography>
            <Typography color="textSecondary">
              Connect Microsoft Teams to receive real-time alerts and notifications directly in your preferred channels.
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
                  <ListItemText>Send Test Notification</ListItemText>
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
        <DialogTitle>Set Up Microsoft Teams Webhook</DialogTitle>

        <DialogContent>
          <ControlledTextField required name="webhook_url" label="Webhook Url" fullWidth />

          <DialogContentText sx={{ mt: 2, minWidth: '400px' }}>
            To enable webhook notifications, create an <strong>Incoming Webhook</strong> in Microsoft Teams.
            Follow the official guide to generate your webhook URL and paste it here.
            <br />
            <Link href="https://support.microsoft.com/en-us/office/create-incoming-webhooks-with-workflows-for-microsoft-teams-8ae491c7-0394-4861-ba59-055e33f75498" target="_blank">Learn more</Link>
          </DialogContentText>

          <Typography variant="body1" fontWeight="bold" sx={{ my: 2 }}>Environment Overrides</Typography>

          {project?.environments.map((env, index) => (
            <Stack key={env.project_environment_id} gap={1} sx={{ mb: 1 }}>
              <ControlledTextField
                select
                fullWidth
                displayEmpty
                name={`environments.${index}.teams_webhook_config`}
                label={env.name}
                helperText={"Channel url to send reports to for the " + env.name + " environment"}
              >
                <MenuItem value="">Use default webhook Url</MenuItem>
                <MenuItem value="-1">Do not send notifications</MenuItem>
                <MenuItem value="as_defined">Override Webhook Url</MenuItem>
              </ControlledTextField>

              {methods.watch(`environments.${index}.teams_webhook_config`) == 'as_defined' && (
                <ControlledTextField name={`environments.${index}.teams_webhook_override`} label={env.name + ` webhook url`} fullWidth sx={{ ml: 3 }} />
              )}
            </Stack>
          ))}
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
  environments: yup
    .array()
    .of(
      yup.object({
        project_environment_id: yup.number().required(),
        teams_webhook_config: yup
          .mixed()
          .oneOf(["", "-1", "as_defined"], "Invalid selection")
          .required(),
        teams_webhook_override: yup
          .string()
          .when("teams_webhook_config", {
            is: "as_defined",
            then: (schema) =>
              schema
                .trim()
                .url("Must be a valid URL")
                .required("Webhook URL is required when overriding"),
            otherwise: (schema) =>
              schema
                // Normalize empty strings to undefined so tests pass cleanly
                .transform((v) => (v === "" ? undefined : v))
                .test(
                  "empty-when-not-overriding",
                  'Remove the override or choose "Override Webhook Url".',
                  (v) => v == null // must be undefined or null when not overriding
                ),
          }),
      })
    )
}).required();

export default TeamsWebhook;