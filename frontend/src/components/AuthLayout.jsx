import { Outlet } from "react-router";
import { Container, Box } from '@mui/material';

const AuthLayout = () => {
  return (
    <Container maxWidth="xs" sx={{ flexGrow: 1 }}>
      <Box display="flex" justifyContent="center" alignItems="center" height="100%">
        <Box sx={{ p: 4, my: 4, backgroundColor: 'white', boxShadow: 3, borderRadius: '16px', width: '100%' }}>
          <Outlet />
        </Box>
      </Box>
    </Container>
  );
};

export default AuthLayout;