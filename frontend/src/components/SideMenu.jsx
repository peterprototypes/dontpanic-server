import React from 'react';
import useSWR from 'swr';
import { Link, useMatch, useSearchParams } from 'react-router';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import ListItem from '@mui/material/ListItem';
import Button from '@mui/material/Button';
import LinearProgress from '@mui/material/LinearProgress';
import { styled } from '@mui/material/styles';
import CircleIcon from '@mui/icons-material/Circle';

import { OrganizationIcon } from './ConsistentIcons';

const SideMenu = () => {
  const [searchParams] = useSearchParams();
  const { data: organizations, isLoading } = useSWR('/api/organizations');

  const organizationPage = useMatch('/organization/:id/:page/*');
  const selectedOrganizationId = organizationPage?.params.id ?? null;

  const reportsPage = useMatch('reports/:project_id?');
  const selectedProjectId = reportsPage && searchParams.get('project_id');

  return (
    <List component="nav">
      <ListItemButton component={Link} to="/reports" divider selected={reportsPage && !selectedProjectId}>
        <ListItemText primary="All Reports" />
      </ListItemButton>

      {isLoading && <LinearProgress />}

      {organizations?.map((org) => (
        <React.Fragment key={org.organization_id}>
          <ListItemButton
            component={Link}
            to={`/organization/${org.organization_id}/projects`}
            selected={selectedOrganizationId == org.organization_id}
          >
            <OrgListIcon><OrganizationIcon /></OrgListIcon>
            <ListItemText primary={org.name} />
          </ListItemButton>

          {org.projects.map((project) => (
            <ListItemButton
              key={project.project_id}
              component={Link}
              to={`/reports?project_id=${project.project_id}`}
              sx={{ pl: 4 }}
              selected={selectedProjectId == project.project_id}
            >
              <ProjectListIcon><CircleIcon sx={{ fontSize: 10 }} /></ProjectListIcon>
              <ListItemText primary={project.name} />
            </ListItemButton>
          ))}
        </React.Fragment>
      ))}

      <ListItem disableGutters>
        <Button variant="outlined" fullWidth component={Link} to="/add-organization">Add Organization</Button>
      </ListItem>
    </List>
  );
};

const OrgListIcon = styled(ListItemIcon)(() => ({
  minWidth: 30
}));

const ProjectListIcon = styled(ListItemIcon)(() => ({
  minWidth: 20
}));

export default SideMenu;