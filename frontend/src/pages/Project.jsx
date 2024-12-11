import { useSearchParams } from 'react-router';
import { Outlet, useMatch, Link as RouterLink } from 'react-router';
import { Box, Divider, Grid2 as Grid, Tab, Tabs, Link } from '@mui/material';

import SideMenu from 'components/SideMenu';

const Project = () => {
  const [searchParams] = useSearchParams();
  const reportsRoute = useMatch('/reports/:page/*');

  const projectId = searchParams.get('projectId');
  const searchParamsString = searchParams.size ? '?' + searchParams.toString() : '';

  return (
    <Grid container spacing={2}>
      <Grid size={3}>
        <SideMenu />
      </Grid>
      <Grid size={9}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', mt: 2 }}>
          <Link
            component={RouterLink}
            to={`/reports` + searchParamsString}
            variant="h4"
            color={reportsRoute?.params?.page ? 'textPrimary' : 'primary'}
          >
            Reports
          </Link>

          <Tabs value={reportsRoute?.params?.page ?? false}>
            <Tab label="Resolved" value="resolved" component={RouterLink} to={`/reports/resolved` + searchParamsString} />
            {projectId && <Tab label="Notifications" value="notifications" component={RouterLink} to={`/reports/notifications` + searchParamsString} />}
          </Tabs>
        </Box>
        <Divider sx={{ mb: 2 }} />
        <Outlet />
      </Grid>
    </Grid>
  );
};

export default Project;