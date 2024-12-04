import React from "react";
import { Navigate } from "react-router";
import useSWR from "swr";

const UserContext = React.createContext();

export const UserProvider = ({ children }) => {
  const { data, error } = useSWR("/api/auth/user");

  if (error) {
    return <Navigate to="/auth/login" replace />;
  }

  if (!data) {
    return <div>Loading...</div>;
  }

  return (
    <UserContext.Provider value={{ user: data }}>
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