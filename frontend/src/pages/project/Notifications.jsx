
import useSWR from "swr";
import { useSearchParams } from "react-router";
import { Divider, Stack, Typography, Link } from '@mui/material';

import UserNotifications from "components/UserNotifications";
import LoadingPage from "components/LoadingPage";

import Webhook from "components/integrations/Webhook";
import TeamsWebhook from "components/integrations/TeamsWebhook";
import SlackWebhook from "components/integrations/SlackWebhook";
import SlackApp from "components/integrations/SlackApp";

const Notifications = () => {
  const [searchParams] = useSearchParams();
  const projectId = searchParams.get('project_id');

  const { data: project, isLoading } = useSWR(`/api/notifications/project/${projectId}`);

  if (isLoading) {
    return <LoadingPage />;
  }

  return (
    <Stack spacing={1} useFlexGap>
      <UserNotifications projectId={projectId} />

      <Typography variant="h5" sx={{ mt: 5, fontWeight: 'bold' }}>Notification Integrations</Typography>
      <Typography color="textSecondary">
        Connect your project with third-party applications to receive real-time notifications.
      </Typography>
      <Typography color="textSecondary" gutterBottom>
        If none of the available integrations fit your requirements, you can set up a custom webhook or <Link href="https://github.com/peterprototypes/dontpanic-server/issues">suggest a new integration</Link> on GitHub.
      </Typography>
      <Divider sx={{ mb: 2 }} />

      <Webhook project={project} />

      <SlackApp project={project} />

      <TeamsWebhook project={project} />

      <SlackWebhook project={project} />
    </Stack>
  );
};

export default Notifications;