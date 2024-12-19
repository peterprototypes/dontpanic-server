import React from 'react';
import useSwr from 'swr';
import { DateTime } from "luxon";
import useSWRMutation from 'swr/mutation';
import { useSnackbar } from 'notistack';
import { useConfirm } from "material-ui-confirm";
import { useParams, Link as RouterLink } from 'react-router';
import { Stack, Typography, Button, Alert, CircularProgress, Tooltip } from '@mui/material';
import { DataGrid, GridActionsCellItem } from '@mui/x-data-grid';

import NoRowsOverlay from 'components/NoRowsOverlay';
import { EditIcon, DeleteIcon, SendEmailIcon } from 'components/ConsistentIcons';

const Members = () => {
  const { id: organizationId } = useParams();

  const { data, error, isLoading, mutate } = useSwr(`/api/organizations/${organizationId}/members`);

  if (error) return <Alert severity="error">{error.message}</Alert>;

  return (
    <Stack spacing={2}>
      <OrganizationMembers
        data={data?.members}
        mutate={mutate}
        isLoading={isLoading}
      />

      <Typography variant="h5">Pending Invitations</Typography>

      <OrganizationInvites
        organizationId={organizationId}
        data={data?.invitations}
        mutate={mutate}
        isLoading={isLoading}
      />

      <Button
        variant="contained"
        component={RouterLink}
        to={`/organization/${organizationId}/members/invite`}
        sx={{ alignSelf: 'flex-start' }}
      >
        Invite Member
      </Button>
    </Stack>
  );
};

const OrganizationMembers = ({ data, mutate, isLoading }) => {
  const columns = React.useMemo(() => [
    { field: 'email', headerName: 'Email', flex: 2 },
    { field: 'name', headerName: 'Name', flex: 1 },
    { field: 'role', headerName: 'Role', minWidth: 60 },
    {
      field: 'date_added',
      type: 'dateTime',
      headerName: 'Added',
      minWidth: 160,
      valueGetter: (value) => value && DateTime.fromISO(value, { zone: 'UTC' }).toJSDate()
    },
    {
      field: 'actions', headerName: 'Actions', type: 'actions', getActions: (params) => [
        <GridActionsCellItem
          key="edit"
          label="Edit this project"
          icon={<Tooltip title="Edit member"><EditIcon /></Tooltip>}
          component={RouterLink}
          to={`/organization/${params.row.organization_id}/members/manage/${params.row.user_id}`}
        />,
        <DeleteMember key="delete" member={params.row} mutate={mutate} />
      ]
    }
  ], [mutate]);

  return (
    <DataGrid
      rows={data}
      columns={columns}
      loading={isLoading}
      getRowId={(row) => row.user_id}
      hideFooter={true}
      rowSelection={false}
    />
  );
};

const DeleteMember = ({ member, mutate }) => {
  const confirm = useConfirm();
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, isMutating } = useSWRMutation(`/api/organizations/${member.organization_id}/members/delete/${member.user_id}`);

  const onDeleteMember = () => {
    let config = {
      title: 'Are you sure?',
      description: 'Are you sure you want to proceed with this action? This member will no longer have access to the organization.',
      confirmationText: 'Delete Member'
    };

    confirm(config)
      .then(() => trigger({})
        .then(() => {
          enqueueSnackbar('Member removed', { variant: 'success' });
          mutate();
        })
        .catch((e) => enqueueSnackbar(e.message, { variant: 'error' }))
      )
      .catch(() => { });
  };

  return (
    <GridActionsCellItem
      label="Remove this member"
      icon={isMutating ? <CircularProgress size="14px" /> : <Tooltip title="Remove this member"><DeleteIcon /></Tooltip>}
      onClick={() => onDeleteMember()}
    />
  );
};

const OrganizationInvites = ({ data, mutate, isLoading }) => {
  const columns = React.useMemo(() => [
    { field: 'email', headerName: 'Email', flex: 2 },
    { field: 'role', headerName: 'Role', flex: 1 },
    {
      field: 'created',
      type: 'dateTime',
      headerName: 'Invited',
      minWidth: 160,
      valueGetter: (value) => value && DateTime.fromISO(value, { zone: 'UTC' }).toJSDate()
    },
    {
      field: 'actions', headerName: 'Actions', type: 'actions', getActions: (params) => [
        <ResendInvitation key="resend" invitation={params.row} />,
        <DeleteInvitation key="delete" invitation={params.row} mutate={mutate} />
      ]
    }
  ], [mutate]);

  return (
    <DataGrid
      rows={data}
      columns={columns}
      loading={isLoading}
      getRowId={(row) => row.organization_invitation_id}
      hideFooter={true}
      rowSelection={false}
      slots={{
        noRowsOverlay: () => <NoRowsOverlay primaryText="No pending invitations" />,
      }}
    />
  );
};

const ResendInvitation = ({ invitation }) => {
  const confirm = useConfirm();
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, isMutating } = useSWRMutation(`/api/organizations/${invitation.organization_id}/members/resend-invite/${invitation.email}`);

  const onResendInvitation = () => {
    let config = {
      title: 'Are you sure?',
      description: 'Are you sure you want to resend this invitation?',
      confirmationText: 'Resend Invitation'
    };

    confirm(config)
      .then(() => trigger({})
        .then(() => enqueueSnackbar('Invitation resent', { variant: 'success' }))
        .catch((e) => enqueueSnackbar(e.message, { variant: 'error' }))
      )
      .catch(() => { });
  };

  return (
    <GridActionsCellItem
      label="Resend this invitation"
      icon={isMutating ? <CircularProgress size="14px" /> : <Tooltip title="Resend Invitation Email"><SendEmailIcon /></Tooltip>}
      onClick={() => onResendInvitation()}
    />
  );
};

const DeleteInvitation = ({ invitation, mutate }) => {
  const confirm = useConfirm();
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, isMutating } = useSWRMutation(`/api/organizations/${invitation.organization_id}/members/delete-invite/${invitation.organization_invitation_id}`);

  const onDeleteInvitation = () => {
    let config = {
      title: 'Are you sure?',
      description: 'Are you sure you want to withdraw this invitation?',
      confirmationText: 'Remove Invitation'
    };

    confirm(config)
      .then(() => trigger({})
        .then(() => {
          enqueueSnackbar('Invitation removed', { variant: 'success' });
          mutate();
        })
        .catch((e) => enqueueSnackbar(e.message, { variant: 'error' }))
      )
      .catch(() => { });
  };

  return (
    <GridActionsCellItem
      label="Withdraw this invitation"
      icon={isMutating ? <CircularProgress size="14px" /> : <Tooltip title="Withdraw this invitation"><DeleteIcon /></Tooltip>}
      onClick={() => onDeleteInvitation()}
    />
  );
};

export default Members;