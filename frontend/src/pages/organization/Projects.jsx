import React from 'react';
import useSwr, { useSWRConfig } from 'swr';
import useSWRMutation from 'swr/mutation';
import { useSnackbar } from 'notistack';
import { useConfirm } from "material-ui-confirm";
import { useParams, Link as RouterLink } from 'react-router';
import { Stack, Typography, Link, Button, Paper } from '@mui/material';
import { DataGrid, GridActionsCellItem } from '@mui/x-data-grid';

import { EditIcon, DeleteIcon, ProjectIcon } from 'components/ConsistentIcons';

const Projects = () => {
  const confirm = useConfirm();
  const { enqueueSnackbar } = useSnackbar();

  const { id: organizationId } = useParams();

  const { data, isLoading, mutate } = useSwr(`/api/organizations/${organizationId}/projects`);

  const { mutate: mutateGlobal } = useSWRConfig();
  const { trigger: deleteProject } = useSWRMutation(`/api/organizations/${organizationId}/projects/delete`);

  const onDeleteProject = (row) => {
    let config = {
      title: 'Are you sure?',
      description: 'You\'re about to permanently delete this project and all data associated with it. This action cannot be undone.',
      acknowledgement: 'I understand',
      confirmationText: 'Delete Project'
    };

    confirm(config)
      .then(() => deleteProject({ project_id: row.project_id })
        .then(() => {
          enqueueSnackbar('Project deleted', { variant: 'success' });
          mutateGlobal('/api/organizations');
          mutate();
        })
        .catch((e) => enqueueSnackbar(e.message, { variant: 'error' }))
      )
      .catch(() => { });
  };

  const columns = React.useMemo(() => [
    { field: 'name', headerName: 'Project', flex: 1 },
    { field: 'api_key', headerName: 'API Key', filterable: false, flex: 2 },
    { field: 'created', headerName: 'Created', flex: 1 },
    {
      field: 'actions', headerName: 'Actions', type: 'actions', getActions: (params) => [
        <GridActionsCellItem
          key="edit"
          label="Edit this project"
          icon={<EditIcon />}
          component={RouterLink}
          to={`/organization/${params.row.organization_id}/projects/manage/${params.row.project_id}`}
        />,
        <GridActionsCellItem
          key="delete"
          label="Delete this project"
          icon={<DeleteIcon />}
          onClick={() => onDeleteProject(params.row)}
        />
      ]
    }
  ], []);

  if (data && data.length === 0) {
    return <NoProjects organizationId={organizationId} />;
  }

  return (
    <Stack spacing={2}>
      <DataGrid
        rows={data}
        columns={columns}
        loading={isLoading}
        getRowId={(row) => row.project_id}
        hideFooter={true}
        rowSelection={false}
      />

      <Button
        variant="contained"
        component={RouterLink}
        to={`/organization/${organizationId}/projects/manage`}
        sx={{ alignSelf: 'flex-start' }}
      >
        Create Project
      </Button>
    </Stack>
  );
};

const NoProjects = ({ organizationId }) => {
  return (
    <Paper sx={{ px: 5, py: 4, backgroundColor: 'accentBackground' }}>
      <Stack spacing={1} alignItems="center" useFlexGap>
        <ProjectIcon sx={{ fontSize: 60 }} />
        <Typography variant="h5" textAlign="center">Projects</Typography>
        <Typography variant="body2" textAlign="center">
          Upon creating a project within your organization, you will receive an API key to integrate the
          {' '}
          <Link href="https://crates.io/crates/dontpanic" title="Crates.io - dontpanic">dontpanic</Link>
          {' '}
          library into your application, enabling you to start sending panic reports.
        </Typography>
        <Button variant="contained" sx={{ my: 1 }} component={RouterLink} to={`/organization/${organizationId}/projects/manage`}>Create Project</Button>
      </Stack>
    </Paper>
  );
};

export default Projects;