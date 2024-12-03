import { Container } from "@mui/material";
import { Outlet } from "react-router";
import { UserProvider } from "../context/user";

const AppLayout = () => {
  return (
    <UserProvider>
      <Container maxWidth="md" sx={{ flexGrow: 1 }}>
        <h1>App Layout</h1>
        <Outlet />
      </Container>
    </UserProvider>
  );
};

export default AppLayout;