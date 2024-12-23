import React, { useEffect } from "react";
import * as yup from "yup";
import useSWR, { useSWRConfig } from "swr";
import useSWRMutation from "swr/mutation";
import { useSearchParams } from "react-router";
import { useSnackbar } from "notistack";
import { FormProvider, useForm } from "react-hook-form";
import { yupResolver } from '@hookform/resolvers/yup';
import { Paper, SvgIcon, Typography, Stack, Box, Button, Dialog, DialogTitle, DialogContent, IconButton, DialogActions, Menu, MenuItem, Divider, ListItemText, ListItemIcon, DialogContentText, CircularProgress, Link, LinearProgress, Alert } from "@mui/material";

import RadioButtonCheckedIcon from '@mui/icons-material/RadioButtonChecked';
import RadioButtonUncheckedIcon from '@mui/icons-material/RadioButtonUnchecked';

import MoreVertIcon from '@mui/icons-material/MoreVert';

import SlackIcon from "./assets/Slack.svg?react";

import { ControlledTextField } from "components/form";
import { SaveIcon, DeleteIcon, TestIcon } from "components/ConsistentIcons";
import { LoadingButton } from "@mui/lab";

const SlackApp = ({ project }) => {
  const { enqueueSnackbar } = useSnackbar();

  const [menuOpen, setMenuOpen] = React.useState(null);
  const [dialogOpen, setDialogOpen] = React.useState(false);

  const { mutate } = useSWRConfig();

  const { data: config, isLoading, error: configError, mutate: configMutate } = useSWR(`/api/notifications/${project.project_id}/slack-app/config`);

  const { trigger: triggerRemove, isMutating: isRemoving } = useSWRMutation(`/api/notifications/${project.project_id}/slack-app/delete`);
  const { trigger: triggerTest, isMutating: isTesting } = useSWRMutation(`/api/notifications/${project.project_id}/slack-app/test`);

  const { trigger, error, isMutating } = useSWRMutation(`/api/notifications/${project.project_id}/slack-app/save`);

  const methods = useForm({
    resolver: yupResolver(SlackAppSchema),
    errors: error?.fields,
    values: {
      slack_channel: project?.slack_channel || config?.slack_chats[0]?.id || "",
    },
  });

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
        configMutate();
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

  const loading = isMutating || isRemoving || isTesting || isLoading;

  if (configError) {
    return <Typography color="error">{configError.toString()}</Typography>;
  }

  return (
    <FormProvider {...methods}>
      <Paper sx={{ p: 2 }}>
        <Stack direction="row" spacing={2} alignItems="center">

          {!loading && (project.slack_bot_token ? <RadioButtonCheckedIcon color="success" /> : <RadioButtonUncheckedIcon />)}

          {loading && <CircularProgress size="20px" />}

          <SvgIcon sx={{ fontSize: 40, filter: project.slack_bot_token ? '' : 'grayscale() opacity(0.5)' }} component={SlackIcon} inheritViewBox />

          <Box>
            <Typography variant="h6">Slack App</Typography>
            <Typography color="textSecondary">Slack messages are a great way to promptly inform the entire team of a panic or error.</Typography>
          </Box>

          {!isLoading && !project.slack_bot_token && <AppNotAdded config={config} onAppAdded={() => setDialogOpen(true)} />}

          {project.slack_bot_token && !project.slack_channel && <ChannelNotSet onClick={() => setDialogOpen(true)} />}

          {project.slack_bot_token && project.slack_channel && (
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
          )}
        </Stack>
      </Paper>

      <Dialog open={dialogOpen} onClose={() => setDialogOpen(false)} component="form" noValidate onSubmit={methods.handleSubmit(onSubmit)}>
        <DialogTitle>Set Slack Channel</DialogTitle>
        <DialogContent>
          <ControlledTextField
            select
            required
            fullWidth
            name="slack_channel"
            label="Slack Channel"
          >
            {config?.slack_chats.map((chat) => (
              <MenuItem key={chat.id} value={chat.id}>{chat.name}</MenuItem>
            ))}
          </ControlledTextField>

          <DialogContentText sx={{ mt: 2, minWidth: '400px' }}>
            Select which Slack channel to send messages to
          </DialogContentText>

          {config?.slack_chats.length === 0 && (
            <Alert color="warning" sx={{ mt: 2 }} action={<Button color="inherit" size="small" onClick={() => configMutate()}>Refresh</Button>}>
              No channels found. Please type <pre>{`/invite @Don't Panic`}</pre> in the channel intended to receive notifications.

            </Alert>
          )}
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

const AppNotAdded = ({ config, onAppAdded }) => {
  const { enqueueSnackbar } = useSnackbar();
  const [searchParams, setSearchParams] = useSearchParams();
  const code = searchParams.get('code');

  const { mutate } = useSWRConfig();

  const { trigger, isMutating } = useSWRMutation(`/api/notifications/${config.project_id}/slack-app/config`);

  useEffect(() => {
    if (!code) return;

    // we end up here from Slacks OAuth2 redirect with code parameter

    searchParams.delete('code');
    setSearchParams(searchParams);

    trigger({ code }).then(() => {
      mutate(`/api/notifications/project/${config.project_id}`);
      onAppAdded();
    }).catch((e) => {
      enqueueSnackbar(e.message, { variant: 'error' });
    });

  }, [code, trigger, searchParams, setSearchParams, enqueueSnackbar, onAppAdded, mutate, config.project_id]);

  if (isMutating) {
    return <LinearProgress />;
  }

  return (
    <Button
      variant="outlined"
      color="inherit"
      sx={{ width: '260px' }}
      startIcon={<SvgIcon component={SlackIcon} inheritViewBox />}
      component={Link}
      href={`https://slack.com/oauth/v2/authorize?scope=chat%3Awrite%2Cchannels%3Aread%2Cgroups%3Aread&redirect_uri=${encodeURIComponent(config.slack_redirect_uri)}&client_id=${config.slack_client_id}`}
    >
      Add To Slack
    </Button>
  );
};

const ChannelNotSet = ({ onClick }) => (
  <Button
    variant="outlined"
    color="secondary"
    sx={{ width: '260px' }}
    onClick={onClick}
  >
    Set Channel
  </Button>
);

const SlackAppSchema = yup.object({
  slack_channel: yup.string().required("Channel URL is required"),
}).required();

export default SlackApp;