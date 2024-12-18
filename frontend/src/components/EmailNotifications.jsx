import React from "react";
import useSWR from "swr";
import useSWRMutation from 'swr/mutation';
import { useSnackbar } from 'notistack';
import { useNavigate, Link as RouterLink } from "react-router";
import { TableContainer, Typography, TableCell, TableRow, Table, TableHead, TableBody, Checkbox, Link, Stack } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { SaveIcon } from "components/ConsistentIcons";

const EmailNotifications = ({ projectId }) => {
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();

  const [userIds, setUserIds] = React.useState([]);

  const { trigger, isMutating } = useSWRMutation(`/api/notifications/email/${projectId}`);
  const { data, isLoading, } = useSWR(`/api/notifications/email/${projectId}`);

  React.useEffect(() => {
    if (data) {
      setUserIds(data?.members.filter((member) => member.notify_email !== null).map((member) => member.user_id));
    }
  }, [data]);

  if (!projectId) {
    navigate('/reports');
  }

  const onSave = () => {
    trigger({ user_ids: userIds }).then(() => {
      enqueueSnackbar("Preferences saved", { variant: 'success' });
    }).catch((e) => {
      enqueueSnackbar(e.message, { variant: 'error' });
    });
  };

  const toggle = (user_id) => {
    if (userIds.includes(user_id)) {
      setUserIds(userIds.filter((id) => id !== user_id));
    } else {
      setUserIds([...userIds, user_id]);
    }
  };

  return (
    <TableContainer>
      <Table>
        <TableHead>
          <TableRow>
            <TableCell>Organization Member</TableCell>
            <TableCell>Role</TableCell>
            <TableCell align="right">Send Email?</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {data?.members.map((member) => (
            <TableRow key={member.user_id}>
              <TableCell sx={{ fontWeight: 'bold' }}>{member.name ?? member.email}</TableCell>
              <TableCell>{member.role}</TableCell>
              <TableCell align="right">
                <Checkbox checked={userIds.includes(member.user_id)} onChange={() => toggle(member.user_id)} />
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
    </TableContainer >
  );
};

export default EmailNotifications;