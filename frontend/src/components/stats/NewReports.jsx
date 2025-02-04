import React from "react";
import useSwr from 'swr';
import { Stack, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { BarChart } from '@mui/x-charts/BarChart';

import LoadingPage from '../LoadingPage';

const NewReportsStats = ({ organizationId }) => {
  const [grouping, setGrouping] = React.useState("daily");

  const { data: projects, isLoading: projectsLoading } = useSwr(`/api/organizations/${organizationId}/projects`);
  const { data, isLoading } = useSwr(`/api/organizations/${organizationId}/stats?grouping=${grouping}&category=new_project_report`);

  if (isLoading || projectsLoading) {
    return <LoadingPage />;
  }

  const findProjectName = (projectId) => {
    return projects.find((project) => project.project_id == projectId)?.name ?? '#' + projectId;
  };

  return (
    <Stack spacing={2} sx={{ border: 1, borderColor: 'divider', borderRadius: 1, py: 2, px: 2 }}>
      <Stack direction="row" justifyContent="space-between">
        <Typography variant="h6">New Reports</Typography>
        <ToggleButtonGroup value={grouping} exclusive onChange={(e, value) => setGrouping(value)} color="primary">
          <ToggleButton value="daily">Daily</ToggleButton>
          <ToggleButton value="monthly">Monthly</ToggleButton>
        </ToggleButtonGroup>
      </Stack>

      <BarChart
        dataset={data.dataset}
        xAxis={[{ scaleType: 'band', dataKey: 'date' }]}
        series={data.names.map((projectId) => ({ dataKey: projectId.toString(), label: findProjectName(projectId), stack: 'total', stackOrder: 'appearance' }))}
        height={250}
      />

      <Typography variant="body2" gutterBottom>
        New reports received for the last {grouping === 'daily' ? '30 days' : '12 months'}.
      </Typography>
    </Stack>
  );
};

export default NewReportsStats;