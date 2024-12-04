// import useSWR from 'swr';
import { Grid2 as Grid } from '@mui/material';

import SideMenu from 'components/SideMenu';

const ReportsList = () => {
  //const { data, error, isLoading } = useSWR('http://localhost:8080/reports');

  return (
    <Grid container spacing={2}>
      <Grid size={3}>
        <SideMenu />
      </Grid>
      <Grid size={9}>
        <h1>Reports List</h1>
      </Grid>
    </Grid>
  );
};

export default ReportsList;