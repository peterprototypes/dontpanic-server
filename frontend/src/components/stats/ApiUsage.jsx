import React from "react";
import useSwr from 'swr';
import { Stack, ToggleButton, ToggleButtonGroup, Typography } from "@mui/material";
import { BarChart } from '@mui/x-charts/BarChart';

import LoadingPage from '../LoadingPage';

const ApiUsage = ({ organizationId }) => {
  const [grouping, setGrouping] = React.useState("daily");

  const { data, isLoading } = useSwr(`/api/organizations/${organizationId}/stats?grouping=${grouping}`);

  if (isLoading) {
    return <LoadingPage />;
  }

  return (
    <Stack spacing={2} sx={{ border: 1, borderColor: 'divider', borderRadius: 1, py: 2, px: 2 }}>
      <Stack direction="row" justifyContent="space-between">
        <Typography variant="h6">API Requests</Typography>
        <ToggleButtonGroup value={grouping} exclusive onChange={(e, value) => setGrouping(value)} color="primary">
          <ToggleButton value="daily">Daily</ToggleButton>
          <ToggleButton value="monthly">Monthly</ToggleButton>
        </ToggleButtonGroup>
      </Stack>

      <BarChart
        dataset={data.dataset}
        xAxis={[{ scaleType: 'band', dataKey: 'date' }]}
        series={[{ dataKey: 'total_count' }]}
        height={250}
      />

      <Typography variant="body2" gutterBottom>
        API usage statistics for the last {grouping === 'daily' ? '30 days' : '12 months'}.
      </Typography>
    </Stack>
  );
};

export default ApiUsage;