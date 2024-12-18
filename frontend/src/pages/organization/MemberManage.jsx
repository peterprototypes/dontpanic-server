import * as yup from "yup";
import useSwr, { useSWRConfig } from 'swr';
import useSWRMutation from 'swr/mutation';
import { useNavigate, useParams, Link as RouterLink } from 'react-router';
import { useSnackbar } from 'notistack';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Stack, Typography, Button, MenuItem } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { FormServerError, ControlledTextField } from "components/form";
import { SaveIcon } from 'components/ConsistentIcons';

const MemberManage = () => {
  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();

  const { id: organizationId, memberId } = useParams();

  const { mutate: mutateGlobal } = useSWRConfig();
  const { data, isLoading } = useSwr(`/api/organizations/${organizationId}/members/${memberId}`);
  const { trigger, error, isMutating } = useSWRMutation(`/api/organizations/${organizationId}/members/${memberId}`);

  const methods = useForm({
    resolver: yupResolver(MemberSchema),
    errors: error?.fields,
    values: {
      role: data?.role ?? "member",
    },
  });

  const onSubmit = (data) => {
    trigger(data)
      .then(() => {
        enqueueSnackbar("Member updated", { variant: 'success' });
        navigate(`/organization/${organizationId}/members`);
        mutateGlobal(`/api/organizations/${organizationId}/members`);
      })
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  return (
    <FormProvider {...methods}>
      <Typography variant="h6" sx={{ mt: 2 }}>Edit Member</Typography>

      <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap alignItems="flex-start">

        <Typography color="textSecondary">{data?.email}</Typography>

        <ControlledTextField
          required
          fullWidth
          name="role"
          label="Role"
          select
        >
          <MenuItem value="member">Member</MenuItem>
          <MenuItem value="admin">Admin</MenuItem>
          <MenuItem value="owner">Owner</MenuItem>
        </ControlledTextField>

        <ul>
          <Typography sx={{ mb: 1 }} variant="body2" component="li"><strong>Member</strong> can view, archive and delete reports, manage notifications, can create and edit projects in the organization.</Typography>
          <Typography sx={{ mb: 1 }} variant="body2" component="li"><strong>Admin</strong> can invite and remove other admins and members, can delete projects in the organization.</Typography>
          <Typography sx={{ mb: 1 }} variant="body2" component="li"><strong>Owner</strong> can do all of the above plus add other owners and delete the organization.</Typography>
        </ul>

        <Stack sx={{ width: '100%' }} direction="row" justifyContent="space-between">
          <Button
            variant="contained"
            color="grey"
            component={RouterLink}
            to={`/organization/${organizationId}/members`}
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
            Save
          </LoadingButton>
        </Stack>

        <FormServerError sx={{ width: '100%' }} />
      </Stack>
    </FormProvider>
  );
};

const MemberSchema = yup.object({
  role: yup.string().oneOf(["member", "admin", "owner"]).required(),
}).required();

export default MemberManage;