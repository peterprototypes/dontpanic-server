import React from 'react';
import useSWR from 'swr';
import List from '@mui/material/List';
import ListItemButton from '@mui/material/ListItemButton';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import ListItem from '@mui/material/ListItem';
import Button from '@mui/material/Button';
import { styled } from '@mui/material/styles';

import OrgIcon from '@mui/icons-material/CorporateFareOutlined';
import CircleIcon from '@mui/icons-material/Circle';

const SideMenu = () => {
  const { data: organizations } = useSWR('/api/organizations');

  return (
    <List component="nav">
      <ListItemButton divider>
        <ListItemText primary="All Reports" />
      </ListItemButton>

      {organizations?.map((org) => (
        <React.Fragment key={org.organization_id}>
          <ListItemButton>
            <OrgListIcon><OrgIcon /></OrgListIcon>
            <ListItemText primary="Cytec Bg Ltd." />
          </ListItemButton>

          {org.projects.map((project) => (
            <ListItemButton key={project.project_id} sx={{ pl: 4 }}>
              <ProjectListIcon><CircleIcon sx={{ fontSize: 10 }} /></ProjectListIcon>
              <ListItemText primary={project.name} />
            </ListItemButton>
          ))}
        </React.Fragment>
      ))}


      <ListItem disableGutters>
        <Button variant="outlined" fullWidth>Add Organization</Button>
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