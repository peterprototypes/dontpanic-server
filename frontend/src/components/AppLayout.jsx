import { Container } from "@mui/material";
import { Outlet } from "react-router";

const AppLayout = () => {
  return (
    <Container maxWidth="md" sx={{ flexGrow: 1 }}>
      <h1>App Layout</h1>
      <Outlet />
    </Container>
  );
};

export default AppLayout;