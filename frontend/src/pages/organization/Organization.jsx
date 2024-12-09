
import { Outlet, useMatch, Link } from 'react-router';
import { Box, Divider, Grid2 as Grid, Typography, Tab } from '@mui/material';
import { TabList, TabContext } from "@mui/lab";

import SideMenu from 'components/SideMenu';

const Organization = () => {
  const { params } = useMatch('/organization/:id/:page/*');

  return (
    <TabContext value={params.page}>
      <Grid container spacing={4}>
        <Grid size={3}>
          <SideMenu />
        </Grid>
        <Grid size={9}>
          <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', mt: 2 }}>
            <Typography variant="h4">Organization</Typography>
            <TabList value={params.page}>
              <Tab label="Projects" value="projects" component={Link} to={`/organization/${params.id}/projects`} />
              <Tab label="Members" value="members" component={Link} to={`/organization/${params.id}/members`} />
              <Tab label="Settings" value="settings" component={Link} to={`/organization/${params.id}/settings`} />
            </TabList>
          </Box>
          <Divider sx={{ mb: 2 }} />
          <Outlet />
        </Grid>
      </Grid>
    </TabContext>
  );
};

export default Organization;