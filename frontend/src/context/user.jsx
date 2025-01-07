import React from "react";
import { Navigate } from "react-router";
import useSWR from "swr";

import LoadingPage from "components/LoadingPage";

const UserContext = React.createContext();

export const UserProvider = ({ children }) => {
  const { data, error, isLoading } = useSWR("/api/account");

  if (error) {
    return <Navigate to="/auth/login" replace />;
  }

  if (isLoading) {
    return <LoadingPage />;
  }

  const user = {
    getRole: (orgId) => data.org_roles[orgId],
    ...data,
  };

  return (
    <UserContext.Provider value={{ user }}>
      {children}
    </UserContext.Provider>
  );
};

export const useUser = () => {
  const context = React.useContext(UserContext);

  if (!context) {
    throw new Error("useUser must be used within an UserProvider");
  }

  return context;
};