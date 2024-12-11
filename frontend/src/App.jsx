import { BrowserRouter, Routes, Route, Navigate } from "react-router";
import { Box } from "@mui/system";

import AppLayout from "components/AppLayout";
import AuthLayout from "components/AuthLayout";
import Footer from "components/Footer";

import Login from "./pages/auth/Login";
import Register from "./pages/auth/Register";

import PasswordResetRequest from "./pages/auth/PasswordResetRequest";
import PasswordReset from "./pages/auth/PasswordReset";
import VerifyEmail from "./pages/auth/VerifyEmail";
import Account from "./pages/Account";

import Organization from "./pages/organization/Organization";
import AddOrganization from "./pages/organization/Add";
import Projects from "./pages/organization/Projects";
import Settings from "./pages/organization/Settings";
import Members from "./pages/organization/Members";
import ProjectManage from "./pages/organization/ProjectManage";
import MemberInvite from "./pages/organization/MemberInvite";
import MemberManage from "./pages/organization/MemberManage";

import Project from "./pages/Project";
import ReportsList from "./pages/project/ReportsList";
import Notifications from "./pages/project/Notifications";

const App = () => {
  return (
    <Box height="100vh" display="flex" flexDirection="column">
      <BrowserRouter>
        <Routes>
          <Route element={<AppLayout />}>
            <Route path="reports" element={<Project />}>
              <Route index element={<ReportsList />} />
              <Route path="resolved" element={<ReportsList resolved={true} />} />
              <Route path="notifications" element={<Notifications />} />
            </Route>
            <Route path="account" element={<Account />} />
            <Route path="add-organization" element={<AddOrganization />} />
            <Route path="organization/:id" element={<Organization />}>
              <Route path="projects">
                <Route index element={<Projects />} />
                <Route path="manage/:projectId?" element={<ProjectManage />} />
              </Route>
              <Route path="members">
                <Route index element={<Members />} />
                <Route path="invite" element={<MemberInvite />} />
                <Route path="manage/:memberId" element={<MemberManage />} />
              </Route>
              <Route path="settings" element={<Settings />} />
            </Route>
            <Route index element={<Navigate to="/reports" replace />} />
          </Route>

          <Route element={<AuthLayout />} path="/auth">
            <Route path="login" element={<Login />} />
            <Route path="register" element={<Register />} />
            <Route path="verify-email/:hash" element={<VerifyEmail />} />
            <Route path="request-password-reset" element={<PasswordResetRequest />} />
            <Route path="password-reset/:hash" element={<PasswordReset />} />
          </Route>
        </Routes>
      </BrowserRouter >

      <Footer />
    </Box>
  );
};

export default App;
