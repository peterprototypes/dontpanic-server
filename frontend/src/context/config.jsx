import React from "react";
import useSWR from "swr";

const ConfigContext = React.createContext();

export const ConfigProvider = ({ children }) => {
  const { data, isLoading } = useSWR("/api/config");

  const loadingData = {
    version: "0.0.0",
    registration_enabled: false,
    can_send_emails: false,
    pushover_enabled: false
  };

  return (
    <ConfigContext.Provider value={{ config: isLoading ? loadingData : data }}>
      {children}
    </ConfigContext.Provider>
  );
};

export const useConfig = () => {
  const context = React.useContext(ConfigContext);

  if (!context) {
    throw new Error("useConfig must be used within an ConfigProvider");
  }

  return context;
};