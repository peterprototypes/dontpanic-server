import useSWRMutation from 'swr/mutation';
import { useSWRConfig } from "swr";
import * as yup from "yup";
import { useNavigate } from 'react-router';
import { useSnackbar } from 'notistack';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Divider, Grid2 as Grid, Stack, Typography } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import SideMenu from 'components/SideMenu';
import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';

const Route = () => {
  return (
    <Grid container spacing={4}>
      <Grid size={3}>
        <SideMenu />
      </Grid>
      <Grid size={9}>
        <AddOrganization />
      </Grid>
    </Grid>
  );
};

const AddOrganization = () => {
  const navigate = useNavigate();
  const { mutate } = useSWRConfig();
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, error, isMutating } = useSWRMutation('/api/organizations');

  const methods = useForm({
    resolver: yupResolver(OrganizationSchema),
    errors: error?.fields,
    defaultValues: {
      name: "",
    },
  });

  const onSubmit = (data) => {
    trigger(data)
      .then((response) => {
        mutate('/api/account');
        enqueueSnackbar("Organization created", { variant: 'success' });
        navigate(`/organization/${response.organization_id}/projects`);
      })
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  return (
    <FormProvider {...methods}>
      <Typography variant="h4" sx={{ mt: 2 }}>Create New Organization</Typography>
      <Divider />

      <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap alignItems="flex-start">

        <Typography color="textSecondary">
          Organizations allow you to manage and collaborate across multiple projects. Members of an organization have access to all of its projects.
        </Typography>

        <ControlledTextField
          fullWidth
          required
          name="name"
          label="Organization name"
          placeholder="My awesome organization"
          helperText="Max 80 characters."
        />

        <LoadingButton
          type="submit"
          variant="contained"
          loading={isMutating}
          loadingPosition="start"
          startIcon={<SaveIcon />}
        >
          Create Organization
        </LoadingButton>

        <FormServerError sx={{ width: '100%' }} />
      </Stack>
    </FormProvider>
  );
};

const OrganizationSchema = yup.object({
  name: yup.string().required("Organization name is required"),
}).required();

export default Route;