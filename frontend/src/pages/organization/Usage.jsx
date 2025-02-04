import { useParams } from "react-router";

import NewReports from 'components/stats/NewReports';
import ApiUsage from 'components/stats/ApiUsage';
import { Stack } from "@mui/material";

const Usage = () => {
  const { id: organizationId } = useParams();

  return (
    <Stack spacing={2}>
      <ApiUsage organizationId={organizationId} />
      <NewReports organizationId={organizationId} />
    </Stack>
  );
};

export default Usage;