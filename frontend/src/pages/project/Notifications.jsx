
import useSWR from "swr";
import { useSearchParams } from "react-router";
import { Divider, Stack, Typography, Link } from '@mui/material';

import EmailNotifications from "components/EmailNotifications";
import LoadingPage from "components/LoadingPage";

import Webhook from "components/integrations/Webhook";
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
      <EmailNotifications projectId={projectId} />

      <Typography variant="h5" sx={{ mt: 5, fontWeight: 'bold' }}>Integrations</Typography>
      <Typography color="textSecondary">
        Integrations allow you to receive notifications in third-party applications.
        If the available integrations don&lsquo;t meet your needs, consider using a webhook or <Link href="https://github.com/peterprototypes/dontpanic-server/issues">open an issue</Link> on GitHub.
      </Typography>
      <Divider sx={{ mb: 2 }} />

      <Webhook project={project} />

      <SlackApp project={project} />

      <SlackWebhook project={project} />
    </Stack>
  );
};

export default Notifications;