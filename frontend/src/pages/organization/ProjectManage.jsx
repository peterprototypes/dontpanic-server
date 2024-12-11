import * as yup from "yup";
import useSwr, { useSWRConfig } from 'swr';
import useSWRMutation from 'swr/mutation';
import { useNavigate, useParams, Link as RouterLink } from 'react-router';
import { useSnackbar } from 'notistack';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Stack, Typography, Link, Button } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';

const ProjectManage = () => {
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();

  const { id: organizationId, projectId } = useParams();

  const { mutate: mutateGlobal } = useSWRConfig();
  const { data, isLoading, mutate } = useSwr(projectId ? `/api/organizations/${organizationId}/projects/${projectId}` : null);
  const { trigger, error, isMutating } = useSWRMutation(`/api/organizations/${organizationId}/projects`);

  const methods = useForm({
    resolver: yupResolver(ProjectSchema),
    errors: error?.fields,
    values: {
      project_id: projectId ? parseInt(projectId) : null,
      name: data?.name ?? "",
    },
  });

  const onSubmit = (data) => {
    trigger(data)
      .then((response) => {
        enqueueSnackbar("Project saved", { variant: 'success' });
        navigate(`/organization/${organizationId}/projects`);
        mutateGlobal('/api/organizations');
        mutate(response, { revalidate: false });
      })
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  return (
    <FormProvider {...methods}>
      <Typography variant="h6" sx={{ mt: 2 }}>Create New Project</Typography>

      <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap alignItems="flex-start">

        <Typography color="textSecondary">
          Projects are a way to group panic reports together. Each project has its own API key, which you can use to integrate the
          {' '}
          <Link href="https://crates.io/crates/dontpanic" title="Crates.io - dontpanic">dontpanic</Link>
          {' '}
          library into your application.
        </Typography>

        <ControlledTextField
          fullWidth
          required
          name="name"
          label="Project name"
          placeholder="My Awesome Project"
          helperText="Max 80 characters."
        />

        <Stack sx={{ width: '100%' }} direction="row" justifyContent="space-between">
          <Button
            variant="contained"
            color="grey"
            component={RouterLink}
            to={`/organization/${organizationId}/projects`}
          >
            Cancel
          </Button>

          <LoadingButton
            type="submit"
            variant="contained"
            loading={isMutating || isLoading}
            loadingPosition="start"
            startIcon={<SaveIcon />}
          >
            {projectId ? 'Save' : 'Create'} Project
          </LoadingButton>
        </Stack>

        <FormServerError sx={{ width: '100%' }} />
      </Stack>
    </FormProvider>
  );
};

const ProjectSchema = yup.object({
  name: yup.string().required("Project name is required"),
}).required();

export default ProjectManage;