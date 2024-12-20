import React from "react";
import useSWR from "swr";

const ConfigContext = React.createContext();

export const ConfigProvider = ({ children }) => {
  const { data } = useSWR("/api/config");

  return (
    <ConfigContext.Provider value={{ config: data }}>
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