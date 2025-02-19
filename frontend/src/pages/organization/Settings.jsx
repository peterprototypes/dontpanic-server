
import useSwr, { useSWRConfig } from 'swr';
import useSWRMutation from 'swr/mutation';
import * as yup from "yup";
import { useNavigate, useParams } from 'react-router';
import { useSnackbar } from 'notistack';
import { useConfirm } from "material-ui-confirm";
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Divider, Stack, Typography, Alert } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';

const Settings = () => {
  const { id: organizationId } = useParams();
  const { enqueueSnackbar } = useSnackbar();

  const { data: organizations, mutate } = useSwr(`/api/organizations`);
  const { trigger, error, isMutating } = useSWRMutation(`/api/organizations/${organizationId}`);

  let org = organizations?.find((org) => org.organization_id === parseInt(organizationId));

  let methods = useForm({
    resolver: yupResolver(SettingsSchema),
    errors: error?.fields,
    values: {
      name: org?.name ?? "",
      requests_alert_threshold: org?.requests_alert_threshold ?? "",
    }
  });

  const onSubmit = (data) => {
    let limit_treshold = parseInt(data.requests_alert_threshold);

    trigger({ ...data, requests_alert_threshold: limit_treshold == 0 ? null : limit_treshold })
      .then(() => {
        enqueueSnackbar("Organization updated", { variant: 'success' });
        mutate();
      })
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  return (
    <Stack spacing={2}>
      <FormProvider {...methods}>
        <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap alignItems="flex-start">

          <ControlledTextField
            required
            fullWidth
            name="name"
            label="Organization Name"
            placeholder="My Awesome Organization"
            helperText="Max 80 characters."
          />

          <ControlledTextField
            required
            fullWidth
            name="requests_alert_threshold"
            label="Daily Requests Alert Threshold"
            placeholder="0"
            helperText="All owners will get notified if this organization receives more than the specified amount of API requests per day. Set to 0 to disable."
          />

          <LoadingButton
            type="submit"
            variant="contained"
            loading={isMutating}
            loadingPosition="start"
            startIcon={<SaveIcon />}
          >
            Save
          </LoadingButton>

          <FormServerError sx={{ width: '100%' }} />
        </Stack>
      </FormProvider>

      <Divider />

      <DeleteOrganization organizationId={organizationId} />
    </Stack>
  );
};

const SettingsSchema = yup.object().shape({
  name: yup.string().required("Organization name is required").max(80),
}).required();

const DeleteOrganization = ({ organizationId }) => {
  const confirm = useConfirm();
  const navigate = useNavigate();
  const { mutate } = useSWRConfig();
  const { enqueueSnackbar } = useSnackbar();
  const { trigger, error, isMutating } = useSWRMutation(`/api/organizations/${organizationId}/delete`);

  const onDeleteOrganization = () => {
    let config = {
      title: 'Are you sure?',
      description: 'You\'re about to permanently delete this organization. This action cannot be undone.',
      acknowledgement: 'I understand',
      confirmationText: 'Delete Organization'
    };

    confirm(config)
      .then(() => trigger({})
        .then(() => {
          mutate('/api/organizations');
          navigate('/');
        })
        .catch((e) => enqueueSnackbar(e.message, { variant: 'error' }))
      )
      .catch(() => { });
  };

  return (
    <Stack sx={{ mt: 4 }} spacing={1} alignItems="flex-start" useFlexGap>
      <Typography variant="h5" color="error">Delete Organization</Typography>
      <Typography variant="body1" color="textSecondary">Deleting an organization will remove all it&lsquo;s data and cannot be undone.</Typography>

      <LoadingButton
        variant="outlined"
        color="error"
        sx={{ mt: 2 }}
        onClick={onDeleteOrganization}
        loading={isMutating}
      >
        Delete Organization
      </LoadingButton>

      {error && <Alert severity="error" sx={{ width: '100%', mt: 2 }}>{error.message}</Alert>}
    </Stack>
  );
};

export default Settings;