import useSWRMutation from 'swr/mutation';
import * as yup from "yup";
import { useNavigate, useParams, Link as RouterLink } from 'react-router';
import { useSnackbar } from 'notistack';
import { useForm, FormProvider } from 'react-hook-form';
import { yupResolver } from '@hookform/resolvers/yup';
import { Stack, Typography, MenuItem, Button, Alert } from '@mui/material';
import { LoadingButton } from "@mui/lab";

import { useUser } from 'context/user';
import { FormServerError, ControlledTextField } from "components/form";
import { SendEmailIcon } from 'components/ConsistentIcons';

const MemberInvite = () => {
  const { id: organizationId } = useParams();
  const { user } = useUser();

  const navigate = useNavigate();
  const { enqueueSnackbar } = useSnackbar();

  const { trigger, error, isMutating } = useSWRMutation(`/api/organizations/${organizationId}/members/invite`);

  const methods = useForm({
    resolver: yupResolver(InvitationSchema),
    errors: error?.fields,
    defaultValues: {
      email: '',
      role: 'member',
    },
  });

  const onSubmit = (data) => {
    trigger(data)
      .then(() => {
        enqueueSnackbar("Member invited", { variant: 'success' });
        navigate(`/organization/${organizationId}/members`);
      })
      .catch((e) => methods.setError('root.serverError', { message: e.message }));
  };

  if (user.getRole(organizationId) === 'member') {
    return (
      <Alert severity="warning" action={<Button component={RouterLink} color="inherit" to={`/organization/${organizationId}/members`}>Back</Button>}>
        You do not have permission to invite members to this organization.
      </Alert>
    );
  }

  return (
    <FormProvider {...methods}>
      <Typography variant="h6" sx={{ mt: 2 }}>Invite Member</Typography>

      <Stack component="form" spacing={2} sx={{ mt: 2 }} onSubmit={methods.handleSubmit(onSubmit)} noValidate useFlexGap alignItems="flex-start">

        <Typography color="textSecondary">
          Invite a new member to your organization.
        </Typography>

        <ControlledTextField
          fullWidth
          required
          name="email"
          label="Email"
          placeholder="john.doe@example.com"
          helperText="We'll send an email to this address with an invitation link."
          autoFocus
        />

        <ControlledTextField
          select
          required
          fullWidth
          name="role"
          label="Role"
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
            loading={isMutating}
            loadingPosition="start"
            startIcon={<SendEmailIcon />}
          >
            Send Invite
          </LoadingButton>
        </Stack>

        <FormServerError sx={{ width: '100%' }} />
      </Stack>
    </FormProvider>
  );
};

const InvitationSchema = yup.object({
  email: yup.string().email("Please enter a valid email address").required("Email is required"),
  role: yup.string().oneOf(['admin', 'member', 'owner']).required(),
}).required();

export default MemberInvite;