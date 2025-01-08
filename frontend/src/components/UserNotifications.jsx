import React from "react";
import useSWR from "swr";
import useSWRMutation from 'swr/mutation';
import { useSnackbar } from 'notistack';
import { useNavigate, Link as RouterLink } from "react-router";
import { TableContainer, Typography, TableCell, TableRow, Table, TableHead, TableBody, Checkbox, Link, Stack, Alert } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { useConfig } from "context/config";
import { SaveIcon } from "components/ConsistentIcons";

const UserNotifications = ({ projectId }) => {
  const { config } = useConfig();
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();

  const [userSettings, setUserSettings] = React.useState([]);

  const { trigger, isMutating } = useSWRMutation(`/api/notifications/per-user/${projectId}`);
  const { data, isLoading, } = useSWR(`/api/notifications/per-user/${projectId}`);

  React.useEffect(() => {
    if (data) {
      setUserSettings(data?.settings);
    }
  }, [data]);

  if (!projectId) {
    navigate('/reports');
  }

  const onSave = () => {
    const settings = userSettings.map((e) => {
      return {
        user_id: e.user_id,
        notify_email: e.notify_email,
        notify_pushover: e.notify_pushover,
      };
    });

    trigger({ settings }).then(() => {
      enqueueSnackbar("Preferences saved", { variant: 'success' });
    }).catch((e) => {
      enqueueSnackbar(e.message, { variant: 'error' });
    });
  };

  const emailEnabled = (user_id) => {
    return userSettings.find((e) => e.user_id == user_id)?.notify_email ?? false;
  };

  const toggleEmail = (user_id) => {
    setUserSettings(userSettings.map((e) => e.user_id == user_id ? { ...e, notify_email: !emailEnabled(user_id) } : e));
  };

  const pushoverEnabled = (user_id) => {
    return userSettings.find((e) => e.user_id == user_id)?.notify_pushover ?? false;
  };

  const togglePushover = (user_id) => {
    setUserSettings(userSettings.map((e) => e.user_id == user_id ? { ...e, notify_pushover: !pushoverEnabled(user_id) } : e));
  };

  return (
    <TableContainer>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>Organization Member</TableCell>
            <TableCell>Role</TableCell>
            {config.pushover_enabled && <TableCell align="right">Pushover Notification?</TableCell>}
            <TableCell align="right">Send Email?</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {data?.settings.map((member) => (
            <TableRow key={member.user_id}>
              <TableCell sx={{ fontWeight: 'bold' }}>{member.name ?? member.email}</TableCell>
              <TableCell>{member.role}</TableCell>
              {config.pushover_enabled && (
                <TableCell align="right">
                  <Checkbox checked={pushoverEnabled(member.user_id)} onChange={() => togglePushover(member.user_id)} />
                </TableCell>
              )}
              <TableCell align="right">
                <Checkbox checked={emailEnabled(member.user_id)} onChange={() => toggleEmail(member.user_id)} disabled={!config.can_send_emails} />
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
      <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ mt: 3 }}>
        <Typography variant="body2" color="textSecondary">
          Click <Link component={RouterLink} to={`/organization/${data?.organization_id}/members`}>here</Link> to invite members to this organization.
        </Typography>
        <LoadingButton
          variant="contained"
          loading={isMutating || isLoading}
          loadingPosition="end"
          endIcon={<SaveIcon />}
          onClick={onSave}
        >
          Save Preferences
        </LoadingButton>
      </Stack>
      {!config.can_send_emails && (
        <Alert severity="warning" sx={{ mt: 2 }}>
          Email notifications are disabled due to missing <strong>EMAIL_URL</strong> environment variable. See the
          {' '}
          <Link href="https://github.com/peterprototypes/dontpanic-server/tree/main?tab=readme-ov-file#environment-variables" target="_blank">README</Link>
          {' '}
          for more information.
        </Alert>
      )}

    </TableContainer >
  );
};

export default UserNotifications;