import React from "react";
import useSWRMutation from "swr/mutation";
import { AppBar, Container, IconButton, Toolbar, Typography, Menu, MenuItem, Button, ListItemIcon, ListItemText, Divider, GlobalStyles } from "@mui/material";
import { Outlet, Link, useNavigate } from "react-router";

import AccountCircleIcon from '@mui/icons-material/AccountCircleOutlined';
import LogoutIcon from '@mui/icons-material/LogoutOutlined';

import { UserProvider, useUser } from "context/user";

import Logo from "./Logo";

const AppLayout = () => {
  return (
    <UserProvider>
      <AppBar position="static" sx={{ bgcolor: "accentBackground", color: "text.primary" }} elevation={1}>
        <Container maxWidth="md">
          <Toolbar variant="dense" disableGutters={true}>
            <IconButton edge="start" sx={{ mr: 1 }} component={Link} to="/">
              <Logo sx={{ height: '25px' }} />
            </IconButton>
            <Typography variant="h5" fontWeight="bold" sx={{ flexGrow: 1 }}>
              Don&lsquo;t Panic
            </Typography>
            <ProfileMenu />
          </Toolbar>
        </Container>
      </AppBar>
      <Container maxWidth="md" sx={{ flexGrow: 1, py: 3 }}>
        <Outlet />
      </Container>

      <GlobalStyles
        styles={{
          body: { backgroundColor: "white" }
        }}
      />
    </UserProvider>
  );
};

const ProfileMenu = () => {
  const navigate = useNavigate();
  const { user } = useUser();
  const { trigger: logout } = useSWRMutation('/api/auth/logout');
  const [anchorEl, setAnchorEl] = React.useState(null);

  const handleMenu = (event) => {
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const onLogout = () => {
    logout().then(() => {
      navigate("/auth/login");
    });
  };

  return (
    <div>
      <Button
        size="large"
        onClick={handleMenu}
        onMouseOver={handleMenu}
        variant="text"
        endIcon={<AccountCircleIcon color="primary" />}
        disableRipple
        color="inherit"
        sx={{ textTransform: 'none', fontWeight: 300 }}
      >
        {user.name ?? user.email}
      </Button>
      <Menu
        anchorEl={anchorEl}
        anchorOrigin={{
          vertical: 'bottom',
          horizontal: 'right',
        }}
        keepMounted
        transformOrigin={{
          vertical: 'top',
          horizontal: 'right',
        }}
        open={Boolean(anchorEl)}
        onClose={handleClose}
        MenuListProps={{ onMouseLeave: handleClose }}
        disableAutoFocusItem
      >
        <MenuItem component={Link} to="/account" onClick={handleClose}>My account</MenuItem>
        <Divider />
        <MenuItem onClick={onLogout}>
          <ListItemIcon>
            <LogoutIcon fontSize="small" />
          </ListItemIcon>
          <ListItemText>Log out</ListItemText>
        </MenuItem>
      </Menu>
    </div>
  );
};

export default AppLayout;