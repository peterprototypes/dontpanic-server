import { BrowserRouter, Routes, Route } from "react-router";
import { Box } from "@mui/system";

import AppLayout from "components/AppLayout";
import AuthLayout from "components/AuthLayout";
import Footer from "components/Footer";

import Login from "./pages/Login";
import Register from "./pages/Register";
import ReportsList from "./pages/ReportsList";

const App = () => {
  return (
    <Box height="100vh" display="flex" flexDirection="column">
      <BrowserRouter>
        <Routes>
          <Route element={<AppLayout />}>
            <Route index element={<ReportsList />} />
          </Route>

          <Route element={<AuthLayout />}>
            <Route path="login" element={<Login />} />
            <Route path="register" element={<Register />} />
          </Route>
        </Routes>
      </BrowserRouter >

      <Footer />
    </Box>
  );
};

export default App;
